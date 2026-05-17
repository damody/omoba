## ADDED Requirements

### Requirement: Rust publishes per-entity animation state
`Om UE frontend` SHALL publish a Rust-authored animation state for every visible animated unit that needs AnimBP control. This state SHALL represent gameplay-derived animation intent and timing, not UE-inferred guesses. UE SHALL consume this state to drive AnimBP variables, state machines, montages, or Blueprint animation events.

#### Scenario: Frame includes animation state for a hero
- **WHEN** a frame contains animated hero `saika_magoichi`
- **THEN** the frame MUST expose an animation state record for that entity
- **AND** the record MUST include entity id/gen, locomotion state, locomotion variant, animation overlay/stance, action state, action instance id, animation tag/catalog id, play rate, tick, and sequence

#### Scenario: Idle variants are explicit
- **WHEN** Rust selects an idle variant such as `stand_1`, `stand_2`, or `stand_3`
- **THEN** the animation state MUST expose that variant through a stable catalog/string id or FName-compatible value
- **AND** UE AnimBP MUST NOT need to choose the idle variant from random local state unless configured as a purely visual override

#### Scenario: Walk state is not guessed from UE transform alone
- **WHEN** runtime determines a unit is walking or moving toward a target
- **THEN** animation state MUST expose a locomotion state such as `Walk`
- **AND** UE MAY use velocity for blend speed, but MUST NOT rely on velocity alone to decide authoritative locomotion category

### Requirement: Buffs and modifiers can drive animation overlays
Active buffs and modifiers SHALL be able to affect animation state through deterministic animation overlays and variants. Rust runtime/bridge SHALL resolve the final overlay and locomotion/action variants from authoritative buff/modifier state before publishing `AnimationState`. UE AnimBP SHALL consume the resolved state and SHALL NOT need to inspect buff lists to choose gameplay-dependent animation variants.

#### Scenario: sniper_mode changes walk animation
- **WHEN** hero `saika_magoichi` is moving while buff `sniper_mode` is active
- **THEN** animation state MUST still expose locomotion state `Walk`
- **AND** animation state MUST expose an animation overlay or stance such as `sniper_mode`
- **AND** animation state MUST expose a locomotion variant such as `sniper_walk`
- **AND** UE AnimBP MUST be able to transition to the sniper-mode walk state without inferring it from raw buff events

#### Scenario: Removing sniper_mode returns to normal walk
- **WHEN** `sniper_mode` expires or is removed while the hero is walking
- **THEN** the next compatible animation state MUST remove the `sniper_mode` overlay
- **AND** locomotion variant MUST return to the normal walk variant unless another active buff/modifier overrides it

#### Scenario: Multiple animation overlays resolve deterministically
- **WHEN** multiple active buffs or modifiers declare animation overlays for the same locomotion/action slot
- **THEN** Rust bridge MUST resolve the winning overlay using deterministic priority and tie-break rules from catalog metadata
- **AND** UE MUST receive only the resolved overlay/variant for the current frame

### Requirement: Attack animation exposes windup, impact, and recovery phases
Attack animation timing SHALL be represented explicitly. Rust runtime SHALL publish attack action state and attack phase data derived from authoritative combat timing. UE SHALL use this to drive attack and CriticalAttack animation state machines without changing gameplay hit timing.

#### Scenario: Normal attack enters windup
- **WHEN** a unit starts an accepted normal attack
- **THEN** animation state MUST set action state to `Attack`
- **AND** attack phase MUST be `Windup`
- **AND** phase duration/progress MUST indicate the time until the impact or hit decision tick
- **AND** action instance id MUST change from the previous completed action

#### Scenario: Critical attack uses distinct action state or flag
- **WHEN** gameplay determines the attack is critical before or at the attack action publication point
- **THEN** animation state MUST expose `CriticalAttack` as the action state or set an explicit critical flag with a critical animation tag
- **AND** UE AnimBP MUST be able to transition to a distinct CriticalAttack state or montage section

#### Scenario: Impact aligns with gameplay hit
- **WHEN** gameplay reaches the hit/impact tick for an attack
- **THEN** animation state MUST expose attack phase `Impact` for the corresponding action instance
- **AND** associated `UnitScript::on_attack_landed` or attack-hit visual cue MUST reference the same entity/action context when available
- **AND** UE AnimNotify MUST NOT be required to decide whether the hit occurred

#### Scenario: Recovery remains visible after impact
- **WHEN** an attack has resolved but the unit is still in backswing/recovery
- **THEN** animation state MUST expose attack phase `Recovery`
- **AND** phase progress MUST advance until the unit can transition back to locomotion, idle, or the next action

### Requirement: Animation state is FFI-safe and lease-scoped
The C ABI frame SHALL expose animation states as borrowed arrays with fixed-width scalar fields, catalog ids, and string-table references. Animation state memory SHALL remain valid for the frame lease duration and SHALL NOT expose Rust containers or Rust enum layout.

