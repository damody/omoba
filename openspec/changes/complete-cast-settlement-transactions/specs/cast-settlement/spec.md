# Cast settlement

## ADDED Requirements

### Requirement: Supported summon formations settle as a complete cast
Before any spawn or resource mutation, authority MUST validate the current caster and all sibling placements. Rejected formations MUST retain mana/cooldowns and publish no cast or spawn success. Complete plans MUST use one immutable stats snapshot per sibling and exclusive World access.

#### Scenario: Later sibling is invalid
- WHEN a valid script result contains a later rejected summon
- THEN no earlier sibling, resource debit or successful cue commits

#### Scenario: Dependent managed cast follows a rejected cast
- WHEN its expected mana no longer matches authority
- THEN it is rejected without applying an absolute stale mana write

### Requirement: Script mana callbacks settle their pools and notifications together
All changed original mana pools MUST match current authority before any changed pool or mana notification commits, including non-cast hooks. Notifications MUST retain their invocation order. A stale owner in a multi-owner resource change MUST reject every pool mutation and notification in that change.
