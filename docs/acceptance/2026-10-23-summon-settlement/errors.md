# Errors, root cause and prevention

Batch E1506: spawn facts were derived before serial spawn admission; dead owners and changed player ownership could pass the previous team-only check. Fix common settlement, not authored hero names. Preserve existing permanent duration0 semantics and keep whole-formation resource atomicity explicitly pending.

Discovery mistakes: attempted nonexistent native/world.rs and comp/faction.rs. Correct locations were discovered with rg: scripting/parallel_world_adapter.rs and comp/unit.rs. Prevention: search declarations and rg --files before reading; never infer one-file-per-type layout.

Carry forward E1505: no edits while acceptance readers are active; preserve failed cohorts, exact creation-token retirement, fixed Lua/Rust1.95.0/UE5.8, and image review for UI changes. No network games, quota resets, commit/push or historical frontend work.

RustV1 stopped at compile E0422: the child test module used BlockedRegion but parent only imported BlockedRegions. All readers retired before the fix. Preserve failed-test-v1.rs and core stdout/stderr/summary. Add the precise crate::comp::BlockedRegion import, not a broad wildcard; rerun the complete core cohort asV2. Discovery also attempted nonexistent native/state_initializer.rs; actual rg declaration is native/initialization.rs.

RustV2:923 passed/3 failed. All three new rejection tests expected EntitiesRes::delete to change is_alive immediately. specs defers removal until World::maintain, matching the runtime frame boundary. Production rejected bodies already have no components, hooks or success facts and reservations are released. After all Rust/build/backend readers retired, preserve V2 test and logs, then add maintain before the retirement assertion; do not change gameplay semantics to satisfy a false immediate-deletion assumption. RustV3 reruns the whole core suite; rebuild/stageV2 follows.

Runner audit before launch caught a cloned capture switch accidentally renamed from om-recall-progress-capture to om-spawn-admission-capture. Restored the existing native test switch before any World cohort ran. Prevention: rename output paths independently from stable native command-line contracts.

Final native review:World64 success,4 existing warnings.20 image files exactly match prior accepted pixels; four explicitly viewed differences are animated recall bar, layered map scene, first-kill notice and summon pose. Recall6/4/1 seconds remains readable, map header04:32/KDA0/0/0 preserved, summon mesh/ring visible. Existing first-kill lower-HUD clipping, narrow objective strip and hero/focus nameplate overlaps remain limits. No new UI claim from this authority-only batch.

Monitor audit:the global170-second timer cloned some previous-batch source/progress paths. It still records cadence, but cannot alone prove current RustV3/source-check progress. Current owned cohorts have separate creation-token watchers; manual reads checked actual engine/core/source logs and source-check childPID38812 CPU4.67→7.66→15.86 while buffered stdout stayed empty. No duplicate work was launched. Future batches must replace all health paths with actual current cohorts before starting the timer.
