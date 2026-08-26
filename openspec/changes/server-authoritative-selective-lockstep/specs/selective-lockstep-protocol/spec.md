## ADDED Requirements

### Requirement: Secure match 使用 protocol V2 team stream

Secure fog match SHALL 在 join 前 negotiate protocol V2。Match 內所有 player session SHALL 使用 team-specific stream；V1/V2 player MUST NOT 混用。`TeamGameStart` SHALL 宣告 protocol/schema version、player/team ID、server/replica tick、tick rate、visibility delay、replica buffer 與 filtered team snapshot。

#### Scenario: V2 join 只取得 team bootstrap

- **WHEN** player 加入 secure fog match 並通過 V2 capability negotiation
- **THEN** server 回傳該 player 所屬 team 的 `TeamGameStart`
- **AND** bootstrap 不包含 global snapshot、global `master_seed` 或其他 team state

#### Scenario: V1 client 被 secure match 拒絕

- **WHEN** V1 client 嘗試加入 secure fog match
- **THEN** join 在取得任何 world snapshot 前被拒絕
- **AND** match 不降級成 global-world protocol

### Requirement: Team-scoped replica identity

Player wire SHALL 使用每個 team 獨立的 opaque `ReplicaEntityId` 與 disclosure epoch。Raw `specs::Entity::id()`、generation 與 canonical ID MUST NOT 出現在 player protocol。Replica ID SHALL monotonic、match-local 且 retire 後不重用。

#### Scenario: 不同 team 無法比對 identity

- **WHEN** 同一 canonical entity 對 team A 與 team B disclosure
- **THEN** 兩個 team 收到的 `ReplicaEntityId` 不相同
- **AND** 任一 team frame 都不包含 cross-team mapping

#### Scenario: Stale disclosure epoch 被拒絕

- **WHEN** client 使用舊 disclosure epoch 引用已 re-reveal 的 replica ID
- **THEN** server 拒絕該 reference
- **AND** 不影響目前 incarnation

### Requirement: Canonical `TeamTickFrame` phase 與順序

`TeamTickFrame` SHALL 包含 `server_tick`、`replica_tick`、`team_sequence`、`view_epoch`、`PreStep`、`Step`、`PostStep` 與 compatibility metadata。Transition SHALL 在 `PreStep`，accepted input/public event/random tape/external effect SHALL 在 `Step`，authority repair/hash SHALL 在 `PostStep`。同 phase 內 SHALL 依 event kind、replica ID 與 stable sub-index canonical ordering。

#### Scenario: Server observer 與 remote client 套用相同順序

- **WHEN** frame 同時包含 reveal、accepted input、external damage 與 component repair
- **THEN** server observer 與 remote client 都依 `PreStep -> Step -> PostStep` 套用
- **AND** 兩者計算相同 team-view hash

#### Scenario: Parallel producer 不改變 bytes

- **WHEN** 相同 logical frame 由不同 thread completion order 產生
- **THEN** canonical encoding bytes 完全相同

### Requirement: Filtered snapshot 與 randomness boundary

Filtered team snapshot SHALL 只包含 disclosed deterministic state、public/team-private resource 與 team-scoped identity。Protocol MUST NOT 傳送 global seed 或可延伸到 hidden period 的 PRNG state。Randomness SHALL 使用 already-decided outcome 或綁定 disclosure epoch/tick window 的 bounded random tape。

#### Scenario: Random tape 不能跨 hidden epoch 使用

- **WHEN** entity 的 disclosure epoch 結束
- **THEN** client 不得使用該 epoch random tape 推進後續 hidden tick
- **AND** re-reveal 使用新 epoch/tape 或 authoritative outcome
