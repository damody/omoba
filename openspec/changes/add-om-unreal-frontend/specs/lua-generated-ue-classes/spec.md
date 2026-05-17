## ADDED Requirements

### Requirement: Rust codegen reads Lua content and emits UE C++ source
系統 SHALL 提供 Rust code generator（暫定 `om-codegen`），讀取 `scripts/lua_data` 的同一份 Lua content manifest，並產生 Unreal Header Tool 可編譯的 C++ headers/sources。Generator SHALL use Lua content as the source of truth for hero、tower、ability、buff、summon、creep、projectile visual class declarations、animation metadata and registry entries，類似 `omoba-template-ids` 從 Lua 產生 Rust ids 與 metadata。

#### Scenario: Codegen command produces generated source
- **WHEN** 開發者執行 documented `om-codegen` command
- **THEN** generator MUST read `scripts/lua_data/templates.lua` and included Lua files
- **AND** generator MUST write generated C++ source under `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmGenerated`
- **AND** generated files MUST be marked as auto-generated and not hand-editable

#### Scenario: Generated source is deterministic
- **WHEN** Lua content and generator version are unchanged
- **THEN** repeated codegen runs MUST produce byte-for-byte identical generated C++ files and manifest
- **AND** output ordering MUST follow deterministic content id ordering compatible with `omoba-template-ids`

#### Scenario: Codegen tracks Lua dependencies
- **WHEN** an included Lua file changes
- **THEN** codegen freshness metadata MUST change
- **AND** verification MUST be able to detect stale generated C++ output

### Requirement: Generated UE classes exist per content item
For each active Lua content item that needs UE visual interaction, generator SHALL produce a stable UHT-visible C++ class. Generated classes SHALL inherit from hand-written UE base classes and SHALL be `Blueprintable` so designers can create Blueprint subclasses in UE Editor. Generated class names SHALL be deterministic, sanitized, collision-checked, and stable unless the Lua id changes.

#### Scenario: Hero generates Blueprintable C++ class
- **WHEN** Lua manifest contains active hero id `saika_magoichi`
- **THEN** generator MUST emit a UHT-visible C++ class such as `AOmHeroSaikaMagoichi`
- **AND** the class MUST inherit from the hand-written hero base class
- **AND** the class MUST be usable as a Blueprint parent in UE Editor

#### Scenario: Tower generates Blueprintable C++ class
- **WHEN** Lua manifest contains active tower id `tower_dart`
- **THEN** generator MUST emit a UHT-visible C++ class such as `AOmTowerDart`
- **AND** the class MUST expose tower-specific visual events and metadata accessors

#### Scenario: Ability generates visual binding class
- **WHEN** Lua manifest contains active ability id `sniper_mode`
- **THEN** generator MUST emit a UHT-visible class such as `UOmAbilitySniperModeVisual`
- **AND** the class MUST be usable by hero/tower actor logic to dispatch ability-specific visual cues

#### Scenario: Buff generates visual binding class or registry entry
- **WHEN** Lua manifest contains active buff id `slow`
- **THEN** generator MUST emit a UHT-visible visual binding class or generated registry entry for that buff
- **AND** the generated output MUST allow Blueprint code to handle buff added, removed, refreshed, and updated events for that buff

#### Scenario: Tombstone does not generate usable class
- **WHEN** Lua content marks an entry as `tombstone = true`
- **THEN** generator MUST preserve id ordering where needed
- **AND** generator MUST NOT emit a Blueprintable runtime class for that tombstoned entry

### Requirement: Lua `ue` metadata controls class and Blueprint binding
Lua content MAY define an optional `ue` section for each supported content item. This metadata SHALL allow declaring generated class name override, default Blueprint soft class path, fallback visual asset hints, visual event bindings, `ue.animation` bindings, and editor category metadata. When `ue` metadata is absent, generator SHALL produce deterministic defaults.

#### Scenario: Explicit Blueprint path is emitted
- **WHEN** hero Lua metadata contains `ue.blueprint = "/Game/Generated/Heroes/BP_SaikaMagoichi.BP_SaikaMagoichi_C"`
- **THEN** generated registry MUST store that soft class path for the hero content id
- **AND** runtime auto-load MUST use that path before falling back to native generated class

#### Scenario: Missing ue metadata gets default path
- **WHEN** tower Lua metadata has no `ue` section
- **THEN** generator MUST derive a deterministic generated class name and default Blueprint soft class path from the content id
- **AND** generated registry MUST still contain an entry for that tower

#### Scenario: Invalid class or asset path fails generation
- **WHEN** Lua `ue` metadata declares an invalid C++ identifier, duplicate generated class name, parent traversal path, non-content soft path, or unsupported event binding
- **THEN** codegen MUST fail with a diagnostic naming the content id and invalid field

