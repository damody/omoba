## ADDED Requirements

### Requirement: Passable brush concealment
Compiled maps SHALL carry optional bounded brush regions separately from collision and opaque LOS. Unit disclosure, fog sampling and immediate unit-item validation SHALL require a friendly vision source inside the target's brush, while retaining existing range, detection and explicit override precedence.

#### Scenario: Outside source and concealed enemy
- WHEN an enemy is inside brush and every eligible friendly source is outside
- THEN the enemy SHALL remain undisclosed and a targeted item SHALL fail visibility validation.

#### Scenario: Entry, exit and unobstructed rear
- WHEN a friendly source enters the same brush within vision range
- THEN eligible targets and fog cells SHALL become visible.
- WHEN that source leaves
- THEN brush cells SHALL lose current vision while preserving exploration.
- WHEN a target is outside brush behind the patch
- THEN the brush SHALL NOT behave as an opaque wall.

### Requirement: Executable point-area guidance
The frontend SHALL generate rank-specific impact circles from the disclosed ability's executable area effects and SHALL NOT infer damage geometry from cosmetic cues.

#### Scenario: Rank and loadout change
- WHEN the disclosed rank changes
- THEN the preview SHALL use that rank's authored effect radius in world scale.
- WHEN the ability is unlearned, unavailable, non-point, replaced or retired
- THEN stale point-area geometry SHALL NOT remain active.

#### Scenario: Player viewport
- WHEN the point marker is visible
- THEN the circle SHALL project through the same owning-player viewport and DPI conversion as the range guide, clipping discontinuous runs without connecting through unprojectable points.
