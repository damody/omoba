# Ability cooldown resolution

## Requirement: authoritative checked duration
Successful ability release SHALL resolve authored cooldown using both existing percentage stats, the existing family policy and 102raw multiplier floor. Managed admission and actual cooldown outcome SHALL agree. Invalid, overflowing or over-limit inputs SHALL reject before effects or resource debit. Zero cooldown SHALL remain zero.

### Scenario: preparation then release
Given a prepared ability and a cooldown modifier changed during preparation, when the ability releases successfully, then release-time stats determine the cooldown once. Later modifier changes SHALL NOT rescale the running cooldown. Pause SHALL freeze its remaining game time.

### Scenario: rejected release
Given malformed cooldown stats at release, then no summons, healing, mana debit or cooldown SHALL be committed.