#### Scenario: Frame exposes animation state array safely
- **WHEN** a frame contains one or more animation states
- **THEN** `Frame` MUST expose animation state count and pointer fields
- **AND** each record MUST use FFI-safe scalar fields for locomotion/action/phase, locomotion variant, animation overlay, entity refs, ids, timing, progress, flags, and play rate

#### Scenario: Empty animation states are valid
- **WHEN** no animated entity exists in a frame
- **THEN** animation state count MUST be zero
- **AND** animation state pointer MAY be null

### Requirement: Lua metadata maps animation states to UE animation bindings
Lua content MAY define `ue.animation` metadata for units, heroes, towers, summons, and creeps. Buff and modifier metadata MAY define `ue.animation_overlay` or equivalent animation override metadata. This metadata SHALL describe idle variants, locomotion variants, overlay priorities, AnimBP variable names, state machine state names, montage or section soft paths, attack phase names, critical attack mapping, default play rate, and fallback behavior. When metadata is absent, `Om UE frontend` SHALL still provide generic animation state variables and fallback values.

#### Scenario: Hero defines stand variants
- **WHEN** hero Lua metadata declares `ue.animation.idle_variants = { "stand_1", "stand_2", "stand_3" }`
- **THEN** generated catalog metadata MUST include those variants
- **AND** UE AnimBP adapter MUST be able to expose the selected variant to Blueprint or AnimInstance variables

#### Scenario: Attack phase mapping is generated
- **WHEN** Lua metadata declares attack phase names or montage sections for windup, impact, and recovery
- **THEN** generated registry/catalog MUST expose the mapping for the unit content id
- **AND** UE runtime MUST be able to resolve it without per-frame string parsing

#### Scenario: Buff declares animation overlay
- **WHEN** buff `sniper_mode` declares an animation overlay with `walk = "sniper_walk"` and priority metadata
- **THEN** generated catalog metadata MUST include that overlay and locomotion override
- **AND** bridge animation state derivation MUST be able to resolve it while the buff is active

#### Scenario: Reloadable animation metadata updates without UE rebuild
- **WHEN** development reload changes idle variant names, overlay mappings, locomotion variant names, montage soft paths, play rate, or AnimBP variable mapping without changing UHT-visible payload shape
- **THEN** Runtime Lua reload MUST treat the change as reloadable
- **AND** UE MUST invalidate animation mapping caches for the new catalog generation

### Requirement: UE applies animation state to AnimBP deterministically
UE runtime SHALL provide an AnimBP adapter on the generated actor base or a dedicated component. The adapter SHALL copy frame animation state into AnimInstance-readable values and dispatch optional Blueprint animation events on the UE game thread. It SHALL avoid retriggering one-shot montage/section transitions every frame.

#### Scenario: AnimBP variables are updated from frame state
- **WHEN** UE actor receives animation state `Stand` with variant `stand_2`
- **THEN** adapter MUST update AnimBP-readable values for locomotion state and idle variant
- **AND** an AnimBP state machine can transition to the corresponding stand state without parsing raw FFI pointers

#### Scenario: AnimBP receives sniper walk overlay
- **WHEN** UE actor receives animation state `Walk` with overlay `sniper_mode` and locomotion variant `sniper_walk`
- **THEN** adapter MUST update AnimBP-readable values for locomotion state, animation overlay, and locomotion variant
- **AND** an AnimBP state machine can transition to the sniper walk state without querying active buff components

#### Scenario: Attack action triggers once per instance
- **WHEN** animation state action instance id changes and action state is `Attack`
- **THEN** UE adapter MAY trigger attack montage or Blueprint event once for that instance
- **AND** subsequent frames with the same action instance id MUST update phase/progress without restarting the action

#### Scenario: Phase transition is observable
- **WHEN** attack phase changes from `Windup` to `Impact` or from `Impact` to `Recovery`
- **THEN** UE adapter MUST expose the phase transition to Blueprint or AnimBP
- **AND** phase progress MUST be available for blends, trails, weapon visibility, or timing visual effects

### Requirement: Animation verification covers state and attack timing
The implementation SHALL include tests and smoke scenarios for animation state publication, FFI layout, generated metadata, AnimBP variable updates, action instance gating, and attack windup/impact/recovery timing.

#### Scenario: Rust projection test covers attack phases
- **WHEN** bridge animation projection tests run
- **THEN** tests MUST cover Stand variant, Walk, sniper_mode walk overlay, Attack windup, Impact, Recovery, CriticalAttack, action instance id changes, and empty arrays

#### Scenario: UE smoke drives AnimBP state machine
- **WHEN** a synthetic or TD_1 smoke drives `saika_magoichi` through stand, walk, sniper_mode walk, attack, and critical attack
- **THEN** UE test actor or Blueprint MUST observe the expected AnimBP variables or events
- **AND** attack windup and recovery phase progress MUST be monotonic for the same action instance