#### Scenario: Animation metadata is emitted
- **WHEN** hero Lua metadata declares `ue.animation` with idle variants, AnimBP variable mapping, attack phase mapping, or montage section paths
- **THEN** generated registry/catalog metadata MUST include those animation bindings for the hero content id
- **AND** generated C++/Blueprint surface MUST expose animation state payloads without requiring raw FFI access

#### Scenario: Buff animation overlay metadata is emitted
- **WHEN** buff Lua metadata declares `ue.animation_overlay` for `sniper_mode` with a walk override such as `sniper_walk`
- **THEN** generated registry/catalog metadata MUST include the overlay, priority, and locomotion override
- **AND** runtime animation state derivation MUST be able to reference that metadata while the buff is active

### Requirement: Generated classes expose typed Blueprint visual events
Generated C++ classes SHALL expose typed Blueprint events for visual interaction. Events SHALL receive UHT-visible `USTRUCT(BlueprintType)` payloads generated from the frame/cue contract, not raw FFI pointers. Blueprint events SHALL include animation state/attack phase payloads, buff lifecycle events for add/remove/refresh/update and every mirrored `UnitScript` event hook. Blueprint events SHALL be visual-only and SHALL NOT provide direct mutation access to lockstep gameplay state.

#### Scenario: Frame update event is Blueprint-overridable
- **WHEN** UE actor for a generated hero receives new frame data
- **THEN** generated class MUST provide a Blueprint-overridable event such as `OnFrameState`
- **AND** event payload MUST include typed position, facing, HP, owner, tick, sequence, and content id fields

#### Scenario: Attack and ability cues dispatch to typed events
- **WHEN** frame contains an attack phase, tower fire, ability cast, buff, or projectile visual cue for a generated content id
- **THEN** UE dispatcher MUST call the corresponding generated C++/Blueprint event with a typed payload
- **AND** Blueprint MUST NOT need to parse raw JSON, protobuf bytes, or raw FFI frame pointers

#### Scenario: Animation state dispatches to typed payload
- **WHEN** frame contains animation state for a generated hero or creep
- **THEN** UE dispatcher or AnimBP adapter MUST receive a typed `USTRUCT(BlueprintType)` animation payload
- **AND** payload MUST include locomotion state, locomotion variant, animation overlay, idle variant, action state, attack phase, action instance id, phase progress, critical flag, and play rate
- **AND** Blueprint MUST NOT need to parse raw FFI animation records

#### Scenario: Buff lifecycle events dispatch to typed events
- **WHEN** frame diff produces `BuffAdded` or `BuffRemoved` for a generated buff id
- **THEN** UE dispatcher MUST call generated C++/Blueprint buff lifecycle event with a typed payload
- **AND** payload MUST include target entity id/gen, buff id, visual instance key, remaining seconds, reason, and payload summary

#### Scenario: UnitScript hook events dispatch to typed events
- **WHEN** frame contains a captured `UnitScript` event cue such as `on_attack_start`, `on_damage_taken`, or `on_order`
- **THEN** UE dispatcher MUST call the matching generated C++/Blueprint event
- **AND** payload MUST be UHT-visible and hook-specific
- **AND** Blueprint MUST NOT receive raw Rust pointers or mutable gameplay objects

#### Scenario: Blueprint cannot mutate gameplay state through visual event
- **WHEN** Blueprint handles a generated visual event
- **THEN** payload MUST be read-only from gameplay perspective
- **AND** any gameplay command MUST still go through bridge input submission APIs

### Requirement: Generated classes expose native C++ readable metadata and handlers
Generated C++ SHALL be usable directly from UE C++ without requiring Blueprint inheritance. For each generated content class, generator SHALL emit C++ readable metadata accessors, UHT-visible payload structs, and native virtual or `BlueprintNativeEvent` handlers. Blueprint override MAY sit on top of the native handler, but C++ MUST be able to subscribe to and process the same typed events.

#### Scenario: C++ reads hero and ability metadata
- **WHEN** UE C++ code uses `AOmHeroSaikaMagoichi`
- **THEN** it MUST be able to read hero metadata including content id, display name, title, base stats, render model, muzzle bone, animation source metadata, and ability slots
- **AND** it MUST be able to read metadata for `sniper_mode`, `saika_reinforcements`, `rain_iron_cannon`, and `three_stage_technique`, including ability type, target type, cast type, max level, cooldown, mana cost, cast time, range, and typed extras

#### Scenario: C++ handles generated events natively
- **WHEN** an ability, buff, animation, attack phase, or UnitScript event is dispatched to a generated actor
- **THEN** generated C++ MUST call a native handler or `BlueprintNativeEvent` with a typed payload before or alongside Blueprint dispatch
- **AND** C++ subclasses MUST be able to override the handler without using Blueprint graphs
- **AND** the payload MUST NOT expose raw FFI pointers after the frame lease is released

