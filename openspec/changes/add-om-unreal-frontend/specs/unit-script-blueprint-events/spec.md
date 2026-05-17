## ADDED Requirements

### Requirement: All `UnitScript` event hooks have UE visual event kinds
`Om UE frontend` SHALL mirror every event hook in `scripts/script-abi/src/script.rs::UnitScript` to a UE visual event kind. The metadata/accessor methods `unit_id()` and `tower_metadata()` SHALL NOT be treated as events; their data SHALL be exposed through catalog/registry metadata instead. Mirrored events SHALL be visual-only and SHALL NOT affect gameplay state.

The mirrored hook set SHALL include `on_spawn`, `on_tick`, `on_death`, `on_damage_taken`, `on_damage_dealt`, `on_skill_cast`, `on_attack_hit`, `on_attack_start`, `on_attack_landed`, `on_attack_fail`, `on_attacked`, `on_health_gained`, `on_mana_gained`, `on_spent_mana`, `on_heal_received`, `on_state_changed`, `on_modifier_added`, `on_modifier_removed`, `on_order`, and `on_respawn`.

#### Scenario: Generated event enum contains all UnitScript hooks
- **WHEN** generated C++ and C ABI event kind definitions are inspected
- **THEN** every `UnitScript` event hook listed above MUST have a corresponding stable event kind
- **AND** `unit_id` and `tower_metadata` MUST NOT appear as visual event kinds

#### Scenario: New UnitScript hook causes verification failure
- **WHEN** `UnitScript` adds a new event hook but `Om UE frontend` generated mapping is not updated
- **THEN** codegen or verification MUST fail
- **AND** the failure MUST name the missing hook

### Requirement: Runtime captures UnitScript hook invocations as render-only cues
The bridge/runtime SHALL capture `UnitScript` hook invocations at the script dispatch boundary and publish them as render-only `UnitScriptEvent` cues. These cues SHALL NOT be part of gameplay hashing, SHALL NOT call UE APIs from Rust worker threads, and SHALL NOT allow Blueprint to influence script hook results.

#### Scenario: Spawn hook publishes cue
- **WHEN** runtime invokes `UnitScript::on_spawn` for entity 7
- **THEN** a render-only UnitScript event cue with kind `OnSpawn` MUST be published for entity 7
- **AND** UE Blueprint event dispatch MUST happen later on the UE game thread

#### Scenario: Damage hook publishes final visual payload
- **WHEN** runtime invokes `UnitScript::on_damage_taken` and the hook mutates `DamageInfo`
- **THEN** the visual event payload MUST include the gameplay-accepted final damage info available after the hook
- **AND** Blueprint MUST NOT be able to mutate `DamageInfo` or feed changes back into runtime

#### Scenario: Script event cues do not alter determinism
- **WHEN** UnitScript visual cue capture is enabled
- **THEN** deterministic gameplay state and state hash MUST be identical to the same simulation without UE visual consumers
- **AND** dropping or coalescing visual cues MUST NOT affect gameplay results

### Requirement: C ABI exposes UnitScript event arrays in frames
`Frame` SHALL expose a borrowed array of UnitScript event cues. Each cue SHALL use FFI-safe payloads with fixed-width scalars, entity id/gen pairs, string-table references, optional target data, and explicit event kind. Cue data SHALL remain valid for the frame lease duration.

#### Scenario: Frame contains UnitScript event array
- **WHEN** one or more UnitScript hooks are captured before frame publication
- **THEN** `Frame` MUST expose `unit_script_event_count` and `unit_script_events`
- **AND** each event MUST include event kind, tick, sequence, primary entity id/gen, and hook-specific payload fields

#### Scenario: Hook string fields use string table references
- **WHEN** a UnitScript event carries `skill_id`, `state_id`, `modifier_id`, or `order_kind`
- **THEN** event payload MUST reference frame string table data by offset/length or equivalent FFI-safe reference
- **AND** UE MUST NOT receive raw Rust `RStr` or Rust-owned temporary pointers

#### Scenario: Empty UnitScript events are valid
- **WHEN** no UnitScript hook was captured for a frame
- **THEN** `unit_script_event_count` MUST be zero
- **AND** `unit_script_events` MAY be null

