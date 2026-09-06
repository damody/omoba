## Context

目前 client runtime 每個 presentation tick 產生一份 `FilteredRenderSnapshot`，其中同時包含可覆蓋的連續狀態與一次性的 `RenderMemoryDirective`。runtime 以 `watch::send_replace` 發送 latest snapshot，omfx IPC worker 又使用容量 1 queue 丟棄舊 snapshot。這對位置等狀態合理，卻會讓 `Hide`／`Forget` 在 render thread 套用前永久消失。實際 trace 已確認 server、team replica、runtime 與 renderer IPC 都收到 `Forget`，最後是在 omfx latest queue 被下一幀覆蓋。

這項變更跨越 protobuf、client runtime 與 omfx，但不改 server authoritative、team replica 或玩家可見性計算。

## Goals / Non-Goals

**Goals:**

- lifecycle event 從產生到 render bridge 全程保序且不可丟棄。
- latest snapshot 仍可合併，維持 120 Hz presentation 的低延遲與固定記憶體使用。
- 保留 `disclosure_epoch`，並以 `view_epoch` 隔離重連或切換視圖前後的事件。
- lifecycle event 可重複套用，不會產生分身或重複 remembered presentation。
- renderer 慢速消費時只對低頻 lifecycle lane 施加 backpressure。

**Non-Goals:**

- 不以 snapshot 中缺席的 entity 推導刪除。
- 不改變 server 到 team replica 的投影與可見性規則。
- 不引入 canonical entity ID 到玩家端 IPC。
- 不保證 renderer process 結束後延續舊畫面狀態；新 process 由新 view baseline 開始。

## Decisions

### 使用獨立 `RenderLifecycleBatch`

在 `RendererIpcEnvelope` 新增 lifecycle payload。每個 batch 帶 `team_id`、ticks、`view_epoch`，每個事件帶 `replica_id` 與 `disclosure_epoch`。相較把 directives 繼續塞在 snapshot，這能讓 transport 明確區分可覆蓋狀態與不可丟事件，也能在 log 與測試中直接驗證排序。

### lifecycle 走既有 critical FIFO

runtime 已有 bounded `mpsc` critical lane，且 IPC writer 優先排空 critical 資料。含 lifecycle 的 batch 一律以 `publish_critical().await` 發送；位置等 snapshot 繼續使用 `publish_latest`。若 lifecycle queue 滿載，runtime 等待 renderer 消費，不丟事件。

考慮過把舊 directives 合併進下一份 snapshot，但這會需要處理 Hide→Reveal、epoch 與 remembered payload 的覆寫規則，本質上仍把事件偽裝成狀態，因此不採用。

### omfx 使用 snapshot 與 lifecycle 兩個接收 queue

snapshot queue 保持容量 1 與 replace-latest；lifecycle queue 使用 bounded FIFO。IPC worker 對 lifecycle 使用 blocking `send`，因此不能被後續 snapshot 越過或覆蓋。render thread 每幀先排空 lifecycle，再讀 latest snapshot。

### bridge 提供獨立且冪等的 lifecycle 套用入口

`FilteredRenderBridge` 新增 lifecycle batch/event 套用方法。`Forget` 對不存在的 ID 仍成功；`Hide` 以 `(replica_id, disclosure_epoch)` upsert remembered presentation；`ResetView` 清除舊 deterministic、remembered 與 retired 狀態。snapshot 的一般 entity upsert 流程不負責推導移除。

`Hide` 結束一個 disclosure identity 後，bridge 必須為該 replica ID 記錄已關閉 disclosure epoch 的 high-water mark。相同 canonical entity 重新 Reveal 時可以沿用 replica ID，但 disclosure epoch 必須增加；新 epoch 會清除同 replica ID 的舊 remembered presentation。`Forget` 則永久退休整個 replica ID，Team replica 保證退休 ID 不會在同一個 view epoch 內重用。因此 lifecycle lane 先抵達、較舊 snapshot lane 後抵達時，bridge 必須忽略已結束的舊 disclosure，不能讓舊 snapshot 復活已隱藏或忘記的單位。High-water mark 每個 replica ID 只占一筆，不會因反覆跨越視野邊界而無限累積。

### epoch 與排序

omfx 只接受目前 view epoch 的 lifecycle。首次連線由 runtime 送 `ResetView` 建立 epoch；較舊事件忽略。lifecycle payload 沿用 envelope sequence，接收端拒絕倒退，但 snapshot sequence 不得使較早收到的 critical lifecycle 被丟棄。

## Risks / Trade-offs

- [renderer 長時間停止消費導致 runtime 等待] → lifecycle lane 容量設為足以吸收短暫卡頓，滿載後採 backpressure；連線中斷則結束 IPC session。
- [critical 與 latest 使用共同 sequence，造成合法交錯被誤判] → 接收端分別追蹤 lifecycle 與 snapshot 的接受進度，view epoch 作為跨流隔離。
- [舊 IPC client 不認得 lifecycle payload] → 同一 monorepo 內同步重建 runtime 與 omfx，protocol version 隨 schema 變更提升。
- [Hide 與隨後 Reveal 很接近] → render thread 先依 FIFO 套用 Hide，再用最新 snapshot upsert Reveal；同一 replica identity 的操作保持冪等。
- [lifecycle 與 snapshot 分屬不同 queue，跨 queue 無全域順序] → Hide 將 `(replica_id, disclosure_epoch)` 寫入 tombstone，Forget 退休整個 replica ID；較晚套用的舊 snapshot 不得重新建立舊 disclosure，新 Reveal 只能使用較高 disclosure epoch。

## Migration Plan

1. 擴充 protobuf 並重新產生 Rust schema。
2. runtime 將 directives 從 snapshot 拆成 lifecycle batch，啟動或 view epoch 改變時發送 `ResetView`。
3. omfx 增加 lifecycle FIFO 與 bridge 套用入口。
4. 移除 `removed_render_ids` 作為可靠刪除來源；欄位暫時保留相容讀取但新 runtime 不再填入。
5. 完成測試後同步部署 runtime 與 omfx；回滾時兩者一併回滾。

## Open Questions

無。queue 容量與 protocol version 依現有常數及測試結果選定，不影響外部玩法契約。
