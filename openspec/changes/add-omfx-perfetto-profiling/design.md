## Context

omfx native frontend 目前用 `simplelog` 初始化 `log` crate，並輸出 `omfx_app.log` 與終端機；Fyrox 自有 log 另寫 `omfx.log`。`native.rs` 的 `FrameProfile` 每 60 frame 輸出 `omfx_frame` / `omfx_render`，`sim_runner.rs` 每秒輸出 `sim_runner_profile`。這些 log 適合長時間低成本觀察，但只能看到平均數，無法回答「哪個函數/區段在單一 frame 或 tick 中拖慢」。

`tracing-perfetto` 可把 `tracing` spans/events 寫成 Perfetto trace，讓使用者在 Perfetto UI 中看到 thread tracks、nested slices、duration 與 fields。這很適合 TD_STRESS 這類需要看 frontend main thread 與 `omfx-sim-runner` pipeline 的效能調查，但不適合預設常駐，因為大量 spans 會增加 CPU overhead 並產生大型 trace 檔。

`omfx/game/src/sim_runner.rs` 不是只在 omfx crate 內運算；它會呼叫 `omoba_core::runtime` 的 `build_phase3_dispatcher`、多個 `drain_pending_*`、`process_outcomes`、`run_script_dispatch`、metadata/snapshot helpers，以及 `omoba_core::kcp` 的 lockstep client/input types。若只在 omfx call site 放 spans，Perfetto 會看到「core runtime 花了一段時間」，但看不到 shared core 內部是哪個 drain、outcome 或 script/world adapter 區段造成成本。因此本 change 需要把 omfx 會走到的 `omoba-core` hot paths 也納入 instrumentation，但 Perfetto output layer 仍只在 omfx executor opt-in 初始化。

## Goals / Non-Goals

**Goals:**
- 在 native omfx 提供 opt-in Perfetto trace 輸出，能看到 frontend main thread 與 `omfx-sim-runner` thread 的 nested spans。
- 覆蓋主要 frame/update/render path、sim tick path，以及 omfx 直接呼叫的 `omoba-core` runtime/KCP hot paths，讓使用者能從 Perfetto UI 判斷耗時最高的區段。
- 以環境變數控制啟用、輸出路徑、granularity 與可選錄製時間；預設完全關閉。
- 保留既有 `omfx_frame`、`omfx_render`、`sim_runner_profile` 與一般 log workflow。
- 讓 profiling disabled 的一般 build/run 不需要設定任何 profiling 變數，也不產生 trace 檔。

**Non-Goals:**
- 不做 Rust 全函數自動 instrumentation；以手動 spans 覆蓋重要 hot paths。
- 不在一般 `run.bat`、stress run 或 CI 預設產生 trace。
- 不把 trace 上傳外部服務，不整合遠端 Perfetto tracing daemon。
- 不改變 gameplay、lockstep protocol、sim cadence、render pacing 或 snapshot 格式。
- 不要求 wasm/android target 支援 Perfetto profiling。
- 不把整個 backend 或所有 `omoba-core` API 全面 instrumentation；優先限於 omfx native frontend 實際走到、且已知可能昂貴的共享 runtime/client 路徑。

## Decisions

1. 使用 `tracing` spans 表示 profiling 邊界，使用 `tracing-perfetto` 作為 native-only trace output。

   `tracing` 可在 `omfx/game` 與 `omfx/executor` 之間共享 instrumentation API，`tracing-perfetto` 則集中在 native executor 初始化。span fields 會帶上 frame/tick、entity counts、queue length、draw calls 等可關聯場景複雜度的資料。

   Alternative considered: 繼續擴充 `Instant::now()` 手寫 profiler。這較便宜，但只能得到平均 log，無法提供 thread timeline、nested slices 與單次 frame/tick timing。

2. 在 `omfx/executor` 統一初始化 logging/profiling，避免 `log` subscriber 衝突。

   目前 executor 使用 `CombinedLogger::init(...)` 設定 global `log` logger。導入 `tracing` 後，實作應選擇一個全域初始化策略：保留 `simplelog` 給 `log` 並另外安裝 `tracing_subscriber` registry/layer，或改為 `tracing_subscriber` + `tracing_log::LogTracer` 統一收集 `log` macros。設計目標是避免重複初始化造成 panic、避免 log 重複，也避免 Perfetto 啟用失敗阻止一般 logging。

   Alternative considered: 在 `omfx/game` plugin 內初始化 Perfetto。這太晚，會漏掉 executor startup，也更容易與 dynamic plugin/dylib reload 邊界混在一起。

3. 使用環境變數作為主要控制面，並允許 launcher 只是薄 wrapper。

   建議控制項：`OMFX_PERFETTO_TRACE=1` 啟用、`OMFX_PERFETTO_PATH=<path>` 指定輸出、`OMFX_PERFETTO_DETAIL=frame|deep` 控制 granularity、`OMFX_PERFETTO_MAX_SECONDS=<seconds>` 作為可選短錄製保護。未設定 `OMFX_PERFETTO_PATH` 時，預設使用 `omfx/target/profiles/omfx-<timestamp>.perfetto-trace` 或 executor current dir 下等價的 `target/profiles`。

   Alternative considered: 僅使用 Cargo feature。feature 可以完全移除依賴與 macro 成本，但每次 profile 都要重編，不利於快速效能調查。若 dependency 或 target 相容性成為問題，可再把 Perfetto output layer 置於 `perfetto-profiling` feature 下，但 runtime opt-in 仍保留。

