## ADDED Requirements

### Requirement: Rank policy owns damage interruption
Each ability rank SHALL default cast_interrupt_on_damage to false. True SHALL require cast_time in [1/1024, 60] seconds. Non-boolean values SHALL be rejected consistently by Rust and Lua content loaders.

#### Scenario: Actual pending opt-in cast loses HP
- WHEN a committed damage path causes positive HP loss while that cast remains pending
- THEN the normal cancellation path clears pending effects without mana or cooldown debit and clears public preparation before the next projection.

#### Scenario: Damage is fully absorbed or cast has already released
- WHEN shields absorb the full packet, no HP is lost, or the effect was already released
- THEN this policy SHALL NOT cancel or roll back effects.

### Requirement: Policy is disclosed in the official frontend
Generated metadata SHALL preserve the per-rank policy, and learned ability tooltips SHALL disclose it only for a positive preparation duration and opted-in rank.
