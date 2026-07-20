# Three-Tower Active Abilities Design

**Date:** 2026-07-12  
**Status:** Approved design  
**Scope:** Complete Bloons-style upgrades for Boomerang, Arty, and Cake Splash, including one active ability per tower and a global ability bar in `omfx`.

## Context

Omoba currently ships seven TD towers. Dart, Bomb, Tack, and Ice already have three four-level upgrade paths with scripted behavior. Boomerang and Arty have upgrade metadata and partial behavior, but some effects are simplified or inconsistent with their descriptions. Cake Splash has a base attack only.

The existing upgrade rule remains unchanged: a tower may reach level 4 on one primary path and level 2 on one secondary path; the third path stays at level 0. This change does not add a fifth upgrade tier or redesign the four completed towers.

## Goals

- Complete all three upgrade paths for Boomerang, Arty, and Cake Splash.
- Give each of these towers one level-4 active ability.
- Add an authoritative, reusable tower-active-ability runtime.
- Add a bottom-center global ability bar that represents individual tower instances.
- Keep active abilities instant-cast; no ground or unit targeting mode is required.
- Use a 10-second cooldown for all three abilities during testing.

## Non-Goals

- Rebalancing Dart, Bomb, Tack, or Ice.
- Converting all tower upgrades to five tiers.
- Adding arbitrary ability-effect DSLs.
- Adding aimed abilities or a targeting cursor.
- Final production cooldown balancing.

## Gameplay Design

### Boomerang

Boomerang remains a mobile multi-target physical tower.

- Path 1, ricochet: range, ricochet, double glaives, then MOAB Press.
- Path 2, turbo: attack speed, projectile speed, triple burst, then Turbo Charge.
- Path 3, shuriken: wider and stronger projectiles, improved damage, double shuriken, then a three-projectile ricochet storm.

**Turbo Charge** is unlocked by Path 2 level 4. It has a 10-second cooldown and lasts 5 seconds. Activation immediately cancels the current attack backswing. While active, attack interval is multiplied by `0.35` and each normal attack emits two additional projectiles. The previous permanent level-4 turbo attack-speed bonus is removed so that the active window does not compound with an equivalent permanent effect.

Existing behavior flags such as `glaive_ricochet`, `glaive_lord`, `moab_press`, `bionic_burst`, `shuriken`, `double_shuriken`, and `storm_shuriken` remain valid. Their implementation and metadata must agree.

### Arty

Arty remains a long-range area-damage and control tower.

- Path 1, heavy ordnance: progressively increases range, damage, and splash radius.
- Path 2, control shells: stun duration progresses through 1, 2, and 3 seconds; level 4 also applies a 50% slow.
- Path 3, reload: progressively increases attack speed; level 4 unlocks Fire at Will.

**Fire at Will** is unlocked by Path 3 level 4. It has a 10-second cooldown and lasts 3 seconds. It schedules six additional shells at 0.5-second intervals. Each pulse selects the living enemy nearest to the route exit at that moment. If no valid enemy exists, that pulse is not spent; remaining shells may fire at later pulses until the 3-second window ends. Extra shells inherit the tower's current damage, splash radius, stun, slow, and cross-path modifiers.

The Path 2 level-4 slow must be implemented as described. The current metadata/script mismatch must not remain.

### Cake Splash

Cake Splash becomes a short-range magical area tower with damage, rhythm, and control paths.

#### Path 1: Baking Power

1. Damage +25%.
2. Attack radius +40 world units.
3. Hits apply a 3-second burn that deals 20% of the triggering hit per second.
4. Burn increases to 40% per second and pulse damage gains +50%.

#### Path 2: Party Rhythm

1. Attack interval ×0.8.
2. Each normal attack adds one secondary pulse for 50% damage.
3. Each normal attack adds two secondary pulses for 50% damage each.
4. Unlocks Dessert Party.

**Dessert Party** has a 10-second cooldown and lasts 5 seconds. It pulses every 0.5 seconds, dealing 50% of current attack damage in the tower's current attack radius. During the active window, other friendly towers inside that radius receive 25% attack speed. The casting Cake Splash cannot receive its own aura, preventing feedback loops.

#### Path 3: Frosting Control

1. Hits slow by 20% for 2 seconds.
2. Slow increases to 35%.
3. Frosted enemies take 15% additional damage from all sources.
4. Slow increases to 50%, incoming damage increases to 25%, and hits refresh the duration.

A burn from the same Cake Splash refreshes rather than stacks. Frosting from multiple Cake Splash towers uses the strongest applicable slow and vulnerability; it does not multiply. These rules bound performance and prevent exponential scaling in stress scenes.

## Metadata and Runtime Model

Lua tower upgrade metadata remains the source of content truth. A level-4 upgrade may include one active-ability declaration:

```text
ability_id
display_name
description
icon
cooldown_seconds
duration_seconds
pulse_interval_seconds (optional)
pulse_count (optional)
```

The Lua compiler and script ABI expose this declaration as fixed metadata. It is not a generic effect language. Tower-specific results stay in `base_content.dll`.

The native tower component stores at most one unlocked active ability because this design provides one active path per tower:

```text
TowerActiveAbilityState
  ability_id
  cooldown_remaining
  active_remaining
  pulse_accumulator
  pulses_remaining
  activation_serial
```

`activation_serial` changes on every accepted cast. It lets scripts observe a new activation without relying on floating-point comparisons or duplicated network events. Time fields use the simulation's deterministic fixed-point representation and advance with authoritative game time. Pausing stops cooldowns; game-speed changes affect them consistently with the rest of the simulation.

The script ABI adds narrowly scoped tower hooks:

- Return optional active-ability metadata with tower metadata.
- Handle activation for a tower instance.
- Handle scheduled active-ability pulses.
- Query whether an active window is currently running and its remaining duration.

