## ADDED Requirements

### Requirement: snapshot exposes hero command and path render data
`SimWorldSnapshot` SHALL expose hero command render data for local and observed heroes when such data exists. The data SHALL include command kind, destination when applicable, target entity id when applicable, queued command count or queue summary when applicable, queue limit, and active waypoint data or next waypoint. Extraction SHALL read deterministic sim state and MAY drain only existing render-only queues allowed by the snapshot invariant; it SHALL NOT mutate gameplay components while projecting command/path data.

#### Scenario: MoveTo snapshot includes waypoint
- **WHEN** a hero has an active path-following move command
- **THEN** the next snapshot contains command kind `move` or equivalent
- **AND** the snapshot contains destination and next waypoint data

#### Scenario: AttackTarget snapshot includes target id
- **WHEN** a hero has an active specified attack command
- **THEN** the next snapshot contains command kind `attack-target`
- **AND** the snapshot contains the target entity id

#### Scenario: queued commands snapshot includes count
- **WHEN** a hero has one active command and two queued commands
- **THEN** the next snapshot contains the active command render data
- **AND** the snapshot exposes queued command count `2` or an equivalent queue summary
- **AND** the snapshot exposes or lets omfx infer the queue limit `16`

#### Scenario: extraction remains read-only
- **WHEN** `extract_snapshot` projects hero command/path data
- **THEN** it does not write hero command, path, position, attack, damage, or entity lifecycle components
- **AND** gameplay state hash is unaffected by snapshot projection

### Requirement: snapshot exposes tower target priority
`EntityRenderData` or an equivalent tower extension in `SimWorldSnapshot` SHALL expose the current target priority for Tower entities. omfx SHALL mirror this field into its snapshot-backed `network_entities` tower state and use it for tower panel display.

#### Scenario: selected tower mirror contains priority
- **WHEN** a snapshot contains a Tower entity with target priority `highest-health`
- **THEN** omfx tower mirror stores `highest-health` for that entity
- **AND** selecting that tower displays the same priority in the tower panel

#### Scenario: removed tower clears priority mirror
- **WHEN** a tower is removed and its id appears in `removed_entity_ids`
- **THEN** omfx removes the tower mirror entry including target priority state
- **AND** selecting the old location cannot show stale priority UI
