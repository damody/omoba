# All-Tower Skill Repairs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every shipped tower upgrade reachable and ensure its runtime behavior matches authored metadata without unbounded projectile chains.

**Architecture:** Strengthen generated/runtime metadata validation across all seven towers, apply stat aggregation consistently in the script adapter, carry projectile provenance through hit dispatch, and repair tower scripts with deterministic cooldowns and bounded effects.

**Tech Stack:** Rust 1.95.0, specs 0.20, abi_stable, Fixed64, Lua-generated omoba-template-ids.

## Global Constraints

- Ignore `omfue` completely.
- Work only in `D:/code/omoba/.worktrees/three-tower-active-abilities` on `codex/three-tower-active-abilities`.
- Use TDD: add and observe a failing regression test before each production fix.
- Keep three paths × four levels and the primary-4/secondary-2 rule.
- Preserve deterministic Fixed64 simulation and bounded per-hit work.
- Do not hand-edit generated protobuf or template output.

---

### Task 1: Seven-Tower Registry and Attack-Speed Semantics

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`
- Modify: `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs`
- Modify: `scripts/lua_data/templates/towers.lua`
- Modify: `omoba-template-ids/tests/generated.rs`
- Test: the same Rust files.

**Interfaces:**
- Produces a registry containing 84 upgrade definitions and a scripted-tower effective attack interval of `base_interval / final_attack_speed_mult`.

- [ ] Add failing tests that require 7 tower kinds × 3 paths × 4 levels, lint every flag/stat, and assert an `AttackSpeedMultiplier=1.2` buff changes a 1-second scripted interval to approximately 0.833 seconds.
- [ ] Run the focused tests and record the expected failures: registry has only 48 definitions and adapter returns the unchanged base interval.
- [ ] Include Arty, Boomerang, and Cake in registry construction and supported-flag validation. Add Cake's approved 12 upgrade definitions to Lua.
- [ ] Normalize every tower `AttackSpeedMultiplier` to speed-multiplier semantics: values greater than 1 make attacks faster and use `op="mul"`. For descriptions that state total path speed (+25%, +50%, +100%, +200%), encode incremental ratios so cumulative products equal 1.25, 1.5, 2.0, and 3.0.
- [ ] Make `parallel_world_adapter::get_asd_interval` return `TAttack.asd.v / UnitStats::final_attack_speed_mult(entity)`, clamped to a positive minimum.
- [ ] Repair the three known stale generated tests: expect the current authored Dart display name, current authoritative TD_STRESS HP, and compare normalized repository-relative paths so `.worktrees` does not trigger the legacy-story-path assertion.
- [ ] Run `cargo test --manifest-path omoba-template-ids/Cargo.toml` and focused `omoba-core` registry/adapter tests; expect all pass.
- [ ] Commit with `feat(core): enable upgrades for all seven towers`.

### Task 2: Bounded Projectile Hit Provenance

**Files:**
- Modify: `scripts/script-abi/src/types.rs`
- Modify: `scripts/script-abi/src/script.rs`
- Modify: `omoba-core/src/runtime/native/comp/projectile.rs`
- Modify: `omoba-core/src/runtime/native/comp/outcome.rs`
- Modify: `omoba-core/src/runtime/native/tick/projectile_tick.rs`
- Modify: `omoba-core/src/runtime/native/game_processor.rs`
- Modify: `scripts/base_content/src/towers/bomb.rs`
- Modify: `scripts/base_content/src/towers/boomerang.rs`
- Test: focused ABI/core/base_content tests.

**Interfaces:**
- Adds projectile `generation: u8` and `kind_id` to a projectile-hit context delivered to scripts; all primary shots use generation 0 and child projectiles increment with saturation.

- [ ] Add failing tests proving frag children cannot spawn another ordinary frag wave and a two-target ricochet terminates after its configured bounce budget.
- [ ] Observe RED with the current unlimited `on_attack_hit` recursion.
- [ ] Add an ABI-safe projectile-hit hook carrying kind and generation; keep generic melee/non-projectile `on_attack_hit` behavior intact.
- [ ] Dispatch tower projectile logic through the provenance-aware hook. Bomb ordinary frag paths trigger only from generation 0; recursive cluster creates exactly four generation-2 children and stops. Homing fragments use `PathSpec::Homing` with deterministic target selection.
- [ ] Give Boomerang ordinary ricochet a one-bounce budget and storm a two-bounce budget, never selecting an already-hit immediate target.
- [ ] Run focused scripts/core tests and commit with `fix(content): bound tower projectile chains`.

### Task 3: Existing Four-Tower Effect Correctness

**Files:**
- Modify: `scripts/script-abi/src/world.rs`
- Modify: `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs`
- Modify: `omoba-core/src/runtime/native/tick/tower_tick.rs`
- Modify: `scripts/base_content/src/towers/bomb.rs`
- Modify: `scripts/base_content/src/towers/tack.rs`
- Modify: `scripts/base_content/src/towers/ice.rs`
- Modify: `scripts/lua_data/templates/towers.lua`
- Test: focused core/base_content tests.

**Interfaces:**
- Exposes deterministic tower internal-cooldown get/start calls backed by `Tower.ultimate_cooldown`; ticks it once per authoritative simulation tick.

- [ ] Add failing tests for Bomb Assassin cooldown, Ice Absolute Zero duration/cooldown, Tack burn DoT without slow, Bomb missile speed tiers, and Ice aura values.
- [ ] Observe each regression fail for the audited reason.
- [ ] Add `get_tower_internal_cooldown` and `start_tower_internal_cooldown` ABI calls; tick cooldown with Fixed64 and pause/game-speed semantics.
- [ ] Make Bomb Assassin trigger at most once per authored cooldown, fix missile tier speed, create exactly the described recursive children, and use homing at the final frag tier.
- [ ] Replace Tack burn's fake slow with `dot_damage` buffs (5/10 DPS) and apply burn from Inferno Ring.
- [ ] Make Absolute Zero apply the authored 2-second freeze and internal cooldown. Make Ice aura percentages and Cryo cadence match Lua; remove duplicate every-normal-attack Cryo damage.
- [ ] Run focused tests and commit with `fix(content): align four tower upgrade effects`.

### Task 4: Arty Cumulative Stats and Control

**Files:**
- Modify: `scripts/lua_data/templates/towers.lua`
- Modify: `scripts/base_content/src/towers/arty.rs`
- Test: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`
- Test: `scripts/base_content/src/towers/arty.rs`

**Interfaces:**
- Makes Arty level-by-level final values equal descriptions and implements level-4 50% slow.

- [ ] Add failing tests for Path 1 final attack/splash/range at every level and Path 2 level-4 projectile stun/slow.
- [ ] Observe RED: cumulative modifiers overshoot and slow is absent.
- [ ] Encode incremental modifiers so cumulative final values equal each description; replace unknown `SlowMultiplier` with a supported behavior flag.
- [ ] Make Arty level-4 control shells carry 3-second stun, slow factor 0.5, and an authored finite slow duration.
- [ ] Run registry and Arty tests and commit with `fix(content): align arty upgrade behavior`.
