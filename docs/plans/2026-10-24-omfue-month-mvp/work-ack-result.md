# work-ack-result：輸入結果分清「已收進這一步」與「已拒絕＋原因」

```text
日期：2026-10-25
目前工作 ID／狀態：checklist 4.3「成敗清楚」／cast 與 upgrade 已完成，待主 session 重建與 L3
玩家或作者新增能做到的事：施放或升級被權威規則拒絕時，renderer 收到 accepted=false 和原因碼（ON_COOLDOWN、NOT_ENOUGH_MANA、OUT_OF_RANGE…），不再假裝成功。
repo HEAD／本次修改：未 commit；見「修改檔案」
來源英雄與 variant：不適用
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh；L3 未執行（主 session 負責）
gate 與原 11 主項的變化：4.3 的 cast／upgrade 失敗回報可測；move／attack 不變
失敗、原因、尚未修的風險：見「限制」
下一個工作 ID：UE／bridge 接 reason（見「UE 端要接的欄位」）；本 session 回到 G-2 channel
人工／硬體需求與是否已取得：無
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward
```

## 原本的語意

`omoba-client-runtime/src/shop_presentation.rs::settled_inputs` 把權威 step 的 `accepted_inputs` 裡每個自己的輸入都回成 `accepted=true, "APPLIED_TO_PRESENTATION"`。這只代表「權威把這個輸入排進這一 tick」，不代表施放或升級真的成立。split6 的 server log 有 `AbilityCast failed ... still on cooldown`，renderer 卻收到 accepted。

## 做法（不改協定）

權威端與 owner replica 跑同一套 `player_input_tick` → `drain_pending_ability_*` → `run_script_dispatch` → `process_outcomes`，而且 replica 已經用 `AdmittedCommandIdentities` 把每個輸入的 `input_id` 對齊到 `PendingPlayerInputs`。因此：

1. 新模組 `omoba-core/src/runtime/native/command_outcome.rs`：`OwnerCommandOutcomes` journal，依 (player_id, input_id) 記拒絕碼。容量有上限（256 列）；correlation 0（bot、legacy）不記。
2. `PendingAbilityCast`／`PendingAbilityUpgrade` 加上 `correlation`（由 `player_input_tick` 帶入 admitted input id）。
3. 拒絕點都會寫碼：
   - upgrade（同步）：INVALID_SLOT、NO_HERO、GAME_NOT_ACTIVE、HERO_INACTIVE、ABILITY_NOT_BOUND、SKILL_POINTS、MAX_LEVEL、LEVEL_REQUIRED、UPGRADE_REJECTED。
   - cast admission（同步）：INVALID_SLOT、NO_HERO、ABILITY_NOT_BOUND、NOT_LEARNED、ON_COOLDOWN。
   - cast 同 tick 後段（dispatch）：GAME_NOT_ACTIVE、ALREADY_CASTING、NOT_LEARNED、ON_COOLDOWN、NOT_ENOUGH_MANA、SILENCED、HERO_INACTIVE、OUT_OF_RANGE、INVALID_TARGET、CAST_REJECTED。handler 回傳 `RErr` 時，以 `classify_handler_error` 判斷（字串開頭若是已知碼加 `:` 就用該碼；否則依 generic effect 的固定字樣判斷範圍或目標）。
   - cast transaction 在 `process_outcomes` 序列 admission 失敗：CAST_REJECTED。
4. client：`SpecsDisclosedWorldStepper::take_owner_command_rejections` → `ReplicaHost::take_owner_command_rejections` → `shop_presentation::apply_owner_rejections`，把同一個 input id 的結果改成 `(id, false, CODE)`，再經既有 `CriticalInputResult { accepted, result_code }` 送給 renderer。

正確性依據：replica 的拒絕只會發生在權威也會拒絕的情況（同一函式、同一份經 hash 驗證的 disclosed owner 狀態）。replica 少了 MobaMatch 那幾項 lane 檢查，所以可能漏報，但不會誤報。

補充（G-2 實作時發現並修正）：replica 沒有 MobaMatch，也不跑 host 生命週期（recovery／pulse channel 一律 stage 失敗），而且看不到隱藏資訊。所以在非權威 world（沒有 `MobaMatch`）裡，dispatch 後段和 transaction 的拒絕只寫入 `command_outcome::owner_state_code` 允許的碼（NOT_LEARNED、ON_COOLDOWN、NOT_ENOUGH_MANA、SILENCED、HERO_INACTIVE、ALREADY_CASTING），也就是只靠 owner 自己英雄狀態判定的碼。其他碼（CAST_REJECTED、OUT_OF_RANGE、INVALID_TARGET、GAME_NOT_ACTIVE）只在權威 world 記錄；權威端的同一筆 journal 仍完整。測試：`a_non_authoritative_world_reports_only_owner_state_rejections`。

## 結果語意（renderer）

| `accepted` | `result_code` | 意義 |
|---|---|---|
| true | `APPLIED_TO_PRESENTATION` | 已收進這一步，這一步沒有規則拒絕它（不保證效果已落地，例如 windup 之後才被打斷） |
| false | `command_outcome` 碼 | 權威規則拒絕，附原因 |
| 商店／ward／undo | 既有碼 | 不變 |

