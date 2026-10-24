## ADDED Requirements

### Requirement: Bounded current hero placement
All current validated projected visible heroes SHALL retain nameplates. Stable priority SHALL attempt nearby vertical positions using current measured geometry and bounded grid work. World anchors SHALL remain unchanged. Nonheroes SHALL avoid actual final hero footprints. Hidden/retired/rebound/unprojected heroes SHALL not retain stale displacement.

#### Scenario: Two colliding heroes
- WHEN space is available near a shared projected anchor
- THEN the lower priority label is placed at a non-overlapping nearby position
- WHEN anchors separate
- THEN a fresh layout restores zero displacement

#### Scenario: Dense capacity
- WHEN nearby space or fixed cell budget is exhausted
- THEN every hero retains its original visible label
- AND work remains bounded; remaining overlap is allowed
