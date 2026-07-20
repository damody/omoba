# Active Task 2 Report: Runtime Active-Ability Metadata

## Status

DONE for the Task 2 runtime registry and snapshot scope.

## RED evidence

- Added registry test `exactly_three_shipped_active_abilities_are_valid` and snapshot test `tower_upgrade_snapshot_exposes_active_ability_metadata` before production changes.
- Command: `cargo test --manifest-path omoba-core/Cargo.toml exactly_three_shipped_active_abilities_are_valid`
- Result: expected compile failures `E0609`; neither `TowerUpgradeDef` nor `TowerUpgradeDefSnapshot` exposed `active_ability`.

## GREEN evidence

- Registry active metadata test: 1 passed, 0 failed.
- Snapshot active metadata test: 1 passed, 0 failed.
- `cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade_registry`: 10 passed, 0 failed; includes all seven towers / 84 definitions strict lint.
- `cargo test --manifest-path omoba-core/Cargo.toml`: 123 passed, 1 pre-existing scope-external failure (details under Concerns).
- `git diff --check`: exit 0; only Git LF-to-CRLF conversion warnings.

## Implementation summary

- Added serializable `TowerActiveAbilityDef` with all display and deterministic `Fixed64` runtime timing fields.
- Added `TowerUpgradeDef::active_ability` and converted `ActiveAbilityConst` in `TowerUpgradeRegistry::new_with_cost_multiplier`.
- Mirrored the complete active metadata into `TowerUpgradeDefSnapshot`.
- Extended strict registry validation: level 4 only; exactly one declaration for Arty, Cake Splash, and Boomerang; globally unique non-empty IDs; exact 10-second cooldown; positive duration; paired pulse interval/count.
- Preserved the behavior-flag allowlists for every shipped Boomerang, Arty, and Cake flag; the seven-tower/84-definition lint passes.
- Corrected stale core assumptions: an effects-empty upgrade is valid when it declares an active ability, and Arty attack-speed milestones stop after L3 at cumulative 1.25/1.5/2.0 while L4 adds no permanent `AttackSpeedMultiplier`.

## Self-review

- Confirmed ordinary upgrades map to `active_ability: None` and all three scoped L4 upgrades map to `Some`.
- Confirmed snapshot data owns cloned strings/metadata rather than borrowing generated constants.
- Confirmed active IDs are validated globally and per-tower declaration counts are enforced independently.
- Confirmed pulse-less Boomerang metadata accepts only the zero/zero pair, while Arty and Cake require both positive fields.
- Confirmed no `omfue` file or file outside the requested Task 2/report scope remains modified.

## Commit

- Subject: `feat(core): expose tower active ability metadata`
- Hash: reported in the task handoff after commit creation.

## Concerns

- The full core suite has one reproducible pre-existing failure: `runtime::native::tick::creep_wave::tests::td_wave_clear_awards_btd_easy_round_income_to_heroes` panics because its test world lacks `PendingDebugCreepSpawnQueue`. The isolated test fails identically. This task does not touch creep-wave initialization, so it remains scope-external.
- Core builds emit the existing non-failing warning: `protoc not found; using src/generated/game.rs fallback`.
