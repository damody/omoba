# Tower Skill Event Audit and Repair Design

## Goal

Restore tower upgrade and active-skill effects that are defined correctly but do not execute through the live combat event pipeline. The reported acceptance case is the Sugar Ball Gunner (`tower_dart`) third path: tier-four critical hits deal a 60-radius, 60-damage splash, and `dart_heavy_burst` increases both to 120 for its five-second active window.

The audit covers all seven scripted towers and their three upgrade paths, on-hit behavior, and tier-four active abilities. Hero abilities are outside this change.

## Diagnosis

Projectile impacts now emit provenance-aware `ProjectileHit` events. Sugar Ball Gunner and Ice Crystal Teddy still implement their projectile-dependent effects in the older `on_attack_hit` hook. Live projectile attacks therefore bypass those effects, although unit tests pass because they invoke `on_attack_hit` directly instead of exercising the runtime event path.

Other projectile towers already consume `on_projectile_hit`. Globally dispatching both hook types for every projectile would risk duplicate fragment, ricochet, burn, and other chained effects.

## Approach

Repair each affected tower at the script boundary instead of adding a global compatibility dispatch:

1. Compare every authored Lua behavior flag and active ability ID with its Rust consumer.
2. Move projectile-dependent Sugar Ball Gunner and Ice Crystal Teddy behavior to `on_projectile_hit` while preserving projectile provenance.
3. Keep generic non-projectile `on_attack_hit` semantics unchanged.
4. Add integration coverage that follows projectile impact through outcome processing, the script event queue, registry dispatch, and resulting damage or buffs.

This is the smallest safe change because existing Bomb, Tack, and Boomerang projectile chains already rely on provenance-aware dispatch and must not be invoked twice.

## Behavior Requirements

### Sugar Ball Gunner

- Third-path critical-strike bonuses must execute when a dart projectile hits.
- `mega_crit` must add a physical splash centered on the struck enemy.
- Without the active window, splash radius and damage are both 60.
- During `dart_heavy_burst`, splash radius and damage are both 120 for five seconds.
- A projectile impact must trigger the effect once, including when other path upgrades alter projectile count or trajectory.

### Ice Crystal Teddy

- `embrittle_15` and `embrittle_25` must apply their authored on-hit vulnerability through projectile impacts.
- `refreeze` must execute through projectile impacts without duplicating the projectile's built-in slow or stun behavior.
- Icicle and other projectile variants must preserve their current projectile-kind and generation behavior.

### Remaining Towers

- Lua flags, Rust flag lookups, and active ability IDs must match exactly.
- Each tier-four active must be reachable through the runtime ability dispatcher.
- Projectile chains must remain bounded: fragments and ricochets must not recursively create unintended extra effects.
- Direct-area towers must retain their current non-projectile execution paths.

## Verification

Verification will include:

- Static flag and active-ID parity checks across all seven tower definitions.
- Focused runtime integration tests for Sugar Ball Gunner and Ice Crystal Teddy projectile-hit effects.
- Existing `base_content` tower tests.
- Tower upgrade registry tests.
- Active ability runtime and outcome-dispatch tests in `omoba-core`.
- The relevant script ABI tests if hook signatures or shared test support change.

The change is complete when the reported Sugar Ball Gunner behavior works through the live event path, the Ice Crystal Teddy effects are covered through the same path, all authored tower effects have a matching consumer, and the targeted test suites pass without duplicate chained effects.

## Non-Goals

- Changing tower balance values or descriptions.
- Auditing hero or summon abilities.
- Redesigning the combat event API.
- Adding a compatibility layer that dispatches both hit hooks for every projectile.
