# Three-Tower Active Abilities Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Bloons-style upgrades for Boomerang, Arty, and Cake Splash, with one authoritative active ability per tower and a bottom-center global ability bar.

**Architecture:** Lua remains the tower-content source of truth. Generated template metadata feeds the shared native runtime, which owns unlocks, deterministic cooldowns, pulse scheduling, validation, and render snapshots; `base_content.dll` owns tower-specific effects, while `omfx` renders and dispatches the global bar without tower-kind conditionals.

**Tech Stack:** Rust 1.95.0, specs 0.20 ECS, abi_stable, omoba-sim `Fixed64`, prost/KCP lockstep input, Lua-generated `omoba-template-ids`, Fyrox/omfx, eui.

## Global Constraints

- Use Rust 1.95.0 for the host and script DLL; do not change `rust-toolchain.toml`.
- Keep the existing three-path, four-level, primary-4/secondary-2 upgrade rule.
- All three test cooldowns are exactly 10 seconds and are authored in Lua.
- Active abilities are instant-cast; do not add a targeting cursor.
- Simulation state and timers use deterministic `Fixed64`, stop while paused, and follow authoritative game speed.
- Do not add tower-kind branches to `omb` or `omfx`; tower-specific effects stay in `base_content.dll`.
- Preserve the existing untracked/modified `omfue` submodule state.
- Rebuild and stage `base_content.dll` before host or end-to-end verification.
- Any new `.bat` file must use CRLF, though this plan creates no batch file.

---

### Task 1: Generated Active-Ability Metadata

**Files:**
- Modify: `omoba-template-ids/src/lib.rs`
- Modify: `omoba-template-ids/build.rs`
- Modify: `omoba-template-ids/src/runtime_content.rs`
- Modify: `scripts/lua_data/templates/towers.lua`
- Test: `omoba-template-ids/src/lib.rs`

**Interfaces:**
- Consumes: existing `UpgradeDefConst`, Lua `upgrades`, and deterministic `Fixed64` conversion.
- Produces: `ActiveAbilityConst` and `UpgradeDefConst::active_ability: Option<ActiveAbilityConst>` for the runtime registry.

- [ ] **Step 1: Add a failing generated-metadata test**

Add a test that looks up the three active upgrades and asserts their IDs, cooldowns, durations, and pulse configuration:

```rust
#[test]
fn three_towers_publish_active_ability_metadata() {
    let cases = [
        (TOWER_BOOMERANG, 1, 3, "boomerang_turbo_charge", 5, 0, 0),
        (TOWER_ARTY, 2, 3, "arty_fire_at_will", 3, 500, 6),
        (TOWER_CAKE_SPLASH, 1, 3, "cake_dessert_party", 5, 500, 10),
    ];
    for (tower, path, level, id, duration, pulse_ms, pulse_count) in cases {
        let def = active_tower_upgrades(tower).unwrap()[path][level];
        let ability = def.active_ability.expect("active ability");
        assert_eq!(ability.ability_id, id);
        assert_eq!(ability.cooldown, Fixed64::from_i32(10));
        assert_eq!(ability.duration, Fixed64::from_i32(duration));
        assert_eq!(ability.pulse_interval, Fixed64::from_raw(pulse_ms * 1024 / 1000));
        assert_eq!(ability.pulse_count, pulse_count);
    }
}
```

- [ ] **Step 2: Run the test and confirm failure**

Run: `cargo test -p omoba-template-ids three_towers_publish_active_ability_metadata`

Expected: compile failure because `active_ability` and `ActiveAbilityConst` do not exist.

- [ ] **Step 3: Add the const metadata type and parser model**

Add this exact public type and field:

```rust
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ActiveAbilityConst {
    pub ability_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
    pub cooldown: Fixed64,
    pub duration: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulse_count: u16,
}

pub struct UpgradeDefConst {
    // existing fields
    pub active_ability: Option<ActiveAbilityConst>,
}
```

Extend both build-time and runtime Lua models with optional `active_ability`. Reject an ability unless it is on level 4, has a non-empty globally unique ID, positive cooldown/duration, and either both pulse fields are positive or both are zero. Generate `None` for ordinary upgrades and `Some(ActiveAbilityConst { ... })` for active upgrades.

- [ ] **Step 4: Author the three Lua declarations and Cake's twelve upgrades**

Use these exact active declarations:

```lua
active_ability = {
  ability_id = "boomerang_turbo_charge", display_name = "渦輪充能",
  description = "5 秒內攻擊間隔 ×0.35，每次攻擊額外投射 2 枚回力鏢",
  icon = "assets/ui/abilities/boomerang_turbo_charge.png",
  cooldown = 10.0, duration = 5.0,
}
```

```lua
active_ability = {
  ability_id = "arty_fire_at_will", display_name = "火力全開",
  description = "3 秒內額外發射 6 發砲彈",
  icon = "assets/ui/abilities/arty_fire_at_will.png",
  cooldown = 10.0, duration = 3.0, pulse_interval = 0.5, pulse_count = 6,
}
```

```lua
active_ability = {
  ability_id = "cake_dessert_party", display_name = "甜點狂歡",
  description = "5 秒內每 0.5 秒造成脈衝並強化附近友軍塔",
  icon = "assets/ui/abilities/cake_dessert_party.png",
  cooldown = 10.0, duration = 5.0, pulse_interval = 0.5, pulse_count = 10,
}
```

Keep existing Boomerang and Arty paths except: remove Boomerang's permanent level-4 turbo multiplier, make Arty Path 3 level 4 active-only, and make Arty control Path 2 level 4 expose `arty_slow_50`. Add Cake's exact damage/rhythm/frosting values from the approved design spec.

- [ ] **Step 5: Run generator tests**

Run: `cargo test -p omoba-template-ids`

Expected: all tests pass; the three active metadata cases pass and ordinary upgrades still generate `None`.

- [ ] **Step 6: Commit**

```powershell
git add omoba-template-ids scripts/lua_data/templates/towers.lua
git commit -m "feat(content): define three tower active abilities"
```

---

### Task 2: Runtime Metadata Registry and Validation

**Files:**
- Modify: `omoba-core/src/tower_meta.rs`
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`
- Modify: `omoba-core/src/runtime/native/snapshot.rs`
- Test: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`

**Interfaces:**
- Consumes: `UpgradeDefConst::active_ability` from Task 1.
- Produces: `TowerActiveAbilityDef`, `TowerUpgradeDef::active_ability`, and snapshot metadata used by runtime and UI.

- [ ] **Step 1: Write failing registry validation tests**

```rust
#[test]
fn exactly_three_shipped_active_abilities_are_valid() {
    let reg = TowerUpgradeRegistry::new();
    let active: Vec<_> = reg.iter_all().filter_map(|d| d.active_ability.as_ref()).collect();
    assert_eq!(active.len(), 3);
    assert!(active.iter().all(|a| a.cooldown == Fixed64::from_i32(10)));
    let ids: std::collections::HashSet<_> = active.iter().map(|a| &a.ability_id).collect();
    assert_eq!(ids.len(), 3);
}
```

- [ ] **Step 2: Verify the test fails**

Run: `cargo test --manifest-path omoba-core/Cargo.toml exactly_three_shipped_active_abilities_are_valid`

Expected: compile failure on missing `active_ability`.

- [ ] **Step 3: Add runtime and snapshot types**

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TowerActiveAbilityDef {
    pub ability_id: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub cooldown: Fixed64,
    pub duration: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulse_count: u16,
}
```

Add `pub active_ability: Option<TowerActiveAbilityDef>` to `TowerUpgradeDef` and mirror all display/runtime fields in `TowerUpgradeDefSnapshot`. Convert generated const metadata in `TowerUpgradeRegistry::new_with_cost_multiplier`.

- [ ] **Step 4: Add strict validation**

Extend `validate_all_upgrade_metadata` to assert: active declarations only occur at level 4; exactly one occurs for each of the three scoped tower kinds; IDs are unique/non-empty; cooldown equals 10 seconds; duration is positive; pulse interval/count are paired; all Boomerang, Arty, and Cake behavior flags are enumerated in `supported_behavior_flags`.

- [ ] **Step 5: Run focused and full tests**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade_registry`

Expected: all tower upgrade registry tests pass.

- [ ] **Step 6: Commit**

```powershell
git add omoba-core/src/tower_meta.rs omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs omoba-core/src/runtime/native/snapshot.rs
git commit -m "feat(core): expose tower active ability metadata"
```

---

