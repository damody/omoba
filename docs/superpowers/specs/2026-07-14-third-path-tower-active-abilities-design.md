# Third-Path Tower Active Abilities Design

## Goal

Add one active ability to Dart, Bomb, Ice, and Tack so every shipped tower has at least one active ability. For each of these four towers, the active ability is unlocked only by the fourth upgrade on the third path. Existing permanent and automatic upgrade effects remain unchanged.

The four abilities are instant-button abilities: the player selects a tower and presses its ability button without choosing a target or position.

## Scope

This change adds active-ability metadata, tower-script behavior, automated tests, and generated-catalog expectations for:

| Tower | Required upgrade | Ability ID | Display name |
|---|---|---|---|
| Dart | Path 3 level 4, `mega_crit` | `dart_heavy_burst` | 重裝爆裂 |
| Bomb | Path 3 level 4, `frag_homing` | `bomb_cluster_overload` | 集束超載 |
| Ice | Path 3 level 4, `icicle_impale` | `ice_crystal_nova` | 冰晶新星 |
| Tack | Path 3 level 4, `needles_32` | `tack_blade_maelstrom` | 刀刃漩渦 |

The change does not add a targeting mode, network message, script ABI method, active-ability UI flow, or new icon artwork. Each metadata record uses a conventional icon path under `assets/ui/abilities/`; the frontend's existing missing-icon fallback remains responsible until artwork is added separately.

## Architecture

Each path-3 level-4 upgrade in `scripts/lua_data/templates/towers.lua` receives a `TowerActiveAbilityDef`. The existing upgrade runtime creates a `TowerActiveAbilityState` when that upgrade is purchased. The existing client snapshot and command path exposes the button and sends `TowerAbilityCastInput`.

The authoritative flow remains:

1. The client sends the selected tower entity and ability ID.
2. The server verifies ownership, tower existence, matching unlocked ability, and ready cooldown.
3. `TowerActiveAbilityState` starts its cooldown, active window, and optional pulse schedule.
4. Script dispatch calls the tower's existing activation or pulse callback.
5. The tower script emits deterministic buffs, projectiles, damage, or status outcomes.

No active ability uses the legacy `Tower::ultimate_cooldown`. Cooldown and active-window timing are owned only by `TowerActiveAbilityState`.

## Ability Designs

### Dart: Heavy Burst

`dart_heavy_burst` has a 12-second cooldown and a 5-second active duration.

Activation applies a tower-local marker buff for the authoritative remaining active duration. While the marker is present, the existing `mega_crit` explosion produced by `on_attack_hit` changes from 60 damage in a 60-unit radius to 120 damage in a 120-unit radius. The ordinary critical bonus damage and all projectile behavior remain unchanged.

This ability strengthens the critical-explosion identity of the third path without borrowing the multi-projectile identity of the second path.

### Bomb: Cluster Overload

`bomb_cluster_overload` has a 12-second cooldown and a 5-second active duration.

Activation applies a tower-local marker buff for the authoritative remaining active duration. While active, both first-generation homing fragments and their second-generation recursive children deal 1.5 times their normal damage and travel at 1.5 times their normal speed. Fragment counts and generation limits remain unchanged.

This preserves the third path's cluster identity while placing a hard upper bound on projectile count. It must not increase the existing 16-primary-fragment or 4-recursive-child counts.

### Ice: Crystal Nova

`ice_crystal_nova` has a 12-second cooldown and no sustained active window.

On activation, the tower emits 16 evenly spaced straight icicle projectiles over 360 degrees. Each projectile:

- travels 600 units;
- uses the existing deterministic angle lookup;
- deals four times the tower's final attack damage;
- has a 75-unit splash radius; and
- freezes affected enemies for 1.5 seconds.

The projectile uses the existing icicle projectile kind and hit pipeline, so path-3 interactions such as embrittlement and refreeze continue to apply normally. The nova fires even when there is no currently selected or nearby enemy because it is radial and untargeted.

### Tack: Blade Maelstrom

`tack_blade_maelstrom` has a 12-second cooldown, a 0.4-second active window, a 0.1-second pulse interval, and four pulses.

Every pulse emits 16 evenly spaced straight blade projectiles over 360 degrees. Across the complete cast this produces exactly 64 projectiles. Each blade:

- travels 600 units;
- deals three times the tower's final attack damage; and
- reuses the existing blade hit width, projectile kind, and burn-on-hit behavior.

Spreading the cast across four pulses limits the per-tick creation spike and makes the cast visually legible. Each pulse is derived from the active state's pulse index and must be consumed once only.

## Script Boundaries and Determinism

Each tower callback first checks its exact ability ID and emits no outcomes for any other ID. Duration-based scripts read the authoritative active remaining time through `TowerActiveAbilityAccess` instead of duplicating the duration as script state.

All time, distance, multipliers, and damage calculations use `Fixed64`. Radial projectiles use the existing deterministic angle type and trigonometric lookup. Projectile counts are constants: Bomb adds no projectiles, Ice emits 16 per cast, and Tack emits 16 per pulse for four pulses.

Existing fourth-level behavior remains active outside and during the new ability windows. The new marker buffs augment only the documented parameters and expire without modifying permanent upgrade stats.

Invalid or stale tower entities are skipped by the existing dispatch path. Script panics retain the existing containment and error logging behavior; this feature adds no new recovery mechanism.

## Metadata and Catalog

The four `active_ability` definitions are placed only on path 3 level 4 in `towers.lua`. Their icon strings are:

- `assets/ui/abilities/dart_heavy_burst.png`
- `assets/ui/abilities/bomb_cluster_overload.png`
- `assets/ui/abilities/ice_crystal_nova.png`
- `assets/ui/abilities/tack_blade_maelstrom.png`

The generated unit and script API catalog must expose seven shipped tower active abilities total: the existing Boomerang, Arty, and Cake abilities plus these four.

## Testing

Script-level tests cover:

- exact ability-ID filtering for all four towers;
- Dart's normal and active explosion damage and radius, plus expiration;
- Bomb's active damage and speed multipliers for both allowed fragment generations, unchanged fragment counts, and expiration;
- Ice's exact count, deterministic radial endpoints, damage multiplier, splash radius, freeze duration, and no-enemy activation;
- Tack's 16 projectiles per pulse, four-pulse total, projectile parameters, and burn interaction;
- inactive behavior remaining identical to the pre-change path-3 behavior.

Metadata and runtime tests cover:

- active metadata existing only at path 3 level 4 for each affected tower;
- upgrade purchase installing the matching active ability state;
- ownership, mismatch, missing-tower, active-window, and cooldown rejection remaining handled by the shared runtime; and
- catalog smoke expectations changing from three to seven active abilities and checking all seven IDs.

Verification runs the relevant scripts-workspace tests, `omoba-core` and backend tests touched by metadata/runtime behavior, and the ignored gen-docs smoke pipeline after staging a freshly built `base_content.dll` as required by the repository build flow.

## Acceptance Criteria

1. Dart, Bomb, Ice, and Tack each expose exactly one active ability after buying path 3 level 4.
2. No active is unlocked by either of the first two paths or before level 4.
3. All abilities cast immediately without a secondary target-selection step.
4. Cooldowns and active durations are visible through the existing tower snapshot and UI flow.
5. Existing passive and automatic fourth-level effects are preserved.
6. Projectile work is bounded by the counts in this specification.
7. All relevant automated tests pass, and the generated catalog lists seven tower actives.
