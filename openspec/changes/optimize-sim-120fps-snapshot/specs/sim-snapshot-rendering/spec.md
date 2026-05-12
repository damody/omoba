## ADDED Requirements

### Requirement: extract_snapshot is not used by omfx sim_runner

`omfx` sim_runner SHALL NOT call `omoba_core::runtime::extract_snapshot`. Initialization seed data and runtime render updates SHALL be produced by explicit lightweight paths instead of a full ECS snapshot dump.

#### Scenario: initialization seeds static data without extract_snapshot
- **WHEN** sim_runner finishes loading scripts, creating the world, and preparing static metadata
- **THEN** sim_runner seeds bootstrap render data without calling `extract_snapshot`
- **AND** the seeded data contains static render data needed by omfx, such as paths, blocked regions, tower templates, tower upgrades, and ability definitions

#### Scenario: runtime ticks do not call extract_snapshot
- **WHEN** sim_runner receives and applies TickBatch ticks N through N+119 during one healthy second after initialization
- **THEN** `extract_snapshot` is not called for those runtime ticks
- **AND** no `[sim_runner]` or `[mirror-snapshot]` full snapshot scan log is emitted from runtime extraction

### Requirement: runtime render state uses lightweight publication

Runtime sim updates SHALL publish render-facing dynamic state without invoking `extract_snapshot`. The runtime publication path SHALL update the dynamic fields consumed by omfx render/HUD/input pairing while preserving static bootstrap data from initialization seed data.

#### Scenario: runtime state advances without full snapshot extraction
- **WHEN** sim_runner applies a newer TickBatch after initialization
- **THEN** omfx observes updated runtime tick and dynamic render data through the lightweight publication path
- **AND** paths, blocked regions, ability definitions, tower templates, and tower upgrade definitions remain available from initialization seed data
- **AND** `extract_snapshot` is not invoked

#### Scenario: removed ids and FX remain delivered
- **WHEN** runtime systems delete entities or enqueue render FX after initialization
- **THEN** the lightweight publication path delivers removed entity ids and render FX to omfx consumers
- **AND** each queue item is drained or deduplicated according to the existing render semantics

### Requirement: repeated runtime tick consumption does not repeat expensive render work

omfx SHALL detect when the latest available runtime render tick is the same as the tick already consumed for expensive render-facing work and SHALL skip or defer tick-driven scene/entity/UI updates for that unchanged tick. This SHALL NOT prevent frame-local UI countdowns, input handling, lockstep event draining, or sim_runner progress.

#### Scenario: unchanged tick reuses rendered state
- **WHEN** `Plugin::update` observes the same runtime render tick as the previous expensive render update
- **THEN** omfx does not run full tick-driven entity scene updates again for that tick
- **AND** omfx continues processing input/network events and frame-local UI state

#### Scenario: new tick renders immediately
- **WHEN** sim_runner publishes runtime render data with a tick greater than the last expensive render update
- **THEN** omfx consumes that runtime data on the next eligible update
- **AND** stale tick skipping does not delay the newer runtime data by an extra frame interval
