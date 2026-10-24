# Requirements

- Valid active zones expose a canonical label and remaining time; malformed, expired or over-budget entries do not.
- Multiple zones aggregate deterministically without source identities, locations or private JSON.
- Refresh/countdown/pause/expiry/death propagate through authoritative public status.
- Actual hero details show localized status and clear it on scope retirement.
- Existing catalog IDs remain stable; new status is append-only.
