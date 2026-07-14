# Final Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Correct tower ability countdown interpolation, icon fallback, multiplayer cast-result retention, and stale documentation without changing `omfue`.

**Architecture:** Keep deterministic simulation results in an ordered per-player resource and expose both the compatibility latest result and a stable per-player snapshot list. Keep wall-clock-only UI state in a small pure state model, while deriving simulated elapsed time from the authoritative pause and speed snapshot. Resolve icon fallback before texture loading so missing authored assets render each tower's own base image and ability initial.

**Tech Stack:** Rust 1.95, specs ECS resources, Fyrox UI, Cargo tests, Lua-authored metadata.

---

### Task 1: Simulation-scaled countdown interpolation

**Files:**
- Modify/Test: `omfx/game/src/native.rs`

- [ ] Add focused tests for `ability_bar_elapsed_sim(elapsed_wall, paused, speed)` at paused, 1x, and 2x, plus an authoritative correction test that rebuilds from the new remaining value.
- [ ] Run `cargo test --manifest-path omfx/Cargo.toml -p game ability_bar_` and confirm the new assertions fail because elapsed wall time is currently passed directly.
- [ ] Add `fn ability_bar_elapsed_sim(elapsed_wall: f32, is_paused: bool, speed: u32) -> f32` returning zero while paused and nonnegative elapsed multiplied by speed otherwise; store pause/speed alongside the snapshot timestamp and pass simulated elapsed into `ability_bar_items_with_names`.
- [ ] Re-run the focused tests and commit the passing omfx behavior after the remaining omfx tasks.

### Task 2: Missing active icon fallback

**Files:**
- Modify/Test: `omfx/game/src/native.rs`

- [ ] Add a focused test using a nonempty missing icon path that expects the resolved model to retain the tower base-image fallback and ability initial rather than the generic placeholder.
- [ ] Run the focused test and confirm RED against current `AbilityBarIcon::Asset` selection/load fallback.
- [ ] Add a pure icon-resolution helper that accepts authored icon loadability and returns `Asset` only when it loads; otherwise return `Fallback { tower_unit_id, initial }`. Update UI texture selection to use the tower template base image and preserve fallback text.
- [ ] Re-run focused icon tests and verify GREEN.

### Task 3: Per-player cast results and readable rejection lifetime

**Files:**
- Modify/Test: `omoba-core/src/runtime/native/comp/lockstep_resources.rs`
- Modify/Test: `omoba-core/src/runtime/native/game_processor.rs`
- Modify/Test: `omoba-core/src/runtime/native/snapshot.rs`
- Modify/Test: `omfx/game/src/native.rs`

- [ ] Add a core drain test where players 7 and 8 each submit one result in a single drain and both remain keyed by player; add snapshot ordering/compatibility assertions.
- [ ] Run focused core tests and confirm RED because one global result is overwritten.
- [ ] Replace the scalar result resource with a deterministic `BTreeMap<u32, TowerAbilityCastResult>` plus global serial, update a player's entry on each drain, initialize it identically on server and replica, and snapshot a sorted `tower_ability_cast_results` vector while preserving `latest_tower_ability_cast_result`.
- [ ] Re-run core tests and verify GREEN.
- [ ] Add pure omfx rejection-state tests for local-player selection, persistence across ordinary snapshots, expiration at 3.0 wall-clock seconds, and immediate removal when its keyed tower disappears.
- [ ] Run focused omfx tests and confirm RED because rejection is cleared at every authoritative boundary and has no timestamp.
- [ ] Store rejection `shown_at`, reconcile only the local player's per-player snapshot result, retain it until 3.0 wall-clock seconds or missing key, and remove the unconditional snapshot clear.
- [ ] Re-run focused omfx tests and verify GREEN.

### Task 4: Documentation cleanup

**Files:**
- Modify: `scripts/base_content/src/towers/arty.rs`
- Modify: `scripts/base_content/src/towers/dart.rs`

- [ ] Correct Arty path documentation and list `arty_slow_50`.
- [ ] Remove the obsolete Dart attack-speed TODO now that final attack-speed aggregation is implemented.
- [ ] Run `cargo test --manifest-path scripts/Cargo.toml -p base_content` without rebuilding/staging fake assets or changing the staged DLL.

### Task 5: Verification, commits, and report

**Files:**
- Append: `.superpowers/sdd/final-review-fix-report.md`

- [ ] Run focused RED/GREEN commands plus full affected omfx, core, backend, and base_content suites; record exact exit status and counts.
- [ ] Review diffs for deterministic ordering, initialization parity, compatibility field preservation, `omfue` exclusion, missing assets, and unintended files.
- [ ] Commit `omb` if it changed, commit `omfx`, then commit root pointer/core/docs/report changes.
- [ ] Append RED/GREEN evidence, final commands/results, hashes, and self-review to the requested report.
