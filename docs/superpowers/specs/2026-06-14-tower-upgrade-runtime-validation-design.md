# Tower Upgrade Runtime Validation Design

Date: 2026-06-14
Status: Approved for implementation planning

## Goal

Add automated runtime tests proving that tower upgrade application is atomic and correct at the gameplay-state boundary.

This phase validates `handle_tower_upgrade_from_input` after the metadata layer has already proven the 48 upgrade definitions are complete and well-formed.

## Scope

The tests cover the direct runtime handler:

```text
omoba-core/src/runtime/native/game_processor.rs::handle_tower_upgrade_from_input
```

The handler must:

- Accept a valid owner upgrade request.
- Deduct the correct hero gold.
- Increment only the requested tower path level.
- Apply stat effects to `BuffStore`.
- Add behavior flags to `Tower.upgrade_flags`.
- Reject invalid requests before mutating state.

This phase intentionally does not test the pending queue drain, lockstep input routing, script behavior, frontend snapshots, or visual feedback.

## Current Context

`handle_tower_upgrade_from_input` already performs the full direct runtime application:

- Validates `path`.
- Finds the tower entity and `ScriptUnitTag`.
- Verifies `FactionType::Player`.
- Verifies `PlayerOwner` matches the requesting player.
- Applies Bloons 2.5 path rules through `tower_upgrade_rules::validate_upgrade`.
- Looks up the next `TowerUpgradeDef`.
- Finds the requesting player's hero.
- Checks and deducts `Gold`.
- Writes permanent stat payloads into `BuffStore`.
- Adds `BehaviorFlag` values into `Tower.upgrade_flags`.
- Writes `Tower.upgrade_levels[path] = next_level`.

Existing tests already verify one negative case: non-owner sell/upgrade rejection does not mutate `upgrade_levels`. The missing coverage is the successful upgrade path and broader failure atomicity.

## Test Strategy

Add focused unit tests in the existing `#[cfg(test)] mod tests` inside:

```text
omoba-core/src/runtime/native/game_processor.rs
```

Reuse the current helpers:

- `world_for_owner_tests()`
- `add_owned_hero(...)`
- `add_owned_tower(...)`

Extend them only if needed to insert the resources required for a successful tower upgrade, especially `TowerUpgradeRegistry`.

## Success Path Test

Create a test that upgrades an owned Dart tower and asserts all intended state changes.

Recommended case:

```text
tower_dart path 0 level 1
```

Expected assertions:

- The handler returns `Ok(())`.
- Player hero gold decreases by the exact level-1 Dart upgrade cost.
- `Tower.upgrade_levels` changes from `[0, 0, 0]` to `[1, 0, 0]`.
- A permanent upgrade buff is present in `BuffStore`.
- The buff payload contains the expected stat effect from the upgrade definition.
- No unrelated path levels change.

If path 0 level 1 has only stat effects, use a second success check for a behavior-flag upgrade, such as Dart path 1 level 3, only after applying the required level 1 and level 2 upgrades on that path.

The flag assertion should prove:

- `Tower.upgrade_flags` contains the expected flag.
- Applying later upgrades does not duplicate existing flags.

## Failure Atomicity Tests

Failure cases should snapshot relevant state before calling the handler, then assert state is unchanged afterwards.

Each failure should verify:

- Handler returns `Err`.
- Hero gold is unchanged.
- `Tower.upgrade_levels` is unchanged.
- `Tower.upgrade_flags` is unchanged.
- No new relevant `BuffStore` payload was applied to the tower.

Required failure cases:

- Invalid path, e.g. `path = 3`.
- Non-owner upgrade request.
- Insufficient gold.
- Bloons 2.5 rule rejection, e.g. attempting a second primary path.

The existing non-owner test may be extended, but a clearer standalone atomicity helper is preferred if it keeps assertions easier to read.

## Helper Design

Use small test helpers only inside `game_processor.rs` tests:

- `hero_gold(world, hero) -> i32`
- `tower_levels(world, tower) -> [u8; 3]`
- `tower_flags(world, tower) -> Vec<String>`
- `tower_buff_count(world, tower) -> usize` or a more specific BuffStore query if available.

Prefer using public or existing `BuffStore` query methods. Do not add production-only APIs for tests.

If the existing `BuffStore` does not expose a clean count method, assert using known stat aggregation from the target upgrade effect, such as `sum_add` with the stat key expected from the registry.

## Verification

Primary command:

```text
cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade
```

Broader regression:

```text
cargo test --manifest-path omoba-core/Cargo.toml
cargo test --manifest-path omb/Cargo.toml -p omobab --lib
```

No script DLL rebuild is required.

## Non-Goals

This phase does not:

- Test `PendingTowerUpgradeQueue` or `drain_pending_tower_upgrades`.
- Test `player_input_tick` wire routing.
- Test tower script behavior such as triple shot, fragments, burn, slow, or aura effects.
- Test frontend snapshots, pips, panels, icons, or VFX.
- Tune upgrade prices or balance.
- Change metadata unless a runtime test exposes a concrete inconsistency.

## Acceptance Criteria

Implementation is complete when:

- A successful owner upgrade test proves gold, levels, stat buffs, and flags are applied correctly.
- Failure tests prove invalid upgrades do not partially mutate gold, tower levels, tower flags, or tower buffs.
- `cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade` passes.
- `cargo test --manifest-path omoba-core/Cargo.toml` passes.
- `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` passes.
