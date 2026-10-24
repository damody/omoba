# Design

Register the existing damage shield as append-only native buff `damage_shield`, retaining its bounded fixed-point payload. Rendering accepts generic `shield_remaining_raw`, not item or hero identities. Decode only registered, finite, live exact-generation data. Reject ambiguous sources, invalid JSON, nonintegral/out-of-range fixed-point values and bad string references. Complete snapshots clear absence; control-only frames retain accepted state.

Show a separate shield bar relative to max HP, capped visually at full width, with the full numeric amount and authoritative remaining time. No healing or inferred expiry. Reuse normal nameplate preferences, admission budgets and retirement; cache same-display seconds. No network matches or omfx changes.