### Requirement: High-frequency `on_tick` visual events are coalesced safely
`on_tick` SHALL have a Blueprint event surface like other UnitScript hooks. To protect UE frame time, the bridge MAY coalesce multiple `on_tick` invocations for the same entity between UE frame publications into a single visual event that carries accumulated delta time, hook count, first tick, and latest tick. Non-tick UnitScript events SHALL NOT be coalesced unless the event payload explicitly carries count/repetition metadata.

#### Scenario: on_tick coalesces within one UE frame
- **WHEN** entity 7 receives three `on_tick` hook invocations before the next UE frame acquire
- **THEN** bridge MAY publish one `OnTick` visual event for entity 7
- **AND** payload MUST include accumulated `dt`, hook count `3`, first tick, and latest tick

#### Scenario: Discrete attack events remain distinct
- **WHEN** entity 7 emits two `on_attack_landed` hooks in the same publication window
- **THEN** bridge MUST publish two distinct attack landed events
- **AND** UE dispatch order MUST preserve simulation order for those events

### Requirement: Generated C++ classes expose Blueprint events matching UnitScript hooks
`om-codegen` SHALL generate Blueprint-visible event declarations for every mirrored UnitScript hook on generated unit classes or their base classes. Event payloads SHALL be UHT-visible structs, not raw FFI pointers. Blueprint subclasses SHALL be able to override these events for visual effects.

#### Scenario: Blueprint can override attack landed
- **WHEN** designer opens a Blueprint inheriting `AOmHeroSaikaMagoichi`
- **THEN** a Blueprint event corresponding to `UnitScript::on_attack_landed` MUST be available
- **AND** the event payload MUST expose attacker, victim, damage, tick, sequence fields, and action/attack instance id when available

#### Scenario: Blueprint can override modifier events
- **WHEN** a unit Blueprint handles modifier changes
- **THEN** generated events for `on_modifier_added` and `on_modifier_removed` MUST be available
- **AND** payload MUST include target entity id/gen and modifier id

#### Scenario: Blueprint can override order event
- **WHEN** a player order is mirrored from `UnitScript::on_order`
- **THEN** generated Blueprint event MUST expose order kind and target payload
- **AND** Blueprint MUST be able to play selection/order feedback without mutating the command

### Requirement: UnitScript Blueprint events are visual-only
Blueprint events generated from UnitScript hooks SHALL be presentation hooks only. They SHALL NOT expose mutable runtime references, `GameWorldDyn`, `DamageInfo` mutable references, or any API that changes deterministic gameplay state. Gameplay commands SHALL continue to be submitted through bridge input APIs.

#### Scenario: Blueprint receives copies/projections only
- **WHEN** `OnScriptDamageTaken` is dispatched
- **THEN** Blueprint payload MUST be a copy/projection of visual data
- **AND** it MUST NOT expose mutable `DamageInfo` or Rust pointers

#### Scenario: Blueprint effect does not change gameplay
- **WHEN** Blueprint plays VFX in response to `OnScriptAttackFail`
- **THEN** gameplay hit/miss result MUST already be decided by Rust runtime
- **AND** Blueprint execution MUST NOT alter the authoritative outcome

### Requirement: UnitScript event verification
The implementation SHALL include verification for hook coverage, C ABI payload safety, Blueprint event availability, ordering, `on_tick` coalescing, and determinism isolation.

#### Scenario: Hook coverage test passes
- **WHEN** tests inspect `UnitScript` hook names and generated UnitScript event mapping
- **THEN** all event hooks MUST be covered
- **AND** metadata/accessor methods MUST be excluded

#### Scenario: UE smoke receives script event
- **WHEN** TD_1 or a synthetic runtime triggers `on_attack_start` and `on_attack_landed`
- **THEN** UE test Blueprint or native test class MUST receive corresponding visual events
- **AND** event payload MUST reference the expected entity ids

#### Scenario: Determinism smoke is unchanged
- **WHEN** UnitScript visual cue capture is enabled in bridge
- **THEN** existing deterministic sim verification MUST still pass
- **AND** disabling UE visual consumption MUST not change script hook gameplay results
