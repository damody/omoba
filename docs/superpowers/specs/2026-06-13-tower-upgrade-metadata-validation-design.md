# Tower Upgrade Metadata Validation Design

Date: 2026-06-13
Status: Approved for implementation planning

## Goal

The first stabilization slice for the Bloons-style TD core is to make the 48 tower upgrade definitions trustworthy before deeper runtime, script, balance, or visual work continues.

This design adds automated validation for the metadata and registry layer:

- Four towers exist in the upgrade registry: Dart, Tack, Bomb, Ice.
- Each tower has exactly three upgrade paths.
- Each path has exactly four consecutive levels.
- Each definition has valid names, descriptions, costs, effects, stat keys, and behavior flags.
- The registry output is safe for both runtime upgrade application and frontend upgrade UI snapshots.

The immediate priority is functional correctness and player-visible clarity at the data boundary. Balance tuning is intentionally deferred until metadata and effect intent are reliable.

## Current Context

Tower upgrade metadata now flows through shared runtime code:

- `scripts/lua_data/templates.lua` owns tower upgrade content.
- `omoba-template-ids/build.rs` generates const upgrade data and lookup helpers.
- `omoba-template-ids::active_tower_upgrades(id)` returns active build-time or runtime content.
- `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` converts const POD data into `TowerUpgradeDef`.
- `omoba-core/src/tower_meta.rs` defines shared `TowerUpgradeDef`, `UpgradeEffect`, `StatOp`, and `upgrade_cost`.
- `omoba-core/src/runtime/native/snapshot.rs` exposes tower upgrade snapshots for frontend UI.

Existing registry tests already cover basic presence, duplicate count, and cost formula. The missing piece is a stricter metadata lint layer that catches content mistakes before they become invisible runtime bugs.

## Recommended Approach

Use a registry plus schema-lint test suite in `omoba-core`, colocated with the runtime registry or placed in an integration-style test module if the assertions grow too large.

This is preferred over a pure registry test because the risky failures are not just missing definitions. A definition can exist but still be wrong in ways that only surface during play:

- `StatOp::Add` paired with a `_multiplier` key.
- `StatOp::Mul` paired with a `_bonus` key.
- Empty behavior flag strings.
- Behavior flags not supported by scripts.
- Empty descriptions that make upgrade UI unclear.
- Missing effects that produce a purchasable but inert upgrade.

Catalog or HTML reporting can come next, after these rules are codified.

## Validation Rules

### Shape Rules

The test suite must verify:

- Registry contains exactly 48 definitions.
- Valid tower IDs are exactly `tower_dart`, `tower_tack`, `tower_bomb`, and `tower_ice`.
- For every tower, paths `0..=2` exist.
- For every path, levels `1..=4` exist with no gaps.
- No definition uses an out-of-range path or level.
- Registry keys match the contained definition fields.

### Text Rules

Every `TowerUpgradeDef` must have:

- Non-empty `name`.
- Non-empty `description`.
- Description that is not identical to the name.
- Positive `cost`.
- At least one `effect`.

Descriptions do not need final balance wording yet, but they must be useful enough for the current frontend panel and future catalog view.

### Cost Rules

Every upgrade cost must match:

```text
upgrade_cost(base_tower_cost, level)
```

The base costs come from generated tower stats:

- `TOWER_DART_STATS.cost`
- `TOWER_TACK_STATS.cost`
- `TOWER_BOMB_STATS.cost`
- `TOWER_ICE_STATS.cost`

This rule already exists in a broad form and should remain part of the stricter lint suite.

### Stat Effect Rules

For every `UpgradeEffect::StatMod`:

- `key` must be non-empty.
- `value` must be finite.
- `value` must not be zero.
- `StatOp::Add` is allowed for keys ending in `_bonus`.
- `StatOp::Mul` is allowed for keys ending in `_multiplier`.
- Explicit absolute-value stat keys are allowed only through a small allowlist.

Initial absolute-key allowlist:

- `crit_chance`
- `crit_bonus`

This keeps the current Dart crit data legal while preventing accidental new absolute keys from slipping in unnoticed.

### Behavior Flag Rules

For every `UpgradeEffect::BehaviorFlag`:

- `flag` must be non-empty.
- `flag` must be present in a supported flag allowlist.

Initial supported flag allowlist:

- Dart: `sharp_pierce`, `spike_o_pult`, `triple_shot`, `fan_club`, `always_crit`, `mega_crit`
- Bomb: `bomb_stun`, `missile`, `moab_assassin`, `frag_8`, `frag_12`, `frag_recursive`, `frag_homing`
- Tack: `blade_shooter`, `burn_tier1`, `burn_tier2`, `ring_of_fire`, `inferno_ring`, `needles_12`, `needles_16`, `needles_32`
- Ice: `deep_freeze`, `absolute_zero`, `arctic_aura_20`, `snowstorm`, `cryo_cannon`, `embrittle_15`, `refreeze`, `embrittle_25`, `icicle_impale`

The allowlist is intentionally strict. If a new script flag is added, the test must be updated in the same change, making the content/script contract explicit.

## Test Placement

Implementation should start in:

```text
omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
```

If the helper functions make the module noisy, move the lint helpers to:

```text
omoba-core/tests/tower_upgrade_metadata.rs
```

The tests should use public registry APIs where practical. If key-field consistency cannot be checked through `iter_all`, add a narrow test-only helper or assert via the known lookup matrix rather than exposing internal map structure as production API.

## Verification Commands

Primary verification:

```text
cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade
```

Broader regression check:

```text
cargo test --manifest-path omoba-core/Cargo.toml
cargo test --manifest-path omb/Cargo.toml -p omobab --lib
```

No `scripts/base_content.dll` rebuild is required for this metadata-only phase unless implementation reveals generated constants are stale.

## Follow-Up Phases

After this metadata validation phase passes:

1. Runtime application tests: upgrade command deducts gold, increments `Tower.upgrade_levels`, applies `BuffStore` stat mods, and records flags.
2. Script behavior tests: upgraded towers produce observable behavior changes such as multi-shot, fragments, slows, burns, splash, or aura effects.
3. Visual/manual checklist: each upgrade path has visible frontend feedback through panel text, pips, projectile behavior, VFX, and combat readability.
4. Balance pass: prices, wave pressure, path identity, and late-game difficulty are tuned after correctness and readability are stable.

## Non-Goals

This phase does not:

- Tune damage, attack speed, range, wave HP, or economy balance.
- Prove that script behavior fully implements every flag.
- Add new UI, catalog HTML, or documentation pages.
- Change tower upgrade data unless validation exposes a concrete defect.
- Modify lockstep input, KCP transport, or frontend rendering.

## Risks

- The flag allowlist can drift if scripts add behavior without updating validation. This is acceptable because the failed test is the intended warning.
- Some stat keys may be deliberately absolute rather than additive/multiplicative. Those keys must be added to the explicit allowlist with a short rationale in the test.
- Runtime content overrides may introduce upgrade data after build-time generation. Validation should call `active_tower_upgrades` through `TowerUpgradeRegistry::new()` so it tests the same source that the game uses.

## Acceptance Criteria

The implementation is complete when:

- The stricter metadata lint suite is present.
- `cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade` passes.
- Test failures identify the exact tower, path, level, and offending field.
- No production behavior changes are introduced unless a metadata defect must be corrected to satisfy the approved rules.
