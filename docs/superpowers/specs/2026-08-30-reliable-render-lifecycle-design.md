# 可靠 Render Lifecycle 通道設計

## 問題

玩家視角 replica 正確產生 `Hide` 與 `Forget`，但 client runtime 與 omfx 之間把這些一次性事件夾在可合併的 presentation snapshot 裡。runtime 的 latest watch 與 omfx 容量 1 queue 都可能用下一幀覆蓋上一幀，導致 renderer 永遠沒有刪除舊的 replica render identity；敵方英雄反覆進出視野時便會留下分身。

## 核准方案

連續狀態與生命週期事件分流：位置、HP、霧區等資料仍走可覆蓋的 latest snapshot；`Hide`、`Forget` 與 `ResetView` 則走保序、不可丟棄的 critical lifecycle lane。renderer 每幀先排空 lifecycle queue，再套用最新狀態。正確性不依賴 snapshot 差集掃描。

## 資料契約

- `RenderLifecycleBatch` 帶有 `team_id`、`authoritative_tick`、`replica_tick`、`view_epoch` 與一組有序事件。
- `RenderLifecycleEvent` 明確攜帶 `replica_id` 與 `disclosure_epoch`；`Hide` 另帶 remember policy 與 sanitized presentation。
- `ResetView` 清除前一個 view epoch 的 deterministic 與 remembered presentation，供重連或 epoch 切換使用。
- lifecycle event 必須冪等；重複 `Hide`、`Forget` 或 `ResetView` 不得產生重複節點或錯誤。
- latest snapshot 不再承擔 `Hide`／`Forget` 的可靠傳遞責任。

## 資料流

1. Team replica 在完成一個 tick 後產生 presentation source。
2. Client runtime 先取出 lifecycle directives，建立 lifecycle batch 並透過既有 critical FIFO 傳送；一般 snapshot 則透過 latest lane 傳送。
3. omfx IPC worker 將 snapshot 放入容量 1 的 latest queue，將 lifecycle batch 放入獨立 FIFO queue。FIFO 滿載時施加 backpressure，不丟棄舊事件。
4. render thread 先依 sequence 排空 lifecycle queue，再消費最新 snapshot。
5. view epoch 改變或 renderer 建立新視圖時，先套用 `ResetView`，再接受新 epoch 的事件與狀態。

## 錯誤處理

- lifecycle sequence 重複時忽略；跳號視為 protocol error 並記錄，不以 snapshot 猜測遺失內容。
- 舊 view epoch 的事件直接忽略；較新的 epoch 必須先經過 `ResetView`。
- FIFO 關閉或 renderer 中斷時停止該 IPC session；重新連線建立新的 view baseline。
- queue 達容量上限時等待消費者，不得覆蓋或丟棄 lifecycle。

## 測試

所有完整測試集中在實作最後執行。單元測試涵蓋事件 codec、epoch、冪等與 queue backpressure；整合測試刻意讓 renderer 慢於 120 Hz，驗證 snapshot 可被合併但所有 lifecycle 事件仍依序到達；玩家情境測試反覆跨越視野與遮擋邊界，確認同一敵方玩家最多只保留一個 deterministic hero，離開視野後舊 identity 必定移除。

## 自我檢查

本設計沒有依賴完整 snapshot 清除缺席 entity，沒有改變 server authoritative 或 team replica 的可見性規則，也不增加 canonical identity 暴露。事件通道只承擔低頻生命週期資料，連續狀態仍可合併，因此不會把 120 Hz 的全部 presentation 變成不可丟棄流量。
