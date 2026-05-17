## ADDED Requirements

### Requirement: UE gameplay input event generator
`Om UE frontend` SHALL provide a UE-side gameplay input event generator that converts player intent from viewport clicks, keyboard modifiers, HUD/Blueprint controls, and C++ calls into typed gameplay input events. These events SHALL cover move, queued/Shift move, attack move, attack target, cast ability, upgrade ability, item use, place tower, upgrade/sell tower, set tower target priority, and start-round actions. The generator SHALL emit commands only through the bridge input path and SHALL NOT mutate deterministic gameplay state directly.

#### Scenario: Move event is generated from viewport hit
- **WHEN** the player clicks a valid world location with move intent
- **THEN** the UE generator MUST produce a move input event containing player id, backend-world target point, target tick policy metadata when available, and no stale UI selection payload
- **AND** the event MUST be routed to the bridge/Rust receiver exactly once

#### Scenario: Shift move produces a queued command
- **WHEN** the player issues move intent while Shift is held or an equivalent queue modifier is active
- **THEN** the UE generator MUST mark the event as queued/append
- **AND** the receiver MUST preserve the command as an ordered queued move rather than replacing the current order
- **AND** non-Shift move MUST remain the default replacement/current-order behavior

#### Scenario: Attack event targets an authoritative entity
- **WHEN** the player selects attack intent on a visible unit or tower actor
- **THEN** the UE generator MUST include the target entity id/gen observed from the latest frame
- **AND** the event MUST be rejected before submission if the target has no valid entity identity
- **AND** UE MUST NOT infer combat resolution locally

#### Scenario: Attack-move event carries queue semantics
- **WHEN** the player arms attack-move intent and confirms a valid world point
- **THEN** the UE generator MUST produce an attack-move input event containing player id, backend-world target point, and queued/append flag
- **AND** the receiver MUST preserve attack-move as an attack-move `PlayerInput`, not downgrade it to move or attack-target

#### Scenario: Ability cast event supports target forms
- **WHEN** the player casts an ability from HUD, hotkey, or Blueprint
- **THEN** the UE generator MUST include ability catalog id or slot, player id, cast kind, and target payload
- **AND** target payload MUST support at least no-target/self, target point, and target entity forms
- **AND** invalid ability id, unavailable slot, or non-finite target coordinates MUST be rejected with a diagnostic status

#### Scenario: Ability upgrade event is distinct from ability cast
- **WHEN** the player requests an ability upgrade from HUD, Shift-hotkey, Blueprint, or C++ call
- **THEN** the UE generator MUST produce an upgrade-ability input event containing player id and ability slot/index
- **AND** the receiver MUST NOT interpret the event as a cast and MUST preserve the upgrade ability action kind

#### Scenario: Item use event supports optional targets
- **WHEN** the player uses an item from hotbar, HUD, Blueprint, or C++ call
- **THEN** the UE generator MUST produce an item-use input event containing player id, inventory slot, and optional point or entity target payload
- **AND** invalid item slot or invalid target payload MUST be rejected before enqueue

#### Scenario: Tower placement event uses catalog ids and world point
- **WHEN** the player confirms a tower placement
- **THEN** the UE generator MUST include tower catalog id, player id, backend-world placement point, and optional placement path/slot metadata
- **AND** Rust/local replica MUST be the authority for placement validity, cost, collision, blocked region, and final spawn result
- **AND** UE MAY show preview visuals but MUST NOT create authoritative tower state before the frame result

#### Scenario: Tower target priority event uses authoritative selection
- **WHEN** the player changes target priority for a selected tower
- **THEN** the UE generator MUST include player id, selected tower entity id/gen, and the requested target priority enum
- **AND** stale tower selection, non-owned tower, or invalid priority MUST be rejected or reported before enqueue when detectable from the latest frame/catalog
- **AND** backend/local replica MUST remain the final authority for ownership and priority application

#### Scenario: UI event guard prevents duplicate world events
- **WHEN** HUD or Blueprint controls generate an ability/tower/start-round event
- **THEN** the same pointer/key action MUST NOT also generate move, attack, or selection world events
- **AND** the generator MUST expose a guard/consumed flag or equivalent route to prevent duplicate submission

### Requirement: Bridge/Rust gameplay input event receiver
`om-bridge` SHALL provide a Rust-side gameplay input event receiver that accepts UE-generated input events through C ABI, validates the payload, converts it to shared `PlayerInput`, assigns or returns input ids, and enqueues the result for the lockstep/local replica pipeline. Receiver behavior SHALL be deterministic with respect to accepted commands and SHALL report rejected commands without changing gameplay state.

#### Scenario: Receiver validates event shape
- **WHEN** UE submits a gameplay input event
- **THEN** the receiver MUST validate struct size, ABI version, event kind, player id, catalog/entity ids, flags, slot/path values, and finite coordinates
- **AND** malformed events MUST return an error status and MUST NOT enter the lockstep input queue

#### Scenario: Receiver converts all required events
- **WHEN** valid move, queued move, attack move, attack target, cast ability, upgrade ability, item use, place tower, upgrade/sell tower, set tower target priority, or start-round events are submitted
- **THEN** the receiver MUST convert them to the shared Rust `PlayerInput` representation used by the backend/local replica
- **AND** the conversion MUST preserve action kind, target payload, queue/Shift flag, and player id

#### Scenario: Receiver returns acknowledgement metadata
- **WHEN** an input event is accepted
- **THEN** the receiver MUST return or publish input id, accepted status, observed tick, target tick, and queue length
- **AND** future frames or diagnostics MUST expose enough applied input metadata for UE UI to correlate input latency and success/failure

#### Scenario: Receiver is safe across runtime lifecycle
- **WHEN** UE submits input before runtime start, after stop, during disconnect, or while the input queue is full
- **THEN** the receiver MUST return a clear status such as not-started, disconnected, busy, or invalid state
- **AND** the receiver MUST NOT panic, block indefinitely, or mutate stale runtime memory

### Requirement: Gameplay input event verification
The implementation SHALL include verification for the UE event generator contract, bridge/Rust receiver validation, conversion to shared `PlayerInput`, Shift queue semantics, duplicate UI guard behavior, and rejection diagnostics.

#### Scenario: Synthetic UE-to-Rust input smoke covers core actions
- **WHEN** automated smoke submits synthetic UE input events
- **THEN** move, Shift move, attack move, attack target, point ability cast, entity ability cast, no-target ability cast, ability upgrade, item use, tower placement, tower upgrade, tower sell, target priority change, and start round MUST each reach the receiver and produce accepted or authoritative rejection statuses
- **AND** accepted events MUST appear in applied input metadata or equivalent diagnostics

#### Scenario: Invalid input smoke does not mutate state
- **WHEN** automated smoke submits invalid ids, stale entity gen, invalid slot/path, non-finite coordinates, or unsupported event kind
- **THEN** the receiver MUST reject the event
- **AND** no corresponding `PlayerInput` MUST be enqueued
