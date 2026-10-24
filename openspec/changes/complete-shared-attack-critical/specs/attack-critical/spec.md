## ADDED Requirements

### Requirement: Shared critical data is consumed by ordinary authority attacks
Native damage SHALL preserve its existing base critical policy and consume shared chance/multiplier. Generic skills SHALL honor their existing can_crit flag. Explicit hero/script ordinary projectile attacks SHALL commit critical physical amount once at launch and never multiply visual copies or source misses.

#### Scenario: Modifiers change while projectile is in flight
- WHEN a critical ordinary projectile has launched and its source buff expires or source retires
- THEN impact retains the committed amount and existing current-target evasion/defense policy.

#### Scenario: Ability opts out
- WHEN an ability or true-damage instance forbids critical damage
- THEN shared critical modifiers do not multiply its physical, magical or pure amounts.
