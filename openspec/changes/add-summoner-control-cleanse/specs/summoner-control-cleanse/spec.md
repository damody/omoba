# Summoner control cleanse

## ADDED Requirements

### Requirement: Explicit self control cleansing
An equipped SelfControlCleanse operation SHALL remove current canonical stun/root/silence entries from its authenticated living owner while preserving all other buffs and units. It SHALL work under those controls, retain movement actions, grant no immunity and consume authored cooldown even with no removable control. Rejected requests SHALL leave controls and cooldown unchanged.

#### Scenario: Controlled owner casts
- WHEN the owner is playing, not paused, alive and the equipped slot is ready
- THEN the existing three control entries are removed and cooldown commits once
- AND subsequent control may apply normally

#### Scenario: Cast rejected
- WHEN owner, phase, pause, life, shape, resources or cooldown validation fails
- THEN no controls are removed and no new cooldown is charged
