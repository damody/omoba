# Targeted control item errors and prevention

## E1530 — read-only source discovery

Two reads guessed nonexistent paths: OmRuntime/Public/OmItemCatalog.h and scripts/base_content/src/recovery_channel_tests.rs. No files were modified by these reads. Actual generated catalog lives in OmGenerated/Public; existing formal item fixtures live in single_lane_match_tests.rs. Discover with `rg --files` before reading a new path. No build/test failure has occurred yet in this batch. Failed validation evidence will be appended without overwriting prior runs.

Follow-up: a guessed native/buff.rs read also failed; discover the actual buff store source before reading. This repeats the same read-only discovery error and reinforces the prevention rule.

Two further read-only guesses (`native/mod.rs`, `OmHealingReductionItemTests.cpp`) failed. Actual files were subsequently discovered as `runtime/native.rs` and `OmHealingReductionTests.cpp`. No production or test result was affected. All paths in acceptance preparation must be asserted with `is_file` before use.

## Final validation

Full 8-scope Rust1849passed/0failed/9existingignored; formal backend and UE5.8 build passed; native78 completed with0failed/0notrun,1existingwarningcase. No compile/test failure or production repair was required. Ten frozen formal inputs and four generated outputs are bound to the final3443-source fingerprint;3429 accepted prior inputs unchanged. Nine new/changed images actually viewed,36 identical accepted images inherited. Existing worldless first-kill HUD crop and map focus-label overlaps remain explicitly pending. Native warnings and Rust ignored cases are not hidden or represented as new failures.
