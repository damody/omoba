# Chronological zone pulse settlement

Ground-zone catch-up pulses currently leave BuffStore grouped by source/key. When two zones become due at different offsets in the same simulation tick, a later low-ID source can be settled before an earlier high-ID source. Fix the common scheduler's ordering so actual due time precedes canonical tie-breaking. Preserve zone admission, source-bound cancellation, per-zone clocks, lifetime integration and ordinary damage attribution. No new match or content special case.
