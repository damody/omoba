## ADDED Requirements

### Requirement: UE TD control events are exposed to Blueprint
`Om UE frontend` SHALL expose C++ and Blueprint-facing presentation/input-intent events for TD controls that exist in the native frontend: tower shop selection, tower placement preview, tower placement confirmation, selected tower changes, tower upgrade request, tower sell request, tower target priority request, and start-round request. These events SHALL be UI/input intent events; authoritative gameplay results SHALL still arrive from bridge frames after Rust/local replica processing.

#### Scenario: Tower shop selection emits a typed event
- **WHEN** the player selects a tower from the UE tower shop UI, hotkey, Blueprint, or C++ call
- **THEN** UE MUST emit a typed tower shop selection event containing tower catalog id, content id/name, display label, cost, current gold, affordability state, and input source
- **AND** the event MUST update presentation selection state without spawning an authoritative tower actor

#### Scenario: Placement preview emits validity changes
- **WHEN** a tower is selected for placement and the cursor or target point changes
- **THEN** UE MUST emit a placement preview event containing tower catalog id, backend-world point, footprint radius, attack range, affordability, validity, and invalid reason when known
- **AND** preview validity MUST be treated as advisory because Rust/local replica remains authoritative for final placement

#### Scenario: Placement confirmation routes through gameplay input
- **WHEN** the player confirms a tower placement from preview state
- **THEN** UE MUST emit a placement confirmation event and route it through the gameplay input event generator
- **AND** the same click MUST be marked consumed so it cannot also emit move, attack, or selection events

#### Scenario: Selected tower changes are observable
- **WHEN** the player selects, changes, clears, sells, or loses a tower because the frame removed it
- **THEN** UE MUST emit a selected tower changed event containing previous entity ref, new entity ref, tower catalog id when available, owner/player information, upgrade levels, range, target priority, and clear reason
- **AND** stale or removed tower refs MUST be cleared before UI emits management requests

#### Scenario: Tower management requests are typed
- **WHEN** the player requests tower upgrade, sell, or target priority change from UE UI, hotkey, Blueprint, or C++ call
- **THEN** UE MUST emit a typed request event containing selected tower entity id/gen, player id, requested path/level or priority, source, and UI-consumed state
- **AND** accepted requests MUST route through the gameplay input event generator instead of mutating tower state locally

### Requirement: UE HUD and entity overlay events are exposed to Blueprint
`Om UE frontend` SHALL expose C++ and Blueprint-facing events or view-model updates for hero HUD, ability bar, active buff list, entity health/name overlays, selected tower range overlays, and runtime diagnostics. These events SHALL be presentation-only and SHALL NOT submit gameplay commands unless routed through the gameplay input event generator.

#### Scenario: Hero HUD state changes are emitted
- **WHEN** bridge frames update the local hero stats, round/lives/gold state, inventory, active command, or active buffs
- **THEN** UE MUST emit a hero HUD state event containing stable hero entity ref, health/max health, level, XP, skill points, gold, lives, attributes, combat stats, active command summary, inventory state, and active buff summaries
- **AND** Blueprint widgets MUST be able to refresh visuals without reading Rust memory after the frame lease is released

#### Scenario: Ability HUD state changes are emitted
- **WHEN** ability catalog metadata, level, cooldown, range, icon path, upgrade availability, or tooltip-relevant data changes
- **THEN** UE MUST emit an ability HUD state event per affected slot
- **AND** the event MUST distinguish ability cast request, ability upgrade request, cooldown display, and tooltip display so widgets do not infer gameplay actions from presentation updates

#### Scenario: Entity overlay state changes are emitted
- **WHEN** visible entity name, health, max health, selection state, owner, upgrade levels, or range overlay state changes
- **THEN** UE MUST emit an entity overlay state event containing entity id/gen, display name, health values, overlay visibility, world/screen anchor data when available, range radius, and throttling visibility reason
- **AND** overlay events MUST clear when the entity is removed or recycled

#### Scenario: Runtime diagnostics changes are emitted
- **WHEN** runtime diagnostics change connection state, tick/sequence, sim TPS, frame publication counters, active leases, input queue length, network bytes, RTT, Lua generation/hash, missing Blueprint count, or last error
- **THEN** UE MUST emit a diagnostics state event suitable for a development HUD or log sink
- **AND** diagnostics display MUST NOT require acquiring a frame lease

### Requirement: UE UI and overlay event verification
The implementation SHALL include synthetic or UE automation coverage for TD control events, HUD state events, entity overlay cleanup, diagnostics updates, and the boundary between presentation events and gameplay input events.

#### Scenario: UI events do not mutate authority
- **WHEN** automated tests emit tower shop selection, placement preview, selected tower changed, hero HUD, ability HUD, overlay, or diagnostics events
- **THEN** no deterministic gameplay state MUST change unless a corresponding gameplay input event is explicitly submitted through the input generator

#### Scenario: UI guard prevents duplicate gameplay submission
- **WHEN** automated tests trigger tower placement, tower management, ability, item, or start-round UI controls
- **THEN** the intended gameplay input MUST be submitted at most once
- **AND** the originating pointer/key action MUST NOT also submit move, attack, selection, or camera commands
