# work-extra-action-slot：英雄第 5 技能欄（extra_action_slot，D 欄，ability index 4）

```text
日期：2026-10-25
目前工作 ID／狀態：G-3 extra_action_slot／Rust 全部完成並通過測試；UE 原始碼已改並寫好 Editor 測試，尚未編譯（需主 session 建置）；2026-10-25 23:07 著陸
玩家或作者新增能做到的事：英雄可以有第 5 個技能（被動或主動，可設等級門檻，例如 16 級才能學）。學習、施放、冷卻、法力消耗、施法進度和揭露都與 Q～R 相同。HUD 會多出第 5 張技能卡，預設按鍵 X。4 技能英雄完全不受影響：沒有第 5 張卡、沒有多送任何 fact，hash 與 payload 也不變。
repo HEAD／本次修改：未 commit；見「修改檔案」
L1／L2／L3／L4 證據：L1 fresh（Rust）；UE Editor 測試已寫、未執行；L3 由主 session 在 quick map 執行
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward/shadow-target
```

## 資料格式（已通知 source-index，H04／H06 照這個格式準備）

- 英雄的 `abilities` 最多 5 個，第 5 個（Lua index 5，runtime slot 4）就是 D 欄。authored `hero_updates.abilities_by_slot`／`ranks_by_slot` 可以有第 5 項（heroes.lua 迴圈改為 1..5）。
- `moba_loadout.ranks` 可以寫 4 或 5 項（`omoba_content_model::MobaLoadout` 自訂反序列化）。只寫 4 項時，第 5 欄出生等級為 0。預設值為 `[1,1,1,1,0]`。
- 等級門檻沿用每級的 `levels[i].required_hero_level`，例如 16 級才能學就寫 `levels={{required_hero_level=16,...}}`。出生等級 > 0 卻有門檻時，content validation 會拒絕。
- 被動（`ability_type='passive'`）可以學習，但不能施放；主動技的規則與 Q～R 相同。handler 寫法不受 slot 影響。

## 各層改動

| 層 | 內容 |
|---|---|
| content-model | `LEGACY_HERO_ABILITY_SLOTS=4`、`HERO_ABILITY_SLOTS=5`；`MobaLoadout.ranks: [u8;5]`（接受 4 或 5 項）；loadout 上限 5 |
| template-ids | `HERO_ABILITY_SLOTS=5`（build.rs 以 const assert 與 content-model 比對）；`MobaLoadoutConst.ranks: [u8;5]` |
| Lua | `heroes.lua` 的 authored 合併迴圈 1..5 |
| 權威端 | `admit_ability_upgrade`／`admit_ability_cast` 接受 index < 5（沒有第 5 技能的英雄回 `ABILITY_NOT_BOUND`，沒有任何副作用）；birth loadout 上限 5；bot 技能 slot < 5；施法進度 `cast_windup::emit_progress` 涵蓋 slot 4；`snapshot` 的英雄技能陣列為 5 |
| 法力 HUD metric | 新增 slot 4 namespace `MMC4`／`MMR4`。Q～R 沿用原本每位英雄 9 個 ordinal 的區塊，slot 4 用獨立區塊 20000+；只有英雄真的有第 5 技能時才送，4 技能的對局 fact 完全不變 |
| 揭露 | `CommittedAbilityRanks.ranks` 改為 `Vec<i32>`：4 技能英雄仍是 16 bytes，有 D 的英雄是 20 bytes。replica 接受 16 或 20；英雄有 5 技能卻只收到 16 bytes 時視為 malformed。`CommittedCooldown`／`CommittedCharge` 的 slot 上限改為 < 5 |
| client runtime | 輸入 `AbilityCast`／`AbilityUpgrade` 接受 index < 5；HUD projection 取前 5 個技能；`cast_progress` 支援 slot 4 |
| bridge CABI | **ABI 37 → 38**：`OmMobaHud.abilities[5]`（插在 struct 中間，後面欄位的 offset 會位移，所以必須升版）；新增 `#define OM_ABILITY_SLOT_COUNT 5`；`OM_MAX_ABILITY_SLOT=4`；driver 驗證上限 5。已用 cbindgen 0.29.4 重新產生 staged header。所有 pin 都已同步：lib/driver/projection 測試的 37→38 共 9 處，smoke 加 `OM_ABI_VERSION == 38` 與 abilities 陣列大小斷言 |
| UE | 新增 `OmAbilitySlots.h`（Count=5、Legacy=4、Extra=4，並在 subsystem `static_assert` 等於 `OM_ABILITY_SLOT_COUNT`）。`EOmAbilitySlot::Extra`；新 action `CastExtra`／`UpgradeExtra`／`SelfCastExtra`／`CastOverrideExtra`；cast mode 陣列為 5，存檔 key 為 `"Extra"`（舊存檔沒有這個 key 時為 Normal）。HUD、輸入、瞄準、bridge actor、autopilot 的所有 ability 陣列和迴圈都改為 5（隊友 4 格、道具 6 格不動）。第 5 張技能卡只在 slot 4 有技能時顯示，出現或消失時重新排版；排版量測跳過隱藏的卡 |

