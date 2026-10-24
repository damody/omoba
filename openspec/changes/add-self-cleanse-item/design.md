# Design

Add closed `self_control_cleanse` to validated item effects, generated Rust constants, native runtime and native item metadata/self-hotbar classification. No arbitrary buff names, healing, shield or immunity fields. Execute existing BuffStore::cleanse_controls only after normal living-owner/item/cooldown/target-contract/resource preflight. It does not clear or replace movement/attack commands. Existing item-use control policy remains; no spell cast-control gate is added.

Append item14 moba_purification_charm / 淨化護符, cost1400, cooldown45seconds, self/no-target. Chinese description states supported removals, unrelated debuff retention and absence of immunity. Cover actual60Hz purchase/use while controlled, unrelated buffs and foreign entity retained, cooldown/rejection/empty use/reapplied control, public status removal and native self input. Full Rust/backend/UE/native validation follows implementation.

Formal rejection coverage exposed a legacy admission hole: self items bypassed match-phase/pause/roster context checked by other active hero actions. Reuse the existing active-hero context helper for item transactions too, before mutations, preserving its no-match fallback and Stop semantics. Both legacy shield and new cleanse receive explicit pause/phase/roster atomic rejection coverage.
