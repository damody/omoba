# Periodic damage errors

## Discovery

- Existing buff tick removed expired entries before integrating DoT, losing final active time. Fixed-point multiplication truncated every frame, making fractional DPS depend on frequency. Remedy: integrate clipped lifetime first with per-entry fractional carry; tests must include nonintegral DPS, partial expiry and all official profiles.
- Discovery attempted unconfirmed paths moba_archetypes.rs / omoba-content-codegen and a PowerShell wildcard passed as an rg path. They do not exist / are not expanded by rg. No files were changed by those failed reads. Prevention: use rg --files and exact returned paths before reads; use rg globs via -g.

## Scope

- V1 scripts: 302 passed / 2 failed in 182.46 seconds. Both differences were 170 raw HP: the new 10 DPS on one 17-raw-second tick. Mana/CD fixture now includes that known ongoing damage; independent four-slot fixture resets target buffs alongside HP. New all-rank lifetime/pause/shield and removal input fixtures passed. Raw failures and ten input sources preserved before repairs, after worker/watch retirement and reap.
- V1 template suite omitted runtime-lua-content and ran 42 tests, not the reported 81. V2 enables that feature and reruns the full final scope; do not reuse the earlier count.
- Presentation audit found dynamic DoT lacked a canonical public binding. Added generic_dot identity and checked damage_over_time_enemy source family, distinct from the same ability's slow channel, plus appended public bleed entry. Projection exposes catalog ID/remaining only. No C ABI layout changes.
- Two apply_patch attempts rejected: duplicate file operations, then hunks ordered backwards within the file. Reads confirmed neither changed sources. Consolidate each file into one operation and order its hunks by source line before retrying.
- Another discovery read guessed nonexistent scripts/gen_template_ids.lua. Generator is Rust build.rs; use exact rg --files entries, not inferred script names.

- A copied health helper retained the previous cohort's escaped filename pattern. The timer interval ran, but the first samples referenced prior logs. Retired/reaped v1 and started v2 with literal prefix comparisons; v2 samples now contain PeriodicDamage logs. Preserve both timer histories, do not count v1 file-progress records as evidence of this cohort. Manual process/log checks continued within three minutes.
- Focused baseline passed core4 and generic-effects11. Before final acceptance, added source/kind/profile fraction reset on a reused key, deterministic expiration callback order, stricter duplicate author checks and formal 60Hz gameplay fixtures. Final acceptance must rebuild from this final revision rather than reuse the earlier focused source revision.

This batch implements ongoing damage through existing buffs, not sustained channel input or persistent ground-zone entities. Those remain pending.

## Follow-up found during settlement review

Final counts must aggregate template lib81 plus integration23+9+2=115, not only lib81. Final full V2 Rust scope: ABI15/content42/templates115/codegen68/core941/base304/bridge135+1ignored/client151+8ignored =1771 passed/9 existing ignored. Base suite182.71sec. A later read repeated a wildcard path passed to rg; corrected to `rg PATTERN DIRECTORY -g '*.stdout.log'`. Always pass directory + -g on this Windows shell; no wildcard path arguments.

Shared legacy script damage and DoT both submit Outcome::Damage directly. handle_damage applies incoming packet modifiers/shields, while the ability DamageInstance path already computes base armor/resistance earlier. Direct scripts currently do not traverse that base-defense calculation (parallel_world_adapter.rs deal_damage around 911; damage_tick.rs around 77/114; game_processor.rs handle_damage around 5429). Do not claim LoL armor/resistance parity from the zero-defense lifetime fixtures, or globally reapply defenses to all Outcome::Damage: that would double-reduce already-settled attacks/ability packets. predeclared is also false on both paths and cannot distinguish them. Next batch should introduce a clear raw-versus-settled common damage contract and validate both routes, TD profiles, shields and source attribution. Current acceptance freezes this batch's lifetime/presentation scope and retains existing settlement semantics; fixed-profile equality is raw scheduled damage, not arbitrary post-modifier HP rounding.


Final acceptance sealed and independently verified: 165份證據／29PNG、3425輸入一致、health 13 samples／max171秒。6 final＋2 historical owner/watchers retired，兩timer retired/reaped，final CIM clean。All four tasks checked; next action is the raw/settled damage contract identified above. 2026-10-22T22:03:45Z
