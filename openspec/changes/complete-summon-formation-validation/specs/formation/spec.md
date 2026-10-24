# Formation contract

- Resolve omitted fields from the requested authored rank; never silently coerce malformed supplied fields.
- Require whole count2..128 divisible by row count, positive bounded lifetime/spacing, bounded formation extent and living owner with position.
- Produce deterministic row-major facing-relative requested coordinates before mutation; terrain adjustment uses existing shared placement policy.
- Invalid data emits no spawn. Managed mid-group rejection discards all requests/reservations and charges no mana/cooldown.
- Preserve all four authored ranks and target compatibility. Document that full outcome-settlement atomicity is outside this admission batch.
