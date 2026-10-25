# Design

A Lua include produces guidance keyed by authored summon IDs, using only the closed target_priority policy. Absent policy means nearest, owner_focus explains the owner's active explicit attack, visibility/range admission and nearest fallback after Stop. The summoning skill references its summon at authoring time and appends the guidance to its normal description. No runtime name branches, new wire fields or special widget implementation.

Preserve the previous sealed acceptance and all generated outputs before editing. Test policy changes, arbitrary IDs, invalid values and actual skill composition; regenerate metadata and run maintained Rust/codegen, backend and UE checks once after implementation. Archive source consistency, receipts and errors.

Implementation inspection exposed raw newline emission in the common C++ string generator. Extend the existing escape function for C0 controls and DEL using standard named escapes and fixed three-digit octal fallbacks. Keep Unicode readable and do not change authored multiline descriptions. Capture failed V1 unchanged, then run corrected V2 against a newly frozen source cohort.

Actual HUD tracing also found descriptions absent from the shared ability registry and not appended by BuildAbilityTooltip. Publish Description in that existing generated registry and append it through common HUD logic only for matching numeric/content identities, preserving upgrade and cast guidance. Add native automation for the actual widget and all rank/lifecycle identity cases. V3 closes this path after immutable V2 retires; metadata-only success is not accepted as player-visible completion.

V3 exposed a module-layer error: OmRuntime cannot import OmGenerated. Final wiring uses generated producer→existing local UE HUD payload→runtime widget. The producer publishes the description bound to numeric/content IDs and clears it before every population; the widget checks both bound IDs against the current skill. No generated-header dependency is added to OmRuntime and no network/Rust ABI field is added. V4 is the corrected frozen cohort; V3 is retained as a failed compile, not completion.
