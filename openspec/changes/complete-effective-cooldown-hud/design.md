# Design

Optional serde-default Hero map records actual cooldown total on start, clears on explicit reset/expiry, and survives snapshots. HUD reads validated running total independent of later modifier changes. Old saves with no map use max(base,remaining) as a conservative fallback, not a claimed recovered original duration. Malformed present totals reject the owner hero HUD rather than manufacture a ready spell. No protobuf or C ABI layout change. Existing native cooldown fraction verifies slow/fast admitted durations.
