## ADDED Requirements
### Requirement: Explicit path contract
Special instant point skills MAY declare swept_path with width_key; malformed rank data MUST fail content validation, never fall back to cosmetic radius or hero identity.
### Requirement: Intended capsule
The owner MUST render a finite closed ground-plane capsule at authored width and range-clamped requested length, using existing viewport projection and pending aim lifecycle. Zero/nonfinite input MUST clear geometry. Terrain uncertainty and actual server authority MUST be explicit.
### Requirement: Fresh rank and retirement
Current rank metadata MUST reach owner and tooltip; unlearned, unknown/replaced shapes and retirement MUST clear previous shape data.