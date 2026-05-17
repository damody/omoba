## ADDED Requirements

### Requirement: SaikaMagoichi generated C++ exposes complete hero metadata
`om-codegen` SHALL treat `saika_magoichi` as the first complete hero integration target. Generated UE C++ SHALL expose `AOmHeroSaikaMagoichi` plus typed metadata for the hero, render assets, animation sources, muzzle bone, ability slots, and all Saika-specific runtime event payloads. UE C++ SHALL be able to read this data directly without parsing Lua, JSON, logs, raw FFI frame pointers, or Blueprint graphs.

#### Scenario: C++ reads Saika hero metadata
- **WHEN** UE C++ code queries `AOmHeroSaikaMagoichi` metadata
- **THEN** metadata MUST include hero id `saika_magoichi`, display name, title, base stats, attack range, movement speed, render model/texture, scale/orientation offsets, muzzle bone, and animation source names
- **AND** metadata MUST include animation entries for idle, idle_2, idle_3, move, attack, critical, and sniper when present in Lua content

#### Scenario: C++ reads Saika ability slot metadata
- **WHEN** UE C++ code queries Saika ability metadata
- **THEN** it MUST receive the Lua ability slot order `sniper_mode`, `saika_reinforcements`, `rain_iron_cannon`, `three_stage_technique`
- **AND** each ability entry MUST expose ability type, target type, cast type, max level, icon, display name, description, cooldown, mana cost, cast time, range, and typed extras for every level

### Requirement: Saika ability events are typed and native C++ dispatchable
Runtime ability events for SaikaMagoichi SHALL be published by the bridge and projected into typed UE C++ payloads. Generated C++ SHALL provide native handlers and BlueprintNativeEvent-compatible dispatch for each Saika ability. Events SHALL be visual/read-only and SHALL NOT allow UE to mutate deterministic gameplay state.

#### Scenario: sniper_mode toggle is readable and dispatchable
- **WHEN** Saika casts `sniper_mode` and the runtime toggles the buff on or off
- **THEN** UE C++ MUST receive a typed sniper-mode event containing caster entity id/gen, ability id, level, enabled state, buff id `sniper_mode`, visual instance key when available, and level extras for range bonus, damage bonus, attack speed penalty, move speed penalty, and accuracy bonus
- **AND** generated C++ MUST dispatch a handler such as `HandleSaikaSniperModeChanged`
- **AND** Blueprint MAY override the same event for scope glow, stance VFX, and sniper walk visual changes

#### Scenario: saika_reinforcements cast and summons are readable
- **WHEN** Saika casts `saika_reinforcements`
- **THEN** UE C++ MUST receive a typed event containing caster, ability level, target/facing data, summon count, rows/columns or formation metadata, duration, row/column spacing, front-row distance, and formation positions when available
- **AND** each spawned `saika_gunner` visual spawn MUST be linkable back to ability id `saika_reinforcements`, the summoner entity, and the corresponding cast/cue instance id
- **AND** generated C++ MUST dispatch a handler such as `HandleSaikaReinforcementsCast`

#### Scenario: rain_iron_cannon passive proc is readable
- **WHEN** Saika's `rain_iron_cannon` passive triggers from `on_attack_hit`
- **THEN** UE C++ MUST receive a typed event containing attacker, primary victim, ability level, true damage percent, AoE radius, arc half angle, attack direction, affected entity refs when available, and related damage cue ids when available
- **AND** generated C++ MUST dispatch a handler such as `HandleSaikaRainIronCannonProc`
- **AND** UE C++ MUST also be able to observe passive learned/active state through ability metadata, buff state, or a passive learned event

#### Scenario: three_stage_technique transform and multi-shot are readable
- **WHEN** Saika casts `three_stage_technique`
- **THEN** UE C++ MUST receive a typed event containing caster, ability level, duration, attack bonus percent, multi-shot count, transform buff id, visual effect id, and action/cue instance id
- **AND** generated C++ MUST dispatch a handler such as `HandleSaikaThreeStageChanged`
- **AND** while the transform is active, attack or projectile visual cues MUST expose multi-shot bullet index/count or an equivalent visual repetition payload so C++ can spawn the correct number of muzzle/projectile visuals

### Requirement: Saika action and animation events are complete for UE C++
SaikaMagoichi action and animation events SHALL be available to UE C++ with typed payloads. This includes locomotion, animation overlay, sniper-mode walk, normal attack, critical attack, attack windup, impact, recovery, action instance id, muzzle information, projectile visual cue references, and UnitScript attack hook events.

#### Scenario: C++ receives normal and sniper walk states
- **WHEN** Saika transitions between idle, normal walk, and `sniper_mode` walk
- **THEN** UE C++ MUST receive animation state payloads containing locomotion state, locomotion variant, animation overlay, idle variant, play rate, tick, and sequence
- **AND** `sniper_mode` active while walking MUST resolve to overlay `sniper_mode` and locomotion variant such as `sniper_walk`

#### Scenario: C++ receives attack phase events
- **WHEN** Saika performs an attack
- **THEN** UE C++ MUST receive attack/action payloads for Windup, Impact, and Recovery with action instance id, phase elapsed/duration/progress, target entity when available, attack/critical state, and muzzle bone or muzzle transform reference when available
- **AND** `on_attack_start`, `on_attack_landed`, `on_attack_fail`, and `on_attacked` UnitScript hook cues MUST be linkable to the same action/cue instance id when available

#### Scenario: C++ receives critical attack state
- **WHEN** gameplay resolves a critical attack or critical visual branch for Saika
- **THEN** animation/action payload MUST expose `CriticalAttack` action state or a critical flag with an animation tag
- **AND** UE C++ and AnimBP adapter MUST be able to route to the critical attack state/montage without guessing from damage numbers

### Requirement: Saika C++ integration is verified end-to-end
The implementation SHALL include verification for generated Saika metadata, typed C++ payloads, native handler dispatch, Blueprint override compatibility, animation/action states, and all four Saika abilities.

#### Scenario: Generated C++ contains Saika symbols
- **WHEN** `om-codegen` runs against current Lua content
- **THEN** generated output MUST include `AOmHeroSaikaMagoichi`
- **AND** generated output MUST include typed payloads or equivalent native event APIs for `sniper_mode`, `saika_reinforcements`, `rain_iron_cannon`, and `three_stage_technique`

#### Scenario: Saika smoke receives all required events
- **WHEN** a synthetic or TD_1 smoke drives Saika through spawn, idle, normal walk, sniper-mode walk, normal attack, critical attack, sniper_mode toggle, reinforcements cast, rain_iron_cannon proc, and three_stage transform
- **THEN** a UE native C++ test class MUST receive the expected typed handlers
- **AND** a Blueprint subclass MUST be able to override the same visual events
- **AND** no handler payload MUST depend on raw FFI memory after the frame lease is released
