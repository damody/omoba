# Requirements

- Shared self_shield authoring has finite positive fixed-point amount and duration<=60; none target only, unknown fields rejected.
- Shield resolves and commits through the existing immutable plan and D/F cooldown transaction, never locally in Unreal.
- Existing damage_shield grant/absorption/expiry/public lifecycle is reused; healing reduction must not diminish shield.
- Failed ownership/life/phase/pause/target/resource admission leaves shield and cooldown unchanged; repeat successful casts are rejected by cooldown.
- New catalog identity3 is selectable and rendered with authored explanation; blink1/heal2 and default loadout remain unchanged.
