## Context

實機 `run_2player.bat` log 顯示 Player 1 前 5 筆、Player 2 前 46 筆輸入成功後，後續 `MoveTo` 全部被 server 判定為 late。第一次拒絕已落後 11～14 tick，之後擴大到 71～72 tick；現行 grace 只有 64 ms，也就是 120 Hz 下 8 tick。

Client runtime 原本以已處理的 `TeamTickFrame.server_tick + 2` 排程輸入，並在同一個 frame apply 路徑逐 tick等待 checkpoint KCP write、presentation extraction 與 critical IPC。修正後KCP reader另外發布已解碼的最新server tick，避免尚未套用的queue backlog讓新input沿用舊tick。短暫 stall 產生的 inbound backlog沒有明確追趕模式；replay／rebase 又只處理 sequence 或 hash 錯誤，因此時間落後不會自動修復。

實作期的週期輸入測試另外找到真正會讓輸入永久失效的 server bug：KCP `TickBroadcaster` 與 Specs authoritative world 各自維護一個 120 Hz tick。前者用來判斷 input 是否過期，後者才是產生 TeamTickFrame 與套用 input 的時鐘。兩者在負載下漂移後，即使封包沒有丟失，合法 input 也會被另一個較快的時鐘判為 late。修正後只有 Specs `State::tick` 能推進 authoritative input tick 與取出該 tick 的 InputBuffer；broadcaster 只負責網路廣播，不得再推進或消費 authoritative input。

## Goals / Non-Goals

**Goals:**

- 依選定方案 A 將 late grace 改為 1 秒 wall-clock budget。
- 短暫落後的輸入由 server retarget，不讓英雄永久失去移動能力。
- Runtime 能逐 tick追趕 backlog，且 catch-up 不丟 deterministic frame 或 lifecycle event。
- Checkpoint 與 presentation 輸出不再阻塞每一個 frame apply。
- 所有 lag、retarget 與 rejection 都能從測試和 log 證明。
- Server 只保留一個 authoritative input tick，transport broadcaster 不得建立第二個判定時鐘。
- Team 1／Team 2 observer 必須在第一個 projected frame 前各自完成 bootstrap。

**Non-Goals:**

- 不加入 client movement prediction。
- 不讓 client 決定實際 input tick。
- 不接受超過 1 秒的任意陳舊輸入。
- 不修改 server authoritative、team projection 或 fog 安全邊界。

## Decisions

### Late grace 使用 1000 ms，而不是固定 tick 常數

保留 `late_input_grace_ticks(step_fps)`，把來源常數改為 1000 ms。60、90、120 Hz 因此各自換算成一秒，測試邊界為 grace 內 retarget、grace 外 reject。

只調大 grace 可以立即避免目前 11～72 tick 的輸入失效，但無法消除 backlog，所以仍必須完成下面的 runtime 修正。

### Frame apply 與 checkpoint write 分離

Deterministic frame apply 維持單執行緒、嚴格 sequence，但 checkpoint report 交給 bounded FIFO worker。Worker 依序寫 KCP；queue 滿載才 backpressure，worker 中斷則安全終止 session。相較直接丟 checkpoint，這保留 replica 驗算完整性；相較每 tick await，可釋放 catch-up 吞吐量。

### 使用有限額 catch-up batch

Runtime 從 inbound queue 非阻塞收集已到達的連續 TeamTickFrame，最多處理 32 frame 或 4 ms後讓出。每一 frame 仍執行 Specs step與 hash 驗證，但一般 presentation snapshot 只保留 batch 最新狀態。

Hide／Forget directives 由 replica runtime 累積，在 batch 尾端依原順序送 lifecycle critical FIFO。若 batch 包含本機 input，對應 input-bearing state 與 applied result保持 critical FIFO。Renderer input、shutdown、rebase 與 secure result優先於下一批 catch-up，避免追趕餓死互動。

### 明確記錄 input submit outcome

Server 對 Accepted、Retargeted、RejectedLate 記錄結構化欄位與計數。Retargeted 必須包含 original、effective、current tick；RejectedLate 必須包含 late-by。Runtime／renderer 的 transport success仍只表示 forwarded，authoritative applied只能由後續 frame證明。

本 change 不要求 client自動重送技能或物品；方案 A 的 1 秒 retarget與 catch-up已涵蓋本次移動失效。超過 1 秒維持 fail closed。

### Telemetry 不進 deterministic state

Queue depth、batch size、lag tick、retarget count與耗時只放在 transport/runtime diagnostics，不加入 ECS component、resource、script ABI、outcome或 state hash。

### Specs world 是唯一 authoritative input clock

`State::tick` 每步先把共享 `LockstepState.current_tick` 設成當前 Specs tick，再直接從共享 `InputBuffer` 取出這個 tick 的輸入，交給 `PendingPlayerInputs`、team projection 與 accepted-input metadata。這讓 late 判定、實際套用與對玩家發布的 tick 使用同一個來源。

`TickBroadcaster` 保留自己的 `broadcast_tick`，只用來維持舊 transport 訊息的發送節奏。正式 builder 將 InputBuffer 標為由 authoritative world 擁有；broadcaster 即使執行，也不能取走或提前消耗遊戲輸入。

### 固定兩隊 observer 在第一個 frame 前 bootstrap

Server 啟動時即建立 Team 1 與 Team 2 observer。各 observer 在收到第一個 projected frame 前先套用該隊 bootstrap；不能等玩家連線後才建立，否則 Team 2 可能先看到 sequence 1 frame，卻缺少初始 world，造成假的 coverage gap 與 rebase。

### 驗證範圍以玩家 session 最後 applied tick 為界

三方 checkpoint 比較只要求 `replica_tick <= session_last_applied_tick` 的 server checkpoint 完整配對。玩家正常離線後 server 繼續產生的 checkpoint 不屬於該 session，不能誤判為缺 report；但 session 已套用範圍內缺任一方仍為 `UNVERIFIED`。

## Risks / Trade-offs

- [1 秒內的舊 MoveTo 較晚生效] → KCP 是可靠有序通道，同 tick多筆狀態型指令仍依到達順序套用；server保留 input ID 與 tick診斷。
- [Checkpoint worker queue 滿載] → bounded FIFO施加 backpressure並告警，不丟驗算資料。
- [Catch-up batch讓 renderer input等待] → 每批限制32 frame或4 ms，下一批前先回到 select loop。
- [合併 snapshot漏掉 lifecycle] → directives獨立累積並走 critical FIFO；以 Hide／Forget反覆跨界測試驗證。
- [Trace log本身造成負載] → 正常只記門檻事件與摘要，詳細 trace由環境變數啟用。

## Migration Plan

1. 先更新 grace常數、邊界測試與 outcome diagnostics。
2. 加入 checkpoint FIFO worker與故障處理。
3. 重構 runtime frame處理為有限額 catch-up batch。
4. 補上 lifecycle、input result與lag telemetry測試。
5. 同步重建 server、runtime、renderer，執行至少3分鐘雙玩家 release驗證，並人工暫停 replica 500 ms。
6. 若需回滾，server與runtime一併回滾；wire schema未變時 renderer可維持同版。

## Open Questions

無。Grace依使用者選擇固定為1秒；catch-up初始上限採32 frame／4 ms，後續只可依量測調整，不改外部契約。
