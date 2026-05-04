## Purpose

Define the snapshot data contract used by omfx to render lockstep simulation state, including HUD state, entity removal, hero and tower UI data, VFX, blocked regions, and path styling.

## Requirements

### Requirement: `SimWorldSnapshot` structure and read-only-except-queues invariant

`omfx/game/src/sim_runner.rs::SimWorldSnapshot` SHALL contain all render-facing state needed by omfx, including tick, entities, paths, removed entity ids, round data, lives, blocked regions, explosions, ability definitions, tower templates, and tower upgrade definitions.

The snapshot entity data SHALL include optional hero extension data, optional tower upgrade levels, and render-safe fixed-point conversions. `extract_snapshot` SHALL treat the sim ECS world as read-only except for producer-consumer queue drains using `std::mem::take(&mut q.pending)`. It SHALL NOT write components, create entities, delete entities, or mutate unrelated resources. Boundary values SHALL be converted from fixed-point to render `f32` through the project fixed-point helpers.

#### Scenario: extract_snapshot only drains outcome queues

- **WHEN** `omfx/game/src/sim_runner.rs::extract_snapshot` is searched for `write_storage`, `write_resource`, `entities.create`, and `entities.delete`
- **THEN** the only permitted writes are `mem::take` drains for `RemovedEntitiesQueue` and `ExplosionFxQueue`
- **AND** there are no component writes, entity creates, or entity deletes

#### Scenario: omoba-sim determinism tests pass

- **WHEN** `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features` is run
- **THEN** the omoba-sim determinism test suite passes, including pin-hash tests

#### Scenario: `Outcome::EntityRemoved` deletes in the same tick boundary

- **WHEN** a system pushes `Outcome::EntityRemoved { entity: e }` into the world outcome resource
- **THEN** `process_outcomes` pushes `e.id()` into `RemovedEntitiesQueue.pending` and calls `entities().delete(e)`
- **AND** after `world.maintain()` the entity is no longer alive
- **AND** state hashing no longer includes the deleted entity after that tick boundary

### Requirement: `removed_entity_ids` drain from `RemovedEntitiesQueue`

`extract_snapshot` SHALL populate `SimWorldSnapshot.removed_entity_ids` by draining `RemovedEntitiesQueue.pending` with `std::mem::take`. The drain SHALL leave the queue empty. `extract_snapshot` SHALL NOT use a cross-tick `prev_alive: HashSet<u32>` diff algorithm.

#### Scenario: drain leaves queue empty

- **WHEN** the sim worker completes a tick and `RemovedEntitiesQueue.pending` contains N ids
- **THEN** `extract_snapshot` moves all N ids into `snapshot.removed_entity_ids`
- **AND** `RemovedEntitiesQueue.pending` is empty after extraction

#### Scenario: no deletion produces an empty removed list

- **WHEN** no `Outcome::EntityRemoved` was processed before a snapshot
- **THEN** `snapshot.removed_entity_ids` is empty

#### Scenario: removed ids are not repeated across ticks

- **WHEN** tick N records removed entity id 2
- **THEN** the next snapshot includes id 2
- **AND** the following snapshot does not include id 2 unless a new removal for id 2 was recorded

### Requirement: HUD reads round, lives, and running state from snapshots

`extract_snapshot` SHALL read round, total rounds, round running state, and lives from sim ECS resources aligned with `omobab::comp`: `CurrentCreepWave` and `PlayerLives`. omfx HUD SHALL read these values from the snapshot and SHALL NOT use legacy heartbeat or mirror state for these fields.

#### Scenario: HUD lives and round reflect sim state

- **WHEN** TD_1 is running and a creep leaks through
- **THEN** the next snapshot has a decremented `lives` value
- **AND** the omfx HUD updates the displayed lives value

#### Scenario: round_is_running controls wave UI

- **WHEN** the player starts a round
- **THEN** the next snapshot has `round_is_running == true` and an updated `round`
- **AND** omfx updates the start button and wave counter UI accordingly

### Requirement: hero stats are aggregated into `HeroStatsExt`

