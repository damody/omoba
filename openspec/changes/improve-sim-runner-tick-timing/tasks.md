## 1. Server Cadence Config

- [x] 1.1 在 `omb/game.toml` 的 `[server]` 區段新增預設 `STEP_FPS = 120`，並確認 `game_stress.toml` 或 stress swap config 需要時同步合理預設。
- [x] 1.2 更新 backend config loader，讀取並驗證 `STEP_FPS` 只接受 `120`、`90`、`60`，且不新增 FPS 環境變數 override。
- [x] 1.3 將 `omb` authoritative clock、`State::tick()` dt 與 `TickBatch` broadcaster cadence 改為使用 `STEP_FPS` 推導的 runtime timing。
- [x] 1.4 將 state hash、snapshot、visibility diff、diagnostic sampling 與 tick retention windows 改為依 configured step FPS 保持相同 wall-clock duration。

## 2. Lockstep Metadata And Client Obedience

- [x] 2.1 更新 `proto/game.proto` 的 `GameStart` 或等價 lockstep start metadata，加入 server 宣告的 `step_fps` 或 `tick_period_us`。
- [x] 2.2 更新 server `GameStart` 發送端，把 `omb/game.toml [server].STEP_FPS` 的實際值送給 client。
- [x] 2.3 更新 `omfx` lockstep client 與 sim_runner 啟動流程，將 server 宣告 cadence 傳入 local replica timing。
- [x] 2.4 確認 `omfx/game.toml` 不提供 simulation step FPS override；client simulation dt、HUD tick-to-time 與 wait deadlines 都服從 server metadata。

## 3. Frontend Game Config Ownership

- [x] 3.1 新增 `D:/omoba/omfx/game.toml`，只保留 frontend-local/client-owned 設定，不複製 `STEP_FPS`、vision/collision 或其他 server-only 設定。
- [x] 3.2 將 native `omfx` 預設 config path 從 repo-local shared/script config 改為 frontend package 內的 `game.toml`，避免以環境變數作為本 change 的 FPS 設定來源。
- [x] 3.3 保留或調整 dev launcher，確保 backend 使用 `omb/game.toml`，frontend 使用 `omfx/game.toml`，兩者在分離啟動時各自持有 config。

## 4. Sim Runner Wait Loop

- [x] 4.1 檢查 `omfx/game/src/sim_runner.rs` 目前的 `wait_tick_batch` trace span 與 channel wait flow，確認最小可修改範圍。
- [x] 4.2 新增最小必要 helper，使用 server cadence 計算 tick interval、remaining budget、sleep duration 與約 2ms precision yield window 的分段結果。
- [x] 4.3 將 channel 空時的長時間 blocking wait 改為 deadline-based wait：先短 sleep 到 `deadline - precision_window`，再進入 yield loop。
- [x] 4.4 在 sleep 前、sleep 後與 yield loop 內重複檢查 `tick_input_rx`，確保已到達的 `TickBatchPayload` 不被 pacing 額外延遲。
- [x] 4.5 保留 `Disconnected` exit path、starvation diagnostic 與既有 profile counters 的語意，避免 yield loop 產生高頻 log。

## 5. Tests And Verification

- [x] 5.1 新增 backend config tests，覆蓋 `STEP_FPS = 120`、`90`、`60` 可用，以及不支援值會明確失敗。
- [x] 5.2 新增或調整 protocol/client tests，確認 `GameStart` cadence metadata 由 server 傳到 omfx，且 omfx local timing 使用該值。
- [x] 5.3 新增或調整 sim_runner 單元測試，覆蓋 120 FPS 約 8.33ms、90 FPS 約 11.11ms、60 FPS 約 16.67ms 下保留約 2ms precision window 的 sleep 計算。
- [x] 5.4 測試 remaining budget 小於或等於 precision window 時不進行長 sleep，改由 yield wait path 處理。
- [x] 5.5 測試 channel 已有 `TickBatchPayload` 時 wait path 不額外 sleep 一個 lockstep interval。
- [x] 5.6 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`。
- [x] 5.7 執行 `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p game` 或最接近的 `omfx` 測試目標；若該 package 名稱不同，先用 `cargo metadata` 或 workspace manifest 確認正確命令。
- [x] 5.8 執行 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor`，確認 native frontend build 通過。
