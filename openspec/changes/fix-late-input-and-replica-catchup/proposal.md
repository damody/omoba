## Why

本機雙玩家在長時間 120 Hz 執行後，client replica tick 會落後 authoritative server，超過現行 64 ms late grace 後，所有新 `MoveTo` 都被永久拒絕。除了 client backlog，實作驗證也確認 server 原本同時存在 transport broadcaster 與 Specs world 兩個 tick 時鐘，兩者漂移會把合法輸入誤判為 late。必須依選定方案 A 將 grace 擴為 1 秒、統一 authoritative input clock，並讓 client replica 在短暫 stall 後追上。

## What Changes

- 將 server late input grace 改為固定 1 秒的 wall-clock 語意，依實際 tick rate 換算。
- 在 grace 內由 server retarget 到下一個 authoritative tick，超過 1 秒才拒絕。
- 讓 Accepted、Retargeted、RejectedLate 與 late-by tick 可被可靠診斷。
- 將 client deterministic frame apply 與 checkpoint／presentation 輸出解耦。
- 新增有限額 catch-up batch：逐 tick 執行 simulation，但合併一般 presentation snapshot。
- 保留 lifecycle 與 input result 的 critical FIFO，不因 catch-up 遺失或亂序。
- 新增 replica backlog、catch-up、checkpoint queue 與 late input telemetry。
- 讓 Specs world 成為唯一 authoritative input clock，transport broadcaster 不再推進 tick 或消費正式 InputBuffer。
- 讓固定 Team 1／Team 2 observer 在第一個 projected frame 前先完成各自 bootstrap。
- 讓三方驗算以玩家 session 最後 applied tick 為邊界，離線後的 server checkpoint 不誤報缺失。
- 新增長時間雙玩家與人工 stall 測試，確認輸入會恢復且兩隊 hash、視野隔離不變。

## Capabilities

### New Capabilities

- `replica-backlog-catchup`: 定義玩家 replica 的 backlog 偵測、有限額追趕、非阻塞 checkpoint 與 presentation 合併規則。

### Modified Capabilities

- `player-input-routing`: late coordinate input 在 1 秒內由 server retarget，並保留可觀察的處理結果。
- `lockstep-cadence`: 120 Hz input late grace 改為 1 秒 wall-clock budget，且短暫 stall 後必須恢復 cadence。
- `input-latency-metric`: 增加 retarget／rejection、replica backlog 與 catch-up phase 診斷。

## Impact

影響 `omb` authoritative tick/input buffer 與 KCP transport、`omoba-core` lockstep wire-edge schema／client、`omoba-client-runtime` 主迴圈、checkpoint 與 presentation bridge，以及雙玩家 Lua 驗證流程。Script ABI 與 team visibility 安全邊界不變。
