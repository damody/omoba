# M7：items.dll 的 host 接線（設計，工作線 A，2026-10-30）

`scripts/items`（items.dll，`v3_rust.lua --items`）已放回 live，可產生 66／168 件。host 目前還完全照內容目錄（`MOBA_ITEM_CATALOG` 的 `ActiveEffect` 與被動 modifier）執行道具，items.dll 沒有被載入。本文件是接線的做法，請 B 審過再實作（會動到 `game_processor.rs`、`scripting/dispatch.rs`）。

## 原則

- 只有 items.dll 有註冊的道具走 script，其餘照舊走目錄；同一件道具不會兩邊都跑。
- authority 與 replica 走同一條路：判斷「是不是 script 道具」只看 registry（兩邊載入同一組 DLL，握手的 set hash 已保證）。
- 成本（冷卻、充能、消耗品次數、法力）仍由 host 依目錄的數值結算；script 只負責效果，跟英雄技能的 `execute` 一樣：`RErr` 時 host 回滾成本。

## 第一階段：主動道具

1. **載入**：`scripts/script_dlls.lua` 的 `modules()` 在 base_content 之後、英雄之前加 `{ module = 'items', file = 'items.dll' }`（沒有 hero 欄位）。launch staging、scripts-manifest.json、set hash、握手、`cargo_args` 會自動帶上。`hero_dlls.lua` 不加。
2. **哪些道具走 script**：安裝 registry 時，把 items 模組註冊的 ability id（就是道具 id）存成 World resource `ScriptedItems(BTreeSet<String>)`，authority 與 replica 都是。
3. **AbilityRegistry**：`populate_ability_registry` 跳過 `ScriptedItems` 裡的 id（或 AbilityDef 加 `kind = item`），讓英雄技能 UI、bot、施放准入不會把道具當成技能。
4. **使用**：
   - `apply_resolved_item_use` 遇到 `ScriptedItems` 的道具，不要求目錄的 `active`。
   - 照目錄的 target 形狀驗證目標（沿用 `catalog_item_target`、`commit_targeted_item_use` 的目標檢查）。
   - 結算冷卻、充能、消耗品、法力。
   - 送出擴充後的 `ScriptEvent::ItemUsed { owner, item_id, target }`。
5. **效果**：`dispatch.rs` 處理 `ItemUsed` 時，如果 registry 有 `item_id` 的 AbilityScript，就以 cast 的方式（同一套 adapter、pulse 規則以外的施放結算）呼叫 `execute(owner, target, 1, level_json)`；`RErr` 時回滾第 4 步的成本（或改成先 execute 再提交成本，與英雄技能一致，看 B 偏好）。
   - 原本給 unit hook 的 `on_skill_cast("item:<id>")` 照舊。
6. **測試**：一件 script 道具（例如 FC-037 的禦神刀主動）用 SimulationDriver 端對端：使用 → 效果套上、進冷卻；`RErr` → 不扣成本；replica 得到相同結果。

## 第二階段：被動與狀態觸發

- 目錄已經從 v3 數值產生被動 modifier（屬性加成照舊）。items.dll 只負責目錄做不到的部分：被動觸發（on_attack_hit、on_take_damage、periodic、pre 等）。
- 裝備（含合成產生、撿起、交換到背包）時呼叫 `on_learn(owner, 1)`；產生碼的 `passive_marker` 放上 learn marker 與事件標記。卸下時移除 marker（`<item id>`），並移除 `<item id>` 開頭的被動列。
- 同名多件：marker 以道具 id 為 key，多件共用一個 marker（v3 的 `unique_group` 與 `scope = item_instance` 在這之後再細分）。

## 不做的

- 不改商店、合成、價格：仍由 `item_shop_v3` 與目錄負責。
- 不產生「尚未開放」的道具：它們不在 items.dll，照目錄或照規定不可購買。
