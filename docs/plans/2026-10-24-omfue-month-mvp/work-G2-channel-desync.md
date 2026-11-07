# work-G2-channel-desync：G-2 引導在實機造成 observer mismatch（split16）

```text
日期：2026-10-25
目前工作 ID／狀態：split16 desync／已修正並著陸（2026-10-25 19:11 著陸），待主 session 重建三端並跑 split17
玩家或作者新增能做到的事：引導技（lumen_tremor、lumen_tether）在 secure lockstep 下兩隊 replica 逐 tick 與權威一致；被敵方暈眩時，自己隊伍的 replica 也知道，不再「本地准入、權威拒絕」；引導期間 rebase 後狀態完整還原。
repo HEAD／本次修改：未 commit；見「修改檔案」
已執行命令、exit code、build identity：見「測試」
L1／L2／L3／L4 證據：L1 fresh（parity 12 個 + lane soak 1 個 + 全套回歸）；L3 未執行（split17 由主 session 跑）
owned process／資源清理結果：未啟動任何 process；cargo 只寫 target/session-ward/shadow-target
```

## 證據（split16 server.stdout.log）

- tick 3706：pid=1 施放 `lumen_tremor`，權威 `AbilityChannel start`。同一行 `AbilityCast ok` 出現兩次（權威與 observer replica 各一次），但 `AbilityChannel start` 只有一次：replica 端引導沒有開始。
- tick 3720：`observer mismatch team=1 ... entity=1 schema=0x464f4705`（DISCLOSED_HERO），也就是施法者本人。
- tick 6438 第二次施放，同樣在施法者（entity 373）mismatch，rebase 後又失敗，最後 terminate。

## 根因

1. **replica 拒絕了自己隊伍的引導施法。** `stage_ability_channel`、`ability_channel::live`／`can_start` 都要求 `MobaMatch`（phase、hero slot），`begin_ability_channel` 直接 `write_resource::<MobaMatch>()`。replica 沒有 `MobaMatch`，所以施法在 replica 被拒絕：不扣魔力、不進冷卻。權威卻扣了，從第 0 tick 起 `ability_cooldowns`／`ability_cooldown_totals` 就不一致（parity 測試重現：`actual={} expected={"lumen_tremor":30720}`）。
2. **己方英雄被敵方控制時，replica 不知道（通用缺口，tremor 讓它變得很常見）。** 敵方造成的 stun／silence／root 只存在權威的 BuffStore，replica 的 BuffStore 從來沒有。被暈的英雄再施法時，權威拒絕（stunned），replica 准入，冷卻分歧。tremor 每 0.25 秒暈 0.2 秒，對面英雄幾乎一直處於暈眩，autopilot／bot 一施法就分歧。這和引導本身無關，任何敵方暈眩都會造成，只是先前的訓練英雄沒有持續暈眩技能。
3. **rebase 會丟失引導狀態。** replica 的 specs BuffStore 不在 bootstrap 中，rebase 之後引導狀態（hero_tick 據此暫停攻擊）與控制狀態都會消失。

## 修正

1. replica 用相同的施法者狀態准入引導施法：`stage_ability_channel` 與 `ability_channel::live` 只在有 `MobaMatch` 時才檢查 phase／recall／hero slot；`begin_ability_channel`（以及 recovery 版本）改用 `try_fetch_mut`。己方的冷卻與魔力因此由 accepted input 確定性重算，符合「己方冷卻由己方輸入決定」的既有設計。replica 也照跑引導生命週期，作為預測。
2. 新增 team-private fact `FactKind::OwnerControl = 40`／`ObservableFact::OwnerControl { source, batch, row }`（`omoba-core/src/runtime/native/owner_control.rs`）：
   - 權威在每次 `process_outcomes` 結束時（每個 active tick 有兩個 batch：pre-script、post-script），對每個英雄送出精確的 stun／silence／root 剩餘時間，以及 `ability_channel`／`recovery_channel` 狀態的剩餘時間與總長（0 = 沒有）。只在有任一值時送，最後再送一次全 0 代表結束。audience = 該英雄的隊伍（projector 另外檢查只送給自己隊伍）。
   - replica 在自己對應的 `process_outcomes` 之後套用同一個 batch，所以之後每個 phase 讀到的控制與引導狀態都和權威相同。權威說引導已結束時，replica 也會丟掉本地的預測列（`ability_channel::forget`），由權威決定引導何時結束（包括 anchor 進入迷霧、隱藏位移等 replica 無法判斷的原因）。
   - payload 57 bytes：`batch u8`、3 × i64 控制剩餘、2 × (i64 剩餘, i64 總長)，全部 LE；decode 會檢查邊界（≤ 60 秒、剩餘 ≤ 總長、batch ≤ 7）。
3. rebase／bootstrap：specs 建立新的 local life 時（bootstrap、rebase、reveal），用 owner-only 的 `buff_visual_state` baseline（本來就帶精確剩餘時間與引導總長）播種同一組控制與引導狀態，rebase 後的第一個 phase 就正確。引導的效果（傷害、暈眩敵人）本來就以權威 fact 揭露給 replica，replica 不需要重建引導列。

這對應主 session 要求的兩條路：己方冷卻／魔力由 replica 確定性重算；replica 無法看見的狀態（敵方控制、引導何時結束）以 committed fact 揭露。

## 「cancelled」的原因（已確認）

