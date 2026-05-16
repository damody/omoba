## Context

omb 目前有 lockstep input path、KCP/transport session path 與 snapshot store。`SnapshotStore` 由 simulation tick 週期性更新，transport 可在收到 state/snapshot 類 request 時回傳目前 snapshot。這讓 snapshot request 容易被拿來補 gameplay input 後的狀態確認，但玩家進入遊戲後的權威流程應該是 lockstep input 進入 sim，再由既有 tick batches、outcome queues 與 render-facing state 推進。

這次變更的重點不是移除 snapshot 資料結構，而是收斂「誰可以觸發 snapshot send」。snapshot 仍然可作為新外部連線加入時的初始化 state，但 gameplay input 指令不得觸發 snapshot response。

## Goals / Non-Goals

**Goals:**

- 將 snapshot send 的觸發點限制在外部 client/session 連線初始化流程。
- 讓 `PlayerCommand`、lockstep `InputSubmit` 或其他 gameplay input path 不再呼叫 snapshot request handling。
- 保留新玩家連進來時取得初始化 snapshot 的能力。
- 增加測試或 guard，防止 input handling path 重新引入 snapshot send。

**Non-Goals:**

- 不重寫 `SimWorldSnapshot` schema 或 snapshot extraction 內容。
- 不改變 lockstep `PlayerInput` wire schema。
- 不用 snapshot 當作 input acknowledgement 或成功/失敗回覆。
- 不處理 snapshot extraction 的效能最佳化；本變更只調整觸發時機。

## Decisions

1. 將 snapshot request 視為 connection bootstrap 行為。

   外部 session 建立或訂閱成功時，server 可從 `SnapshotStore` 讀取最近一次 snapshot 並回傳給該 session。這符合「給新玩家連進來」的用途，也避免每次 input 都造成 snapshot traffic。

   Alternative considered: 保留 client 主動 `SnapshotReq`，但在 input 後由前端決定是否呼叫。這會把 lifecycle policy 分散到 client，且無法防止其他 client 或未來 input handler 重新觸發 snapshot。

2. input path 只提交排程後的 gameplay intent，不送 snapshot。

   KCP `InputSubmit`、legacy `PlayerCommand` 與 omb 的 player input tick 應只負責 buffering、validation、logging 與進入 ECS entry points。成功或失敗由後續 authoritative state 觀察，不以 snapshot response 作 ack。

   Alternative considered: input handler 回傳空 snapshot 或 stale snapshot。這仍保留錯誤語意，會讓呼叫端誤以為 input 後 snapshot 是合法 contract。

3. `SnapshotStore` 繼續由 sim tick 更新，但 transport 只有 bootstrap endpoint 讀取並傳送。

   這避免改動 simulation runner 與 snapshot extraction 的既有 contract。即使 store 持續更新，也不代表任何 input command 能觸發 send。

   Alternative considered: 只有連線進來時才即時 extraction。這會在 transport path 中引入 ECS access/timing 問題，也可能破壞現有 read-only extraction invariant。

4. 測試以行為測試加 grep guard 組合。

   行為測試確認 input submit 不產生 snapshot response；grep guard 確認 input command branches 不讀 `SnapshotStore` 或建構 snapshot response。這比只靠單一測試更能防止未來 routing refactor 退化。

## Risks / Trade-offs

- [Risk] 某些 client 目前依賴 input 後 snapshot 立即刷新 UI。→ Mitigation: 實作時檢查 omfx input forwarding 與 pending input diagnostics，改用既有 tick/snapshot publication 或 applied input metadata，而不是 input-triggered snapshot。
- [Risk] 新玩家 bootstrap 如果過早發送，可能拿到空或舊 snapshot。→ Mitigation: bootstrap response 使用 `SnapshotStore` 目前 tick，必要時等待已存在的 latest snapshot 或回傳明確的 no-snapshot-yet 狀態，但不在 input path 補送。
- [Risk] KCP、gRPC 或 legacy command path 觸發點不一致。→ Mitigation: 逐一檢查 transport implementations，將 snapshot send 集中到 connection/subscription bootstrap helper。
- [Trade-off] `SnapshotStore` 仍週期性更新，因此 CPU extraction 成本不會因本變更消失。→ 這次目標是修正 lifecycle 與 wire behavior；效能最佳化另案處理。
