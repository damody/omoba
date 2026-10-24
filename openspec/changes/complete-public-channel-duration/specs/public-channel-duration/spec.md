## ADDED Requirements

### Requirement: Versioned bounded public channel total
Current maintained replicas SHALL decode both legacy BVS1 and strict BVS2, with exact length, catalog/ordering/count and duration checks. Only current canonical channel total may be rendered as sanitized JSON. Unknown/private payload SHALL NOT cross the boundary.

#### Scenario: Filtered midway snapshot
- WHEN a canonical3-second channel has1.5seconds remaining
- THEN its owner-team public state and replica retain total3/remaining1.5
- AND the native shared strip shows50percent elapsed.

#### Scenario: Cancel or legacy data
- WHEN the public state is empty or lacks a total
- THEN no previous channel total is retained or inferred.

### Requirement: Existing visibility boundary
Enemy/hidden/retired identities SHALL NOT gain owner-team Buff metadata. Existing schema/event gates and transactional rejection of malformed baseline/update data SHALL remain in effect.