The host owns unlock state, cooldown validation, scheduling, and pulse counts. Scripts own damage, buffs, projectile creation, target selection, and visuals. This prevents `omb` and `omfx` from accumulating tower-kind conditionals.

If an upgrade path that owns the ability is unlocked, the state is created with zero cooldown. Selling or destroying the tower deletes it. Save/load and reconnect snapshots include the remaining cooldown and active window.

## Command and Event Flow

The global ability bar is driven by authoritative per-player snapshots.

1. Unlocking, selling, destroying, reconnecting, or materially changing ability state causes the backend to publish the player's tower-ability snapshot.
2. The client creates one button per tower instance.
3. Clicking a button or pressing its shortcut sends `tower_id` and `ability_id` through the existing player-command transport.
4. The backend verifies player identity, ownership, tower existence, unlock state, ability ID, cooldown, and game state.
5. A valid command starts the active window, initializes pulse scheduling, sets cooldown to 10 seconds, invokes the script activation hook, and broadcasts the new state.
6. An invalid command returns a rejection reason without changing cooldown or effect state.

The concrete command topic is `player/tower/ability`. Ability snapshots and cast results use the existing game-event envelope rather than introducing a second transport. Exact payload encoding should follow the current tower-upgrade command/event convention.

The backend never trusts client countdowns. The frontend may decrement displayed cooldown locally between snapshots, but every backend update replaces the displayed value.

## Global Ability Bar

The bar is anchored at the bottom center of the game view and does not replace the selected-tower upgrade panel.

- Buttons are ordered by tower construction order, with a stable entity identifier as the tie-breaker.
- Only unlocked abilities belonging to living player-owned towers are shown.
- Duplicate tower types receive a compact instance suffix such as `Boomerang #2` in their tooltip.
- Each button shows tower identity, skill icon, shortcut, ready state, and a one-decimal cooldown overlay.
- Keys `1` through `6` activate the six currently visible buttons.
- More than six abilities use horizontal paging or scrolling; shortcuts apply to the visible page.
- Missing skill art falls back to the tower icon plus the skill's first character.
- A rejected cast briefly displays the backend rejection reason, such as cooldown active or tower no longer present.
- Removal of a tower immediately removes and reflows its button.

The layout must avoid per-frame scene-node creation. Buttons are diffed by stable `(tower_id, ability_id)` identity, and only changed cooldown text or state is updated.

## Error Handling and Edge Cases

- Duplicate or rapid cast commands: only the first valid command starts the cooldown; later commands are rejected.
- Stale button after a tower sale: backend rejects the command and the following snapshot removes the button.
- Ability metadata missing at runtime: the upgrade remains purchasable only if validation accepts its metadata; otherwise script loading fails with a content diagnostic.
- No enemy during Fire at Will: do not consume that scheduled shell; expire unused shells at the end of the active window.
- Tower destroyed during an active window: cancel future pulses and remove its UI entry.
- Pause during an active window: active duration, pulse accumulator, buffs, and cooldown all stop.
- Reconnect: rebuild the entire bar from a full authoritative snapshot.
- Multiple frosting or party auras: use stable source IDs and strongest-effect aggregation; never compound the same modifier from repeated refreshes.

## Validation and Testing

### Metadata validation

- Boomerang, Arty, and Cake Splash each expose three paths with four levels.
- Exactly one level-4 upgrade per tower exposes an active ability.
- Ability IDs are non-empty and globally unique.
- Cooldown is exactly 10 seconds for the initial test configuration.
- Duration, pulse interval, and pulse count are internally consistent.
- Every behavior flag is recognized by the matching tower script.

### Pure runtime tests

- Unlocking creates a ready state.
- Cooldown starts once and reaches zero using authoritative game time.
- Pause and game-speed behavior match simulation time.
- Ownership, existence, unlock, ability-ID, and cooldown rejection paths do not mutate state.
- Pulse scheduling does not double-fire across tick boundaries.

### Script tests

- Turbo Charge cancels backswing, lasts 5 seconds, changes interval by ×0.35, and adds two projectiles.
- Fire at Will fires at most six inherited-stat shells, selects the enemy nearest the exit, and preserves shells when no target exists.
- Dessert Party emits ten half-damage pulses over 5 seconds and buffs only other friendly towers in range.
- Cake burn refreshes per source; frosting selects the strongest slow and vulnerability.
- Arty level-4 control shells actually apply the documented 50% slow.

### Integration and frontend tests

- Upgrade purchase creates an ability button without reconnecting.
- Cast command executes the script effect and updates cooldown.
- Selling or destroying a tower removes its ability and cancels active pulses.
- Reconnect restores order, cooldown, and active state.
- Six buttons receive shortcuts; additional buttons scroll/page correctly.
- Cooldown interpolation is corrected by authoritative snapshots.

### Manual smoke test

Build and stage `base_content.dll`, run the game, unlock each active ability, and verify effects at normal speed, accelerated speed, pause, resume, sale during activation, and reconnect. Test cooldowns remain at 10 seconds until a later balance pass explicitly changes the Lua values.

## Implementation Boundaries

Expected areas of change are:

- `scripts/lua_data/templates/towers.lua` for upgrade and active-ability metadata.
- `scripts/script-abi` for ABI-safe metadata and tower ability hooks.
- `scripts/base_content/src/towers/{boomerang,arty,cake_splash}.rs` for behavior.
- `omoba-core` native tower state, validation, scheduling, commands, and snapshots.
- `omb` integration with the shared native runtime and transport.
- `omfx` and `eui` for the bottom-center global ability bar.

No unrelated refactor is included. ABI changes require rebuilding both the scripts workspace and host workspace with Rust 1.95.0 and staging the rebuilt DLL before end-to-end verification.