## 預設按鍵（寫入 profile）

- Classic：施放 **X**、升級 **Ctrl+X**、對自己施放 **Alt+X**、Shift 覆寫 **Shift+X**。不用 T、G 的原因：T 是表情輪盤、G 是 ping 輪盤，D／F 是召喚師技能。X 在 classic profile 原本沒有用途。
- 舊存檔：沿用 `ReleaseOccupiedDefault` 機制。玩家的存檔若已把 X（或 Ctrl／Alt／Shift+X）綁給其他動作，新動作就保持未綁定並產生 MigrationConflict 警告，不搶既有綁定。
- WASD 鍵盤方案（`SeedKeyboard`）：X 在這個方案是 Stop，所以第 5 技能的四個動作預設未綁定，玩家需要自行設定。這是已知限制。
- 設定畫面：「施放第五技能」等標籤與說明，以及「第五技能」施法模式列（Tag `CastMode.Extra`）。

## autopilot

`ChooseUpgrade`：R 和 D 只要能升就優先升（D 通常有等級門檻，未達門檻時 `bCanUpgrade` 為 false）；其餘不變。`ChooseCast` 的順序為 R、D、Q、W、E。intent 名稱為 `upgrade_extra`、`cast_extra_*`。

## 測試

shadow tree，`CARGO_TARGET_DIR=target/session-ward/shadow-target`（依使用者指示，只跑到能確認功能成功的程度）：

