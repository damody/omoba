## Requirements

- Damage integrates the final partial interval exactly once and never after expiration/removal.
- Fixed-point fractional carry makes total raw damage invariant across supported tick profiles.
- Invalid DPS, kind/profile or source identity emits no damage; malformed identities never narrow into valid handles.
- Authored DoT validates target allegiance, range, lifetime and all extras before applying any effect.
- Existing shield, mitigation, source attribution, death and pause settlement policies remain authoritative.
