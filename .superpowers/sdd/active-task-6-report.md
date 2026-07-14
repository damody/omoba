# Active Task 6 Report — Boomerang and Arty Active Behaviors

## Outcome

Implemented Turbo Charge and Fire at Will in commit `da414e9` (`feat(content): complete boomerang and arty abilities`).

- Turbo Charge records the accepted five-second active window as a transient marker buff, resets attack backswing through `TowerActiveAbilityAccess`, derives the effective interval as the current final interval multiplied by fixed-point `0.35` (`358/1024`), and adds two projectiles after permanent count upgrades.
- Fire at Will exposes the authored six-pulse / 0.5-second constants, uses the existing authoritative scripted-tower target selector (the same path-remaining-distance metric used by `TowerTargetPriority::First`), and reuses the normal Arty shell construction so current final damage, splash, stun, and the finite 50%/5-second control slow are retained.
- A no-target Fire at Will callback returns `false`. Under the inspected scheduler contract this preserves the charge for the next due interval inside the active window, while the scheduler still limits the activation to exactly six interval opportunities and expires unused charges after the three-second window/backlog.
- No permanent attack-speed buff is added or changed. Existing Boomerang ricochet provenance limits and all unrelated tower implementations remain unchanged.

## Files

- `scripts/base_content/src/towers/boomerang.rs`
- `scripts/base_content/src/towers/arty.rs`
- `scripts/base_content/src/towers/mod.rs` — test-only fixture repair: Task 5 made `ParallelAdapterCache` fetch `PlayerOwner`, so the shared tower fixture needed to register that storage before any tower behavior test could execute.

## Strict TDD evidence

### RED 1 — required pure helpers/constants

Command:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content towers::
```

Result: exit 1. Compilation failed on exactly the four missing required symbols:

- `boomerang_projectile_count`
- `turbo_interval`
- `ARTY_FIRE_AT_WILL_PULSES`
- `ARTY_FIRE_AT_WILL_INTERVAL`

### RED 2 — active behavior

After adding only the minimal pure helpers/constants, the same command initially exposed a pre-existing fixture panic: `MaskedStorage<PlayerOwner>` was not registered. The adapter introduced by Task 5 now fetches that storage. Registering it in the shared test fixture removed the setup error without changing production behavior.

The rerun reached the intended behavioral assertions: exit 1, 17 passed / 4 failed. The four failures were:

- Turbo activation did not create the transient five-second marker/reset outcome.
- Turbo plus Bionic Burst emitted 3 projectiles instead of 5.
- Fire at Will returned `true` with no target.
- Fire at Will emitted no authored projectile for the route-leading target.

### GREEN

Command:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content towers::
```

Result: exit 0, 21 passed / 0 failed.

Full package command:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content
```

Result: exit 0, 21 passed / 0 failed.

Formatting/diff checks:

```powershell
cargo fmt --manifest-path scripts/base_content/Cargo.toml
git diff --check
```

Result: exit 0.

An additional strict Clippy probe was run:

```powershell
cargo clippy --manifest-path scripts/Cargo.toml -p base_content --all-targets -- -D warnings
```

It did not pass because the package already contains unrelated warnings in `lib.rs`, Bomb/Ice/Tack tests, and hero scripts. The Task 6 warnings it identified (an extracted unused Arty local and unnecessary `mut` in the new tests) were removed. No unrelated warning cleanup was bundled into this task.

## Self-review

- Confirmed Turbo reads the final host-computed interval first, then applies only the transient `0.35` multiplier; permanent upgrade buffs are not mutated.
- Confirmed the `+2` is applied after the existing 1/2/3 projectile cross-path calculation, including Bionic Burst, Glaive Lord, Double Shuriken, and Storm Shuriken.
- Confirmed projectile kind, speed, hit radius, slow, and ricochet hooks remain on the existing paths; only fan spacing is generalized for the new 4/5-projectile cases.
- Confirmed normal Arty and Fire at Will share one shell builder, avoiding drift in final damage/splash/stun/slow behavior.
- Confirmed the Fire at Will test distinguishes the route-leading enemy from the spatially nearest enemy by assigning different authoritative `path_remaining_distance` values.
- Confirmed no `omfue`, frontend, or unrelated tower behavior was edited.

## Concerns

- `protoc` is not installed in this environment; builds use the repository's checked-in generated protobuf fallback, as reported by the existing build warning.
- The required tower test command was blocked until the test-only `PlayerOwner` registration was added. This is one extra related fixture file beyond the task brief's two behavior files, with no production effect.
- Strict package-wide Clippy remains non-green due to pre-existing unrelated warnings listed above; the required and full package tests are green.

## Important Review Fix — Fire at Will Always Uses First

### Finding and RED evidence

The original Fire at Will implementation called `GameWorld::query_nearest_enemy`. Despite its legacy name, that API intentionally honors the tower's current player-selected `target_priority`; therefore an Arty tower configured as `Nearest`, `HighestHealth`, and so on could override Fire at Will's authored First targeting.

The focused test now explicitly sets the tower priority to `TowerTargetPriority::Nearest`, places the spatially nearest enemy at 10 units with `path_remaining_distance = 100`, and places the route-leading enemy at 20 units with `path_remaining_distance = 10`.

Command:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content towers::arty::tests::fire_at_will_uses_first_priority_and_current_control_shell -- --exact
```

RED result: exit 1, 0 passed / 1 failed. The emitted projectile targeted `Entity(1)` at 10 units (the player-selected Nearest result), while the assertion required route-leading `Entity(2)`.

### Fix

Commit: `968d720` (`fix(content): force fire at will first targeting`).

Files:

- `scripts/script-abi/src/world.rs` — added the narrow active-ability query `query_first_enemy_in_range`.
- `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs` — routed the new query through the existing `select_script_tower_target(TowerTargetPriority::First, ...)` comparator and extracted the common enemy-query plumbing so ordinary selected-priority targeting uses the same unchanged path.
- `scripts/base_content/src/towers/arty.rs` — changed Fire at Will to the extension-aware pulse hook and used the new active-only First query; strengthened the focused test with an explicit non-First tower priority.

### GREEN and regression evidence

Focused GREEN command:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content towers::arty::tests::fire_at_will_uses_first_priority_and_current_control_shell -- --exact
```

Result: exit 0, 1 passed / 0 failed.

Covering content tests:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p base_content towers::
```

Result: exit 0, 21 passed / 0 failed.

ABI tests:

```powershell
cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi
```

Result: exit 0, 11 passed / 0 failed; doc-tests 0 failed.

Affected core adapter tests:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml runtime::native::scripting::parallel_world_adapter::tests::
```

Result: exit 0, 12 passed / 0 failed (140 filtered out). This includes `query_nearest_enemy_uses_selected_tower_priority_for_scripted_towers`, confirming ordinary tower target selection still honors the player's selected priority.

### Fix self-review

- Fire at Will now hard-codes only the comparator choice (`First`) at the narrow active-ability access boundary; it does not duplicate or approximate route progress.
- The authoritative comparator still uses minimum `Creep::path_remaining_distance` with the existing stable entity-id tie break.
- Ordinary `GameWorld::query_nearest_enemy` still resolves the tower's configured target priority before calling the shared query helper.
- The no-target branch still returns `false`, preserving the scheduler opportunity semantics within the active window.
- `fire_shell` remains the single normal/active projectile builder, so current final attack, splash, 3-second stun, and 50% slow for 5 seconds are unchanged.
