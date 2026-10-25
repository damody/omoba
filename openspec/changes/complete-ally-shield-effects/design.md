# Design

ShieldAlly has positive bounded per-rank amount, duration and unit cast range; it shares Ally allegiance with HealAlly and supports existing RestoreManaSelf composition. All selected-target effects preflight before resources or other outcomes. Use the existing add_stat_buff transport with a closed versioned ally-shield grant packet containing source ID/generation, amount and range. Never write raw shield_remaining_raw from the script, bypass canonical refresh or extend the full GameWorld trait table.

The shared native packet decoder validates envelope/shape/types. Managed serial settlement binds packet source to the actual caster and rechecks current source/target life, team and distance before any cost/CD/effects commit. Apply revalidates and calls existing BuffStore::grant_damage_shield. Existing canonical policy already takes maximum remaining amount and duration; retain it, not a new shield accumulator. Internal packet data never becomes public buff payload.

Existing lumen_aid retains140/220/300/380 healing, caster mana10/15/20/25, original cost/CD/range600 and IDs. Add ally/self shield70/100/130/160 for2seconds. No magic-attack or mana-burn scope; extra spend API has spent-mana semantics and must not be relabeled as hostile mana loss. Rank hints derive the new typed effect; current public damage_shield metadata/rendering is reused.

Fixed Lua generator contracts, arbitrary-name schema/generic tests, late serial rejection/atomic resource coverage, real60Hz ranks/team/absorption/pause/expiry/replay, full maintained regression/build/catalog/native/capture and independently sealed provenance precede completion. No true network matches, quota reset, commits or historical frontend work.
