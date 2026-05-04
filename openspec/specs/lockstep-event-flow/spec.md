## Purpose

Define which legacy network render events are forbidden or retained, and establish outcome queues as the authoritative bridge from omb simulation events to omfx snapshot rendering.

## Requirements

### Requirement: legacy render-event emits are forbidden

omb ECS tick and handler systems SHALL NOT send legacy render-state `TypedOutbound` events for state that is now derived from omfx lockstep simulation and `SimWorldSnapshot` extraction.

The forbidden emit list includes:

- `EntityFacing`, `CreepStall`, `CreepSlow`, `CreepCreate`, `CreepMove`, `CreepHp`, `ProjectileCreate`, `ProjectileDestroy`, `UnitCreate`, entity `Miss`, and `GameExplosion` legacy render payloads.
- `TypedOutbound::EntityDeath`, `TypedOutbound::TowerCreate`, `TypedOutbound::TowerUpgrade`, `TypedOutbound::GameRound`, `TypedOutbound::HeroStatic`, and `TypedOutbound::HeroHot`.
- Builder functions for those payloads, including entity death, tower create, tower upgrade, game round, hero static, hero hot, and game explosion builders.

All equivalent render state SHALL come from the local omfx sim ECS world and extracted snapshots.

#### Scenario: TD_STRESS wire traffic stays low

- **WHEN** `run_smoke_long.bat` runs for 60 seconds with `STORY = "TD_STRESS"`
- **THEN** the sampled `kcp-p7 .* bytes_per_sec` values in `omb_app.log` remain below 5000 bytes per second
- **AND** `omb_app.log` contains zero `Removed disconnected KCP session` lines
- **AND** `omfx_app.log` contains zero `no TickBatch in 1.0s` lines

#### Scenario: forbidden TypedOutbound variants are not constructed

- **WHEN** `omb/src/` is searched for `TypedOutbound::EntityDeath`, `TypedOutbound::TowerCreate`, `TypedOutbound::TowerUpgrade`, `TypedOutbound::GameRound`, `TypedOutbound::HeroStatic`, and `TypedOutbound::HeroHot`
- **THEN** no `OutboundMsg::new_typed*` construction uses those variants as payload
- **AND** the corresponding KCP routing entries and dead builder functions are absent

#### Scenario: omb lib tests pass

- **WHEN** `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib` is run
- **THEN** the omb library test suite passes

### Requirement: retained HUD broadcasts are allowlisted

omb SHALL retain only gameplay broadcasts that are still required as player-visible acknowledgements or one-shot terminal events.

The allowlist is:

- `TypedOutbound::GameLives` / `make_game_lives` for life-loss acknowledgement.
- `TypedOutbound::GameEnd` / `make_game_end` for game end overlays.

Explosion VFX SHALL NOT be broadcast and SHALL use `Outcome::Explosion` into `ExplosionFxQueue` and `snapshot.explosions`.

#### Scenario: life loss remains visible

- **WHEN** a TD_1 creep reaches the base and reduces lives
- **THEN** omb broadcasts the retained `game.lives` event
- **AND** omfx HUD reflects the new lives value

#### Scenario: game end remains visible

- **WHEN** TD_1 ends because the game is completed or lives reach zero
- **THEN** omb broadcasts the retained `game.end` event
- **AND** omfx displays the game end overlay

### Requirement: entity removal uses `Outcome::EntityRemoved` as the only delete channel

`omb/src/comp/outcome.rs` SHALL define `Outcome::EntityRemoved { entity: Entity }` and `RemovedEntitiesQueue { pending: Vec<u32> }`. `process_outcomes` SHALL be the only code path that calls `entities().delete()`. The `EntityRemoved` arm SHALL push `entity.id()` into `RemovedEntitiesQueue.pending` and delete the entity in the same outcome-processing pass.

All systems that need to remove an entity SHALL push `Outcome::EntityRemoved { entity }` into the world `Vec<Outcome>` resource. Script boundary despawn calls SHALL route through the same outcome resource. Direct entity deletion outside `process_outcomes` is prohibited.

`extract_snapshot` SHALL drain `RemovedEntitiesQueue` into `SimWorldSnapshot.removed_entity_ids`. omfx render code SHALL release per-entity render caches for those ids, including scene nodes, labels, and collision rings.

#### Scenario: creep death removes the omfx scene node

- **WHEN** a TD_1 creep dies in sim
- **THEN** death handling enqueues `Outcome::EntityRemoved { entity }`
- **AND** `process_outcomes` deletes the entity and records its id in `RemovedEntitiesQueue.pending`
- **AND** the next snapshot includes the creep id in `removed_entity_ids`
- **AND** omfx releases render caches for that id
- **AND** omb does not broadcast `EntityDeath`

#### Scenario: sold tower is removed through the outcome channel

- **WHEN** a TD_1 player sells a tower
- **THEN** sell handling enqueues `Outcome::EntityRemoved { entity }`
- **AND** `process_outcomes` deletes the tower and records its id
- **AND** the next snapshot includes the tower id in `removed_entity_ids`
- **AND** the tower's scene node, label, and collision ring disappear

#### Scenario: process_outcomes is the only delete sink

- **WHEN** `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --test delete_entity_outcome_only` is run
- **THEN** the grep guard passes and finds no `entities().delete(` or `.delete_entity(` calls outside the allowed outcome-processing sink

### Requirement: explosion VFX use `Outcome::Explosion` and a drainable queue

`omb/src/comp/outcome.rs` SHALL define `Outcome::Explosion { pos: omoba_sim::Vec2, radius: omoba_sim::Fixed64, duration: omoba_sim::Fixed64 }` and `ExplosionFxQueue { pending: Vec<ExplosionFx> }`. `process_outcomes` SHALL push explosion data into `ExplosionFxQueue`. Script boundary explosion calls SHALL push into the same queue and SHALL NOT send a legacy network event.

`extract_snapshot` SHALL drain the queue into `SimWorldSnapshot.explosions`. omfx render code SHALL create a red-circle VFX for each explosion, animate scale and alpha over `duration_ms`, and release the scene node when the duration ends.

#### Scenario: bomb tower explosion renders locally

- **WHEN** a bomb tower hits a creep and emits an explosion outcome
- **THEN** the next snapshot contains an explosion entry with position, radius, duration, and spawn tick
- **AND** omfx renders a fading red-circle VFX
- **AND** the VFX node is released when its duration ends
- **AND** omb does not broadcast `GameExplosion`

#### Scenario: legacy explosion builders are absent

- **WHEN** `omb/src/` is searched for `Outcome::Explosion`, `ExplosionFxQueue`, and legacy game explosion builders
- **THEN** `Outcome::Explosion` and `ExplosionFxQueue` are present
- **AND** legacy `make_game_explosion` builders are absent except comments
- **AND** extraction drains the queue into `snapshot.explosions`
