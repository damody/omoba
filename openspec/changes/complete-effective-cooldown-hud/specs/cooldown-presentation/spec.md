# Cooldown instance presentation

## Requirement: preserve admitted duration
Starting a cooldown SHALL store its actual admitted duration. Refresh SHALL replace it, explicit clear and expiry SHALL retire it. Tick and subsequent modifier changes SHALL NOT recalculate the stored total. Nonpositive time SHALL NOT advance it.

### Scenario: slow or haste
A 20-second base skill admitted as 10 or 40 seconds SHALL expose that actual total through existing owner HUD fields, including after serialization and pause. Half elapsed SHALL present half progress with the correct total.

## Requirement: legacy and invalid inputs
Missing legacy totals SHALL use a conservative max(base,remaining) bound without claiming recovery of the original duration. Present totals outside valid bounds or smaller than remaining SHALL reject the owner hero HUD, not expose a fabricated ready ability. Other owners SHALL not acquire the row.
