## ADDED Requirements

### Requirement: Shared diagonal map
The authority and Unreal MUST consume the same authored lane endpoints, tower layers and jungle positions. Each lane MUST share both base endpoints. Base and hero spawning MUST follow the authored mid lane rather than a horizontal axis.

#### Scenario: Both sides spawn
- WHEN the compiled training map starts
- THEN the two bases occupy opposite diagonal corners and each hero spawns inward from its own corner.

### Requirement: Champion protection
Nearby allied lane NPCs MUST prioritize an enemy champion who initiates a basic attack against their champion. Towers MUST additionally retaliate for actual champion damage. The protection window MUST expire deterministically and MUST NOT persist across entity generations.

#### Scenario: Retaliation and disengagement
- WHEN a champion attacks another champion near defending minions or a tower
- THEN eligible defenders prioritize the aggressor within their acquisition range
- AND dead, expired or out-of-range aggressors stop receiving protection priority.

### Requirement: Jungle reset
Returning camps MUST reject damage and new aggression until arriving home; home recovery and respawn MUST use the shared simulation outcome lifecycle.