`EntityRenderData` SHALL include `hero_ext: Option<Box<HeroStatsExt>>` for hero entities. `HeroStatsExt` SHALL include armor, magic resist, attack damage, attack range, move speed, attack speed seconds, bullet speed, mana, max mana, buffs, inventory, ability levels, and ability ids needed by omfx UI.

For each Hero entity, `extract_snapshot` SHALL use `omobab::ability_runtime::UnitStats::from_refs(...)` and final stat accessors to populate `HeroStatsExt`. omfx hero panel UI SHALL read hero stats from snapshot entity data for the local hero. Buff countdown display MAY decrement locally between snapshots and SHALL be reset by authoritative snapshot values.

#### Scenario: hero panel displays expected reference stats

- **WHEN** TD_1 loads the reference hero scene
- **THEN** the hero panel displays the authoritative armor, attack damage, attack speed, range, and move speed values from `hero_ext`

#### Scenario: finite buff countdown is smooth between snapshots

- **WHEN** a hero has a buff with 5 seconds remaining
- **THEN** the snapshot reports `remaining_secs == 5.0`
- **AND** omfx may decrement the displayed value by frame delta between snapshots
- **AND** the next snapshot resets the display to the authoritative remaining time

#### Scenario: toggle buff does not count down

- **WHEN** a hero has a toggle or indefinite buff
- **THEN** the snapshot reports `remaining_secs == -1.0`
- **AND** omfx does not display a countdown for that buff

### Requirement: tower upgrade levels render from `EntityRenderData.upgrade_levels`

`EntityRenderData` SHALL include `upgrade_levels: Option<[u8; 3]>`, populated only for Tower entities. `extract_snapshot` SHALL read upgrade levels from the sim ECS Tower component. omfx SHALL render tower upgrade state from this snapshot value.

#### Scenario: upgraded tower exposes levels in snapshot

- **WHEN** a player upgrades path 0 of a tower to level 2
- **THEN** the next snapshot has `upgrade_levels == Some([2, 0, 0])` for that tower
- **AND** omfx reflects the upgraded state in tower UI

### Requirement: tower selection and upgrade panel use a snapshot-backed mirror

omfx render code SHALL mirror `EntityKind::Tower` snapshot entities into `network_entities: HashMap<u32, NetworkEntity>` after acquiring a snapshot. The mirror SHALL map tower entity type, render position, tower kind, upgrade levels, collision radius, and attack range. After mirroring, omfx SHALL remove stale tower entries that are no longer present in the current snapshot.

Tower click hit-testing, sell/upgrade panel rendering, and attack-range display SHALL consume `network_entities` rather than directly locking the snapshot in UI handlers.

#### Scenario: clicking a TD tower opens the tower panel

- **WHEN** a TD_1 player left-clicks an existing tower
- **THEN** `selected_tower_entity` is set from the snapshot-backed mirror
- **AND** the sell button and three upgrade buttons are shown
- **AND** the selected tower attack range is shown when available

#### Scenario: selling a tower removes its mirror entry

- **WHEN** a sold tower no longer appears in the next snapshot
- **THEN** `network_entities` no longer contains that tower id
- **AND** clicking the old tower position does not select the sold tower

#### Scenario: upgrading a tower updates the mirror

- **WHEN** a tower upgrade is applied and the next snapshot has updated levels
- **THEN** the corresponding `network_entities` entry has the same updated `upgrade_levels`
- **AND** the upgrade panel text reflects the new next-level state

### Requirement: tower upgrade definitions are shared through snapshot Arc data

`SimWorldSnapshot.tower_upgrades` SHALL be an `Arc<Vec<TowerUpgradeDefSnapshot>>` built from `TowerUpgradeRegistry`. `TowerUpgradeDefSnapshot` SHALL include tower kind, path, level, name, and cost. The sim worker SHALL build this data once and clone the `Arc` for snapshots. omfx SHALL cache these definitions by `(unit_id, path, level)` for sell refund and upgrade button text.

#### Scenario: omfx sell refund matches omb