### Requirement: Generated registry maps content ids to native and Blueprint classes
Generator SHALL emit a generated content registry that maps stable content ids and numeric ids to display metadata, generated native class, default Blueprint soft class path, fallback class, and supported event/cue capabilities. Runtime SHALL use this registry to spawn or attach the correct visual class for each frame entity/cue.

#### Scenario: Registry resolves entity class by content id
- **WHEN** runtime sees a frame entity with hero or tower content id
- **THEN** UE runtime MUST resolve that content id through generated registry
- **AND** registry MUST return the generated native class and configured Blueprint soft class path

#### Scenario: Unknown content id uses fallback
- **WHEN** frame references a content id absent from generated registry
- **THEN** UE runtime MUST log the unknown id
- **AND** MUST use an explicit generic fallback actor/component instead of crashing

#### Scenario: Registry exposes content metadata
- **WHEN** UI or actor code queries generated registry for a known tower, hero, or ability id
- **THEN** registry MUST expose display name, generated class name, Blueprint soft path, and supported visual event flags

#### Scenario: Registry resolves buff visual metadata
- **WHEN** UI or actor code queries generated registry for a known buff id
- **THEN** registry MUST expose display name, generated visual class or generic binding, Blueprint soft path when configured, and supported buff lifecycle event flags

#### Scenario: Registry resolves animation metadata
- **WHEN** actor or AnimBP adapter queries generated registry for a known hero, tower, creep, or summon id
- **THEN** registry MUST expose animation metadata such as idle variants, locomotion variants, overlay names, state names, attack phase mapping, critical attack binding, and montage/section soft paths when configured

### Requirement: Runtime auto-loads Blueprint subclasses on initialization and spawn
During initialization and entity spawn, `Om UE frontend` runtime SHALL automatically load Blueprint subclasses declared by the generated registry. If a Blueprint subclass exists, runtime SHALL spawn/use it. If it is missing or fails to load, runtime SHALL fall back to the generated native C++ class and report a diagnostic.

#### Scenario: Hero Blueprint loads automatically
- **WHEN** frame first contains hero `saika_magoichi` and registry has Blueprint path `BP_SaikaMagoichi`
- **THEN** UE runtime MUST load that Blueprint class
- **AND** MUST spawn or assign an actor of the Blueprint class for that entity

#### Scenario: Missing Blueprint falls back to generated native class
- **WHEN** registry Blueprint path does not resolve to a valid Blueprint generated class
- **THEN** UE runtime MUST log the missing path and content id
- **AND** MUST use the generated native class for that content id
- **AND** gameplay/render frame processing MUST continue

#### Scenario: Class load is cached
- **WHEN** multiple entities share the same content id
- **THEN** UE runtime MUST cache the resolved Blueprint or fallback class
- **AND** MUST NOT reload the same soft class path every frame

### Requirement: Blueprint inheritance workflow is editor-friendly
Generated classes SHALL be stable enough for Blueprint assets to inherit from them across codegen runs. Generator SHALL avoid deleting or renaming generated classes unless the underlying Lua id or explicit `ue.class` changes. Generated classes SHALL expose editor categories and Blueprint events with names that make the expected visual override points discoverable.

#### Scenario: Existing Blueprint parent remains valid after unchanged codegen
- **WHEN** developer created `BP_SaikaMagoichi` inheriting `AOmHeroSaikaMagoichi`
- **AND** Lua id and generated class name remain unchanged
- **WHEN** codegen runs again
- **THEN** generated class name and module path MUST remain stable
- **AND** Blueprint parent MUST remain valid in UE Editor

#### Scenario: Blueprint sees visual override events
- **WHEN** designer opens a Blueprint inheriting a generated hero or tower class
- **THEN** visual events declared for that content type, including mirrored `UnitScript` hook events, MUST be visible as Blueprint implementable/native events
- **AND** event parameter structs MUST be Blueprint-visible

### Requirement: Generated code build and freshness verification
The implementation SHALL include verification that generated C++ is fresh, UHT-compatible, and aligned with Lua content. UE module build SHALL compile generated classes when UE 5.7 is configured. Rust tests SHALL cover name sanitization, duplicate detection, default Blueprint path derivation, and Lua metadata validation.

#### Scenario: Stale generated C++ fails verification
- **WHEN** Lua content changes but generated C++ was not regenerated
- **THEN** freshness check MUST fail or report stale generated code
- **AND** failure MUST identify the codegen command needed to refresh output

#### Scenario: UE module compiles generated classes
- **WHEN** UE 5.7 path resolves to default `D:\UE5.7` or a configured override and generated C++ is fresh
- **THEN** UE build verification MUST compile `OmGenerated` classes through Unreal Header Tool
- **AND** generated `.generated.h` include ordering MUST be valid

#### Scenario: Codegen tests validate identifiers
- **WHEN** `om-codegen` tests run
- **THEN** tests MUST cover class-name sanitization, duplicate class names, tombstone behavior, Blueprint path validation, and default metadata generation