4. Granularity 分成 `frame` 與 `deep`，預設避免 per-entity spans。

   `frame` 只覆蓋 frame/tick 大區段與 hot path 函數，例如 event drain、snapshot apply、batch mesh update、projectile/VFX、camera/UI、dispatcher/script/snapshot publish。`deep` 才允許 selected inner-loop 或 per-entity spans，且需在文件明確警告會扭曲效能與膨脹 trace。

   Alternative considered: 每個 entity、每次 UI send、每個 batch item 都建立 span。這雖然細，但在 1000+ entity 場景會嚴重改變瓶頸位置，trace 也可能快速變得不可用。

5. Trace 檔寫入失敗不阻止遊戲啟動。

   Profiling 是診斷工具，不是 gameplay 前置條件。若建立資料夾或檔案失敗，executor 應輸出 warning/error，然後以一般 logging 繼續啟動。成功啟用時，log 應包含完整輸出路徑與 `ui.perfetto.dev` 提示。

6. `omoba-core` 只加入 lightweight `tracing` spans，不負責 Perfetto output。

   `omoba-core` 是 server/frontend 共用 crate，不能把 Perfetto writer、檔案路徑或 executor-specific 設定塞進 core。core 內只應使用 `tracing::{span, instrument}` 這類低耦合 instrumentation；是否輸出到 Perfetto 由目前 process 安裝的 subscriber/layer 決定。這讓 omfx 啟用 Perfetto 時能看到 core 內部 spans，也讓 backend 或測試在未安裝 Perfetto layer 時只付出最低限度的 disabled span 成本。

   Alternative considered: 只在 `sim_runner.rs` 包住每個 `omoba_core::*` call。這能快速做出大區段 timing，但無法拆開 `process_outcomes`、`run_script_dispatch`、`GameWorld` adapter 或 KCP receive/send 內部細節，對定位 shared core hot path 幫助有限。

## Risks / Trade-offs

- [Risk] `tracing-perfetto` API 或 crate version 與目前 Rust/toolchain 不相容 → Mitigation: 先以最小 dependency spike 驗證 `cargo build --manifest-path omfx/Cargo.toml -p executor --features runtime-lua-content`，必要時改用相容版本或 feature gate。
- [Risk] `simplelog`、`tracing_subscriber`、`LogTracer` 全域初始化衝突 → Mitigation: 在 executor 封裝單一 initialization function，disabled/enabled 都走同一路徑，失敗時降級並保留原本 `omfx_app.log`。
- [Risk] deep spans 造成 overhead，改變 profile 結果 → Mitigation: default `frame` granularity 不產生 per-entity spans，`deep` 必須明確 opt-in，文件標示只適合短時間診斷。
- [Risk] trace 檔過大或長時間 run 填滿磁碟 → Mitigation: 文件建議短時間錄製，預設輸出到 `target/profiles`，可支援 `OMFX_PERFETTO_MAX_SECONDS` 或提供短錄製 launcher。
- [Risk] native-only dependency 誤影響 wasm/android target → Mitigation: 將 Perfetto output dependencies 放在 `target.'cfg(not(target_arch = "wasm32"))'.dependencies` 或 executor native crate，`omfx/game` 只依賴跨 target 可接受的 `tracing` macros。
- [Risk] `omoba-core` spans 影響 backend 或測試成本 → Mitigation: core 只加入 lightweight `tracing` dependency 與粗粒度 spans，避免預設 per-entity spans；deep spans 必須受 granularity guard 或 helper 控制。

## Migration Plan

1. 在 `omfx/executor` 新增 profiling initialization module 與必要 native dependencies。
2. 調整 executor startup logging，確認 profiling disabled 時維持 `omfx_app.log`、終端機與 `omfx.log` 行為。
3. 在 `omfx/game` 與 `omoba-core` 加入必要的 lightweight `tracing` instrumentation dependency，並在 `native.rs` / `sim_runner.rs` 加入 frame/tick level spans。
4. 在 omfx 會走到的 `omoba-core` runtime/KCP hot paths 加入 core-level spans，Perfetto output 仍由 omfx executor 控制。
5. 逐步加上 `frame` granularity hot path spans，再以 helper/guard 加上 `deep` selected spans。
6. 新增文件或 `run_profile.bat`，說明啟用方式與 Perfetto UI 使用方式。
7. 驗證 disabled build/run、enabled trace 產出與 trace 內容。

## Open Questions

- `tracing-perfetto` 的實際初始化 API 與可用版本需在 implementation spike 中確認。
- 是否要新增 `run_profile.bat`，或先以文件化環境變數命令為主。
- `OMFX_PERFETTO_MAX_SECONDS` 是否在第一版實作自動停止錄製，或先只文件化短時間手動關閉 workflow。
