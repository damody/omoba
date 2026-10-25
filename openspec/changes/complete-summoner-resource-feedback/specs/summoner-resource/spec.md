# Requirements

- Generated resource requirements depend on operation, not identity or D/F slot.
- Missing support, zero capacity, nonfinite/negative/over-capacity mana fails local resource readiness for a mana-dependent operation.
- Empty and full valid pools remain ready; full-pool cooldown policy is unchanged.
- Card, tooltip, self preview and submission guard agree; cooldown precedes resource failure and no local failure consumes an input id or modifies mana/cooldown.
- Non-resource summoners remain usable without mana. Self summoners retain their existing control policy.
- Preserve parent source and original worker exit/failure evidence, verify final source set/hash/mtime, update handoff/tasks.
