# Effective cooldown HUD errors and prevention

Inspection found authority correctly applies cooldown modifiers but owner HUD independently recomputes total from authored base. A slowed duration can exceed total and native SetAbilityVisualState then rejects it. Record actual instance total at start in shared Hero state and use existing HUD total fields; do not recalculate running total from today's buffs or clamp a slowed remaining value to the base. Optional missing legacy field uses conservative max(base,remaining), while malformed present totals are rejected.

Read-only inspection guessed OmGenerated/Private/OmRustBridgeComponent.cpp, which does not exist; actual bridge is OmWorldBridgeActor.cpp. rg failed without mutation. Prevention remains exact rg --files discovery before reads, not guessed component names.

V1 actual E0433 at bridge compiling client library: omoba_sim is a client dev-dependency, though existing cfg(test) code can import it. Core967/base321 passed; bridge failed to build and client tests did not execute. V1 owner/watchers retired/reaped, exact4sources+manifest/rawlogs/ready/false receipt preserved before correction. Use omoba_template_ids::Fixed64, the existing normal-dependency reexport already used by production renderer fixtures; do not add another dependency. Prevention: check normal versus dev dependencies before moving a test-only type path into library code. FullV2 cohort required.

After all RustV2 readers retired/reaped with1819+9ignored, preserved native CPP+SHA receipt before tightening its checks: equal fractions alone could pass while stale total remained. Add exact total tooltip assertion for10/40seconds, and capture existing full viewport with actual40total/20remaining so pixel review can see cooldown20. This is pre-UE test-only strengthening, not a previously observed native failure; Rust sources remain unchanged after accepted V2.

Final accepted: 1819passed/0failed/9existingignored；World71全執行；3433正式輸入一致；150份證據／34PNG；health10 samples／max171秒；正式來源／證據獨立驗證，所有reader/watchers/health退休reap。
