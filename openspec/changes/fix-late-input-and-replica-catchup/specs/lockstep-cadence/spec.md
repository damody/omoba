## MODIFIED Requirements

### Requirement: input lookahead uses 120Hz budget

omfx與external client runtime input submission SHALL保留 `INPUT_LOOKAHEAD_TICKS = 2` 作為預設，target tick SHALL由KCP reader已解碼的latest authoritative frame tick加上lookahead計算；若reader尚未收到新frame，才使用latest applied tick。在 `LOCKSTEP_TPS = 120` 下，2 tick lookahead SHALL代表約16.7 ms排程緩衝。omb SHALL將late input grace定義為1000 ms的wall-clock budget並依active step FPS換算tick；grace內的late input SHALL retarget到 `current_tick + 1`，超過grace才 SHALL拒絕。

#### Scenario: target tick is two 120Hz ticks ahead
- **WHEN** client的KCP reader已解碼latest lockstep tick N並submit input
- **THEN** `InputSubmit.target_tick == N + 2`
- **AND**預估固定lookahead budget約為16.7 ms

#### Scenario: 一秒內 late input 會 retarget
- **WHEN**120 Hz server收到落後不超過120 tick的late input
- **THEN**server將輸入排到 `current_tick + 1`
- **AND**記錄original tick、effective tick、current tick與late-by ticks

#### Scenario: 超過一秒的 late input 會拒絕
- **WHEN**120 Hz server收到落後超過120 tick的input
- **THEN**server拒絕該input且不放入InputBuffer
- **AND**拒絕可與transport starvation或client pending backlog區分

### Requirement: Server 只能有一個 authoritative input tick

Specs authoritative world SHALL是 input late 判定、InputBuffer 取出與 TeamTickFrame 發布所使用的唯一 authoritative tick。Transport TickBroadcaster MUST NOT推進 authoritative tick，也 MUST NOT消費由 Specs world 擁有的 InputBuffer。

#### Scenario: Transport broadcaster 與 Specs loop 同時執行
- **WHEN**TickBroadcaster 的 wall-clock 排程比 Specs world 快或慢
- **THEN**`LockstepState.current_tick` 仍只由 Specs `State::tick` 更新
- **AND**輸入只會在相同的 Specs tick 被取出並套用

#### Scenario: 週期輸入持續三分鐘
- **WHEN**兩隊 client 在120 Hz設定下持續送出週期 MoveTo，且 replica 曾暫停500 ms
- **THEN**後續輸入不會因第二個 tick 時鐘漂移而永久 RejectedLate
- **AND**兩隊 replica 能回到 received tick 等於 applied tick

### Requirement: 固定兩隊 observer 必須先 bootstrap

Server SHALL只建立 Team 1 與 Team 2 observer，且每個 observer MUST在第一個 projected frame 前先套用該隊 bootstrap。

#### Scenario: Team 2 尚未有玩家連線
- **WHEN**server 已開始產生 Team 2 projected frame
- **THEN**Team 2 observer 已有完整的 team bootstrap world
- **AND**第一個 frame 不會造成假的 coverage gap 或 rebase
