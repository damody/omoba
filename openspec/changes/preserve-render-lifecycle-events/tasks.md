## 1. IPC 資料契約

- [x] 1.1 在 `proto/game.proto` 新增 `RenderLifecycleBatch` message。
- [x] 1.2 在 `proto/game.proto` 新增 `RenderLifecycleEvent` 與事件種類。
- [x] 1.3 在 lifecycle event 保留 `replica_id`、`disclosure_epoch` 與 Hide payload。
- [x] 1.4 在 lifecycle batch 加入 team、tick 與 `view_epoch` 欄位。
- [x] 1.5 將 lifecycle batch 加入 `RendererIpcEnvelope` oneof。
- [x] 1.6 提升 renderer IPC protocol version。
- [x] 1.7 重新產生並確認 `omoba-core/src/generated/game.rs`。

## 2. Client runtime 事件分流

- [x] 2.1 新增由 `RenderMemoryDirective` 轉成 lifecycle event 的小型轉換函式。
- [x] 2.2 新增建立 lifecycle batch envelope 的函式。
- [x] 2.3 讓一般 snapshot envelope 不再包含 `removed_render_ids` 或 remembered lifecycle edge。
- [x] 2.4 在 presentation source 抽取後分離 snapshot 與 lifecycle directives。
- [x] 2.5 lifecycle batch 一律呼叫 `publish_critical().await`。
- [x] 2.6 一般狀態 snapshot 維持 `publish_latest()`。
- [x] 2.7 保持 input-bearing snapshot 與 input result 的既有 FIFO 順序。
- [x] 2.8 runtime 建立 renderer view 時送出對應 epoch 的 `ResetView`。
- [x] 2.9 更新 trace，分別顯示 snapshot lane 與 lifecycle lane。

## 3. omfx IPC 接收分流

- [x] 3.1 在 `RendererPresentationHandle` 新增 lifecycle receiver。
- [x] 3.2 在 presentation IPC worker 新增 bounded lifecycle sender。
- [x] 3.3 snapshot payload 繼續使用容量 1 的 replace-latest queue。
- [x] 3.4 lifecycle payload 使用 FIFO `send`，不得呼叫 replace 或丟棄舊事件。
- [x] 3.5 lifecycle 轉換時完整保留 `disclosure_epoch` 與 Hide payload。
- [x] 3.6 snapshot sequence 與 lifecycle sequence 分別追蹤，避免合法交錯互相淘汰。
- [x] 3.7 更新 IPC trace，能辨識 lifecycle enqueue 與 apply sequence。

## 4. Render bridge 套用事件

- [x] 4.1 在 omfx 端定義可直接套用的 lifecycle batch 型別。
- [x] 4.2 為 `FilteredRenderBridge` 新增套用單一 lifecycle event 的方法。
- [x] 4.3 實作冪等 `Forget`，移除 deterministic、remembered 與 identity。
- [x] 4.4 實作冪等 `Hide`，移除 deterministic 並 upsert remembered presentation。
- [x] 4.5 實作 `ResetView`，清除舊 epoch 的 bridge 狀態。
- [x] 4.6 記錄目前接受的 `view_epoch` 並忽略舊 epoch 事件。
- [x] 4.7 native render loop 每幀先排空 lifecycle queue，再處理 latest snapshot。
- [x] 4.8 確認 snapshot entity upsert 會移除相同 identity 的 remembered presentation。
- [x] 4.9 移除把 snapshot 缺席 entity 當成刪除依據的需求與程式分支。

## 5. 最後測試與完整檢查

- [x] 5.1 新增 protobuf lifecycle round-trip 測試。
- [x] 5.2 新增非零 `disclosure_epoch` 不遺失測試。
- [x] 5.3 新增 omfx lifecycle FIFO 在 snapshot replace 時不丟事件測試。
- [x] 5.4 新增 lifecycle queue backpressure 測試。
- [x] 5.5 新增重複 Hide 與 Forget 的 bridge 冪等測試。
- [x] 5.6 新增 `ResetView` 與舊 epoch 忽略測試。
- [x] 5.7 新增 Hide 後 Reveal 只保留一個 deterministic identity 的測試。
- [x] 5.8 新增慢速 renderer 反覆跨越視野邊界的整合測試。
- [x] 5.9 執行 `omoba-core`、`omoba-client-runtime` 與 omfx 相關單元測試。
- [x] 5.10 執行 release build，確認三個 process 的 IPC schema 相容。
- [x] 5.11 執行雙玩家視野情境，檢查同一玩家不會留下多個英雄 identity。
- [x] 5.12 檢查 git diff，確認未修改無關檔案且沒有建置產物進入版控。

## 6. 修正跨 queue 的舊 snapshot 復活

- [x] 6.1 確認 `Hide` 提高 disclosure epoch，`Forget` 才永久退休 `replica_id`。
- [x] 6.2 讓 `Hide` 將舊 `(replica_id, disclosure_epoch)` 寫入 tombstone。
- [x] 6.3 保留 `Forget` 既有的 tombstone 行為。
- [x] 6.4 snapshot 套用 entity 前先檢查 retired ID 與 disclosure tombstone。
- [x] 6.5 snapshot 遇到已結束的 disclosure identity 時略過 deterministic upsert。
- [x] 6.6 新 Reveal 的較高 epoch 可建立，並清除同 replica ID 的舊 remembered presentation。
- [x] 6.7 `ResetView` 繼續清空上一個 view epoch 的 tombstone。

## 7. 最後測試與完整檢查

- [x] 7.1 新增 `Forget` 先到、舊 snapshot 後到的回歸測試。
- [x] 7.2 新增 `Hide` 先到、舊 snapshot 後到的回歸測試。
- [x] 7.3 新增舊 disclosure 被擋住、較高 epoch Reveal 可建立的回歸測試。
- [x] 7.4 修正慢速 renderer 測試，使它涵蓋跨 queue 反向抵達順序。
- [x] 7.5 執行 omfx 單元測試。
- [x] 7.6 執行 release build，更新 `run_2player.bat` 使用的執行檔。
- [x] 7.7 執行 release 雙玩家 smoke，確認 server、兩隊 lockstep、資料隔離與程序生命週期；跨 queue 分身由 7.4 的確定性整合測試驗證。
- [x] 7.8 檢查 git diff 與建置產物，確認只保留必要原始碼變更。

### 驗證紀錄

- omfx：141 個測試通過，0 個失敗。
- release executor：建置成功，`run_2player.bat` 會使用更新後的執行檔。
- release 雙玩家 smoke：兩隊三方 lockstep、對手 sentinel 隔離、程序生命週期皆通過。
- smoke 的自動 MoveTo 有送達 runtime，但驗證器未觀察到 applied marker，因此該次執行沒有產生 Hide／Forget，不把它當作視野邊界分身的證明；視野邊界由「lifecycle 先到、舊 snapshot 後到」的確定性整合測試直接覆蓋。
