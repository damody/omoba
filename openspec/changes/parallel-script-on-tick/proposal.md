## Why

`omoba_core::runtime::script_on_tick` 目前以單一迴圈串行執行所有 `ScriptUnitTag` entity 的 `UnitScript::on_tick`，在 TD_STRESS 這類 1000 座 scripted tower 同步 ready 的場景會偶發達到約 9ms，超過 120Hz 的 8.33ms tick budget。

這會讓 `omfx` 的 local sim_runner 在部分 tick 落後、累積 TickBatch backlog，進一步造成 render frame p99/max spike；需要把 script tick 改為可平行計算，並將 ECS mutation 後移到 deterministic outcome apply 階段。

## What Changes

- 新增 parallel script tick execution 能力，讓 `UnitScript::on_tick` 的主要計算可在多執行緒上處理。
- 將 script `GameWorld` mutation API 從「即時寫 ECS」改為「記錄 script outcome / command」，由統一 apply 階段在固定順序寫回 ECS。
- 保留 script 對世界狀態的讀取能力，但在 parallel tick 階段只允許讀取 snapshot / read-only view。
- 將 projectile spawn、facing/pos/asd_count 更新、damage/splash、FX queue、buff/stat mutation 等副作用納入 deterministic outcome pipeline。
- 維持 authoritative backend 與 omfx local sim_runner 的 lockstep determinism，不引入非決定性排序或 thread-local RNG 分歧。
- 不變更現有 script ABI 對 content 作者的高層語意；若 ABI trait surface 必須新增型別，會同步更新 `scripts/script-abi`、host adapter 與 `base_content`。

## Capabilities

### New Capabilities
- `parallel-script-tick`: 定義 script `on_tick` 如何以 read-only compute + deterministic outcome apply 的模式平行執行。

### Modified Capabilities
- `unit-attack-phase-timing`: scripted tower 的 attack phase 仍必須遵守既有 windup / impact / backswing 語意，但狀態寫回改由 outcome apply 完成。

## Impact

- 影響 `omoba-core/src/runtime/native/scripting/dispatch.rs`、`world_adapter.rs`、`comp/outcome.rs`、`game_processor.rs` 與 tick profile instrumentation。
- 影響 mirror implementation：`omb/src/scripting/*`、`omb/src/comp/game_processor.rs`，需避免 backend 與 core runtime 分歧。
- 影響 `scripts/script-abi` 的 `GameWorld` host implementation；若 ABI surface 變更，需要完整重建 `scripts/base_content.dll` 與 omb/omfx。
- 影響 scripted towers / summons：`scripts/base_content/src/towers/*`、`scripts/base_content/src/summons/*` 需要驗證結果一致。
- 驗證需涵蓋 `cargo check`、相關 unit tests，以及 TD_STRESS 下 `script_on_tick`、`sim_runner_profile`、`omfx_frame_slo` 指標。
