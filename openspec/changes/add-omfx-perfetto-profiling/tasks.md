## 1. Dependency And Startup Setup

- [x] 1.1 確認 `omfx/executor`、`omfx/game`、`omoba-core` 的 target/features 與現有 logging 初始化，決定 `tracing`、`tracing-subscriber`、`tracing-log`、`tracing-perfetto` 依賴放置位置，避免影響 wasm/android target。
- [x] 1.2 在 `omfx/executor` 新增 profiling initialization module，解析 `OMFX_PERFETTO_TRACE`、`OMFX_PERFETTO_PATH`、`OMFX_PERFETTO_DETAIL` 與可選的 `OMFX_PERFETTO_MAX_SECONDS`。
- [x] 1.3 調整 executor startup 的 logging/profiling 初始化，避免 `simplelog`、`tracing_subscriber` 或 `LogTracer` 重複初始化，並保留 `omfx_app.log`、terminal log 與 `omfx.log` 行為。
- [x] 1.4 實作 trace output path resolution，預設輸出到 `target/profiles` 類目錄，啟用成功時 log 完整路徑與 `ui.perfetto.dev` 提示。
- [x] 1.5 實作 profiling 初始化失敗降級：路徑或 layer 建立失敗時輸出 warning/error，並繼續一般 omfx 啟動。

## 2. Frontend Instrumentation

- [x] 2.1 在 `omfx/game` 加入必要的 `tracing` instrumentation dependency，並確認 wasm/android target 不會引用 native-only Perfetto output dependency。
- [x] 2.2 在 `omfx/game/src/native.rs` 的 `Plugin::update` 建立 frame-level root span，附帶 frame/tick、network entity count、projectile count、draw calls 或 triangles 等 fields。
- [x] 2.3 替 automatic hooks/input submission、lockstep event drain、snapshot consumption 與 render bridge update 加入 structured spans。
- [x] 2.4 替 sim batch update、body/HP batch mesh update、VFX/projectile update、camera update、UI update 與 frame statistics recording 加入 structured spans。
- [x] 2.5 實作 granularity guard，確保 default `frame` 不產生 per-entity spans；只有 `OMFX_PERFETTO_DETAIL=deep` 時才啟用 selected inner-loop spans。

## 3. Sim Runner Instrumentation

- [x] 3.1 在 `omfx/game/src/sim_runner.rs` 的 tick loop 建立 tick-level root span，附帶 lockstep tick、`tick_input_rx.len()`、input count 與 runtime publish flag。
- [x] 3.2 替 `TickBatch` receive、DEV Lua reload check、input metadata retention、input push/apply 與 time resource update 加入 structured spans。
- [x] 3.3 替 dispatcher execution、pending queue drains、pre-script `process_outcomes`、script dispatch、post-script `process_outcomes` 加入 structured spans。
- [x] 3.4 替 metadata snapshot refresh、runtime snapshot extraction、render FX retention、snapshot publish 與 applied input metadata publish timestamp 加入 structured spans。
- [x] 3.5 確認既有 `sim_runner_profile` log 保留，Perfetto trace 與文字 profile 可同時使用且不重複初始化 logging。

## 4. omoba-core Instrumentation

- [x] 4.1 在 `omoba-core` 加入 lightweight `tracing` instrumentation dependency；core 不解析 `OMFX_PERFETTO_*`，也不建立 Perfetto writer。
- [x] 4.2 在 `omoba-core/src/runtime/native/game_processor.rs` 的 `drain_pending_moves`、`drain_pending_ability_upgrades`、`drain_pending_ability_casts`、`drain_pending_tower_spawns`、`drain_pending_tower_sells`、`drain_pending_tower_upgrades`、`drain_pending_item_uses` 與 `process_outcomes` 加入 coarse spans 與必要 counts。
- [x] 4.3 在 `omoba-core/src/runtime/native/scripting/dispatch.rs` 與必要的 `world_adapter.rs` hot paths 加入 script dispatch / adapter spans，default granularity 不做 per-unit 或 per-API-call spans。
- [x] 4.4 在 omfx sim runner 使用到的 runtime snapshot、metadata 或 world initialization 路徑加入 coarse spans，協助區分 content reload、metadata rebuild 與 runtime snapshot extraction 成本。
- [x] 4.5 在 `omoba-core/src/kcp/client.rs` 的 lockstep join/receive/send、seq-gap `StateReq` 與 inbound dispatch hot paths 加入 spans/events，讓 omfx trace 能關聯 network receive 與 tick processing。
- [x] 4.6 確認 backend 或其他使用 `omoba-core` 的 binary 未安裝 Perfetto layer 時不會產生 trace 檔，且 spans 不改變 runtime 行為。

## 5. Workflow And Documentation

- [x] 5.1 文件化 profiling 啟用方式、環境變數、trace 預設輸出路徑、Perfetto UI 開啟方式與主要 spans/tracks 對照。
- [x] 5.2 在文件中標示 `omoba-core` spans 的來源與限制：core 只輸出 `tracing` spans，Perfetto 檔案由 omfx executor 控制。
- [x] 5.3 若新增 `run_profile.bat`，確認檔案為 CRLF 行尾，且不改變 `run.bat` / `run_stress.bat` 的預設行為。
- [x] 5.4 文件化 `OMFX_PERFETTO_DETAIL=deep` 的 overhead 與 trace size 風險，建議只做短時間錄製。

## 6. Verification

- [x] 6.1 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content` 或等價 frontend native build，確認 profiling disabled 不需要任何環境變數。
- [x] 6.2 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 或涵蓋 `omoba-core` runtime 的等價測試，確認 core instrumentation 未改變 backend/shared runtime 行為。
- [x] 6.3 以 profiling disabled 短 run 驗證不產生 trace 檔，且 `omfx_app.log`、terminal log、`omfx_frame` / `omfx_render` 仍可用。
- [x] 6.4 以 `OMFX_PERFETTO_TRACE=1` 短 run 驗證可產生 Perfetto trace，startup log 包含輸出路徑與 `ui.perfetto.dev` 提示。
- [x] 6.5 用 Perfetto UI 開啟 trace，確認可看到 frontend main thread nested frame spans、`omfx-sim-runner` tick pipeline spans，以及 `omoba-core` runtime/KCP spans。
- [x] 6.6 驗證 `OMFX_PERFETTO_PATH` 指定自訂路徑有效，且不可寫路徑只會 warning/error，不阻止遊戲啟動。
- [x] 6.7 驗證 default granularity 不產生 per-entity spans；若實作 `deep`，確認它必須明確 opt-in。