### Task 3: Deterministic Tower Ability State and Scheduler

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower.rs`
- Create: `omoba-core/src/runtime/native/tick/tower_ability_tick.rs`
- Modify: `omoba-core/src/runtime/native/tick/mod.rs`
- Modify: `omoba-core/src/runtime/native/initialization.rs`
- Test: `omoba-core/src/runtime/native/tick/tower_ability_tick.rs`

**Interfaces:**
- Consumes: `TowerActiveAbilityDef`.
- Produces: `TowerActiveAbilityState`, `TowerAbilityPulseOpportunity`, `acknowledge_pulse(consumed)`, and `tick_tower_abilities(world, dt)`.

- [ ] **Step 1: Add failing state-machine tests**

Cover ready state, activation, duplicate activation rejection, pause via zero `dt`, exact pulse count across uneven ticks, expiry, and cooldown reaching zero:

```rust
#[test]
fn scheduler_emits_each_pulse_once_across_tick_boundaries() {
    let mut state = TowerActiveAbilityState::ready("arty_fire_at_will");
    state.activate(Fixed64::from_i32(10), Fixed64::from_i32(3),
        Fixed64::from_raw(512), 6, 7).unwrap();
    let mut total = 0;
    for _ in 0..30 {
        if state.advance(Fixed64::from_raw(103)).pulse_due {
            state.acknowledge_pulse(true);
            total += 1;
        }
    }
    assert_eq!(total, 6);
    assert_eq!(state.pulses_remaining, 0);
}
```

- [ ] **Step 2: Run the test and confirm failure**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_tick`

Expected: compile failure because the state and scheduler do not exist.

- [ ] **Step 3: Add the serializable state**

```rust
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TowerActiveAbilityState {
    pub ability_id: String,
    pub cooldown_remaining: Fixed64,
    pub active_remaining: Fixed64,
    pub pulse_accumulator: Fixed64,
    pub pulse_interval: Fixed64,
    pub pulses_remaining: u16,
    pub activation_serial: u32,
}
```

Add `active_ability: Option<TowerActiveAbilityState>` to `Tower` with `#[serde(default)]`. Implement `activate`, `advance`, and `acknowledge_pulse` as pure methods. Activation increments `activation_serial`, initializes the window and pulses, and starts cooldown. `advance` subtracts `dt` with saturation and produces at most one due opportunity at each interval. `acknowledge_pulse(true)` decrements `pulses_remaining`; `acknowledge_pulse(false)` leaves the charge available for the next interval. This is what preserves Arty shells when no enemy exists.

- [ ] **Step 4: Add the ECS tick wrapper**

`tick_tower_abilities` advances each tower once per authoritative simulation tick and enqueues `(entity, ability_id, activation_serial, pulse_index)` records into a new `PendingTowerAbilityPulseQueue`. A destroyed entity naturally disappears before future pulses. Dispatch records before advancing the same tower again; feed the script's consumed/not-consumed result to `acknowledge_pulse`. Register the queue wherever other lockstep queues are inserted.

- [ ] **Step 5: Run tests**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_tick`

Expected: all scheduler tests pass.

- [ ] **Step 6: Commit**

```powershell
git add omoba-core/src/runtime/native/comp/tower.rs omoba-core/src/runtime/native/tick omoba-core/src/runtime/native/initialization.rs
git commit -m "feat(core): add deterministic tower ability scheduler"
```

---

### Task 4: Lockstep Cast Input and Authoritative Validation

**Files:**
- Modify: `proto/game.proto`
- Modify: `omoba-core/src/runtime/native/comp/lockstep_resources.rs`
- Modify: `omoba-core/src/runtime/native/tick/player_input_tick.rs`
- Modify: `omoba-core/src/runtime/native/game_processor.rs`
- Modify: `omoba-core/src/runtime/native.rs`
- Test: `omoba-core/src/runtime/native/game_processor.rs`

**Interfaces:**
- Consumes: tower active metadata/state from Tasks 2–3.
- Produces: `TowerAbilityCastInput`, `PendingTowerAbilityCastQueue`, `TowerAbilityCastResult`, `handle_tower_ability_cast_from_input`, and `drain_pending_tower_ability_casts`.

- [ ] **Step 1: Add failing validation tests**

Create fixtures for a player-owned upgraded tower and assert acceptance plus rejections for wrong player, missing tower, locked ability, mismatched ability ID, and cooldown. Every rejection must preserve the old state.

```rust
assert!(handle_tower_ability_cast_from_input(
    &mut world, tower.id(), "boomerang_turbo_charge", owner_pid
).is_ok());
assert_eq!(world.read_storage::<Tower>().get(tower).unwrap()
    .active_ability.as_ref().unwrap().cooldown_remaining, Fixed64::from_i32(10));
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_cast`

Expected: compile failure because the cast input path does not exist.

- [ ] **Step 3: Extend the protobuf without renumbering existing tags**

```proto
message TowerAbilityCastInput {
  uint32 tower_entity_id = 1;
  string ability_id = 2;
}

