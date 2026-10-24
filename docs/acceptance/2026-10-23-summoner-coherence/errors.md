# Summoner coherence errors and prevention

Root cause: summoner operations were build-time generated, while RuntimeContent and ContentShape omitted their catalog from compatibility checks. Lua could claim a new content revision while authority and native presentation retained compiled summoner values. Compare the entire shared validated serialization at both boundaries; changes require rebuilding peers.

Preserve fixed Lua workflow, Rust1.95.0, no live source edits during acceptance. No new matches, quota resets, commits or omfx work. Full base_content acceptance uses --release. Prior screen-spacing tasks are already complete and are not repeated.

Discovery error: guessed scripts/lib/process.lua does not exist. Process helper is under tools/lua/lib; discover files before reads. No test/build failure caused by that lookup.

Repeated discovery mistake: a backend receipt prefix was mistaken for a script filename. The worker is diagonal-backend-build.lua. Read owner/receipt or enumerate actual helpers, never infer script paths from cohort labels.

Deployment boundary: dedicated backend build intentionally stages only scripts/base_content.dll. An Unreal consumer must additionally receive that same accepted base_content.dll in its plugin Binaries folder. Copy only that accepted artifact using fixed Lua and verify full moba_stage_contract parity; do not claim dedicated backend staging updated every consumer or rebuild unrelated native sources. The compiled-content-only bridge has no runtime Lua content code path; its existing artifact remains unchanged. No native/ABI/source asset change in this batch.

Concentrated acceptance: content40/template81/codegen68/core930/base_content300 =1419 passed, zero failures/ignored. Fullbase --release finished181.46 seconds after optimized recompilation. Official dedicated authority/moba-config build succeeded; current script DLL copied to both formal consumers and stage consistency passed. Source3420 exact set/SHA/size/mtime unchanged. Three workers plus watchers retired/reaped;170-second health timer retired/reaped. No runtime test/build failure this batch. Existing unusedTDflags and protoc/generated-source fallback warnings retained in raw logs. Unreal65/25PNG is the prior shield batch, not rerun or counted as current acceptance.
