## Why

`extract_snapshot` 是完整 ECS → render snapshot 的重型 dump 工具；sim_runner 不需要用它來驅動 runtime render。現在它在 runtime tick loop 中持續呼叫，會讓 1000 tower / 1000 creep stress path 每個 tick 重掃世界並輸出大量 snapshot 診斷，直接拖累把 sim/render 推到 120 FPS/TPS 的目標。這個 change 要切斷 sim_runner 對 `extract_snapshot` 的依賴。

## What Changes

- sim_runner 不再呼叫 `extract_snapshot`；初始化 seed 與 runtime updates 都改由明確的輕量路徑建立。
- 從 sim_runner per-tick loop 移除 `extract_snapshot` 呼叫與 `[sim_runner]` / `[mirror-snapshot]` 類完整 snapshot 掃描診斷。
- runtime tick 後若需要更新 render-facing state，改走輕量 incremental/mirror publication path，不重建完整 `SimWorldSnapshot`。
- 保留 lockstep TickBatch 驅動本地 sim_runner 的 determinism，不更動 KCP/lockstep protocol。
- 加入 guard/diagnostic，若 sim_runner 又呼叫 `extract_snapshot`，能在開發 log 或測試中被抓到。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `render-sim-cadence`: 明確要求 healthy path 下 sim/render 以 shared 120 cadence 運作，且 diagnostics 能辨識 runtime 是否還在做完整 snapshot extraction。
- `sim-snapshot-rendering`: 明確要求 omfx sim_runner 不得呼叫 `extract_snapshot`，runtime render updates 必須避免完整 snapshot 重建。

## Impact

- `omfx/game/src/sim_runner.rs`: 移除 `extract_snapshot` 呼叫，改為初始化 static seed 與 runtime lightweight publication。
- `omfx/game/src/native.rs`: 若目前 render 依賴每 tick `SimWorldSnapshot.entities`，需改接 runtime lightweight mirror 或在無新 full snapshot 時不重跑 snapshot-driven work。
- `omoba-core/src/runtime/native/snapshot.rs`: 保留 `extract_snapshot` 資料契約，但它不再是 omfx sim_runner publication path。
- `omoba-core/src/lockstep_timing.rs`: 確認 120 TPS source of truth 被 sim/render 共用。
- 測試與驗證：相關 cargo tests/build、TD_1 smoke、TD_STRESS smoke，並確認沒有 sim_runner `extract_snapshot` runtime log。
