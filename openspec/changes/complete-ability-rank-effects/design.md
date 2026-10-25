# Design

Codegen produces a bounded, rank-ordered effect summary from validated AbilityEffect declarations and their per-rank extras. Closed semantic labels distinguish selected ally healing, self healing/mana, additional mana spend, typed damage, periodic damage, channel pulses, control and mana modifiers. Empty custom handlers have no invented summary. Values are authored bases; do not imply effective post-modifier settlement.

Extend generated rank metadata and the existing typed HUD rank base. Copy summaries only for exact catalog/string identity and valid rank, clear on unknown/replacement/reset/retirement. Current rank effects remain useful at maximum rank; unlearned skills only show the next rank. Use existing wrapped shared tooltip for narrow layouts. No reverse OmRuntime dependency on OmGenerated, new per-entity loops or hero-name rules.

Use fixed-Lua owned acceptance with exclusive inputs/logs, complete codegen/backend/UE/catalog and native widget regressions plus actual Slate capture. Preserve original failures and source fingerprints, independently seal evidence, then update tasks/handoff/audit. Unchanged backend/gameplay tests may retain the previous2104/9 baseline with explicit fresh scope if only presentation code changes.
