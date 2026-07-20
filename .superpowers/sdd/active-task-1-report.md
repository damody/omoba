# Active Task 1 Report: Generated Active-Ability Metadata

## Status

DONE for the Task 1 generator scope.

## RED evidence

- Command: `cargo test --manifest-path omoba-template-ids/Cargo.toml -p omoba-template-ids three_towers_publish_active_ability_metadata`
- Result: expected compile failure `E0609`; `UpgradeDefConst` had no field `active_ability`.
- The brief's root-level `cargo test -p ...` form could not start because this monorepo has no root `Cargo.toml`, so the crate manifest was supplied explicitly.

## GREEN evidence

- Targeted metadata test: 1 passed, 0 failed.
- `cargo test --manifest-path omoba-template-ids/Cargo.toml -p omoba-template-ids`: 27 passed, 0 failed (1 unit, 19 generated, 7 hero-ability tests; doc tests empty).
- `cargo test --manifest-path omoba-template-ids/Cargo.toml -p omoba-template-ids --features runtime-lua-content`: 36 passed, 0 failed (10 unit/runtime, 19 generated, 7 hero-ability tests; doc tests empty).
- `git diff --check`: exit 0; only Git's existing LF-to-CRLF conversion warnings.

## Changed files

- `omoba-template-ids/src/lib.rs`
- `omoba-template-ids/src/lua_content.rs`
- `omoba-template-ids/build.rs`
- `omoba-template-ids/src/runtime_content.rs`
- `scripts/lua_data/templates/towers.lua`

`src/lua_content.rs` is the shared serde model used by both build-time and runtime loading, so its minimal addition is required even though the brief named the two consumers rather than this shared model file.

## Implementation summary

- Added the exact public `ActiveAbilityConst` and optional `UpgradeDefConst::active_ability` field.
- Added optional Lua parser fields and deterministic `Fixed64` conversion in generated and runtime-loaded metadata.
- Both build-time and runtime loading validate the complete tower manifest with one global ID set: active upgrades must be level 4, IDs must be non-empty and globally unique, cooldown/duration positive, and pulse interval/count either both positive or both zero.
- Ordinary upgrades generate `None` and are covered by the metadata regression.
- Added the exact Boomerang, Arty, and Cake active declarations.
- Preserved Cake's existing twelve approved upgrades and Arty's `arty_slow_50` repair.
- Removed only Boomerang L4's permanent `AttackSpeedMultiplier`; no sub-one speed stat was authored.
- Made Arty Path 3 L4 active-only as required.

## Self-review

- Confirmed active IDs are checked across all towers, not independently per tower.
- Confirmed all authored cooldown/duration/pulse values quantize through the existing 1024-scale conversion.
- Confirmed Cake's existing 12 upgrade definitions were not rewritten.
- Confirmed no `omfue` file was touched.

## Commit

- Subject: `feat(content): define three tower active abilities`
- Hash: reported in the task handoff after commit creation.

## Concerns

- The existing `omoba-core` registry tests have not yet integrated active metadata. Running `cargo test --manifest-path omoba-core/Cargo.toml tower_upgrade_registry -- --nocapture` produced 7 passed and 2 failed: the old Arty cumulative-speed test still requires an L4 multiplier, and strict lint still rejects an empty-effects active upgrade. Task 1 deliberately does not modify `omoba-core`; the later registry integration must treat `active_ability` as valid metadata and update the stale L4 speed expectation.
- `protoc not found; using src/generated/game.rs fallback` remains a pre-existing non-failing warning in the core test run.

## Review fix: deterministic active-ability quantization

### RED evidence

- Added `active_ability_rejects_positive_values_that_quantize_to_zero` for `0.0001` cooldown and pulse interval values.
- Added `active_ability_rejects_non_finite_fixed_values` for `NaN`, positive infinity, and negative infinity.
- Command: `cargo test --manifest-path omoba-template-ids/Cargo.toml -p omoba-template-ids --features runtime-lua-content active_ability_rejects -- --nocapture`
- Result: expected compile failure `E0425`; the wished-for shared `validate_active_ability_quantization` API did not exist.

### GREEN evidence

- The same targeted command passed both negative tests: 2 passed, 0 failed.
- Final default and `runtime-lua-content` suite results are recorded in the task handoff after fresh verification.

### Fix implementation and self-review

- Added one shared quantization validator in `src/lua_content.rs`, which is compiled into both `build.rs` and the runtime crate.
- It rejects every non-finite fixed-point input before conversion, quantizes with the deterministic 1024 scale, requires cooldown/duration raw values greater than zero, and pairs pulse count with the quantized pulse raw value.
- Build-time generated constants and runtime-loaded constants now use the validator's returned raw values directly, ensuring validation and construction cannot disagree.
- Confirmed ordinary zero/zero pulse metadata remains valid and the three authored active declarations retain their expected raw values.

### Fix commit

- Subject: `fix(content): validate quantized active ability timings`
- Hash: reported in the task handoff after commit creation.

### Fix concerns

- The previously reported scope-external `omoba-core` active-only registry assertions remain for the later integration task; this fix does not change that status.
