## ADDED Requirements

### Requirement: Shared self damage reduction spell
A valid authored no-target SelfDamageReduction SHALL grant a source-keyed strongest-family reduction and consume cooldown only after successful application. Reduction SHALL be positive<=1, duration positive<=60seconds. Unsupported/missing sink and invalid context SHALL reject atomically. Existing movement/control state SHALL remain.

#### Scenario: Weaker longer item overlaps
- WHEN25%3second spell overlaps12.5%6second item
- THEN only25% applies until spell expires
- THEN12.5% remains until item expires
- AND public status time represents any valid remaining source
- AND pause freezes effect/cooldown