## UE 端要接的欄位

- bridge `omfue/bridge/src/lib.rs::publish_presentation_input_result`：非商店／ward 的 `!result.accepted` 已經寫 `OmAppliedInput.status = InvalidArgument`，所以 autopilot 現在就能記成 rejected，不需改動。
- 要在 HUD 顯示「冷卻中」「法力不足」：在 `OmAppliedInput` 新增 `reason_code: u32`，值取 `omoba_core::runtime::native::command_outcome::rejection_code_number(&result.result_code)`（`REJECTION_CODES` 的 1-based 位置，只能往後加；0 表示沒有拒絕）。目前編號：1 INVALID_SLOT、2 NO_HERO、3 ABILITY_NOT_BOUND、4 NOT_LEARNED、5 ON_COOLDOWN、6 NOT_ENOUGH_MANA、7 SILENCED、8 OUT_OF_RANGE、9 INVALID_TARGET、10 ALREADY_CASTING、11 GAME_NOT_ACTIVE、12 HERO_INACTIVE、13 CAST_REJECTED、14 SKILL_POINTS、15 MAX_LEVEL、16 LEVEL_REQUIRED、17 UPGRADE_REJECTED。這個 ABI 欄位屬於 bridge／UE header（主 session 範圍），本次沒有修改。

## 其他指令種類盤點

| 指令 | 現況 |
|---|---|
| CastAbility、UpgradeAbility | 本次完成 |
| ItemBuy／ItemSell／ShopUndo | 本來就有權威 receipt（SHOP_*、UNDO_*），不變 |
| WardPlacement | 本來就有 ward outcome journal 和 phase，不變 |
| SummonerCast | server 有 receipt（`summoner_world.rs`），但 client 仍回 APPLIED；replica 是否有完整 roster 尚未確認，不能保證不誤報，所以未接 |
| ItemUse、Recall | 失敗只記 log；未接，列為後續 |
| Move／Attack／Stop／Hold | 排進這一步就生效，不需要拒絕碼 |

## 限制

- 有 windup 的施法：開始 windup 就算「已排程」，之後被打斷不會改寫已送出的結果（會由施法回饋呈現）。
- 協定版本未變；舊 client 不送 rejected（行為和以前一樣），新 client 對舊 server 也成立，因為判定在 replica 端。

## 修改檔案

- 新增 `omoba-core/src/runtime/native/command_outcome.rs`
- `omoba-core/src/runtime/native.rs`（mod 宣告）
- `omoba-core/src/runtime/native/comp/lockstep_resources.rs`、`tick/player_input_tick.rs`（correlation）
- `omoba-core/src/runtime/native/game_processor.rs`（admission 改成帶碼；transaction 拒絕）
- `omoba-core/src/runtime/native/scripting/dispatch.rs`、`parallel_world_adapter.rs`、`registry.rs`（dispatch 拒絕碼；測試用 ability 註冊）
- `omoba-core/src/runtime/filtered_specs.rs`（replica 讀取與 parity 測試）
- `omoba-client-runtime/src/replica_host.rs`、`shop_presentation.rs`、`main.rs`

## 測試

所有命令都設 `CARGO_TARGET_DIR=D:\omoba\target\session-ward`。

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test --manifest-path omoba-core/Cargo.toml --lib command_outcome` | 0 | 6 passed：journal 邊界、in-flight、handler 分類、編號固定；`rejected_owner_cast_and_upgrade_report_codes_by_input_id`（真實 `InputSubmit` 編碼 → 解碼 → admitted identity → `SimulationDriver` tick：NOT_LEARNED、SKILL_POINTS、INVALID_SLOT、ON_COOLDOWN，合法升級不留紀錄且等級 +1）；`dispatch_stage_cast_rejections_reach_the_same_input_id`（NOT_ENOUGH_MANA、OUT_OF_RANGE，成功施放不留紀錄且真的進入 CD） |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib owner_replica_reports` | 0 | replica parity：owner replica 解碼自己的 accepted cast，依同一個 input id 回 NOT_LEARNED、ON_COOLDOWN |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib` | 0 | 1119 passed |
| `cargo test --manifest-path omoba-client-runtime/Cargo.toml` | 0 | 161 passed、8 ignored（含 `apply_owner_rejections`） |
| `cargo test --manifest-path omb/Cargo.toml -p omobab` | 101 | lib 192 passed；bin 190 passed，2 個失敗是 `urgent_before_min_batch_still_holds`、`urgent_after_min_batch_short_circuits`（毫秒計時斷言，在機器負載下失敗）。單獨重跑 `--bin omobab urgent_` 時 exit 0，6 passed |
| `cargo test --manifest-path scripts/Cargo.toml -p base_content` | 0 | 396 passed |

修正前的對照：split6 的 server log 有 `AbilityCast failed ... still on cooldown`，同一筆輸入在 renderer 端卻是 `accepted`。新測試用的 `OwnerCommandOutcomes` 在修正前不存在，因此修正前的程式無法產生任何拒絕碼。

## 主 session 需要重建

`omobab`、`omoba-client-runtime`（兩者都用到 omoba-core）。`base_content.dll` 不需要（ABI 未改）。若要接 HUD 原因，再改 bridge／UE。