// PlayerInput.oneof action
TowerAbilityCastInput tower_ability_cast = 16;
```

Regenerate through the repository's existing prost build; never hand-edit `omoba-core/src/generated/game.rs`.

- [ ] **Step 4: Queue and validate casts**

Follow the existing TowerUpgrade queue/drain pattern. Validation order is: entity exists and is a Tower; `PlayerOwner` equals requester; current level-4 definition unlocks the supplied ID; state ID matches; cooldown is zero; game is not ended. On success call `state.activate(...)` using registry metadata and enqueue one activation callback record. Return stable rejection codes: `tower_missing`, `not_owner`, `ability_locked`, `ability_mismatch`, `cooldown_active`, `game_ended`.

Store the latest result per player in an ECS resource:

```rust
pub struct TowerAbilityCastResult {
    pub player_id: u32,
    pub tower_entity_id: u32,
    pub ability_id: String,
    pub accepted: bool,
    pub reason: String,
    pub result_serial: u32,
}
```

Increment `result_serial` for every processed cast, including rejection. Task 8 exposes this value in the render snapshot so the UI displays each backend-derived rejection once.

- [ ] **Step 5: Drain casts at the deterministic boundary**

Drain after player input dispatch and before tower script ticks, then drain scheduled pulses immediately before ordinary tower ticks. Apply the same order in `omb/src/state/core.rs` and `omfx/game/src/sim_runner.rs` so server and replica simulation remain identical.

- [ ] **Step 6: Run tests**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_cast`

Expected: all acceptance, ownership, mismatch, and cooldown tests pass.

- [ ] **Step 7: Commit**

```powershell
git add proto/game.proto omoba-core/src/runtime/native
git commit -m "feat(core): validate tower ability cast inputs"
```

---

### Task 5: Script ABI Hooks and Dispatch

**Files:**
- Modify: `scripts/script-abi/src/script.rs`
- Modify: `scripts/script-abi/src/world.rs`
- Modify: `scripts/script-abi/src/types.rs`
- Modify: `scripts/script-abi/src/lib.rs`
- Modify: `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs`
- Modify: `omoba-core/src/runtime/native/game_processor.rs`
- Test: `omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs`

**Interfaces:**
- Consumes: activation and pulse records from Tasks 3–4.
- Produces: `on_tower_ability_activate`, `on_tower_ability_pulse`, and read-only active-state queries for tower scripts.

- [ ] **Step 1: Add failing adapter tests**

Assert that a script can read active remaining time/serial and that activation/pulse records dispatch exactly once to the registered unit script.

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_dispatch`

Expected: compile failure on missing ABI hooks.

- [ ] **Step 3: Append ABI-safe hooks after the current prefix boundary**

```rust
fn on_tower_ability_activate(
    &self, _tower: EntityHandle, _ability_id: RStr<'_>, _w: &mut GameWorldDyn<'_>
) {}

