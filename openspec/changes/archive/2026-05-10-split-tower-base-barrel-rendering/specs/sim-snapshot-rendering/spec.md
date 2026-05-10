## ADDED Requirements

### Requirement: tower template snapshots expose combat render metadata
`SimWorldSnapshot.tower_templates` SHALL expose render-facing tower combat metadata needed by omfx composite rendering. For each tower template, the snapshot data SHALL include render mode, base image path, barrel image path, barrel frame paths, barrel animation timing, body animation frame paths for animated-area towers, rotation mode, barrel layout, barrel count variants, barrel offset, barrel pivot, muzzle offset, recoil distance, recoil scale, recoil attack duration, recoil return duration, and recoil mode. The metadata SHALL originate from scripts content data and SHALL be shared through `Arc` with the same static template lifecycle as existing tower template snapshot data.

#### Scenario: tower template snapshot contains base and barrel paths
- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_dart`
- **THEN** the snapshot entry contains a non-empty base image path for `tower_dart`
- **AND** the snapshot entry contains a non-empty barrel image path for `tower_dart`
- **AND** omfx can cache the metadata by `unit_id`

#### Scenario: tower template snapshot contains barrel animation frames
- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a tower whose barrel declares animation frames
- **THEN** the snapshot entry contains the ordered barrel frame paths
- **AND** the snapshot entry contains barrel animation timing metadata

#### Scenario: tower template snapshot contains animated-area frames
- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a no-barrel area damage tower
- **THEN** the snapshot entry contains `render_mode = "animated_area"`
- **AND** the snapshot entry contains ordered body animation frame paths
- **AND** the snapshot entry does not require a barrel image path to render safely

#### Scenario: tower template snapshot contains tack fixed rotation mode
- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains `rotation_mode = "fixed"`
- **AND** the snapshot entry contains recoil mode data that allows omfx to play a `scale_pulse` instead of target-facing directional recoil

#### Scenario: tower template snapshot contains tack count variants
- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains a radial barrel layout
- **AND** the snapshot entry contains count variants for 8, 12, and 16 barrels or needle holes
- **AND** each variant contains the image path needed by omfx to render that count state

#### Scenario: tower render metadata is built once and shared
- **WHEN** sim worker emits multiple snapshots after tower templates are available
- **THEN** tower render metadata is contained in the shared `tower_templates` Arc data
- **AND** subsequent snapshots use O(1) `Arc::clone` instead of rebuilding identical render metadata every tick

### Requirement: tower aim direction comes from snapshot-facing data
Tower barrel aiming SHALL use authoritative render-facing data from the simulation snapshot only for tower templates whose rotation mode is target-facing. `EntityRenderData.facing_rad` SHALL be treated as the authoritative tower aim direction when tower systems update it toward attack targets; otherwise, the snapshot SHALL expose an equivalent tower aim direction. For templates whose rotation mode is fixed, such as `tower_tack`, omfx SHALL keep barrel visual rotation fixed even when fire cues include a direction. omfx SHALL use snapshot data for target-facing barrel rotation and SHALL NOT compute target selection independently.

#### Scenario: snapshot exposes tower aim direction
- **WHEN** a tower has an active attack target and the sim has updated its aim direction
- **THEN** the corresponding `EntityRenderData` exposes that direction through `facing_rad` or a tower-specific aim field
- **AND** omfx uses that value to rotate the barrel sprite

#### Scenario: no target preserves last known direction
- **WHEN** a tower loses its target for a snapshot
- **THEN** snapshot/render data keeps a stable last known facing or a default facing
- **AND** omfx does not snap the barrel to a random creep or undefined angle

#### Scenario: fixed rotation tower ignores aim for barrel visual
- **WHEN** `tower_tack` has a fire cue direction and `rotation_mode = "fixed"`
- **THEN** omfx can keep the cue direction available for projectile visuals or diagnostics
- **AND** omfx SHALL NOT rotate the `tower_tack` barrel visual toward that cue direction

### Requirement: tower fire cues are drained as render-only snapshot events
`SimWorldSnapshot` SHALL include render-only tower fire cues for recoil animation. The source queue SHALL follow the `ExplosionFxQueue` pattern: deterministic gameplay processing pushes fire cue data when a tower actually fires, and `extract_snapshot` drains the pending queue with `std::mem::take`. The queue SHALL NOT be read by simulation systems for gameplay and SHALL NOT affect state hashing.

Each fire cue SHALL include at minimum the tower entity id, spawn tick, and firing direction in radians. If multiple projectile outcomes from the same tower occur in the same tick, the render cue producer or omfx SHALL allow them to collapse into one recoil pulse for that tower tick.

#### Scenario: firing tower appears in snapshot fire cues
- **WHEN** a tower attack creates a projectile or equivalent attack outcome at tick N
- **THEN** a tower fire cue for that tower entity is pushed during outcome processing
- **AND** the next `extract_snapshot` includes that cue in `SimWorldSnapshot`
- **AND** the cue contains the tower entity id, tick N, and firing direction

#### Scenario: fire cue queue is empty after drain
- **WHEN** `extract_snapshot` drains pending tower fire cues
- **THEN** the drained cues appear in the snapshot
- **AND** the source queue is empty after extraction
- **AND** the same cue does not repeat in later snapshots unless the tower fires again

#### Scenario: fire cues do not change determinism
- **WHEN** determinism tests hash the sim state before and after tower fire cue extraction
- **THEN** render-only fire cue queue contents are not part of the authoritative gameplay hash
- **AND** draining the cue queue does not mutate gameplay components, resources, entity existence, damage, cooldown, or projectile state

### Requirement: attack phase cues are exposed through render snapshots
`SimWorldSnapshot` SHALL expose render-only attack phase cues for unit attack animation. Each cue SHALL represent an attack windup start and include entity id, attack sequence id, windup duration, impact offset, backswing duration, and target or direction data. The cue source queue SHALL be drained with the same render-only pattern as explosion and tower fire cues.

#### Scenario: attack phase cue appears before impact
- **WHEN** a unit starts attack windup at tick N and impact is scheduled for a later tick or sub-tick offset
- **THEN** the next render snapshot includes an attack phase cue for that unit
- **AND** omfx can start attack animation before projectile spawn or damage impact

#### Scenario: attack phase cue queue drains once
- **WHEN** `extract_snapshot` drains pending attack phase cues
- **THEN** drained cues appear in the snapshot
- **AND** the source queue is empty after extraction
- **AND** the same cue does not appear again in later snapshots unless another attack windup starts
