## ADDED Requirements

### Requirement: 玩家 replica 必須追趕已到達的連續 frame

Client runtime MUST 依 team sequence逐 tick套用所有 authoritative frame，並 MUST 在 inbound queue累積時以有限額 catch-up batch追趕。Catch-up MUST NOT跳過 deterministic step或 hash驗證。

#### Scenario: 短暫 stall 累積 72 frame
- **WHEN** client runtime暫停後累積72個連續 TeamTickFrame
- **THEN** runtime依 sequence逐一套用72個 frame
- **AND** runtime在有限批次間讓出執行權，最後追到最新已收到 tick

#### Scenario: Catch-up 遇到 sequence gap
- **WHEN** catch-up batch下一個 frame sequence不是預期值
- **THEN** runtime停止該批追趕並使用既有 replay流程
- **AND** runtime不得越過缺口套用較新 frame

### Requirement: Catch-up 合併一般 presentation 但保留 critical 事件

Catch-up期間的一般連續 snapshot MUST可以合併為最新狀態；Hide、Forget、ResetView、input-bearing state與 input result MUST維持 FIFO且不得丟失。

#### Scenario: Catch-up 中同時發生 Hide 與 Forget
- **WHEN**一個 catch-up batch內依序產生 Hide與Forget
- **THEN**兩個 lifecycle event依原順序送到 renderer
- **AND**一般位置 snapshot可以只送batch尾端最新狀態

#### Scenario: Catch-up 中套用本機輸入
- **WHEN** batch內某 frame包含本機 accepted input ID
- **THEN**對應 state與 APPLIED result以critical FIFO送出
- **AND**較新的latest snapshot不得越過該結果

### Requirement: Checkpoint 輸出不得阻塞每個 deterministic step

Client runtime MUST使用bounded FIFO將checkpoint write與frame apply解耦。Checkpoint MUST保序且不得靜默丟棄；queue滿載時 MUST backpressure，worker失敗時 MUST安全停止session。

#### Scenario: Checkpoint writer 暫時變慢
- **WHEN** checkpoint writer短暫慢於frame apply
- **THEN** report依序留在bounded queue
- **AND** deterministic frame apply可在queue未滿時繼續追趕

#### Scenario: Checkpoint worker 中斷
- **WHEN** checkpoint writer永久中斷
- **THEN** runtime停止未驗算session並留下明確診斷
- **AND** runtime不得假裝replica仍健康

### Requirement: Replica lag telemetry 不得影響 deterministic state

Runtime MUST記錄received/applied lag、inbound depth、catch-up batch與checkpoint queue資料，且這些diagnostics MUST留在wire edge或runtime外圍。

#### Scenario: Backlog 超過門檻
- **WHEN** latest received tick與last applied tick差距超過設定門檻
- **THEN** runtime輸出包含lag ticks、queue depth與catch-up資料的摘要
- **AND**該資料不進入ECS、script ABI或state hash

### Requirement: Checkpoint coverage 必須符合玩家 session 邊界

三方驗算 MUST要求玩家最後成功 applied tick 以前的每個正式 checkpoint 都有 server expected、server observer 與 external runtime report。玩家 session 結束後 server 繼續產生的 checkpoint MUST NOT被算成該玩家漏報。

#### Scenario: Runtime 正常排空 queue 後離線
- **WHEN**runtime 最後 applied tick 是 N，server 之後繼續執行並產生大於 N 的 checkpoint
- **THEN**驗證只要求 tick 不大於 N 的 checkpoint 完整配對
- **AND**tick 不大於 N 的任何缺 report 仍為 UNVERIFIED
