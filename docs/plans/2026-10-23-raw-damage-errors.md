# Raw damage settlement errors

## Root cause

Outcome::Damage combines already-computed native DamageInstance packets and raw script/DoT packets. handle_damage only applies packet modifiers/shields. The boolean predeclared is false on both paths and cannot indicate defense state. Remedy: explicit raw variant and serialized projectile stage, one shared fixed-point defense formula, existing settled variants unchanged.

## Discovery mistakes

- An rg path used attack*.rs and was not expanded by this PowerShell shell. Always pass a discovered directory with -g, never a wildcard in the path argument.
- A guessed observable_fact.rs path does not exist; the reducer is observable_fact_from_outcome in game_processor.rs. Discover definitions with rg before reads.

## Scope

- Focused V1: core938 passed/9 failed. Five new ECS fixtures omitted TowerTemplateRegistry (required by normal damage bookkeeping); four existing retired-source real-flight credit fixtures used invalid profile0 on pure script projectiles. Preserve 12 original sources/logs after retirement/reap. Initialize the full required registry and use explicit TRUE on the old pure-damage fixture, retaining all original wallet/generation/credit assertions.
- Factory audit found native homing/directional projectiles also contain only outgoing attack values. Explicitly mark new launches Raw; do not mistake source-side bonuses for target-side mitigation. Old serialized projectiles default Settled for compatibility; already-computed native DamageInstance/melee packets remain Settled.
- Remove an arbitrary one-million raw packet cap: authored abilities/DoT already validate their own budgets, while legitimate TD upgrades can exceed that. Raw ingress instead rejects negative channels, invalid masks and an unrepresentable total using i128, without narrowing. Typed checked defenses keep malformed buffs fail-closed. (No cap was shipped; this was revised before final acceptance.)
- A guessed projectile_script_tests.rs discovery path was absent. Keep discovery separate from reads; use rg --files before naming a candidate file.

Preserve existing armor and 75% positive magic-resistance cap semantics. This closes bypass/double-reduction errors, not a global rebalance to all current LoL rules or arbitrary post-mitigation tick-invariant rounding.

Focused V2 core948/0failed accepted after the above fixes; all original credit assertions remained. Before final scope, added two formal-input tests and migrated 11 script-output assertions to RawDamage without changing amounts, hit membership or controls. Their hand-authored native settled inputs remain Damage. Final acceptance must run the full scope on this final revision, not mix the earlier focused source set into final proof.

Full V2 all1780passed/9existingignored, base306passed181.97sec, then serial-lifetime review found a real late raw packet after an earlier explicit Death in the same outcome batch. ECS liveness remains true until batch deletion, so the target's shield could still be consumed. Retire/reap all V2 readers and preserve19 inputs before adding a test-first reproduction: 0passed/1failed, remaining shield0 versus expected10240 raw. Preserve that red test's two inputs/logs after retirement/reap; only then guard RawDamage against target membership in remove_uids. This does not reject retired sources: their legitimate projectiles continue through original provenance. Final V3 must rerun the complete scope on the guarded revision. Prevention: authoritative retirement ledger takes precedence over ECS physical deletion; test both retired targets and retired sources.

A filtered_specs directory was guessed in the same call that discovered filtered_specs.rs. Read the returned exact .rs path in a separate dependent call. The actual contract explicitly excludes authority projectile lifetimes from filtered replica mirroring; new damage_stage is native authority/checkpoint data, not a public wire field. Formal backend ResourceManager delegates to the shared native processor; no duplicate legacy event-dispatch implementation was added.

During acceptance, a follow-up read also guessed scripts/base_content/src/ability_lua_schema as a directory; it was absent. No source mutation occurred. Discover exact filenames first in a separate call before every path-dependent rg/read, including exploratory follow-ups.


Final V3 acceptance: 1781passed/0failed/9existingignored；3426正式輸入一致；221份證據／29PNG；health15 samples／max171秒；backend/UE5.8/World66及staged/import SHA成功，全部reader/timer退休reap，證據獨立驗證通過。
