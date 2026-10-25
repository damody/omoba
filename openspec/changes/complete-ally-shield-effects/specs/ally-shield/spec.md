# Requirements

- Typed ally/self shielding uses valid unit targets, positive per-rank bounded amount/duration/range, and closed ally allegiance.
- Invalid target/life/team/range/data/source life or a later invalid sibling rejects the entire managed cast without HP, shield, mana or CD leakage.
- Canonical shield absorption and max amount/max duration refresh are reused; no amount accumulation, raw payload overwrite or new ABI.
- Public status remains canonical shield_remaining_raw; typed grant packets remain internal.
- Existing support content and rank tooltips expose the new shield without hero-name dispatch, new identities or per-frame global scans.
