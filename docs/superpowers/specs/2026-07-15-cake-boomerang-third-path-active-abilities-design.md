# Cake and Boomerang Third-Path Active Abilities Design

## Goal

Add an active ability to the fourth upgrade of the third path for Cake Splash and Boomerang without moving or removing their existing second-path active abilities. After this change, every shipped tower's third-path level-four upgrade has an active ability, while Cake Splash and Boomerang each retain two route-specific active definitions in total.

Both new abilities are untargeted instant-button abilities. The player selects the tower and presses its ability button without choosing an enemy or ground position.

## Scope

| Tower | Required upgrade | Ability ID | Display name |
|---|---|---|---|
| Cake Splash | Path 3 level 4, `cake_frost_50_vulnerability_25` | `cake_frosting_lockdown` | 糖霜封鎖 |
| Boomerang | Path 3 level 4, `storm_shuriken` | `boomerang_shuriken_storm` | 手裡劍風暴 |

The existing `cake_dessert_party` and `boomerang_turbo_charge` definitions remain on path 2 level 4 and retain their current behavior and balance values.

This change adds metadata, tower-script behavior, automated tests, and generated-catalog expectations. It does not add a targeting mode, network message, script ABI method, multi-slot ability state, or new icon artwork. Conventional icon paths are authored under `assets/ui/abilities/`, and the frontend continues to use its existing missing-icon fallback until artwork is supplied separately.

## Route Exclusivity and Runtime Model

The tower upgrade rules allow at most one primary route at level 3 or 4. A Cake Splash or Boomerang tower therefore cannot own both of its level-four active upgrades in a legal build. `TowerActiveAbilityState` remains singular: buying the chosen primary route's fourth level installs that route's ability, and the other route cannot subsequently reach level 4.

The authoritative cast flow remains unchanged:

1. The client sends the selected tower entity and ability ID.
2. The server resolves the only legal level-four active from the tower's upgrade levels.
3. Ownership, ability-ID match, and cooldown readiness are validated.
4. `TowerActiveAbilityState` starts its cooldown and optional pulse schedule.
5. The tower script emits deterministic damage, status buffs, effects, and projectiles.

No active uses `Tower::ultimate_cooldown`; all timing stays in `TowerActiveAbilityState`.

## Ability Designs

### Cake Splash: Frosting Lockdown

`cake_frosting_lockdown` has a 12-second cooldown and no sustained active window.

On activation, the tower snapshots all enemies currently within its final attack range, then:

- deals magical splash damage equal to twice the tower's final attack damage;
- applies the shared `stun` status for 1.5 seconds; and
- applies a five-second frosting stat buff containing 50% movement slow and 25% incoming-damage amplification.

The frosting payload uses the existing `cake_frosting` aggregation family, so it combines with ordinary Cake frosting through the existing strongest-effect rule instead of multiplying same-family slows or vulnerabilities. Its source-qualified buff ID allows different Cake towers to refresh their own frosting independently. The activation emits its explosion effect and damage opportunity even when no enemy is present, but naturally applies no per-target buffs in that case.

This gives the control-oriented third path a decisive lockdown button without duplicating the second path's repeated damage and friendly attack-speed support.

### Boomerang: Shuriken Storm

`boomerang_shuriken_storm` has a 12-second cooldown, a 0.6-second active window, a 0.2-second pulse interval, and three pulses.

Each pulse emits 12 evenly spaced straight shuriken projectiles over 360 degrees, producing exactly 36 initial projectiles per cast. Each projectile:

- travels to the tower's final attack range;
- uses the tower's final attack damage;
- travels at the normal Boomerang projectile speed, including the existing `faster_rangs` cross-path modifier when present;
- uses the existing 90-unit shuriken hit width and shuriken projectile kind; and
- enters the existing `storm_shuriken` hit callback, retaining its bounded two-generation ricochet behavior.

The three waves are rotated by a deterministic angular offset derived from the pulse index so consecutive waves cover the gaps between prior projectiles. The cast fires even with no nearby enemy. Existing ricochet target selection remains deterministic and bounded to one next target per hit, so the complete cast creates at most 36 initial projectiles and 72 ricochet projectiles.

This emphasizes the third path's radial coverage and ricochet identity without borrowing the second path's attack-speed and extra-projectile self-buff.

## Metadata and Validation

The two `active_ability` records are added only to path 3 level 4 in `scripts/lua_data/templates/towers.lua`:

- `assets/ui/abilities/cake_frosting_lockdown.png`
- `assets/ui/abilities/boomerang_shuriken_storm.png`

Registry validation can no longer require exactly one active per tower. Instead, it must validate an explicit set of nine `(tower, path, level, ability_id)` records, require every active to remain at level 4, reject duplicate ability IDs, and retain positive cooldown and valid pulse-field checks. This ensures the two extra definitions are intentional rather than weakening validation to an unbounded "at least one" rule.

The generated unit and script API catalog must list nine active upgrades across seven towers. Cake Splash and Boomerang each appear twice, once for each active-bearing route; the other five towers appear once.

## Script Boundaries and Determinism

Each callback checks its exact ability ID and emits no outcomes for unrelated IDs. All gameplay numbers use `Fixed64`. The Cake victim list comes from the authoritative range query. Boomerang radial endpoints use the existing deterministic angle and trigonometric lookup.

Existing permanent fourth-level effects remain active outside and during the new casts. The abilities add no script-owned timers, random selection, unbounded searches, or unbounded projectile generation. Invalid or stale entities continue to be skipped by the existing dispatch and adapter behavior.

## Testing

Script-level tests cover:

- exact ability-ID filtering for both new abilities;
- Cake's damage multiplier, range, explosion emission, 1.5-second stun, five-second frosting duration, 50% slow, 25% vulnerability, and no-enemy activation;
- frosting aggregation-family and source-qualified ID behavior;
- Boomerang's three pulses, 12 initial projectiles per pulse, deterministic rotated endpoints, speed, damage, hit width, and shuriken kind;
- Boomerang's no-enemy activation and existing two-generation ricochet bound; and
- unchanged behavior of `cake_dessert_party` and `boomerang_turbo_charge`.

Metadata and runtime tests cover:

- all nine exact active upgrade positions and IDs;
- both new third-path upgrades installing the matching active state;
- legal route exclusivity keeping one active state per built tower;
- existing cast validation continuing to resolve the selected primary route's ability; and
- catalog smoke expectations changing from seven to nine active upgrades.

Verification builds and tests the scripts workspace first, stages the resulting `base_content.dll` into the backend as required by the repository workflow, then runs the relevant `omoba-core`, backend, and ignored gen-docs catalog tests.

## Acceptance Criteria

1. Cake Splash path 3 level 4 unlocks `cake_frosting_lockdown` without changing `cake_dessert_party` on path 2.
2. Boomerang path 3 level 4 unlocks `boomerang_shuriken_storm` without changing `boomerang_turbo_charge` on path 2.
3. All seven shipped towers have an active ability on path 3 level 4.
4. Legal tower builds still expose only one active state and one ability button.
5. Both new abilities cast immediately without secondary targeting.
6. Existing passive and automatic fourth-level effects remain intact.
7. Projectile and query work remains bounded by this specification.
8. All relevant automated tests pass, and the generated catalog lists nine active upgrades.
