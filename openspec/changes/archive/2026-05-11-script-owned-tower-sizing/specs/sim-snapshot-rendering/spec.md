## MODIFIED Requirements

### Requirement: tower template snapshots expose combat render metadata

`SimWorldSnapshot.tower_templates` SHALL expose render-facing tower combat metadata needed by omfx composite rendering. For each tower template, the snapshot data SHALL include render mode, base image path, barrel image path, script-owned `render.visual_size`, script-owned `placement_radius`, barrel frame paths, barrel animation timing, body animation frame paths for animated-area towers, rotation mode, barrel layout, barrel count variants, barrel offset, barrel pivot, muzzle offset, recoil distance, recoil scale, recoil attack duration, recoil return duration, and recoil mode. The metadata SHALL originate from scripts content data and SHALL be shared through `Arc` with the same static template lifecycle as existing tower template snapshot data.

#### Scenario: tower template snapshot contains base and barrel paths

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_dart`
- **THEN** the snapshot entry contains a non-empty base image path for `tower_dart`
- **AND** the snapshot entry contains a non-empty barrel image path for `tower_dart`
- **AND** omfx can cache the metadata by `unit_id`

#### Scenario: tower template snapshot contains script-owned sizing

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_dart`
- **THEN** the snapshot entry contains `render.visual_size` from scripts metadata
- **AND** the snapshot entry contains `placement_radius` from scripts metadata
- **AND** neither value is inferred from `footprint`, image dimensions, global frontend scale, or another snapshot field

#### Scenario: tower template snapshot contains barrel animation frames

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a tower whose barrel declares animation frames
- **THEN** the snapshot entry contains the ordered barrel frame paths
- **AND** the snapshot entry contains barrel animation timing metadata

#### Scenario: tower template snapshot contains animated-area frames

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a no-barrel area damage tower
- **THEN** the snapshot entry contains `render_mode = "animated_area"`
- **AND** the snapshot entry contains ordered body animation frame paths
- **AND** the snapshot entry does not require a barrel image path to render safely

#### Scenario: tower template snapshot contains tack fixed rotation mode

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains `rotation_mode = "fixed"`
- **AND** the snapshot entry contains recoil mode data that allows omfx to play a `scale_pulse` instead of target-facing directional recoil

#### Scenario: tower template snapshot contains tack count variants

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains a radial barrel layout
- **AND** the snapshot entry contains count variants for 8, 12, and 16 barrels or needle holes
- **AND** each variant contains the image path needed by omfx to render that count state

#### Scenario: tower render metadata is built once and shared

- **WHEN** sim worker emits multiple snapshots after tower templates are available
- **THEN** tower render metadata is contained in the shared `tower_templates` Arc data
- **AND** subsequent snapshots use O(1) `Arc::clone` instead of rebuilding identical render metadata every tick
