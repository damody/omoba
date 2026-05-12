## Why

目前 omfx 只有 `omfx_frame`、`omfx_render` 與 `sim_runner_profile` 這類視窗平均 log，能看出大分類耗時，但無法在時間軸上追到是哪個 thread、哪段 frame/update/render 或 sim tick pipeline 造成卡頓。TD_STRESS 與大量 entity 場景需要可匯入 Perfetto UI 的 trace，才能用 nested spans 精準定位 frontend main thread 與 `omfx-sim-runner` 的瓶頸。

## What Changes

- 新增 opt-in omfx Perfetto profiling 模式，使用 `tracing` + `tracing-perfetto` 輸出 `.perfetto-trace` 或等價 Perfetto trace 檔。
- 在 `omfx/executor` 早期初始化 profiling/logging，透過 `OMFX_PERFETTO_TRACE`、`OMFX_PERFETTO_PATH`、`OMFX_PERFETTO_DETAIL` 等明確控制啟用、輸出路徑與 granularity。
- 在 `omfx/game/src/native.rs` 的高成本 frame/update/render 路徑加入 structured spans，至少涵蓋 `Plugin::update`、lockstep event drain、snapshot consumption、render bridge、batch updates、VFX/projectile、camera、UI 與 frame profile 區段。
- 在 `omfx/game/src/sim_runner.rs` 的 `omfx-sim-runner` thread 加入 tick-level spans，涵蓋 `TickBatch` receive、input apply、dispatcher、pending queue drains、script dispatch、snapshot extraction、render FX retention 與 publish。
- 將 omfx 直接使用的 `omoba-core` runtime/KCP hot paths 納入 profiling 範圍，讓 trace 不只停在 omfx call site，也能看到 shared core 內部的 drain、outcome、script dispatch、snapshot 與 client receive/send 區段。
- 保留既有文字 profile log，Perfetto trace 作為 opt-in deep profiling 補充；預設關閉以避免一般 dev/stress run 付出 trace overhead。
- 文件化 profiling workflow：如何啟用、trace 輸出位置、如何用 `ui.perfetto.dev` 開啟，以及主要 tracks/spans 對照。

## Capabilities

### New Capabilities
- `omfx-perfetto-profiling`: 定義 omfx native frontend 以 `tracing-perfetto` 產生可分析 Perfetto trace 的行為、啟用方式、span 覆蓋範圍、granularity 控制、輸出檔處理與驗證方式。

### Modified Capabilities

## Impact

- 影響 `omfx/executor/src/main.rs` 的 startup logging/profiling 初始化，以及可能新增 `omfx/executor/src/perfetto_profile.rs` 或等價小型 module。
- 影響 `omfx/game/src/native.rs` 與 `omfx/game/src/sim_runner.rs` 的 instrumentation，但不應改變 gameplay、lockstep protocol、sim cadence 或 render pacing。
- 影響 `omoba-core/src/runtime/native/**` 與 `omoba-core/src/kcp/client.rs` 中由 omfx native frontend 直接走到的共享 hot paths；新增 spans 不應改變 backend/omfx 共用 runtime 行為，也不應讓 backend 沒啟用 Perfetto 時產生 trace 產物。
- 可能新增 `tracing`、`tracing-subscriber`、`tracing-log`、`tracing-appender`、`tracing-perfetto` 等 native-only 依賴；需避免破壞 wasm/android target declarations。
- 可能新增 `run_profile.bat` 或文件化 PowerShell/cmd 啟動範例；若新增 `.bat`，必須維持 CRLF 行尾。
- Trace 產物可能很大，預設應輸出到明確且可忽略的位置，例如 `omfx/target/profiles/`，不可自動提交產物。