- **WHEN** a player sells a tower after buying upgrades
- **THEN** omfx sell panel refund calculation uses base tower cost and upgrade costs from snapshot tower upgrade definitions
- **AND** the displayed refund matches omb sell logic

#### Scenario: upgrade buttons display next-level names

- **WHEN** a TD_1 player selects an unupgraded dart monkey tower
- **THEN** each path button shows the next level name and cost
- **AND** the button text does not use unsupported unicode pip glyphs

#### Scenario: maxed path displays MAX

- **WHEN** a tower path reaches max level
- **THEN** that path's upgrade button displays `MAX`

### Requirement: tower body labels only show upgrade level summaries

omfx sim-runner-backed entity labels SHALL show Tower labels only when at least one upgrade path is above zero. The label text SHALL be `<L0>/<L1>/<L2>`, such as `2/4/0`. Unupgraded towers SHALL have no tower label. Tower labels SHALL NOT include tower name or HP. Hero and Creep labels MAY keep the existing `name HP/MaxHP` format.

#### Scenario: unupgraded tower has no label

- **WHEN** a TD_1 player places a new tower with `upgrade_levels == [0, 0, 0]`
- **THEN** omfx does not create a name label widget for that tower
- **AND** any stale tower label entry is removed

#### Scenario: upgraded tower label shows N/N/N

- **WHEN** a tower has `upgrade_levels == Some([2, 4, 0])`
- **THEN** the tower label text is `2/4/0`
- **AND** the label contains no tower name, HP, or unsupported pip glyphs

### Requirement: sell and upgrade panel width avoids clipping

The TD sell and upgrade panel text widgets in `omfx/game/src/lib.rs::Game::on_init` SHALL have widths of at least 360.0, and the associated panel width calculation SHALL also be at least 360.0.

#### Scenario: upgrade button text is not clipped

- **WHEN** TD_1 renders the dart monkey path 0 upgrade button
- **THEN** the widget width is at least 360.0
- **AND** the full upgrade name and cost are visible

### Requirement: blocked regions render from snapshots

`SimWorldSnapshot.blocked_regions` SHALL be populated from `omobab::comp::BlockedRegions`. omfx SHALL render polygon outlines and circle outlines from this snapshot data.

#### Scenario: DEBUG_1 displays region outlines

- **WHEN** `STORY = "DEBUG_1"` loads a scene with blocked regions
- **THEN** the snapshot contains blocked region data
- **AND** omfx renders the region outlines visibly

#### Scenario: TD_1 has no region outlines

- **WHEN** TD_1 loads with no blocked regions
- **THEN** `blocked_regions` is empty
- **AND** omfx renders no blocked-region outlines

### Requirement: ability definitions are shared through snapshot Arc data

`SimWorldSnapshot.abilities` SHALL be an `Arc<Vec<AbilityDefSnapshot>>` built once by the sim worker and cloned into later snapshots. `AbilityDefSnapshot` SHALL include ability id, display name, max level, icon path, and other UI metadata. `HeroStatsExt.ability_levels` and `HeroStatsExt.ability_ids` SHALL allow omfx to render Q/W/E/R ability bars from snapshot data.

#### Scenario: hero ability bar reflects level changes

- **WHEN** a TD_1 hero levels up and the player upgrades Q
- **THEN** the next snapshot has an incremented first ability level
- **AND** omfx changes the Q display from `0/4` to `1/4`

#### Scenario: ability definitions are not rebuilt every snapshot

- **WHEN** the sim worker emits N snapshots
- **THEN** the inner ability definition vector is built once
- **AND** each snapshot uses an O(1) `Arc::clone`

### Requirement: path rendering uses the thick cream zigzag style

`omfx/game/src/render_bridge.rs::ensure_paths_drawn` SHALL render paths using line width `64.0 * crate::WORLD_SCALE * 2.0` and color `(170, 140, 90, 255)`. Checkpoint marker dots SHALL NOT be rendered.

#### Scenario: TD_1 path is thick and cream colored

- **WHEN** TD_1 loads
- **THEN** path rendering uses the thick cream zigzag line style
- **AND** no extra checkpoint marker dots are rendered at corners
