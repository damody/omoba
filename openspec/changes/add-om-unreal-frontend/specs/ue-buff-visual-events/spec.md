## ADDED Requirements

### Requirement: Buff state is exposed for all visible units
`om-bridge` SHALL expose active buff state for every visible entity that can carry buffs, not only the local hero HUD. Each buff snapshot SHALL identify target entity id/gen, buff id, stable buff catalog id where available, remaining seconds, payload reference, and a deterministic visual instance key.

#### Scenario: Active buff appears in frame
- **WHEN** local replica `BuffStore` contains buff `slow` on visible creep entity 42
- **THEN** the next published `Frame` MUST include a buff snapshot for entity 42 and buff `slow`
- **AND** the buff snapshot MUST include target entity id/gen, buff id or catalog id, remaining seconds, and visual instance key

#### Scenario: Permanent buff uses infinite sentinel
- **WHEN** a toggle or permanent buff is active
- **THEN** frame buff snapshot MUST expose `remaining_secs == -1.0`
- **AND** Blueprint event payload MUST expose it as permanent/toggle rather than a huge countdown value

#### Scenario: Entity removal clears buff snapshots
- **WHEN** entity 42 is removed from the frame
- **THEN** the bridge/UE lifecycle layer MUST treat all active buff snapshots for entity 42 as removed
- **AND** cleanup events MUST be generated for their visual instance keys

### Requirement: Buff lifecycle events are generated from authoritative buff diff
`Om UE frontend` SHALL generate visual buff lifecycle events by comparing the current active buff set with the previously processed frame. The lifecycle SHALL distinguish add, remove, refresh, and payload update. The source of truth SHALL be authoritative local replica buff state; explicit gameplay/VFX cues MAY supplement metadata but MUST NOT be the only source of cleanup.

#### Scenario: Buff added event
- **WHEN** buff key `(entity_id, entity_gen, buff_id)` is absent in previous processed frame and present in current frame
- **THEN** `Om UE frontend` MUST emit `BuffAdded`
- **AND** the event MUST include target entity id/gen, buff id, visual instance key, remaining seconds, payload summary, and tick/sequence

#### Scenario: Buff removed event
- **WHEN** buff key is present in previous processed frame and absent in current frame
- **THEN** `Om UE frontend` MUST emit `BuffRemoved`
- **AND** the event MUST include the same visual instance key used by the prior `BuffAdded`

#### Scenario: Buff refreshed event
- **WHEN** buff key is present in both frames and remaining duration increases without disappearing
- **THEN** `Om UE frontend` MUST emit `BuffRefreshed`
- **AND** it MUST NOT emit a remove/add pair for the same key

#### Scenario: Buff payload updated event
- **WHEN** buff key is present in both frames but visual payload hash or known visual fields change
- **THEN** `Om UE frontend` MUST emit `BuffUpdated`
- **AND** Blueprint MUST be able to update existing effect parameters instead of rebuilding the effect

### Requirement: C ABI exposes buff snapshots and buff events
The bridge C ABI SHALL expose buff snapshots and lifecycle event arrays as frame-owned borrowed memory. Buff event payloads SHALL be FFI-safe, use fixed-width scalars and string-table references, and remain valid for the frame lease duration. UE SHALL NOT parse raw `BuffStore` internals.

#### Scenario: Frame has buff event arrays
- **WHEN** a frame contains one or more buff lifecycle changes
- **THEN** `Frame` MUST expose buff event count and pointer
- **AND** each event MUST expose event kind, target entity id/gen, buff id/catalog id, instance key, reason, remaining seconds, and payload/string references

#### Scenario: Empty buff events are safe
- **WHEN** no buff lifecycle changes occurred in a frame
- **THEN** buff event count MUST be zero
- **AND** buff event pointer MAY be null
- **AND** UE consumer MUST treat this as valid empty data

#### Scenario: Payload is lease-scoped
- **WHEN** a buff event references payload JSON or visual payload text
- **THEN** the pointer/offset MUST remain valid until the frame lease is released
- **AND** UE MUST NOT retain the raw pointer after release

### Requirement: Generated Blueprint buff events manage add/remove effects
Generated UE C++ classes SHALL expose typed Blueprint events for buff lifecycle. Unit actor base classes SHALL provide an active buff effect map keyed by visual instance key, so Blueprint can create effects on add and remove exactly the matching effects on remove. Cleanup SHALL also run when an actor is despawned or destroyed.

