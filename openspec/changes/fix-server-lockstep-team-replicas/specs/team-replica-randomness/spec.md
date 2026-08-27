## ADDED Requirements

### Requirement: Replica取得相同global seed
`TeamGameStart` SHALL 將同一個`global_seed`提供給authoritative runtime、Team 1 replica、Team 2 replica與omfx replica。戰爭迷霧安全 SHALL 依靠hidden state不被投影，而不是依靠seed保密。

#### Scenario: 兩隊bootstrap取得相同seed
- **WHEN** Team 1與Team 2處理同一場match的`TeamGameStart`
- **THEN** 兩隊取得完全相同的`global_seed`
- **AND** filtered snapshot仍不包含另一隊hidden entity state

### Requirement: 每tick由global seed與tick重建RNG
每個replica tick SHALL 使用`tick_seed = hash(global_seed, tick)`建立新的deterministic RNG stream。實作 SHALL NOT 要求entity ID、system ID或action ID作為seed domain。

#### Scenario: 相同seed與tick產生相同stream
- **WHEN** server與replica使用相同`global_seed`及tick建立RNG
- **THEN** 在相同request順序下產生相同random sequence

#### Scenario: 不同tick不沿用cursor
- **WHEN** runtime從tick T前進到T+1
- **THEN** T+1 RNG由`global_seed`與T+1重新建立
- **AND** tick T消耗多少random value不改變T+1初始stream

### Requirement: 平行system以stable request order消耗RNG
平行Specs system SHALL NOT 直接競爭tick-local RNG。Random request SHALL 帶stable ordering key，在barrier合併排序後依序消耗stream。

#### Scenario: System完成順序不改變RNG結果
- **WHEN** 測試交換兩個平行system的完成順序但request集合相同
- **THEN** 排序後random assignments與team hash保持相同

### Requirement: Hidden random跨界使用external effect
Hidden entity不在team replica建立random request。Hidden random結果影響disclosed state時，server SHALL 以sanitized external effect套用結果；replica不得推測hidden request數量。

#### Scenario: Hidden random呼叫不要求replica重播
- **WHEN** hidden entity在authoritative world內增加random呼叫並影響visible hero
- **THEN** team replica透過external effect取得visible結果
- **AND** team frame不揭露hidden random request或entity identity

