# Requirements

- Authoring rejects nonfinite, nonpositive, out-of-envelope amounts, unknown fields and point shape.
- Generated/runtime catalog identities agree; spell 7 describes restoration 200 and cooldown180 seconds.
- Settlement restores min(amount,missing), checks exact owner/current resource/phase/pause/life, and commits resource and cooldown together.
- Missing or zero-capacity pools reject without publication. Full pools accept zero, preserving normal cooldown policy. Reusing plans or modifying the pool invalidates the plan.
- No healing multiplier applies to mana. No implicit resource initialization or command interruption.
- Fresh maintained Rust suites, backend/UE build, stage/catalog and targeted native tests supply final evidence with original exits and source consistency.
