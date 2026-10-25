# Combat visual world isolation final acceptance

The receiving UOmCombatVisualSubsystem now rejects an owner from a different actual UWorld before floating-text or one-shot deduplication, persistent recall/buff lookup, and shared allocation. Existing channel and ground-layer guards remain. Local hidden/overload cues still consume their legitimate history; foreign events cannot consume a matching local key. This closes a world transition/preview isolation gap for all generated units without hero-name conditions, new timers, world scans, wire or gameplay changes.

Two actual Editor worlds reuse entity 42/generation 1/epoch 1. The native test rejects foreign HP, avoidance and critical text, then accepts the identical local keys; rejects foreign one-shot, recall activation/cancellation, buff activation/removal and channel cancellation; checks the foreign pool remains empty and local retirement clears allocations. Existing health text lifecycle/capacity/pause/fog/reconnect/death regressions and actual restoration/MISS/CRIT glyph rendering remain covered. Generated shield nameplate and pooled combat lifecycle regressions also run.

Fresh UE5.8 local build and three exact native tests passed with zero native warnings. Staging verified. Three fresh 512x256 scene captures were inspected through view_image; glyphs are visible. This is local native test/render evidence, not a real network match or OS DPI claim.

Source proof: 3494 files, 3492 unchanged, two planned C++ files changed, no additions. The whole-source proof binds the accepted slow-cleanse baseline and its index. Rust/backend/generated catalog sources are unchanged: their prior 2123 passing tests / nine existing ignored are inherited, not fresh executions. ABI36/presentation12/selective14 unchanged. MVP remains 33/44 with 11 external acceptance items.

The owned acceptance worker has actual API exit 0, verified identity-bound exit/cleanup/retirement and health maximum gap <=180000ms. Sealer and independent rehash also have actual reaped API exits. Errors and original evidence are preserved under the immutable acceptance index.

Independently rehashed evidence files: 47. Index SHA-256: 76c68eb5403ee1c0bec52d30bb281d48b63f28e1c82cd2e1d3224da81578b3f4.
