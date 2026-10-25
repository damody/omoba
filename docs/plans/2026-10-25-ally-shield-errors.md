# Ally shield errors, causes and prevention

AS01 (read-only audit): passed PowerShell's -ErrorAction to native rg, and assumed scripting/outcome.rs and native/mod.rs existed. None of these reads changed source or launched a build. Actual native entry is runtime/native.rs, with settlement in native/cast_settlement.rs and add-buff handling in native/game_processor.rs. Prevention: rg --files first and read only paths returned by that inventory; native rg options are not PowerShell common parameters.

Audit decisions: rejected hostile mana-burn reuse of spend_mana because it has self-spend notification semantics; did not implement that shortcut. Canonical BuffStore::add already preserves maximum duration and grant_damage_shield preserves maximum amount, so do not rewrite that accepted policy. New ally shielding will use a closed checked grant envelope and the existing canonical helper, not overwrite shield_remaining_raw directly.

AS02: initial multi-file apply_patch verification failed because allegiance detection uses an any() predicate rather than the assumed match arm. Verified no partial writes occurred (new module absent and no ally_shield references), then applied against the actual predicate. Prevention: read exact insertion context before patching; after a multi-file rejection inspect actual state before retrying.

AS03: a later test patch used an assumed existing test-function name and was rejected atomically. Confirmed the new tests were absent, read the actual insertion context and reapplied. Prevention remains exact context first; do not reconstruct a test name from a summarized behavior.

AS04 (pre-execution audit): values(key,true) in the schema enforces the radius10000 envelope, not just positivity. Corrected ShieldAlly to use the shared scalar1000000 envelope plus explicit positive amount/duration checks, matching Lua and native guards. Also used an exact Q10 timer tolerance rather than an unverified float constructor in the new formal test. Prevention: read validator parameter semantics and test numeric boundaries across all consumers before launching builds.

AS05: ProbeV1 failed compiling two exhaustive match tables in an existing damage/self-heal test (E0004). The new ShieldAlly variants were missing from their explicitly unreachable ally-effect arms. Added both variants without wildcard arms. Original failed logs, frozen inputs and retired worker are retained under AllyShieldProbe/v1; original API session62475 was reaped with actual exit1 (chunk a259c1). Prevention: include test-only exhaustive consumers when extending effect enums, and require a fresh focused probe before broad acceptance.

AS06 (read-only evidence preparation): assumed rank-effects-docs-v1.lua rather than inventory-returned rank-effects-final-docs-v1.lua, and attempted to read the final receipt before ProbeV2 completed. No source or worker was changed. Prevention: inventory helper paths first; final receipts only after the original API completes, while active work is checked through existing logs and supervisor health.

AS07 (read-only build monitoring): searched assumed scripts/lib instead of inventorying Lua module paths. That search failed without changing source or the running build. Prevention: use rg --files scripts -g '*.lua' before locating a module directory; do not infer layout from require names.
