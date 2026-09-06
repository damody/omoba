## 1. Server late input grace

- [x] 1.1 將 `LATE_INPUT_GRACE_MS` 改為 1000 ms。
- [x] 1.2 保留依 `STEP_FPS` 換算 grace tick 的 helper。
- [x] 1.3 讓 grace 內的 late input 繼續 retarget 到 `current_tick + 1`。
- [x] 1.4 讓超過 grace 的 input 繼續 fail closed。
- [x] 1.5 在 Retargeted log 加入 player、input ID、original、effective、current tick。
- [x] 1.6 在 RejectedLate log 加入 late-by tick 與 grace tick。
- [x] 1.7 確認 secure coordinate input 與一般 InputSubmit 使用同一 grace 規則。
- [x] 1.8 讓 Specs `State::tick` 更新共享 authoritative current tick。
- [x] 1.9 讓 Specs `State::tick` 直接取出當前 tick 的 InputBuffer。
- [x] 1.10 將取出的輸入同時交給 gameplay pending input 與 team accepted-input metadata。
- [x] 1.11 讓 TickBroadcaster 改用獨立 broadcast tick，不再修改 authoritative tick。
- [x] 1.12 將正式 InputBuffer 標為由 authoritative world 擁有，禁止 broadcaster 提前取走輸入。

## 2. Checkpoint 非阻塞輸出

- [x] 2.1 在 `omoba-core` 定義可 clone 的 checkpoint reporter handle。
- [x] 2.2 Reporter handle 只公開送出 `ClientReplicaCheckpointReport` 的方法。
- [x] 2.3 在 client runtime 建立 bounded checkpoint queue。
- [x] 2.4 建立單一 checkpoint writer task，依 FIFO 寫入 KCP。
- [x] 2.5 將每個 frame 的 checkpoint 改為 enqueue，不在 frame apply 內等待 KCP write。
- [x] 2.6 Queue 滿載時施加 backpressure，不丟 checkpoint。
- [x] 2.7 Writer 失敗時通知主迴圈安全停止 session。
- [x] 2.8 Shutdown 前關閉 queue 並等待 writer 結束。

## 3. Replica backlog 追趕

- [x] 3.1 新增 received tick、applied tick 與 inbound queue depth 的 lag tracker。
- [x] 3.2 每次收到 TeamTickFrame 時更新 latest received tick。
- [x] 3.2a 玩家 input target 使用KCP reader已解碼的latest server tick，避免queue backlog產生陳舊target。
- [x] 3.3 每次成功套用 frame 時更新 last applied tick。
- [x] 3.4 將連續 frame 的追趕上限設為每批最多 32 frame。
- [x] 3.5 將單批追趕時間上限設為 4 ms。
- [x] 3.6 Catch-up 仍逐一呼叫 deterministic Specs frame apply。
- [x] 3.7 Catch-up 遇到 sequence gap 時停止並沿用 replay。
- [x] 3.8 Catch-up 遇到 unsafe frame 或 hash mismatch 時停止並沿用 repair／rebase。
- [x] 3.9 Catch-up 批次之間讓 renderer input、shutdown 與 rebase 先取得執行機會。
- [x] 3.10 讓 Team 1／Team 2 observer 在第一個 projected frame 前各自套用 bootstrap。

## 4. Presentation 與 input 正確性

- [x] 4.1 Backlog 存在時只合併一般連續 presentation snapshot。
- [x] 4.2 Catch-up 中的 Hide directives 依原順序送入 lifecycle FIFO。
- [x] 4.3 Catch-up 中的 Forget directives 依原順序送入 lifecycle FIFO。
- [x] 4.4 `ResetView` 繼續使用 critical FIFO。
- [x] 4.5 本機 input-bearing state 不得被一般 snapshot 越過。
- [x] 4.6 `APPLIED_TO_PRESENTATION` result 不得被一般 snapshot 越過。
- [x] 4.7 Transport `FORWARDED` 與 authoritative applied 診斷保持可區分。

## 5. Lag telemetry

- [x] 5.1 新增 replica lag tick 的摘要 log。
- [x] 5.2 新增 inbound queue depth 的摘要 log。
- [x] 5.3 新增 catch-up batch frame count 與耗時。
- [x] 5.4 新增 checkpoint queue depth 與滿載 warning。
- [x] 5.5 正常無 backlog 時禁止每 tick warning spam。
- [x] 5.6 確認所有 telemetry 都不進 ECS、script ABI、outcome 或 state hash。
- [x] 5.7 更新雙玩家 Lua runner，保留足以驗證 lag 的 log，避免無限制 trace 負載。

## 6. 最後單元測試

- [x] 6.1 新增 60 Hz grace 等於約 60 tick 的測試。
- [x] 6.2 新增 90 Hz grace 等於約 90 tick 的測試。
- [x] 6.3 新增 120 Hz grace 等於約 120 tick 的測試。
- [x] 6.4 新增 grace 邊界內 retarget 測試。
- [x] 6.5 新增 grace 邊界外 reject 測試。
- [x] 6.6 新增 checkpoint FIFO 保序測試。
- [x] 6.7 新增 checkpoint queue backpressure 測試。
- [x] 6.8 新增 checkpoint writer failure 停止測試。
- [x] 6.9 新增 10 frame backlog 追趕測試。
- [x] 6.10 新增 72 frame backlog 追趕測試。
- [x] 6.11 新增 120 frame backlog 追趕測試。
- [x] 6.12 新增 catch-up lifecycle 不遺失測試。
- [x] 6.13 新增 catch-up input result 保序測試。

## 7. 最後整合與完整檢查

- [x] 7.1 執行 `omb` input buffer 與 transport 相關測試。
- [x] 7.2 執行 `omoba-core` 測試。
- [x] 7.3 執行 `omoba-client-runtime` 測試。
- [x] 7.4 執行 omfx presentation 與 input latency 測試。
- [x] 7.5 執行 server、runtime、executor release build。
- [x] 7.6 執行人工 0.5 秒 stall 的雙玩家恢復測試。
- [x] 7.7 執行至少 3 分鐘的 120 Hz 雙玩家週期輸入測試。
- [x] 7.8 確認測試期間沒有永久 `RejectedLate`、英雄分身或視野外資訊洩漏。
- [x] 7.9 檢查兩隊 server／client／observer replica hash 驗算一致。
- [x] 7.10 檢查 git diff，確認沒有無關修改或建置產物進入版控。
- [x] 7.11 確認 OpenSpec tasks 全部完成；任一檢查失敗時回到對應步驟修正並重測。
- [x] 7.12 確認 checkpoint 比較只涵蓋 session 最後 applied tick，且 session 內缺 report 仍會失敗。
