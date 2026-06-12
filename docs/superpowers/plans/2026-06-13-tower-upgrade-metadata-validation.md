# Tower Upgrade Metadata Validation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add automated metadata lint tests that prove all 48 Bloons-style tower upgrades are complete, valid, and safe for runtime/UI consumption.

**Architecture:** Keep this phase test-focused inside `omoba-core` by extending the existing `TowerUpgradeRegistry` test module. The tests validate registry shape, text, costs, stat-effect key/op conventions, and behavior-flag contracts without changing production behavior unless a real metadata defect is found.

**Tech Stack:** Rust 1.95.0, `cargo test`, `omoba-core`, `omoba-template-ids`, shared `TowerUpgradeDef` / `UpgradeEffect` / `StatOp`.

---

## File Structure

- Modify `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`
  - Existing responsibility: convert generated or active runtime tower upgrade const data into runtime `TowerUpgradeDef` values.
  - New test responsibility: host strict metadata lint helpers near the registry they validate.
  - Rationale: tests need access to `reg.defs` for key-field consistency and exact count, and the file already has related registry tests.

No production files, frontend files, script files, `.bat` files, or generated files should change in this phase unless a test exposes invalid metadata that must be corrected.

---

### Task 1: Add One Strict Metadata Lint Entry Point

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`

- [ ] **Step 1: Write the failing test**

In `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`, inside the existing `#[cfg(test)] mod tests`, add this test after `no_duplicate_keys`:

```rust
    #[test]
    fn all_upgrade_metadata_passes_strict_lint() {
        let reg = TowerUpgradeRegistry::new();
        validate_all_upgrade_metadata(&reg);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: FAIL to compile with an error equivalent to:

```text
cannot find function `validate_all_upgrade_metadata` in this scope
```

- [ ] **Step 3: Add the minimal helper with shape, text, and cost validation**

Still inside `#[cfg(test)] mod tests`, add these imports near the top of the module:

```rust
    use crate::tower_meta::{upgrade_cost, StatOp, UpgradeEffect};
    use omoba_template_ids::{
        TOWER_BOMB_STATS, TOWER_DART_STATS, TOWER_ICE_STATS, TOWER_TACK_STATS,
    };
    use std::collections::BTreeSet;
```

Then add this helper block after the existing tests in the same module:

```rust
    fn validate_all_upgrade_metadata(reg: &TowerUpgradeRegistry) {
        validate_registry_shape(reg);
        validate_text_and_costs(reg);
    }

    fn expected_towers() -> [(&'static str, i32); 4] {
        [
            (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
            (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
            (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
            (TOWER_ICE.as_str(), TOWER_ICE_STATS.cost),
        ]
    }

    fn validate_registry_shape(reg: &TowerUpgradeRegistry) {
        assert_eq!(
            reg.defs.len(),
            48,
            "tower upgrade registry must contain exactly 4 towers * 3 paths * 4 levels"
        );

        let expected_tower_ids: BTreeSet<&str> =
            expected_towers().into_iter().map(|(kind, _)| kind).collect();
        let actual_tower_ids: BTreeSet<&str> =
            reg.iter_all().map(|def| def.tower_kind.as_str()).collect();
        assert_eq!(
            actual_tower_ids, expected_tower_ids,
            "tower upgrade registry contains unexpected tower ids"
        );

        for (kind, _) in expected_towers() {
            for path in 0..=2u8 {
                for level in 1..=4u8 {
                    let def = reg.get(kind, path, level).unwrap_or_else(|| {
                        panic!("missing upgrade def for {kind} path {path} level {level}")
                    });
                    assert_eq!(def.tower_kind, kind, "{kind} path {path} level {level}: tower_kind mismatch");
                    assert_eq!(def.path, path, "{kind} path {path} level {level}: path mismatch");
                    assert_eq!(def.level, level, "{kind} path {path} level {level}: level mismatch");
                }
            }
        }

        for def in reg.iter_all() {
            assert!(
                expected_tower_ids.contains(def.tower_kind.as_str()),
                "{} path {} level {}: unexpected tower_kind",
                def.tower_kind,
                def.path,
                def.level
            );
            assert!(
                def.path <= 2,
                "{} path {} level {}: path must be 0..=2",
                def.tower_kind,
                def.path,
                def.level
            );
            assert!(
                (1..=4).contains(&def.level),
                "{} path {} level {}: level must be 1..=4",
                def.tower_kind,
                def.path,
                def.level
            );
        }
    }

    fn validate_text_and_costs(reg: &TowerUpgradeRegistry) {
        for (kind, base_cost) in expected_towers() {
            for path in 0..=2u8 {
                for level in 1..=4u8 {
                    let def = reg
                        .get(kind, path, level)
                        .expect("shape validation guarantees every upgrade exists");
                    let label = upgrade_label(kind, path, level);

                    assert!(!def.name.trim().is_empty(), "{label}: name must not be empty");
                    assert!(
                        !def.description.trim().is_empty(),
                        "{label}: description must not be empty"
                    );
                    assert_ne!(
                        def.name.trim(),
                        def.description.trim(),
                        "{label}: description must not be identical to name"
                    );
                    assert!(def.cost > 0, "{label}: cost must be positive");
                    assert_eq!(
                        def.cost,
                        upgrade_cost(base_cost, level),
                        "{label}: cost must match upgrade_cost(base_cost, level)"
                    );
                    assert!(
                        !def.effects.is_empty(),
                        "{label}: upgrade must contain at least one effect"
                    );
                }
            }
        }
    }

    fn upgrade_label(kind: &str, path: u8, level: u8) -> String {
        format!("{kind} path {path} level {level}")
    }
```

