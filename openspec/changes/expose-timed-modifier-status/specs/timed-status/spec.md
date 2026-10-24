## ADDED Requirements

### Requirement: Timed modifier status is canonical and time-only
The system SHALL disclose valid active shared sprint/damage-reduction families as canonical localized statuses with maximum active remaining lifetime. It SHALL NOT disclose private source IDs or magnitudes, change gameplay aggregation, or promote malformed reserved canonical names.

#### Scenario: Independent source expiration
- **WHEN** a stronger short sprint and weaker longer sprint overlap
- **THEN** the public movement status time represents any remaining valid source
- **AND** actual movement strength continues using the existing strongest-family rules
- **WHEN** all sources expire or the owner scope retires
- **THEN** the status is removed
