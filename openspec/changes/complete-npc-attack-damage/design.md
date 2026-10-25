# Design

Jungle damage resolves UnitStats::normal_attack_physical at commit, or authored base when BuffStore is absent. Lane already resolves it. After each real creep/jungle attack is queued, consume_next_attack_bonus exactly once, including later misses/blocks, matching projectile launch policy. Do not consume on stun/range/recovery/return/death refusal, for UI observations or for buildings. Delayed lane write occurs after read guards drop; camp writes use optional BuffStore. No new payload parser, content-name branch, wire or game.

Core tests cover outgoing add/percent/clamp, armed bonus order/one-shot consumption, refusal preservation, siblings, optional resource and buildings. Formal60Hz uses real binding ItemUse, actual NPC damage through shield/defense pipeline, unchanged modifiers, next attacks, pause and replay. Parent-bound originals, owned focused/consolidated workers, fresh9Rust/backend/UE/catalog/native/source and independent closure retained.
