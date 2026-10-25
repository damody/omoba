# Summon UI errors / prevention

UI01 readonly audit appended a guessed OmRuntime/Private/OmWorldBridgeActor.cpp read although the verified actor lives under OmGenerated/Private. No mutation; continue only with inventoried paths. Do not infer module location from class name.

Design: frame presentation entity_gen is an admitted u32 disclosure epoch; driver refuses overflow instead of truncating it. Do not reinterpret it as canonical generation. Public kind4/EntitySummon and exact owner are required, not team affiliation alone.

Workflow audit: scripts/build_ue_moba.lua currently invokes historical omfue/build_bridge.bat, whose logic uses shell/PowerShell. Port the active generation/staging path to fixed Lua before running any new UE build; no legacy fallback.

UI02 readonly audit inferred scripts/ue_bridge_stage.lua from a tests filename; the actual stage verifier is scripts/moba_stage_contract.lua. No mutation. Use the inventory result as authoritative; a similarly named test is not evidence of a module path.

UI03 a multi-hunk patch listed an earlier ActionOwner array after a later hunk and failed verification. Inspection confirmed the transaction made no changes. Apply file hunks in ascending source order and inspect actual state before retrying.

UI04/UI05 readonly lookups guessed OmEditor/Public/OmAutomationTestActors.h and OmContentTypes.h despite having enough verified identifiers. Actual fixture actor header is OmEditor/Private/OmAutomationTestActors.h; generated IDs were found by directory rg in OmContentIds.h. No mutation. Future reads must use a path already returned by inventory, not an optional guessed follow-up.

UI06 fixed-Lua codegenV1 reached Rust bridge build then compile E0004 in renderer_fixture.rs intent_value logging. Add explicit opaque summon observation JSON (64-bit refs as strings), not a wildcard. Original API27307 actual1 reaped and62 frozen candidates/raw output retained; preserve the additional renderer fixture source against sealed baseline before editing. CodegenV2 freezes63 candidates. No header/staging success claimed for V1.

UI07 codegenV2 compile E0433: the client crate does not re-export game_proto at crate root. Qualify the verified shared type as omoba_core::game_proto::SecureReplicaTarget, consistent with nearby imports. Original direct API actual1 and frozen63/raw output retained; V3 changes only those two type qualifications in renderer observation.
# UI08 — focused client test constructor

ProbeV1 original API 6181 returned actual exit 1 and was reaped. Core focused test passed; client test compilation failed E0599 because the new test called nonexistent InputBridge::new. Use the established resume_after(0), which preserves capability defaults and input sequencing. Frozen 63 inputs and raw compiler output remain under SummonUIProbeV1. Prevention: inspect existing constructor API before writing fixture setup; production bridge compilation does not compile client cfg(test).

# UI09 — generated anonymous C typedef

UEV1 original API 24666 actual exit 1 reaped. OmSummonInput.h forward-declared `struct OmInputCommand`, while cbindgen exports an anonymous struct typedef. C2371 and downstream incomplete-type diagnostics result. Include the generated ABI header under THIRD_PARTY_INCLUDES, matching OmOwnerControlInput.h. Preserve original 63 inputs, full UE stdout/stderr and UBT log. Prevention: inspect generated C declaration rather than assume a tagged struct; Rust compilation cannot verify C++ header compatibility.

## Active executor completeness

Review also found build_ue_moba's default executor still requested om_restart --with-bridge, reaching the historical batch. The default route now explicitly runs the same fixed Lua bridge pipeline and invokes om_restart build without that flag. Local and source-engine routes already use fixed Lua. Historical files are retained; no old batch was executed during this batch.

# UI10 — exact native case enumeration

NativeV1 original API 2782 actual exit 1 reaped, editor itself actual 0. Six existing regressions passed; the three new cases did not run because native_command_file anchors each supplied full test path, while the helper supplied their category prefix. Report validation correctly rejected the missing category; this is not native feature acceptance. Enumerate TypedInput, SelectionAndSlate and ProfileMigration explicitly. Preserve original command file/report/logs. Prevention: use exact registered names with the existing anchored native contract, never category prefixes.

UI11: A Lua -e helper preparation used C-style quote escaping through PowerShell; Lua received a stray backslash and exited 1 before writes. Replaced the one-liner with a saved fixed-Lua preparation file and plain substring replacement. No native worker or formal source was changed by the failed preparation. Prevention: put multiline/quoted workflow code in a Lua file rather than embedding nested quotes in shell command text.

UI12: Readonly follow-up guessed OmUnitFocusTests.cpp and a Saved errors path; both were absent. Actual registered selection regression is OmSelectedCombatDetailsTests.cpp and the error ledger is this docs/plans file. No source mutation. Use the returned directory inventory and exact test registration names; avoid repeated optional guessed reads.

UI13: Consolidated V1 original API 41462 actual exit 1 reaped. ABI/content/templates/codegen/core/base_content suites passed (base_content 371, including existing long local filtered lifecycle tests). Bridge had ten failures: eight ABI35 assertions, one old terminal-field layout assumption and one presentation11 assertion. Current ABI36/presentation12 are intentional. Preserve the full 63-input cohort, 3487-source manifest and raw output. Extend parent preservation for driver.rs/projection.rs against sealed baseline before editing; update exactly eight version literals (lib5/projection2/driver1), assert appended summon field starts at the old owner-control end and total size ends after the new field, and advance the presentation assertion to12. No production behavior changed by these repairs. Prevention: inventory version-pinned and terminal-layout tests across modules before wider build/capture acceptance. Long content tests were checked using advancing logs and CPU, never duplicated or terminated.