#### Scenario: Blueprint adds effect on buff added
- **WHEN** `BuffAdded` for `sniper_mode` is dispatched to a hero Blueprint
- **THEN** generated class MUST call a Blueprint-visible event such as `OnBuffAdded`
- **AND** Blueprint MUST be able to create and attach Niagara/component/material/audio effects associated with the visual instance key

#### Scenario: Blueprint removes effect on buff removed
- **WHEN** `BuffRemoved` for the same visual instance key is dispatched
- **THEN** generated class MUST call a Blueprint-visible event such as `OnBuffRemoved`
- **AND** Blueprint or base class MUST remove/stop effects associated with that key

#### Scenario: Actor destruction cleans remaining buff effects
- **WHEN** a unit actor is despawned, destroyed, or recycled while it has active buff effects
- **THEN** base class MUST cleanup all active buff effect handles
- **AND** no Niagara/component/audio effect created for that actor's buffs SHOULD remain attached to stale actor state

#### Scenario: Refresh does not flicker effect
- **WHEN** `BuffRefreshed` is dispatched for an active buff
- **THEN** generated class MUST call refresh/update event
- **AND** base class MUST keep the existing effect handle active unless Blueprint explicitly replaces it

### Requirement: Lua buff metadata supports UE visual binding
Lua buff definitions MAY include optional `ue` metadata for visual class, Blueprint soft path, attach socket, default effect asset path, material parameter mapping, animation overlay mapping, stacking policy, and event options. Missing `ue` metadata SHALL still generate generic buff visual support.

#### Scenario: Buff Blueprint path is generated
- **WHEN** `templates/buffs.lua` entry `slow` declares `ue.blueprint`
- **THEN** `om-codegen` MUST include that Blueprint soft class path in generated buff registry
- **AND** runtime MUST use it to dispatch or instantiate buff visual logic

#### Scenario: Missing buff metadata uses generic visual
- **WHEN** buff `burn` has no `ue` section
- **THEN** generator MUST still expose buff id/display name and generic buff events
- **AND** runtime MUST NOT fail solely because no buff-specific visual class exists

#### Scenario: Buff metadata can define animation overlay
- **WHEN** buff `sniper_mode` declares `ue.animation_overlay` with a walk override such as `sniper_walk`
- **THEN** generator MUST include that overlay metadata in the generated buff/animation registry
- **AND** runtime animation state derivation MUST apply the overlay while `sniper_mode` is active
- **AND** buff visual add/remove events MUST remain separate from animation state selection

#### Scenario: Invalid buff visual metadata fails codegen
- **WHEN** buff `ue` metadata contains invalid class name, invalid soft path, unsupported attach policy, or duplicate generated class
- **THEN** codegen MUST fail with a diagnostic naming the buff id and invalid field

### Requirement: Buff visual events are visual-only
Buff Blueprint events SHALL be presentation-layer hooks only. They SHALL NOT allow Blueprint to mutate authoritative buff state, remaining time, gameplay stats, damage, movement, or lockstep simulation. Gameplay commands MUST continue to use bridge input APIs.

#### Scenario: Blueprint receives read-only buff payload
- **WHEN** Blueprint handles `OnBuffAdded` or `OnBuffRemoved`
- **THEN** payload MUST be a Blueprint-visible copy/projection of frame data
- **AND** Blueprint MUST NOT receive mutable pointers to Rust `BuffStore` or frame memory

#### Scenario: Gameplay state is unchanged by visual effect
- **WHEN** Blueprint creates or removes a Niagara effect for a buff
- **THEN** local replica gameplay state MUST remain determined only by Rust runtime and lockstep inputs
- **AND** visual effect lifecycle MUST NOT alter buff duration, stats, damage, movement, or targetability

### Requirement: Buff lifecycle verification
The implementation SHALL include tests or smoke scenarios for buff add, remove, refresh, update, entity despawn cleanup, missing Blueprint fallback, and codegen of buff visual metadata.

#### Scenario: Bridge diff test covers lifecycle
- **WHEN** Rust tests feed previous/current buff snapshots into the lifecycle diff
- **THEN** tests MUST verify added, removed, refreshed, updated, and owner-removed events

#### Scenario: Blueprint smoke covers add/remove
- **WHEN** UE smoke uses a generated buff visual Blueprint or test class
- **THEN** applying a buff MUST trigger add event and create a tracked effect
- **AND** removing or expiring the buff MUST trigger remove event and clear the tracked effect

#### Scenario: Refresh test avoids remove/add
- **WHEN** the same buff id refreshes duration on the same entity
- **THEN** verification MUST confirm `BuffRefreshed` occurs
- **AND** no `BuffRemoved`/`BuffAdded` pair occurs for that same key in that frame
