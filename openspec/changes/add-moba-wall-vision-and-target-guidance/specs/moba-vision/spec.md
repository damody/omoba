## ADDED Requirements

### Requirement: Compiled walls occlude authoritative vision
The authority SHALL derive opaque vision polygons from the same compiled terrain bounds used for collision. Public disclosure, fog and immediate unit-item visibility SHALL use those polygons. Legacy campaign descriptor geometry SHALL retain its behavior.

#### Scenario: A wall separates source and target
- WHEN an eligible friendly source and a vision-scoped enemy are on opposite sides of a compiled wall
- THEN that source SHALL NOT disclose the enemy or illuminate cells behind the wall.
- WHEN another eligible friendly source observes from the enemy side
- THEN normal range, brush and detection rules SHALL govern revelation.

### Requirement: Authored unit-target guidance
Unit skill preview SHALL display the disclosed authored ally/enemy requirement without asserting that the hovered candidate meets authority rules.

#### Scenario: Loadout or rank changes
- WHEN the active unit spell requires an ally or enemy
- THEN the shared preview SHALL show that requirement.
- WHEN metadata becomes unknown, unlearned or non-unit
- THEN stale requirements SHALL clear.
