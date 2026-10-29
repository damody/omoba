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

## 實作（影子，2026-10-30，請 B 審）

照 B 的決定做：道具使用採「先 execute 再提交」，與英雄技能同一套 cast transaction（成本與效果一起提交，不靠 RErr 回滾）。AbilityDef 加 `kind`。影子在 `target/lineA/host`，B 審過後用 land_to_live 放回。

- **AbilityDef.kind**：`omoba-core/src/ability_meta.rs` 新增 `AbilityKind { Ability, Item }`，serde 預設 `ability`。
  - `populate_ability_registry`（initialization.rs）跳過 item，英雄技能 UI、bot、SkillCast 都看不到道具。
  - 另外裝 World resource `ScriptedItems`（有主動的 item script id），authority 與 replica 都裝（同一組 DLL）。
- **item 的 def**：`ability_kit::item_ffi(id, passive, script)` 與 `metadata::item_def`。
  - 用 catalog `source_ref` 的 `…:<v3 id>` 對到商店目錄，產生 rank 1 的 cooldown、mana、range 與 target type。
  - 目錄沒有的道具（原作 KV 已註解掉）回傳 None，不註冊。
- **產生器**：
  - 道具的 registry 現在真的列出道具。原本只收 kind ability，所以 items.dll 的 registry 是空的。
  - 道具的 `active`（events、target、flags）現在當成 cast 程式。原本完全沒讀 `active`，主動效果被默默略過，沒有 blocker。
  - cost 的 cooldown、mana、cooldown_groups 由 host 依目錄結算；charges、cooldown_start 不是 on_cast、kind 不是 active 時擋下。
  - `cast.cast_point` 列為 gap：host 使用道具時沒有前搖。
  - 道具產生數因此從 67 件改為 50 件，是實際數字。
- **使用流程**：
  - `drain_pending_item_uses` 遇到 `ScriptedItems` 的道具，改走 `queue_scripted_item_use`：只做准入（擁有者、格子、冷卻、目錄 target 形狀／範圍／隊伍／可見／魔免、法力），不扣任何東西，排 `ScriptEvent::ItemCast`。
  - dispatch 的序列迴圈以 rank 1 呼叫該道具的 `execute`（同一個 adapter 與 mana transaction）。成功才輸出 `Outcome::ScriptItemTransaction`，失敗就丟棄。
- **結算**（game_processor）：`ScriptItemTransaction` 重新確認道具仍在格子且就緒、法力付得起，接著一起提交：
  - 冷卻、共用冷卻、消耗品、法力、shop undo、home speed_up。
  - 推 `ItemUsed`，持有者的 unit hook 與 status ABILITY_EXECUTED 會看到。
  - 然後才結算效果。條件不成立時，使用與效果都不生效。
- **DLL 集合**：`scripts/script_dlls.lua` 的 modules 在 base_content 之後加 `items.dll`。
- **測試**：
  - items_impl：每件產生的道具都以 kind=item 註冊，target 與 cooldown 等於目錄。
  - base_content `scripted_item_use`：把目錄的 active 拿掉，只剩 script 能產生狀態；使用後狀態套上、冷卻開始，冷卻中再用不生效。
  - base_content 中名稱含 item 的 46 個測試都通過。
- **還沒做**：第二階段（被動觸發的 on_learn 與 marker），以及 replica 對 ItemCast 的處理（replica 是否跑 item 佇列）。這些請 B 看 filtered_specs 的需要。

### B 審查（2026-10-30）

- 召喚：道具結算沒有跑 plan_summons，效果含 SpawnSummoned 時整筆拒絕並記 log。目前 6 件主動道具都不召喚。
- 已知 gap：dispatch 已留下的 visual cue，在結算因道具改變而拒絕時不會撤回（只影響呈現）。
- replica 不必另外處理：ItemCast 只在知道擁有者背包的世界排入（authority 與擁有者那一隊的 replica），commit 只讀擁有者自己的 Inventory、Gold 與冷卻。
