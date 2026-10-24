# Design

Keep legacy BuffVisualState/BVS1 unchanged. Introduce BuffVisualSnapshot as the current decoder/producer wrapper. BVS2 rows contain catalog u16, remaining Q10 i64 and optional total Q10 i64;0 means no channel metadata. Emit legacy BVS1 when no total is present. Require sorted unique generated identities,64-row cap, exact lengths, canonical known channel metadata only and0<remaining<=total<=60s. BVS2 without a total is noncanonical and rejected. Legacy clients do not gain BVS2 support; current maintained host/replica/bridge use the new decoder.

Reuse existing schema ID, owner-team filter and OwnerBuffVisual absolute event. Baseline and update validation must accept both versions; rendering reconstructs only canonical sanitized total JSON so existing native typed decoder works. Missing/legacy totals remain unknown. Clear/cancel/tombstone produces an empty legacy state, preventing stale total retention. No new network matches, C ABI fields or private metadata disclosure.
