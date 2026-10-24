# 規格

## ADDED Requirements

### Requirement: Brush attack exposure is bounded and deterministic
An admitted basic attack from brush SHALL expose only its living attacker to the non-neutral opposing victim team for two simulated seconds. Refresh SHALL extend that grant; rejected requests, allied or neutral targets SHALL not grant exposure. Death, retirement, expiry and map reset SHALL retire grants, and replay digest SHALL include the state.

### Requirement: Exposure only bypasses brush
Public disclosure, immediate unit targeting and fog SHALL agree. Range, wall LOS, stealth detection, ServerOnly and ForceHide SHALL remain enforced. Fog SHALL expose only the attacker's cell without revealing other brush occupants or creating an observing source.

### Requirement: Ability tooltips describe only published targeting metadata
Ability tooltips SHALL present known target shape, published ally/enemy requirement, learned-rank range and point impact radius using configured world scale. Invalid, unknown, unlearned, non-point replacement and retired states SHALL not show stale numbers. Existing authored descriptions and legacy logical labels SHALL remain available.

