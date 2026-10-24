## ADDED Requirements

### Requirement: Summon identity remains distinct across official presentation
A native UnitType::Summon SHALL produce Summon snapshots and public render kind 4, map to existing ABI EntitySummon and CatalogSummon, and receive ordinary animation snapshots. Identically named creeps SHALL retain Creep identity. Unknown summon catalog IDs SHALL use generic summon fallback.

#### Scenario: Visible summon moves, hides, returns and retires
- WHEN public snapshots disclose, move, hide, reveal or remove a summon
- THEN the official frontend SHALL maintain correct content class, generation and animation lifetime without reconstructing private summon behavior or replaying stale actions.

### Requirement: Authored summons can use the shared animated-unit pipeline
The existing native_visual schema SHALL configure a summon rig, materials and animation slots through content generation, with the same scaling, health, visibility and reset rules used by animated units.
