# Design

Both primary and member bodies may author `death_children`. The tree is at most two split generations, four children per body and 32 lifetime bodies per camp. Children inherit movement, combat cadence, camp leash and reset policy; each defines HP, damage and its own bounty. Offsets are relative to the parent home and validated cumulatively against the root leash, terrain and coordinate bounds.

Committed lethal retirement queues offspring once; mutable camp tick spawns them at collision-safe offsets from the corpse. Queue state participates in the deterministic digest. The camp cannot start its respawn deadline while offspring are pending or alive. Full respawn reconstructs only the authored initial bodies. Forced retirement and terminal cleanup do not split. Children never inherit objective or personal rewards.
