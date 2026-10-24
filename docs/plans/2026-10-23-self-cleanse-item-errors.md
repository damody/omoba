# Self cleanse item errors and prevention

## E1533 — initial implementation

Reuse the existing bounded BuffStore::cleanse_controls implementation; do not duplicate removal rules or grant undocumented immunity. Preserve strict author schema and current item preflight/control policy. Before native fixtures use private members, inspect the WITH_DEV_AUTOMATION_TESTS friend list; prefer public self-input APIs. Discover files with rg --files and use directory -g filters, never Windows path wildcards. Freeze sources before readers and preserve versioned failure evidence before any repair. Actual RustV1 failure is recorded below.

## E1533.1 — patch validation before readers

One apply_patch request listed two update operations for the same model file; the tool rejected the whole patch without mutation. Consolidated both hunks under one file operation before retry. No build/test reader had started. Prevention: one file operation per path in each patch. Initial Lua insertion was also moved to the physical end so catalog1..13 ordering stays unchanged when appending14.

## E1533.2 — source discovery and command semantics

A read-only search guessed native/buff_visual_state.rs; actual shared public snapshot module lives directly under runtime. No mutation resulted. Use discovered paths, not module-name guesses. Review also found the legacy self-item executor clears command queues after every activation. The new instant defensive cleanse explicitly preserves commands by semantic effect kind; other self-item behavior is unchanged. Cover this with formal input tests rather than assuming reuse automatically preserves orders.

## E1533.3 — legacy self-item admission bypassed match context

FullRustV1 passed ABI/content/templates/codegen/core980; base_content completed330 tests with329passed/1failed. Actual controlled cleanse passed. The new rejection test failed at the retained Warmup rejection assertion (single_lane_match_tests.rs:7890): self-item executor accepted a non-playing match and consumed cleanse. This is a production admission gap, unlike a fixture mismatch. Original six-scope raw logs/summary/false retired receipt and10 exact inputs-v1 are retained; no backend/UE/native run started. All Rust readers/launcher retired/reaped before repair; only the single health monitor remains.

Repair: generalize the existing Stop context helper to active_hero_context_allows and reuse it at common item admission, rejecting pause/non-playing/stale-roster contexts before mutation. Preserve Stop/no-match owner semantics and controlled item use. Keep the failing assertion and add explicit legacy shield/new-cleanse pause/phase/roster regression coverage. Only game_processor.rs and formal test source change from the10-input V1 cohort;8 others remain unchanged. Rerun all8 Rust scopes in a fresh V2 root. Prevention: test common self-item context admission directly; do not assume targeting-only checks cover self effects, and never weaken a phase rejection test to match this bypass.

## E1533.4 — pre-reader repair and helper checks

Initial repair hunk matched an identical owner-life gate in targeted admission instead of the common transaction. Read-back caught placement before readers; removed the redundant targeted addition and inserted the gate using unique common transaction context. Verify function scope, not only matching statements. A documentation hunk used a partial line and rejected the whole patch; retry used full line. The Saved helper then stopped on an incorrect expected text base_content=330 (actual base330) after freezing inputs and preparing runners/sealer; completed the remaining docs edit directly and did not rerun the partially mutating helper. These were pre-reader preparation errors, with no new test acceptance claimed. The openspec executable is absent from PATH; read existing proposal/design/spec/tasks directly rather than installing or changing the fixed workflow.

## E1533.5 — parameterless effect rejected by numeric tooltip guard

UE BuildV1 and stage SHA passed. Actual nativeWorldV1 ran81: the new SelfCleanseItem failed its retained accurate-guidance assertion; all80 other tests completed. Screenshot showed 道具資訊未提供. BuildItemDescription had a legacy unconditional EffectAmount>0 preflight before dispatch; the new parameterless cleanse intentionally generates zero. Retired/reaped world/source-capture/source-check readers before repair; successful prior build/source capture and failed native/source receipts remain untouched, including all original screenshots.

Repair native shared guard by semantic effect type: cleanse requires amount exactly zero and zero duration/interval/radius/control metadata; all existing numeric active effects still require positive amount. Preserve the original accurate-guidance assertion; add five malformed metadata rejection cases. Only shared native tooltip source and native fixture change from frozen inputs-v2; other8 including all Rust sources byte-identical. Freeze inputs-v3, rerun UE build/stage and full81 native acceptance with fresh V2 source fingerprints. Preserve exact RustV2/inputsv2 relationship and failed native evidence. Prevention: enumerate parameterless versus numeric effect contracts before adding effects, and exercise actual generated metadata through full shared tooltip preflight.
