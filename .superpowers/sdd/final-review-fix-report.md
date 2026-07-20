# Final Review Fix Report

Date: 2026-07-14
Branch: `codex/three-tower-active-abilities`
Result: DONE

## Changes

- Tower ability bar interpolation now converts wall-clock elapsed time to simulation elapsed time from the last authoritative pause/speed snapshot: paused = 0, otherwise `elapsed_wall * game_speed_multiplier`. Every new authoritative snapshot remains the correction point.
- Nonempty authored ability icon paths are loaded exactly. A missing or failed ability icon resolves to the owning tower's base image through the tower asset loader and preserves the ability-name initial in the slot text. Existing hero ability icons still explicitly use the generic placeholder as their separate fallback policy. No fake binary icon assets were added.
- `TowerAbilityCastResult` remains the latest-overall compatibility resource and snapshot field. `TowerAbilityCastResults` adds deterministic `BTreeMap<u32, TowerAbilityCastResult>` latest-per-player storage, extracted as a player-sorted `tower_ability_cast_results` snapshot vector. Shared initialization inserts both resources for authoritative and replica worlds.
- The ability bar consumes the local player's keyed result. Rejections survive ordinary snapshots, remain readable for 3.0 wall-clock seconds, and clear immediately if the keyed tower/ability disappears.
- Arty module docs now describe Path1 damage, Path2 control including `arty_slow_50`, and Path3 reload/active behavior. The obsolete Dart attack-speed TODO was removed.
- `omfue` was not touched. The ignored staged `scripts/base_content.dll` was not rebuilt or modified because the script changes are documentation-only.

## Strict TDD Evidence

### Countdown interpolation

RED:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_elapsed_sim --lib` (from `omfx`)

- Exit 1.
- Expected compile failure: `cannot find function ability_bar_elapsed_sim` in paused, 1x, 2x, and correction tests.

GREEN:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_ --lib` (from `omfx`)

- Exit 0.
- 16 passed after the interpolation implementation; paused, 1x, 2x, and authoritative-correction tests all passed.

### Missing authored icon

RED:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_uses_tower_and_initial_fallback_when_nonempty_icon_fails_to_load --lib` (from `omfx`)

- Exit 1.
- Expected compile failure: `cannot find function resolved_ability_bar_icon`.

GREEN:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_ --lib` (from `omfx`)

- Exit 0.
- 17 passed, including the focused nonempty missing-path fallback test.

Self-review then identified that tower base images require the tower loader's `scripts/base_content` candidate paths. A second focused cycle covered loader selection:

- RED: same focused command exited 1 with missing `ability_bar_texture_kind` / `AbilityBarTextureKind`.
- GREEN: same focused command exited 0 with 1 passed; fallback now selects `TowerBase` and production uses `tower_texture_for_key`.

### Latest-per-player cast results

RED:

`cargo test --manifest-path omoba-core/Cargo.toml tower_ability_cast_drain_retains_latest_result_for_each_player --lib`

- Exit 1.
- Expected compile failure: unresolved `TowerAbilityCastResults` import.

GREEN:

- Same command exited 0 with 1 passed.
- One drain retained player 7's serial 1 result and player 8's serial 2 result.

Snapshot ordering RED:

`cargo test --manifest-path omoba-core/Cargo.toml tower_ability_cast_result_snapshots_are_sorted_by_player --lib`

- Exit 1.
- Expected compile failure: missing `tower_ability_cast_results` extraction helper.

Snapshot ordering GREEN:

- Same command exited 0 with 1 passed.
- Results inserted as players 8 then 7 extracted deterministically as `[7, 8]`.

### Rejection visibility lifetime