| 命令 | 結果 |
|---|---|
| `cargo test -p base_content --lib -- extra_action_slot loadout birth upgrade` | 15 passed，含 4 個新測試：level gate（5 級學不了 6 級門檻、6 級可學並施放）、被動 D 可學不可放、4 技能英雄的 slot 4 無副作用，以及 240 tick 兩隊 replica 逐 tick hash 一致（D 學習、施放、冷卻，對手看到 20 bytes ranks 與 slot 4 冷卻） |
| mutation：replica 的 `CommittedCooldown` slot 上限改回 3 | parity 測試失敗（team 2 frame not applied），確認測試真的涵蓋對手視角 |
| omoba-content-model 全部、omoba-template-ids 全部（同步實際樹最新測試後）、omfue/codegen 全部 | 全部通過 |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 通過 |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml --lib` | 164 passed（input_bridge 測試改為 slot 0..5，最後一個 input id 預期由 5 改為 6） |
| `cargo test --manifest-path omfue/bridge/Cargo.toml` | lib 149、integration 2 passed（ABI 38） |
| `cargo check --locked -p omobab --tests` | 通過 |
| 著陸後在實際樹執行 `cargo check` omobab、bridge --tests | 通過 |

base_content 與 omobab 的完整測試沒有重跑（耗時約 50 分鐘；改動範圍內的測試都已涵蓋）。

## UE Editor 測試（已寫，需主 session 編譯並執行）

新檔 `OmEditor/Private/OmExtraAbilitySlotTests.cpp`：
- `Om.Runtime.ExtraAbilitySlotProfile`：四個 Extra 動作的預設 chord 都正確、`AbilitySlot==4`；D／F 不是 X；Extra 施法模式可設定、可存檔、可讀回；舊存檔沒有 `Extra` key 時為 Normal；`OmAbilitySlots::Count == OM_ABILITY_SLOT_COUNT`。
- `Om.Runtime.ExtraAbilitySlotHud`：Q～R 的卡一定顯示；4 技能英雄沒有第 5 張卡；送入 slot 4 的技能後顯示，清除後隱藏；預設快捷鍵顯示為「X」。
- `Om.Runtime.ExtraAbilitySlotAutoPilot`：可升級的 D 會被選上，但 R 仍優先；沒有 D 時永遠不選；D 可用時會施放，冷卻中則改放 Q。
- 既有 `Om.Runtime.AutoPilotAbilityRule` 的 `FSlotCard Cards[4]` 改為 5：陣列參數會退化成指標，維持 4 的話 `ChooseCast` 讀 slot 4 會越界。

## 主 session 需要做的

- 三端加 UE 一起重建（ABI 38：bridge DLL、header 與 UE 必須一起換）。執行上面三個新 Editor 測試，以及既有的 ControlProfile、OwnedViewportIpc、AutoPilot、MatchTermination 測試。
- 實機：4 技能英雄的畫面應該和之前完全一樣，只有 4 張卡。等 source-index 的 H04／H06 資料進來後，再驗證 D 欄（16 級門檻、學習、施放、冷卻）。

## 限制

- 目前內容裡沒有 5 技能英雄。Rust 測試是在 runtime 給 luminary 加上第 5 個技能（`apprentice_lance` 的 6 級門檻、被動用 `rain_iron_cannon`）。沒有新增 fixture 英雄，因為 hero／ability 的數字 ID 依宣告順序配置，插入新項目會讓 authored 內容的 ID 位移。
- 英雄死亡期間，slot 4 的法力 metric 依持久 loadout 決定是否送出。正式內容的持久 loadout 本來就有 5 技能，所以只影響上面那種 runtime 測試。
- 非 shipping 的 ability smoke（`OM_ABILITY_SMOKE`、first-learn smoke）仍然只測 Q～R。

## 追加：auto_learn（來源 NOT_LEARNABLE；2026-10-25 23:19 著陸，UE 尚未編譯）

source-index 回報 H04 `c17_devotion`（被動，出生 rank 1）與 H06 `b08_endless_fang`（16 級自動學會）在來源都是 DOTA_ABILITY_BEHAVIOR_NOT_LEARNABLE。

- **資料**：ability 層級的通用欄位 `auto_learn=true`，適用任何欄位，不限 D。效果：
  - 每一級在英雄等級達到該級的 `required_hero_level` 時自動學會，不消耗技能點。
  - 玩家手動加點會被拒絕，回 `UPGRADE_REJECTED`（不新增拒絕碼，CABI 不變）。
  - 出生 rank 照 `moba_loadout.ranks`。birth loadout 本來就不扣技能點，max_level 1 也不能再加點。
- **權威端與 replica 一致**：XP 只在權威端發放，replica 的等級由 CommittedProgression 得知。因此自動學會不在發 XP 時執行，而是在 `drain_pending_ability_upgrades`（AbilityUpgrades phase）開頭執行 `auto_learn_gated_ranks`。權威端與每個 replica 在同一個 tick、用相同的英雄等級學會，並送出相同的 SkillLearn。死亡中的英雄在重生後補學。
  - 規則抽成純函式 `auto_learned_rank`，並有單元測試：門檻、max level 上限、一次跨多個門檻、rank 不會降低、負值。
- **管線**：`AbilityDefinition.auto_learn`（content-model）→ `AbilityConst.auto_learn`（template-ids，build.rs 與 runtime_content）→ UE codegen 的 `FAbilityVisualEntry.bAutoLearn`（已重新產生 OmVisualRegistry／OmRegistry／manifest）。
  - `PopulateAbilityUpgradeHud` 遇到 auto_learn 時：未達等級顯示 RequiresHeroLevel，達到後顯示 Unavailable，tooltip 為「英雄 N 級時自動學會」。不會出現升級按鈕。
- **測試**：omoba-core `auto_learned_ranks_follow_each_level_gate_and_never_drop`、content-model 68、template-ids、codegen 86、base_content（extra_action_slot、upgrade）12，全部通過；omobab check 通過。真正的端到端測試要等 H06 的 `b08_endless_fang` 資料進來（目前內容中沒有任何 auto_learn 技能）。
