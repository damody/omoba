## ADDED Requirements

### Requirement: Positive cooldown remains visibly positive
Logical, expanded-card and tooltip D/F timer readouts SHALL show positive cooldown below0.1seconds as <0.1 rather than0. The existing compact countdown already rounds positive values upward and SHALL remain unchanged. Tooltip SHALL disclose named remaining and total seconds from the same current row. Invalid negative/nonfinite/inverted pairs SHALL render unavailable rather than plausible timing. Exactzero and existing authoritative readiness policy SHALL remain unchanged. Range scalar semantics SHALL remain separate.

### Requirement: Native verification
Both slots, tiny/fractional timers, malformed pairs and scope clearing SHALL be covered using actual widget state. The shared real tooltip capture SHALL show readable named times; no local countdown or gameplay ACK may be inferred.
