# Next gameplay candidate: targeted friendly control cleansing

Completed after this historical discovery: OpenSpec complete-ally-cleanse3/3, fresh Rust2128/9ignored, UE5.8/native4 and actual Slate capture;401evidence files independently verified. See 2026-10-25-ally-cleanse-final-acceptance.md and handoff.md. Do not repeat this candidate as unimplemented. Current baseline is AllyCleanseV1/source-before.json3496, bound by docs/acceptance/2026-10-25-ally-cleanse/index.json.

Discovery only; no gameplay change, accepted task or launched worker in this audit.

Current accepted baseline is `docs/acceptance/2026-10-25-slow-nameplate/index.json`, SHA-256 `cab5f6e60708c3c97fa77e8056d653f6d0aa54e4bc90b1b4f828f4dcc71a89da`, binding `omfue/Saved/Automation/SlowNameplateV1/source-before.json` (3494 files). Nameplate spacing and slow nameplate are complete. No build/test worker remained at discovery start. Do not rerun their accepted checks as a replacement for new implementation.

New concrete gameplay gap found: current cleansing is self-only (summoner and item). Generic targeted friendly effects support healing and shield, but have no targeted ally control-cleansing operation. This can be implemented and verified on one PC, using the existing canonical BuffStore cleanse semantics, strict friendly target admission and serial transaction rather than a hero-name branch. This is a candidate, not a claim that ally cleansing already exists.

Implementation outline for next batch:

1. Define closed `CleanseAlly` authoring/operation with unit-friendly allegiance, no configurable buff-name list, and positive bounded rank range. Compose with existing valid ally effects and caster-local resource effects; preserve whole-plan validation before cost/cooldown/effects commit.
2. Reuse `native/targeted_effect.rs` for living source/target, exact source generation, team and inclusive range checks. Deliver a strict instant packet through the existing stat-buff outcome transport, bind source to the actual caster during serial admission, consume it without storing a public buff, and invoke `BuffStore::cleanse_controls` only after revalidation. Preserve voluntary stance reductions, periodic/zone clocks, shields, healing reduction and unrelated buffs. No immunity or passive ticking.
3. Decide an authored support skill application from the shared data, preserving stable IDs and existing cost/cooldown unless the concrete design requires otherwise. Do not silently claim a particular ability has changed. Provide generated current/next rank effect text with truthful cleanse semantics; omit unsupported legacy EffectSpec preview rather than pretending it heals or shields.
4. Add strict authoring/packet/invalid sibling and serial source-target boundary coverage; formal60Hz actual ally cast with control/slow removal, retained buffs, cost/cooldown, pause, immediate reapplication and replay. Run appropriate Rust suites plus fixed Lua authoring/generation checks, backend/UE5.8/staging/catalog and native tooltip/target feedback acceptance. Preserve parent inputs before edits and all failed original evidence.

Confirmed source inventory:

- `omoba-content-model/src/lib.rs`: AbilityEffect, `unit_target_allegiance`, validation and target-consumer composition.
- `scripts/gen_hero_registry.lua`: generator selected by `scripts/base_content/build.rs` through the fixed Lua executable; search its existing shield/drain mappings before edits.
- `scripts/base_content/src/generic_effects.rs`: EffectOp/ResolvedEffect/EffectSink, preflight/application/resolution and legacy preview filtering; existing test sinks require explicit new methods and exhaustive arms.
- `omoba-core/src/runtime/native/mana_drain.rs`, `ally_shield.rs`, `targeted_effect.rs`: strict packet/admission patterns.
- `omoba-core/src/runtime/native/cast_settlement.rs` and `game_processor.rs`: source-bound serial preflight and reserved marker consumption.
- `omoba-core/src/runtime/native/cast_settlement_tests.rs`: current source/range/sibling atomicity tests.
- `omoba-core/src/runtime/native/ability_runtime/buff_store.rs`: canonical cleansing, already accepted including formal skill/projectile slows.
- `scripts/base_content/src/single_lane_match_tests.rs`: `ally_shield_formal_60hz_rank_cost_absorption_expiry_public_pause_and_replay` and `ally_heal_60hz_generated_support_rejects_enemy_and_heals_disclosed_teammate` provide actual support/friendly target fixtures.
- `scripts/lua_data/templates/abilities.lua`: current lumen_aid heal+caster mana restore+ally shield declaration.
- `omfue/codegen/src/lib.rs`: closed current/next rank effect summaries.

Read-only discovery error AC-D01: guessed `omoba-content-model/src/abilities.rs` and `native/ability_runtime/cast_settlement.rs`; both reads failed. Actual locations are lib.rs and native/cast_settlement.rs above. No source or worker changed. Prevention: get definition/inventory results first and copy those paths into each read; do not reconstruct remembered paths even for familiar symbols. No-match rg exits in discovery are absence results, not test failures.

Remaining external requirements stay separate: MVP33/44, eleven external acceptance items, no new real network game or quota reset. Deadline remains 2026-10-26 23:59:59 Asia/Taipei.