RED:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_rejection_ --lib` (from `omfx`)

- Exit 1.
- Expected compile failure: undeclared `AbilityBarRejectionState` in all four behavior tests.

GREEN:

- Same command exited 0 with 4 passed.
- Covered local-player result selection, repeated ordinary snapshots at 0.5/1.0/2.9 seconds, expiry exactly at 3.0 seconds, and immediate tower removal.

## Final Verification

- `cargo test --manifest-path omfx/Cargo.toml -p omfx --lib`
  - Exit 0; 95 passed, 0 failed, 0 ignored.
  - Re-run fresh after the tower-loader correction.
- `cargo test --manifest-path omoba-core/Cargo.toml`
  - Exit 0; 157 unit tests passed.
  - Doc tests: 1 passed, 1 ignored.
- `cargo test --manifest-path omb/Cargo.toml -p omobab`
  - Exit 0.
  - Library target: 111 passed, 1 ignored.
  - Binary target: 111 passed, 1 ignored.
  - Integration tests: 4 passed; explicitly ignored network/determinism/budget suites remained ignored.
  - Doc tests: 0 passed, 1 ignored.
- `cargo test --manifest-path scripts/Cargo.toml -p base_content`
  - Exit 0; 26 passed, 0 failed, 0 ignored.
- `git diff --check` and `git -C omfx diff --check`
  - Exit 0; no whitespace errors before commits.
- Authored icon audit:
  - All three Lua metadata paths exist as strings.
  - All three referenced active-icon PNG files are absent.
  - Corresponding tower base images exist under `scripts/base_content/assets/towers`.

The only repeated warning was the repository's expected `protoc not found; using src/generated/game.rs fallback` build warning.

## Commits

- omfx: `8e13b25` (`fix(ui): harden tower ability feedback`)
- omb: unchanged at `473be6e` (no omb-local source change was required)
- root behavior/pointer/docs: `29c68b2` (`fix: preserve tower ability cast feedback`)

## Self-review

- Determinism: per-player storage uses `BTreeMap`; snapshot vector follows map order; serials still advance in drained request order; the compatibility latest-overall result is preserved.
- Initialization parity: both cast-result resources are inserted in the shared `StateInitializer` used by authoritative and replica runtime construction; focused fixtures mirror this initialization.
- UI timing: simulation interpolation uses authoritative pause/speed values, while the rejection timeout intentionally uses `Instant` wall-clock time and does not reset on repeated result snapshots.
- Removal: rejection key validation uses all living locally owned towers with an active ability, not only the current six-item page.
- Assets: authored icon load failures are cached exactly, fallback base images use the established tower loader, and no placeholder binaries were manufactured.
- Scope: no `omfue` changes, no `omb` working-tree changes, no staged DLL rebuild, and formatter-only changes outside the intended files were removed before commit.

## Final Minor Polish: Accepted Cast Clears Rejection

An accepted local `TowerAbilityCastResult` now advances `last_result_serial` and immediately clears any visible earlier rejection, rather than leaving the old reason visible until its 3-second timeout.

RED:

`cargo test --manifest-path Cargo.toml -p omfx ability_bar_accepted_result_clears_visible_rejection_before_timeout --lib` (from `omfx`)

- Exit 1.
- The new rejection-then-acceptance test failed at `assert!(state.visible.is_none())`, proving the accepted serial advanced while the old rejection remained.

GREEN:

- `cargo test --manifest-path Cargo.toml -p omfx ability_bar_rejection_ --lib`
  - Exit 0; 4 passed.
- `cargo test --manifest-path Cargo.toml -p omfx ability_bar_accepted_result_clears_visible_rejection_before_timeout --lib`
  - Exit 0; 1 passed.
- `cargo test --manifest-path Cargo.toml -p omfx --lib`
  - Exit 0; 96 passed, 0 failed, 0 ignored.
- `git diff --check` in `omfx`
  - Exit 0; no whitespace errors.

Polish commits:

- omfx: `6312033` (`fix(ui): clear rejection after accepted cast`)
- root pointer: `855ebad` (`fix: update tower ability feedback client`)

Scope review: only `omfx/game/src/native.rs`, the root `omfx` pointer, and this report changed. `omfue`, backend/core behavior, and binary assets were untouched.