- [ ] **Step 4: Run test to verify it passes or exposes metadata defects**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: PASS if current metadata satisfies shape, text, and cost rules. If it fails, fix only the metadata field named in the panic, then rerun the same command.

- [ ] **Step 5: Commit**

```powershell
git add omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
git commit -m "test(omoba-core): add strict tower upgrade metadata lint entry"
```

---

### Task 2: Add Stat Effect Key and Op Validation

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`

- [ ] **Step 1: Write the failing test expansion**

In `validate_all_upgrade_metadata`, add the stat validation call:

```rust
    fn validate_all_upgrade_metadata(reg: &TowerUpgradeRegistry) {
        validate_registry_shape(reg);
        validate_text_and_costs(reg);
        validate_stat_effects(reg);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: FAIL to compile with an error equivalent to:

```text
cannot find function `validate_stat_effects` in this scope
```

- [ ] **Step 3: Add stat-effect validation helpers**

Inside the same `#[cfg(test)] mod tests`, add this helper block below `validate_text_and_costs`:

```rust
    fn validate_stat_effects(reg: &TowerUpgradeRegistry) {
        for def in reg.iter_all() {
            let label = upgrade_label(&def.tower_kind, def.path, def.level);
            for effect in &def.effects {
                let UpgradeEffect::StatMod { key, value, op } = effect else {
                    continue;
                };

                assert!(!key.trim().is_empty(), "{label}: stat key must not be empty");
                assert!(value.is_finite(), "{label}: stat key {key} value must be finite");
                assert_ne!(*value, 0.0, "{label}: stat key {key} value must not be zero");

                match op {
                    StatOp::Add => assert!(
                        key.ends_with("_bonus") || is_absolute_stat_key(key),
                        "{label}: StatOp::Add key `{key}` must end with `_bonus` or be explicitly allowlisted"
                    ),
                    StatOp::Mul => assert!(
                        key.ends_with("_multiplier"),
                        "{label}: StatOp::Mul key `{key}` must end with `_multiplier`"
                    ),
                }
            }
        }
    }

    fn is_absolute_stat_key(key: &str) -> bool {
        matches!(key, "crit_chance" | "crit_bonus")
    }
```

- [ ] **Step 4: Run test to verify it passes or exposes metadata defects**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: PASS if current stat metadata follows suffix and allowlist rules.

If the test fails because a key is valid but absolute by design, add that exact key to `is_absolute_stat_key` and include a short inline comment explaining why it is absolute. Example:

```rust
    fn is_absolute_stat_key(key: &str) -> bool {
        matches!(
            key,
            "crit_chance" | "crit_bonus" // Dart crit path uses absolute crit values for current content.
        )
    }
```

If the test fails because the key suffix or `StatOp` is wrong, correct the source metadata instead of weakening the lint.

- [ ] **Step 5: Commit**

```powershell
git add omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
git commit -m "test(omoba-core): lint tower upgrade stat effects"
```

---

### Task 3: Add Behavior Flag Contract Validation

**Files:**
- Modify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`

- [ ] **Step 1: Write the failing test expansion**

In `validate_all_upgrade_metadata`, add the behavior flag validation call:

```rust
    fn validate_all_upgrade_metadata(reg: &TowerUpgradeRegistry) {
        validate_registry_shape(reg);
        validate_text_and_costs(reg);
        validate_stat_effects(reg);
        validate_behavior_flags(reg);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: FAIL to compile with an error equivalent to:

```text
cannot find function `validate_behavior_flags` in this scope
```

- [ ] **Step 3: Add behavior flag validation helpers**

Inside the same `#[cfg(test)] mod tests`, add this helper block below `validate_stat_effects`:

```rust
    fn validate_behavior_flags(reg: &TowerUpgradeRegistry) {
        for def in reg.iter_all() {
            let label = upgrade_label(&def.tower_kind, def.path, def.level);
            for effect in &def.effects {
                let UpgradeEffect::BehaviorFlag { flag } = effect else {
                    continue;
                };

                assert!(
                    !flag.trim().is_empty(),
                    "{label}: behavior flag must not be empty"
                );
                assert!(
                    supported_behavior_flags(def.tower_kind.as_str()).contains(flag.as_str()),
                    "{label}: unsupported behavior flag `{flag}` for {}",
                    def.tower_kind
                );
            }
        }
    }

    fn supported_behavior_flags(tower_kind: &str) -> &'static [&'static str] {
        match tower_kind {
            "tower_dart" => &[
                "sharp_pierce",
                "spike_o_pult",
                "triple_shot",
                "fan_club",
                "always_crit",
                "mega_crit",
            ],
            "tower_bomb" => &[
                "bomb_stun",
                "missile",
                "moab_assassin",
                "frag_8",
                "frag_12",
                "frag_recursive",
                "frag_homing",
            ],
            "tower_tack" => &[
                "blade_shooter",
                "burn_tier1",
                "burn_tier2",
                "ring_of_fire",
                "inferno_ring",
                "needles_12",
                "needles_16",
                "needles_32",
            ],
            "tower_ice" => &[
                "deep_freeze",
                "absolute_zero",
                "arctic_aura_20",
                "snowstorm",
                "cryo_cannon",
                "embrittle_15",
                "refreeze",
                "embrittle_25",
                "icicle_impale",
            ],
            _ => &[],
        }
    }
```

- [ ] **Step 4: Run test to verify it passes or exposes metadata defects**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml all_upgrade_metadata_passes_strict_lint
```

Expected: PASS if every behavior flag in metadata is part of the approved script contract.

If the test fails because a flag is misspelled, correct the metadata flag. If the test fails because a genuinely supported script flag is missing from the allowlist, add the exact string to the matching tower list and keep the list sorted by the tower's upgrade path order.

- [ ] **Step 5: Commit**

```powershell
git add omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
git commit -m "test(omoba-core): lint tower upgrade behavior flags"
```

---

### Task 4: Run Focused and Regression Verification

**Files:**
- Verify: `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs`
- Verify: `docs/superpowers/specs/2026-06-13-tower-upgrade-metadata-validation-design.md`

- [ ] **Step 1: Run the focused tower upgrade tests**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade
```

Expected: PASS. The output should include the existing `tower_upgrade_registry` and `tower_upgrade_rules` tests plus the new strict metadata lint test.

- [ ] **Step 2: Run all `omoba-core` tests**

Run:

```powershell
cargo test --manifest-path omoba-core/Cargo.toml
```

Expected: PASS. If unrelated pre-existing tests fail, capture the failing test names and rerun the focused command from Step 1 to prove this change is isolated.

- [ ] **Step 3: Run backend library regression tests**

Run:

```powershell
cargo test --manifest-path omb/Cargo.toml -p omobab --lib
```

Expected: PASS. This confirms the stricter metadata tests did not require runtime behavior changes that break backend code.

- [ ] **Step 4: Check the working tree**

Run:

```powershell
git status --short
```

Expected: only intended files are modified. For this phase, intended files are:

```text
omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs
```

If metadata defects were corrected, include the exact content source files shown by `git status --short` in the final commit and explain why the corrections were necessary.

- [ ] **Step 5: Commit final verification or metadata corrections**

If Task 1-3 commits already contain all changes and no extra metadata corrections were needed, do not create an empty commit.

If metadata corrections were needed after the lint helpers were added, commit them:

```powershell
git add omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs scripts/lua_data/templates.lua omoba-template-ids/src/lib.rs
git commit -m "fix(content): satisfy tower upgrade metadata lint"
```

Only include files that actually changed. If `omoba-template-ids/src/lib.rs` was regenerated by the build and `scripts/lua_data/templates.lua` was not changed, stage the generated file only after confirming this repository normally tracks generated template constants.

---

## Self-Review Notes

- Spec coverage: shape, text, cost, stat effect, and behavior flag validation are all covered by Tasks 1-3.
- Non-goals preserved: no runtime application tests, script behavior tests, frontend work, balance tuning, transport changes, or DLL rebuilds are part of this plan.
- Type consistency: helper snippets use existing `TowerUpgradeRegistry`, `TowerUpgradeDef`, `UpgradeEffect`, `StatOp`, `upgrade_cost`, `TOWER_*`, and `TOWER_*_STATS` names from current `omoba-core` and `omoba-template-ids`.
- Verification: focused and broad commands match the approved design.
