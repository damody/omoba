## 1. Baseline 與驗收目標

- [x] 1.1 確認或建立 `TD_STRESS_10K` 等價 10k render-backed units stress 路徑，記錄 release/profiling-disabled 驗收方式與 measurement window
- [x] 1.2 在現有程式中確認 `omfx::sim_runner::receive_tick_batch` 是否包住 blocking wait，並確認 `native.rs` TickBatch forwarding 是否 prompt
- [x] 1.3 建立修正前 baseline：frame FPS/1% low、sim TPS、queue backlog、`receive_tick_batch`、`omfx::sim_runner::tick`、render frame profile 與可用 Perfetto trace

## 2. Critical Path Diagnostics

- [x] 2.1 將 sim_runner blocking wait、active receive、tick work、snapshot/runtime publish 與 queue backlog/catch-up 拆成可分辨 spans/counters
- [x] 2.2 補低開銷 frame-time histogram / 1% low summary，包含 average FPS、p50/p95/p99 frame time、max frame time、sim TPS 與 queue backlog summary
- [x] 2.3 在 `native.rs` frame path 補足 lockstep drain/forward、snapshot consume、render bridge/entity update、UI/VFX、frame pacing sleep/yield 與 present/engine wait 的 coarse diagnostics
- [x] 2.4 確認 default diagnostics 不產生 per-entity 或 per-frame log spam；deep/per-frame tracing 必須 opt-in

## 3. Receive Loop 與 Backlog 行為

- [x] 3.1 將 blocking wait 改到 `omfx::sim_runner::wait_tick_batch` 或等價 wait/idle span，不再使用 active `receive_tick_batch` 包住 `recv_timeout`
- [x] 3.2 將 active `receive_tick_batch` 移到成功取得 `TickBatchPayload` 後的輕量 bookkeeping 區間
- [x] 3.3 加入 queued batch fast path：queue 有資料時用 non-blocking receive 連續處理，不在 queued batches 之間進入 blocking wait
- [x] 3.4 確認 queued TickBatch 依 tick order 處理，沒有 skip、coalesce、reorder，並保留 timeout/disconnect 既有語意

## 4. Root Cause Fix

- [x] 4.1 用新增 diagnostics 判斷 120 FPS/1% low 未達標主因是 idle span 誤標、sleep/yield overshoot、present/vsync wait、handoff contention、render update hot path、sim backlog 或其他因素
- [x] 4.2 針對最大 contributor 實作最小修正，且不得透過降低 TPS、跳 tick、丟 TickBatch、停更 entity 或降低 unit count 達標
- [x] 4.3 若修正後仍未達 1% low >= 119，重複量測並處理下一個最大 contributor，或記錄環境/工具 blocker 與下一步

  Result: `BuffStore::iter_for` full-table scans were removed, TD leak/game-over log/event spam was suppressed, repeated same-tick sim batch work was gated, and frame pacing was adjusted. Residual p99 frame interval spikes were traced to executor pacing using freshly-reset `previous.elapsed()` instead of accumulated `lag`; using `lag` for the yield/spin decision removed the steady-state ~8.5ms tail. A steady stress run now reports p99/max around 7.72ms with 1% low around 129.5 FPS and healthy sim TPS/queue. A strict 5s process-start window can still include shader/FBX warmup and startup catch-up spikes, so SLO interpretation should use the steady measurement window rather than process startup.

## 5. 驗證

- [x] 5.1 執行 `cargo test --manifest-path omfx/Cargo.toml` 或此 workspace 可用的最接近 omfx test/build command
- [x] 5.2 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor` 或此 repo 慣用 omfx build 驗證
- [x] 5.3 跑 TD_1 或一般 dev smoke，確認 receive wait/active work、frame pacing 與 sim profile diagnostics 正常且低頻
- [x] 5.4 跑 `TD_STRESS_10K` 或最接近 stress release run，確認 average FPS、1% low、sim TPS、queue backlog 與 frame-time histogram；若未達標，回到第 4 節處理下一個 contributor
