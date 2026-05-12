## 1. Baseline 與呼叫路徑確認

- [x] 1.1 確認 `omoba_core::lockstep_timing` 的 shared cadence 為 120 TPS，且 omfx render pacing 與 sim_runner diagnostics 都從 shared helper 取得目標值
- [x] 1.2 追蹤 `omfx/game/src/sim_runner.rs` 中 `extract_snapshot` 呼叫路徑，確認目前 runtime TickBatch loop 仍在呼叫它
- [x] 1.3 對照 `SimWorldSnapshot` consumers，列出 runtime lightweight publication 必須更新的動態欄位

## 2. Init Seed 與 Runtime Lightweight Publication

- [x] 2.1 新增 sim_runner 初始化/seed 路徑，直接建立 static render data，不呼叫 `extract_snapshot`
- [x] 2.2 從 per-tick TickBatch loop 移除 `extract_snapshot` 呼叫與依賴 full snapshot 的 runtime scan log
- [x] 2.3 新增 runtime lightweight update helper，更新 tick、entities、round/lives、removed ids、FX、applied input metadata 與 DEV Lua reload fields，但保留 init seed 的 static Arc/Vec data
- [x] 2.4 確認 removed ids 與 render FX queues 仍被正確 drain/retained/dedup，不依賴 `extract_snapshot`
- [x] 2.5 加入 guard、測試或 diagnostic，確認 sim_runner 不會呼叫 `extract_snapshot`

## 3. Render Stale Runtime Tick 節流

- [x] 3.1 確認 `omfx/game/src/native.rs` 的 lockstep event draining、input submission、auto smoke 與 sim_runner forwarding 在任何 render skip 前完成
- [x] 3.2 對相同 runtime render tick 跳過或延後高成本 tick-driven scene/entity/UI 更新
- [x] 3.3 確認新 runtime tick 仍會在下一個 eligible update 立即消費，不被 stale tick skip 多延遲一個 frame interval
- [x] 3.4 將 stale runtime tick reuse/skip counters 接到既有 `omfx_render` profile window

## 4. Diagnostics 與 Log Cleanup

- [x] 4.1 移除或降級 `[sim_runner]` / `[mirror-snapshot]` 這類 full snapshot scan log
- [x] 4.2 新增低頻 sim_runner profile，顯示 processed ticks、runtime lightweight publishes、latest tick 與 queue starvation 狀態
- [x] 4.3 確認 diagnostics 不產生 per tick、per snapshot 或 per entity log spam

## 5. 驗證

- [x] 5.1 執行 `cargo test --manifest-path omoba-core/Cargo.toml` 或最接近可用的 omoba-core test command
- [x] 5.2 執行 omfx/omb 相關 build，確認 Rust 1.91.0 toolchain 下通過
- [x] 5.3 跑 TD_1 smoke 至少 5 秒，確認 runtime render state 正常更新，且沒有 sim_runner `extract_snapshot` 呼叫/log
- [x] 5.4 跑 TD_STRESS 或 release smoke，確認 log 不再出現反覆 `[sim_runner]` / `[mirror-snapshot]` full snapshot 掃描，並檢查 FPS/TPS bottleneck
