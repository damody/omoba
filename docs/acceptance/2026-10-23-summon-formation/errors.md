# Errors and prevention

- Found permissive JSON fallback, fractional-count truncation and hardcoded rank-one defaults in formation handler. Resolve typed bounded data and all coordinates before mutation; omitted fields use exact current-rank content.
- Diagnostic mistake: guessed scripts/script-abi/src/host.rs; rg showed the actual world.rs. Use rg --files before selecting paths. No production worker failure resulted.
- GameWorld stable trait method table is full; retain existing ABI and managed adapter transaction. Do not add an ABI batch method or simulate rollback with partial despawn events.
- Preserve prior pure Slate layout and bounded reservation implementations. Unattended omoba-48 deadline remains2026-10-25 00:33:38 Taipei.
- V1 backend/build/base compilation failed E0599: Fixed64 intentionally has no primitive abs() API. Waited for all Rust/build readers and watchers to retire, then replaced magnitude checks with symmetric lower/upper comparisons (including test tolerance). Preserve failed V1 logs/receipts; V2 is a new run. Prevention: inspect wrapper API before using primitive methods; range comparisons also avoid minimum-integer absolute overflow.
- A later read-only search included the optional nonexistent OmRuntime/Resources root copied from the source guard's skip-if-missing list. Generated description was correctly found in OmGenerated/om_codegen_manifest.json and Private/OmContentClasses.cpp. Use discovered existing roots; a guarded optional root is not evidence it exists.

Final acceptance: Rust1970 passed /10 existing ignored, World59 success, UE5.8/backend/contracts success,3409 formal inputs unchanged. Failed V1 evidence preserved.
