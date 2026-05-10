## ADDED Requirements

### Requirement: Hero 3D assets are declared by scripts content

Hero 3D visual metadata SHALL be declared in the scripts content hero template data. `saika_magoichi` SHALL declare `render_mode = "model_3d"`, a model path, a texture path, and positive scale metadata that points to the existing assets under `scripts/lua_data/templates/heroes/saika_magoichi/`.

#### Scenario: Saika declares 3D model metadata
- **WHEN** `scripts/lua_data/templates/heroes.lua` is loaded by template codegen
- **THEN** the `saika_magoichi` hero entry contains `render.render_mode = "model_3d"`
- **AND** it references `templates/heroes/saika_magoichi/saika_magoichi.fbx`
- **AND** it references `templates/heroes/saika_magoichi/saika_magoichi_mat.png`
- **AND** it declares a positive model scale

#### Scenario: Saika 3D assets exist at the declared location
- **WHEN** the declared `saika_magoichi` 3D asset paths are checked relative to `scripts/lua_data/`
- **THEN** `scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx` exists
- **AND** `scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi_mat.png` exists

#### Scenario: Heroes without 3D metadata remain valid
- **WHEN** a hero template does not declare a `render` table
- **THEN** template codegen still succeeds
- **AND** that hero remains eligible for existing 2D fallback rendering

### Requirement: Generated hero render metadata is available to runtime code

`omoba-template-ids` SHALL parse optional hero render metadata at build time and expose it through generated Rust lookup APIs. Runtime crates SHALL consume generated data and SHALL NOT load Lua files to discover hero model paths.

#### Scenario: Saika generated metadata returns model data
- **WHEN** runtime code calls `hero_render_metadata(HERO_SAIKA_MAGOICHI)`
- **THEN** the lookup returns metadata with `render_mode = "model_3d"`
- **AND** the metadata contains the Saika FBX path
- **AND** the metadata contains the Saika PNG texture path
- **AND** the metadata contains positive scale data

#### Scenario: Hero without render metadata returns none
- **WHEN** runtime code calls `hero_render_metadata` for a hero that has no `render` table
- **THEN** the lookup returns `None`
- **AND** callers can use existing 2D rendering without special-case errors

#### Scenario: Invalid model metadata fails during codegen
- **WHEN** a hero declares `render_mode = "model_3d"` without a non-empty model path or without positive scale
- **THEN** `omoba-template-ids` build-time codegen fails
- **AND** the error message identifies the invalid hero id and invalid field

### Requirement: Simulation snapshot exposes optional hero 3D render data

`SimWorldSnapshot` entity render data SHALL include optional hero 3D render metadata for hero entities whose template declares `model_3d`. Snapshot extraction SHALL resolve hero render metadata from generated template APIs using the hero template id represented by `ScriptUnitTag.unit_id`.

#### Scenario: Saika entity snapshot contains hero render data
- **WHEN** a `saika_magoichi` hero entity with `ScriptUnitTag.unit_id = "hero_saika_magoichi"` appears in a snapshot
- **THEN** the corresponding `EntityRenderData` contains optional hero render data
- **AND** that render data contains the Saika FBX path, texture path, scale, yaw offset, and z offset

#### Scenario: Non-hero entities do not carry hero render data
- **WHEN** snapshot extraction emits tower, creep, projectile, or other non-hero entities
- **THEN** their optional hero render data is absent
- **AND** existing tower, creep, projectile, and HP bar rendering behavior remains unchanged

#### Scenario: Snapshot extraction does not load Lua at runtime
- **WHEN** `extract_snapshot` resolves hero render metadata
- **THEN** it uses `omoba-template-ids` generated lookup data
- **AND** it does not read or execute files under `scripts/lua_data` at runtime

### Requirement: omfx renders 3D hero models from snapshot metadata

omfx SHALL instantiate and update a Fyrox scene node hierarchy for hero entities that have valid `model_3d` snapshot metadata and successfully loaded assets. The model visual SHALL follow the snapshot entity position and facing while hero UI, HP bars, input, abilities, and gameplay data continue to use snapshot entity data.

#### Scenario: Saika uses a 3D scene node instead of the generic 2D body
- **WHEN** omfx receives a Saika hero snapshot with valid `model_3d` metadata and the model loads successfully
- **THEN** omfx creates or reuses a 3D model node for that entity id
- **AND** omfx positions the model at the hero snapshot position in render space
- **AND** omfx applies the declared scale and facing offset
- **AND** omfx suppresses the generic 2D body quad and facing bar for that hero

#### Scenario: Existing hero HUD remains driven by snapshot data
- **WHEN** Saika is rendered as a 3D model
- **THEN** the hero panel still displays name, title, level, stats, abilities, buffs, inventory, and gold from snapshot data
- **AND** the HP bar still reflects `EntityRenderData.hp` and `EntityRenderData.max_hp`

#### Scenario: Model node is reused across stable snapshots
- **WHEN** the same Saika entity appears in consecutive snapshots
- **THEN** omfx reuses the existing 3D model node for that entity id
- **AND** it updates transforms instead of reloading or re-instantiating the model every frame

#### Scenario: Removed hero cleans up model node
- **WHEN** a hero entity using a 3D model is removed from the snapshot or appears in `removed_entity_ids`
- **THEN** omfx removes the corresponding 3D model node from the scene
- **AND** it releases the per-entity node cache entry

### Requirement: Hero 3D asset failures fall back without gameplay impact

If hero 3D metadata is missing, invalid, or asset loading fails, omfx SHALL keep the hero visible through the existing 2D fallback path. Asset failures SHALL be diagnostic and SHALL NOT panic, disconnect, or alter gameplay simulation.

#### Scenario: Missing model falls back to 2D rendering
- **WHEN** Saika declares `model_3d` metadata but the FBX cannot be loaded
- **THEN** omfx logs a diagnostic message for the failed model path
- **AND** omfx renders Saika using the existing 2D body and facing fallback
- **AND** movement, attack, abilities, selection, HP display, and hero panel continue to work

#### Scenario: Missing texture does not hide the hero
- **WHEN** the Saika FBX loads but the declared PNG texture cannot be loaded or bound
- **THEN** omfx either uses the model material fallback or existing 2D fallback
- **AND** the hero remains visible
- **AND** the failure does not affect backend or lockstep state

### Requirement: Hero 3D rendering remains render-only

Hero 3D rendering SHALL be a client visual feature only. It SHALL NOT mutate ECS gameplay components, deterministic state hashing inputs, hero stats, collision radius, attack range, movement speed, ability behavior, or network protocol payloads.

#### Scenario: Gameplay data is unchanged by 3D metadata
- **WHEN** Saika has `model_3d` metadata enabled
- **THEN** `hero_stats`, abilities, inventory, buffs, movement, attack, collision, and skill behavior remain sourced from existing gameplay systems
- **AND** disabling or removing the metadata only changes the visual representation

#### Scenario: Deterministic simulation remains unaffected
- **WHEN** simulation determinism tests run with Saika 3D metadata present
- **THEN** deterministic gameplay results remain unchanged
- **AND** the 3D model loading path is not part of simulation state hashing
