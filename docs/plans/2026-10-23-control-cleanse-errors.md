# Control cleanse errors

## E1523: inspection path mistakes

Read-only searches mistakenly used a Windows filesystem wildcard summoner* and guessed omoba-client-runtime/src/peer_gate.rs. Neither is a valid discovered path. Searches failed without source mutation or worker launch. Prevent repetition by discovering exact files with rg --files, then search the containing directory with -g '*.rs'. Actual summoner modules were discovered and used afterward. No OpenSpec executable is installed; the established manual artifact workflow continues under the user's autonomous implementation instruction.

## Scope and prevention

### V1 actual failed authoring test

RustV1 ran ABI15 successfully, then content42passed/1failed: Serde's internally tagged unit variant silently accepted an extra immunity field despite enum deny_unknown_fields. Worker13671 and its watcher retired/reaped; CIM showed only the owned health timer. Before correction, frozen-rust-v1 preserved all10 exact modified inputs plus manifest/SHA. All failed original logs, ready/summary and false worker receipt retained. Replaced only the authored operation variant with an empty struct variant SelfControlCleanse {}, which applies strict unknown-field rejection; generated/runtime unit enums and JSON kind remain unchanged. Keep the regression asserting immunity is rejected, and rerun the full8 scopes in V2.

### V2 actual historical evidence scan failure

RustV2 ABI15/content43 passed; template lib tests passed, but generated integration tests22passed/1failed because the whole-repository migration scan descended into docs/acceptance/2026-10-23-effective-mana/history/frozen-rust-v1/.../generated/game.rs. That archived stale source intentionally preserves the old input; the actual current generated/game.rs already has an explicit legacy exemption. Worker63656/watchers retired/reaped; exact11 pre-repair inputs plus manifest saved in frozen-rust-v2. Fix the shared source/manifest scanner to exclude exactly workspace docs/acceptance, preserving all archives and retaining scans of other docs or source directories named acceptance. No historical evidence is rewritten/deleted. Full V3 required; this is a test boundary correction, not a game runtime bypass.

### V3 actual cross-crate fixture compile failure

V3 ABI/content/templates/codegen/core succeeded (core970); base_content test compilation failed E0616 because MobaHeroSlot.summoners is private to core. Worker90216/watchers retired/reaped; all11 exact V3 inputs saved in frozen-rust-v3 before correction. The formal60Hz fixture now reads project_team_summoners + decode_projected_summoners and asserts catalog4/player2/current cooldown, including pause stability. No visibility widening or public test-only accessor. Prefer actual public disclosure when verifying cross-crate HUD state; verify API privacy before writing fixtures. Full V4 required.

### Native buildV1 actual C2248

RustV4 all1829+9ignored and backend build passed. UEbuildV1 failed C2248 because the new FOmSummonerCleanseTest calls private UOmCommandBarWidget::BuildSummonerTooltip without the friend declaration used by the existing shield test.15956 retired/reaped; no other readers remained except health8050. Before correction, pre-native-v2 saved all11 accepted Rust inputs plus the widget header (12 exact sources/manifest), and the raw failed build/false receipt remain. Add only FOmSummonerCleanseTest under the existing WITH_DEV_AUTOMATION_TESTS friend block; no method visibility change, Rust change or gameplay logic change. Rebuild UE in buildV2 and verify all native tests. When extending native fixtures, inspect the associated test friend declarations as well as the sample test body before launching compilation.

Pre-build review caught a patch anchored only on `match self`: it inserted the new operation literal arm into MobaSummonerTarget instead of MobaSummonerOperation. Corrected before starting any build/test reader. No failed cohort was launched. Future enum patches must include the enclosing impl/function name as context; inspect every inserted arm before freezing sources.

This original cleanse removes only the three canonical control BuffIds already used by authority: stun, root and silence. It does not infer debuff status from arbitrary payload keys or hero names, remove all buffs, grant immunity, cleanse damage-over-time, or claim full LoL Cleanse parity. Existing control entries are removed with BuffStore's normal indexed removal path, preserving unrelated entries and their lifetimes.