`ability_channel::cancel` 只由 `cast_windup::cancel` 呼叫，觸發來源有三種：Stop、任何非 queued 的英雄指令（move／attack／attack-move／hold），以及任何通過 slot／learned／charge／cooldown 檢查的 CastAbility（`admit_ability_cast`）。注意：後來才因射程、魔力或目標被拒絕的施法，也已經取消了引導。

split16 兩次都是 autopilot 在引導期間自己下指令：
- tick 3766：pid=1 `CastAbility ability_index=0`（lumen_bolt），3/12 波後 cancelled。
- tick 6460：pid=1 `AttackTarget target_id=98`，1/12 波後 cancelled。

規則本身沒有問題（玩家下新指令就會中斷引導）。建議 OmAutoPilot 在自己英雄有 `ability_channel` 狀態（UE 端 OmChannelProgress 已經看得到）時不要送新指令，否則 split17 仍只會看到被自己打斷的引導。此項屬於主 session 的 OmAutoPilot，本次沒有修改。

## 修改檔案

- `omoba-core/src/runtime/native/owner_control.rs`（新增，含 2 個單元測試）
- `omoba-core/src/runtime/native.rs`（`pub mod owner_control`）
- `omoba-core/src/runtime/native/ability_channel.rs`（`live` 不再要求 MobaMatch、`forget`、模組說明）
- `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs`（`stage_ability_channel` 不再要求 MobaMatch）
- `omoba-core/src/runtime/native/game_processor.rs`（`try_fetch_mut::<MobaMatch>` ×2、`process_outcomes` 結尾呼叫 `owner_control::settle_batch`）
- `omoba-core/src/runtime/stable_fact.rs`（`FactKind::OwnerControl = 40`、`ObservableFact::OwnerControl`）
- `omoba-core/src/runtime/team_projector.rs`（payload、只送自己隊伍）
- `omoba-core/src/runtime/filtered_specs.rs`（注入 OwnerControl 列、新 local life 從 buff_visual_state 播種）
- `scripts/base_content/src/single_lane_match_tests.rs`（新增 `mod channel_replica_parity`、`mod channel_lane_soak`）

沒有 proto、CABI 或 UE 變更：`TeamPublicEvent.event_kind` 本來就是 u32，新的 kind 只有 omoba-core 的 replica（omb 的 observer 與 client runtime 都用它）會讀。舊版 client runtime 會忽略它，等同修正前的行為，所以三端必須一起重建。

## 測試

所有命令都在 shadow tree 執行，`CARGO_TARGET_DIR=target/session-ward/shadow-target`；著陸後另在實際樹跑 `cargo check`（omobab、base_content --tests），結果皆為 Finished。

| 命令 | exit | 結果 |
|---|---|---|
| `cargo test -p base_content --lib channel_replica_parity` | 0 | 12 passed |
| `cargo test -p base_content --lib channel_lane_soak` | 0 | 1 passed（7200 tick 雙 channeler 分路對戰，約 7 分鐘） |
| `cargo test -p base_content`（全部） | 0 | 440 passed |
| `cargo test --manifest-path omoba-core/Cargo.toml --lib --tests` | 101 | lib 1141 passed；唯一失敗的是 `td_autoplay_100`（3 個），原因是 shadow 沒有 `scripts/target/release` 的 DLL，屬於環境問題，與本次修改無關。另外 omoba-core 的 examples 因為 `WaveBReadView` 缺 `brush_regions` 編不過，實際樹也一樣，是既有問題 |
| `cargo test --locked -p omobab` | 0 | 193 passed、1 ignored，其他 test binary 全部 ok |
| `cargo check --locked --manifest-path omoba-client-runtime/Cargo.toml --tests` | 0 | — |
| `cargo test --manifest-path omfue/bridge/Cargo.toml` | 0 | lib 149 passed、integration 2 passed |

`channel_replica_parity` 的 12 個情境（逐 tick 比對兩隊 replica 的 canonical hash 與每個 component；另外直接比對己方英雄在 replica specs BuffStore 中的 stun／silence／root／ability_channel 與權威是否完全相同）：
- tremor 完整 12 波；armed 施法者；移動中斷；敵方 tremor 暈眩中斷；敵方波殺死施法者
- tether 完整 8 波；armed 加移動中斷
- 被敵方暈眩時施放 bolt：權威拒絕，replica 同樣拒絕
- 引導中 rebase（tremor）；引導中 rebase（armed）；tether 中 rebase 後再移動；被暈眩當下 rebase

mutation 驗證：
- 關閉 replica 套用 `OwnerControl`：8 個測試失敗（例如 `STATUS tick 15 team 1 stun: replica=None authority=Some((205, None))`）。
- 關閉 rebase 播種：3 個 rebase 測試失敗（`ability_channel: replica=None authority=Some((2065, total 3072))`）。
- 修正前的程式：4 個原始情境全部在 tick 0 起 mismatch（`ability_cooldowns: actual={} expected={"lumen_tremor":30720}`）。

## 主 session 需要做的

- 重建 server、client runtime 與 base_content DLL（三端一起），再跑 split17。
- split17 確認：`AbilityChannel start` 之後不再出現 `observer mismatch ... 0x464f4705`；若 autopilot 仍在引導中下指令，log 會出現 `reason=cancelled`，這是預期行為。

## 限制

- replica 的引導預測列在 rebase 後不會重建（只還原狀態），所以 rebase 之後 replica 本地不會再跑剩下的波；波的效果本來就由權威 fact 揭露，hash 測試也證實不受影響。
- 己方英雄被敵方位移（擊退）這類隱藏效果，目前仍只靠 movement fact 修正 render 狀態；與本次無關，未處理。