fn on_tower_ability_pulse(
    &self, _tower: EntityHandle, _ability_id: RStr<'_>, _pulse_index: u16,
    _w: &mut GameWorldDyn<'_>
) -> bool { true }
```

Add world queries:

```rust
fn get_tower_ability_active_remaining(&self, e: EntityHandle, ability_id: RStr<'_>) -> Fixed64;
fn get_tower_ability_activation_serial(&self, e: EntityHandle, ability_id: RStr<'_>) -> u32;
fn reset_attack_backswing(&self, e: EntityHandle);
fn query_friendly_towers_in_range(&self, center: Vec2, radius: Fixed64, exclude: EntityHandle)
    -> RVec<EntityHandle>;
```

Implement queries using current ECS storages and faction/player ownership. `reset_attack_backswing` sets the tower attack counter to its effective interval, making the next ordinary attack ready without generating a duplicate impact.

- [ ] **Step 4: Dispatch queued callbacks**

Resolve `ScriptUnitTag.unit_id` through the existing dispatcher. Activation callbacks run once after a cast is accepted. Pulse callbacks run once per scheduler record and return whether a charge was consumed; pass that result to `TowerActiveAbilityState::acknowledge_pulse`. A missing tower/script cancels the state because it can never consume later pulses; log the cancellation once.

- [ ] **Step 5: Rebuild ABI and run tests**

Run: `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability_dispatch`

Expected: both suites pass.

- [ ] **Step 6: Commit**

```powershell
git add scripts/script-abi omoba-core/src/runtime/native/scripting/parallel_world_adapter.rs omoba-core/src/runtime/native/game_processor.rs
git commit -m "feat(script-abi): add tower active ability hooks"
```

---

### Task 6: Boomerang and Arty Active Behaviors

**Files:**
- Modify: `scripts/base_content/src/towers/boomerang.rs`
- Modify: `scripts/base_content/src/towers/arty.rs`
- Test: `scripts/base_content/src/towers/boomerang.rs`
- Test: `scripts/base_content/src/towers/arty.rs`

**Interfaces:**
- Consumes: ABI hooks and active-state queries from Task 5.
- Produces: Turbo Charge and Fire at Will behavior with existing cross-path modifiers.

- [ ] **Step 1: Add failing pure behavior tests**

Extract small pure helpers and test exact values:

```rust
#[test]
fn turbo_adds_two_projectiles_and_multiplies_interval() {
    assert_eq!(boomerang_projectile_count(1, true), 3);
    assert_eq!(turbo_interval(Fixed64::from_i32(1)), Fixed64::from_raw(358));
}

#[test]
fn fire_at_will_has_six_pulses() {
    assert_eq!(ARTY_FIRE_AT_WILL_PULSES, 6);
    assert_eq!(ARTY_FIRE_AT_WILL_INTERVAL, Fixed64::from_raw(512));
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::`

Expected: compile failure on missing helpers/constants.

- [ ] **Step 3: Implement Turbo Charge**

On activation call `reset_attack_backswing`. During the 5-second active window, derive an effective interval of base final interval ×0.35 and add two projectiles after all permanent count upgrades. Do not mutate permanent attack-speed buffs. Preserve ricochet, shuriken, MOAB Press, and cross-path behavior.

- [ ] **Step 4: Implement Fire at Will and control slow**

On each pulse, select the valid enemy nearest the route exit using the same authoritative target-priority metric as `TowerTargetPriority::First`; spawn one projectile carrying current final attack, splash, stun, and the new 50% Path 2 level-4 slow. If no target exists, return without spawning. The host scheduler already preserves the pulse budget only until the 3-second active window; ensure exactly six pulse opportunities exist.

- [ ] **Step 5: Run script tests**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content towers::`

Expected: Boomerang and Arty helper/behavior tests pass.

- [ ] **Step 6: Commit**

```powershell
git add scripts/base_content/src/towers/boomerang.rs scripts/base_content/src/towers/arty.rs
git commit -m "feat(content): complete boomerang and arty abilities"
```

---

### Task 7: Cake Splash Upgrade Behaviors

**Files:**
- Modify: `scripts/base_content/src/towers/cake_splash.rs`
- Test: `scripts/base_content/src/towers/cake_splash.rs`

**Interfaces:**
- Consumes: Cake metadata from Task 1, active pulse hooks and friendly-tower query from Task 5.
- Produces: burn, secondary pulses, frosting, and Dessert Party.

- [ ] **Step 1: Add failing Cake rule tests**

```rust
#[test]
fn cake_party_emits_ten_half_damage_pulses() {
    assert_eq!(CAKE_PARTY_PULSES, 10);
    assert_eq!(CAKE_PARTY_DAMAGE_FACTOR, Fixed64::from_raw(512));
}

#[test]
fn strongest_frosting_wins() {
    assert_eq!(stronger_frosting(FROST_20, FROST_50), FROST_50);
}
```

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content cake_splash`

Expected: compile failure on missing behavior helpers.

- [ ] **Step 3: Implement normal attacks and burn**

Use final attack/range. Path 2 levels 2–3 schedule one/two immediate follow-up `deal_damage_splash` calls at 50% damage without recursively triggering another normal attack. Apply a source-stable burn ID `cake_burn:<tower_entity>` for 3 seconds; refresh that source rather than stacking it. L3 burn is 20% of triggering damage per second and L4 is 40%.

- [ ] **Step 4: Implement frosting aggregation**

Apply stable stat buffs with `movespeed_bonus_percentage` and `incoming_damage_percentage`. L1/L2/L4 use −0.20/−0.35/−0.50 movement modifiers; L3/L4 use +0.15/+0.25 incoming damage. Give all frosting sources the common aggregation family so `BuffStore` selects the strongest magnitude instead of multiplying sources; hits refresh the 2-second duration.

- [ ] **Step 5: Implement Dessert Party**

Every active pulse deals 50% final attack damage over current final range. Query friendly towers in range excluding the caster and refresh a 0.6-second `cake_party_haste:<caster>` buff with 25% faster attack interval. The short refresh duration prevents a stale aura after sale while remaining continuous at 0.5-second pulses.

- [ ] **Step 6: Run Cake and BuffStore tests**

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content cake_splash`

Run: `cargo test --manifest-path omoba-core/Cargo.toml buff_store`

Expected: Cake behavior tests pass and existing buff aggregation tests remain green.

- [ ] **Step 7: Commit**

```powershell
git add scripts/base_content/src/towers/cake_splash.rs
git commit -m "feat(content): add cake splash upgrade paths"
```

---

### Task 8: Render Snapshot and Bottom-Center Ability Bar

**Files:**
- Modify: `omoba-core/src/runtime/native/snapshot.rs`
- Modify: `omfx/game/src/native.rs`
- Modify: `omfx/game/src/sim_runner.rs`
- Test: `omfx/game/src/native.rs`

**Interfaces:**
- Consumes: active state and metadata from Tasks 2–4.
- Produces: `TowerActiveAbilitySnapshot`, stable bar view models, mouse/keyboard cast dispatch.

- [ ] **Step 1: Add failing snapshot/view-model tests**

```rust
#[test]
fn ability_bar_orders_by_spawn_then_entity_and_limits_shortcuts() {
    let items = ability_bar_items(test_snapshot_with_eight_abilities());
    assert_eq!(items.len(), 8);
    assert_eq!(items[0].shortcut, Some(1));
    assert_eq!(items[5].shortcut, Some(6));
    assert_eq!(items[6].shortcut, None);
}
```

Also test one-decimal countdown formatting, missing-icon fallback, duplicate tower suffixes, and removal after the tower disappears.

- [ ] **Step 2: Verify failure**

Run: `cargo test --manifest-path omfx/Cargo.toml -p game ability_bar`

Expected: compile failure because the snapshot/view model is absent.

- [ ] **Step 3: Add snapshot fields**

```rust
#[derive(Clone, Debug)]
pub struct TowerActiveAbilitySnapshot {
    pub ability_id: String,
    pub display_name: String,
    pub description: String,
    pub icon: String,
    pub cooldown_total: f32,
    pub cooldown_remaining: f32,
    pub active_remaining: f32,
    pub activation_serial: u32,
}
```

Add `tower_active_ability: Option<TowerActiveAbilitySnapshot>` and stable `spawn_order` to tower render entities. Include ability data only for living towers; `omfx` filters by `owner_player_id`.

Add `latest_tower_ability_cast_result: Option<TowerAbilityCastResultSnapshot>` to `RenderSnapshot`, copied from the Task 4 resource. The client tracks the last displayed `result_serial`; a new rejected result for the local player shows its stable `reason` on the matching button exactly once.

- [ ] **Step 4: Build a pure bar view model**

Create `AbilityBarItem` keyed by `(tower_entity_id, ability_id)`, sorted by `(spawn_order, tower_entity_id)`. Compute duplicate type suffixes and shortcuts for the visible six-item page. Keep countdown interpolation client-side but replace it whenever a snapshot arrives.

- [ ] **Step 5: Render and diff the bar**

Anchor the bar at bottom center. Reuse widgets by stable key; update only changed icon, cooldown text, overlay, tooltip, or enabled state. Display one decimal while cooling down, `READY` at zero, and a fallback composed from the tower icon plus the first display-name character. Add horizontal wheel/page controls for more than six entries.

- [ ] **Step 6: Dispatch click and keyboard inputs**

Clicks and keys `1`–`6` send `PlayerInput::TowerAbilityCast` with tower entity and ability ID through the existing lockstep input sender. Ignore shortcuts while typing in another UI control. Do not optimistically restart cooldown; wait for the authoritative replica state, but debounce the same button until the next input/snapshot boundary.

- [ ] **Step 7: Run frontend tests**

Run: `cargo test --manifest-path omfx/Cargo.toml -p game ability_bar`

Expected: ordering, paging, fallback, countdown, removal, and shortcut tests pass.

- [ ] **Step 8: Commit in the omfx submodule, then bump the root pointer**

```powershell
git -C omfx add game/src/native.rs game/src/sim_runner.rs
git -C omfx commit -m "feat(ui): add global tower ability bar"
git add omfx
git commit -m "feat(ui): integrate tower ability bar"
```

Do not stage `omfue`.

---

### Task 9: Runtime Integration and End-to-End Verification

**Files:**
- Modify: `omb/src/state/core.rs`
- Modify: `omfx/game/src/sim_runner.rs`
- Test: all affected workspaces

**Interfaces:**
- Consumes: all prior tasks.
- Produces: identical server/replica execution order and a staged runnable build.

- [ ] **Step 1: Add the new deterministic drain order to both runners**

At the existing post-input boundary use the same order in both binaries:

```rust
drain_pending_tower_upgrades(world);
drain_pending_tower_ability_casts(world);
tick_tower_abilities(world, scaled_dt);
drain_pending_tower_ability_callbacks(world);
// existing tower/unit script tick follows
```

Ensure `scaled_dt` is zero while paused and is the same authoritative value already used by hero/tower cooldowns.

- [ ] **Step 2: Rebuild and stage the script DLL**

Run: `cargo build --manifest-path scripts/Cargo.toml -p base_content`

Run: `Copy-Item -Force scripts/target/debug/base_content.dll omb/scripts/base_content.dll`

Expected: both commands succeed and the staged DLL timestamp updates.

- [ ] **Step 3: Run focused workspace tests**

Run: `cargo test -p omoba-template-ids`

Run: `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi`

Run: `cargo test --manifest-path scripts/Cargo.toml -p base_content`

Run: `cargo test --manifest-path omoba-core/Cargo.toml tower_ability`

Expected: all pass.

- [ ] **Step 4: Run backend and frontend tests**

Run: `cargo test --manifest-path omb/Cargo.toml -p omobab`

Run: `cargo test --manifest-path omfx/Cargo.toml -p game`

Expected: all pass with no non-exhaustive `PlayerInput` matches or ABI mismatch.

- [ ] **Step 5: Run metadata/catalog smoke test**

Run: `cargo test --manifest-path omb/Cargo.toml -p omobab --features gen-docs -- --ignored`

Expected: catalog generation smoke test passes and lists all seven towers; the three scoped towers each show twelve upgrades and one active ability.

- [ ] **Step 6: Perform manual gameplay smoke checks**

Run: `run.bat`

Verify each ability unlocks at its Path 2/3 level 4 as specified, appears bottom-center, casts with mouse and `1`–`6`, shows a 10.0-second cooldown, pauses with the game, respects 2× speed, survives reconnect, and disappears on sale. Verify sale during an active window cancels future pulses. Place more than six eligible towers and verify paging without per-frame widget growth.

- [ ] **Step 7: Review repository state and commit final integration**

```powershell
git status --short
git diff --check
git add omb/src/state/core.rs omb/scripts/base_content.dll proto/game.proto omoba-core scripts omoba-template-ids omfx
git commit -m "feat: integrate three tower active abilities"
```

Before committing, remove paths already committed in earlier tasks from the final `git add` list as appropriate. Confirm `omfue` remains unstaged.

---

## Final Acceptance Criteria

- Boomerang, Arty, and Cake Splash each have three complete four-level paths.
- Exactly one level-4 path per scoped tower unlocks an active ability.
- All three cooldowns are Lua-authored at 10 seconds.
- Casts are authoritative, player-owned, deterministic, pause-safe, and replica-safe.
- Turbo Charge, Fire at Will, Dessert Party, Arty slow, Cake burn, and Cake frosting match the approved values.
- The global ability bar is bottom-center, stable-key diffed, supports six shortcuts and overflow, and removes sold towers.
- Script ABI, generated metadata, backend, frontend, and catalog tests pass using Rust 1.95.0.
