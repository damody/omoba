## ADDED Requirements

### Requirement: snapshot 只由外部連線初始化觸發

omb transport layer SHALL only send lockstep/world snapshot data as part of external client/session bootstrap when a new connection or subscription needs initial authoritative state. Gameplay input paths, including `PlayerCommand`, lockstep `InputSubmit`, player input tick handling, tower actions, ability actions and item actions, MUST NOT trigger snapshot extraction, snapshot request handling or snapshot response sending.

`SnapshotStore` MAY continue to be updated by the simulation tick loop, but reading from it for network delivery SHALL be restricted to connection/subscription bootstrap code paths. Input acknowledgement SHALL remain command acceptance, tick batches, applied input metadata or subsequent authoritative state, not a snapshot response caused by the input command.

#### Scenario: 新外部連線收到初始化 snapshot

- **WHEN** external client/session 建立連線或完成訂閱，且 server 已有可用的 latest snapshot
- **THEN** omb 會將該 snapshot 作為 bootstrap state 傳給該 session
- **AND** 該 snapshot send 不需要先收到 gameplay input command

#### Scenario: lockstep input 不觸發 snapshot response

- **WHEN** 已在遊戲中的玩家送出 lockstep `InputSubmit`
- **THEN** omb 只將 input 放入排程 buffer 並回報既有 acceptance/diagnostics
- **AND** omb 不會因該 input 讀取 `SnapshotStore`、呼叫 snapshot extraction 或送出 snapshot response

#### Scenario: legacy player command 不觸發 snapshot response

- **WHEN** 已在遊戲中的玩家送出 legacy `PlayerCommand` 或其他 gameplay command
- **THEN** omb 依既有 command path 處理該指令
- **AND** 該 command path 不會建構或送出 snapshot response

#### Scenario: input 後狀態觀察來自後續權威流程

- **WHEN** 玩家送出 tower、ability 或 item gameplay input 並由 sim 套用
- **THEN** 玩家透過後續 tick batches、snapshot publication、outcome queues 或 render-facing authoritative state 觀察結果
- **AND** 該觀察不依賴 input command 同步觸發的 snapshot
