## ADDED Requirements

### Requirement: Presentation IPC為版本化loopback protobuf

Runtime與renderer SHALL 使用loopback TCP、固定magic/version、big-endian length prefix及protobuf payload。Runtime MUST 拒絕非loopback bind、超過上限的frame、錯誤magic/version與無效protobuf。Schema SHALL 可由Rust omfx及未來Unreal C++產生型別，且 MUST NOT 暴露Rust ABI。

#### Scenario: 不相容client被拒絕
- **WHEN** renderer送出錯誤protocol version或超長frame
- **THEN** runtime關閉該IPC connection並記錄不含hidden資料的原因
- **AND** filtered Specs world繼續由runtime持有

### Requirement: Presentation只包含render-safe資料

Runtime輸出 SHALL 限於team identity、authoritative/replica tick、filtered render entities、removed render IDs、remembered ghosts、10×10 fog tiles、visibility digest、己隊vision circles、安全blocked regions/tree occluders、effects、audio cues、input result及session狀態。Payload MUST NOT 包含canonical Specs Entity ID、hidden位置、hidden component、server-only metadata或完整map entity清單。

#### Scenario: 視野外敵人不在presentation
- **WHEN** Team 2單位未對Team 1揭露且沒有合法LastKnown ghost
- **THEN** Team 1 presentation中不存在其實體、位置、component或canonical identity

### Requirement: IPC queue有界且input不等待snapshot

Simulation SHALL 維持120 Hz；presentation cadence SHALL 支援30/60/120 Hz且預設60 Hz。Snapshot channel SHALL 使用bounded latest-wins slot；critical session/input result SHALL 使用獨立reliable ordered queue。Renderer input MUST 立即送runtime，不得等待下一個presentation tick。

#### Scenario: Renderer落後不累積snapshot
- **WHEN** renderer消費速度低於presentation產生速度
- **THEN** runtime丟棄舊snapshot並保留最新snapshot
- **AND** critical input result仍依序送達

### Requirement: Input IPC只傳玩家意圖

Renderer SHALL 只送MoveTo、AttackMove、AbilityCast、ItemUse、Tower action、ready、consumed sequence及graceful shutdown。Runtime SHALL 驗證格式、owner、disclosure epoch與target membership後才轉送server；server MUST 再做authoritative驗證。

#### Scenario: Hidden target遭雙層拒絕
- **WHEN** renderer送出指向hidden或stale target的input
- **THEN** runtime拒絕該input且不轉送hidden identity
- **AND** server對任何繞過runtime的等價input仍拒絕
