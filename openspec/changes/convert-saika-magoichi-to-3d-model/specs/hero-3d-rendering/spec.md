## ADDED Requirements

### Requirement: Hero 3D assets are declared by scripts content

Hero 3D visual metadata SHALL be declared in the scripts content hero template data. `saika_magoichi` SHALL declare `render_mode = "model_3d"`, a model path, a texture path, and positive scale metadata that points to the existing assets under `scripts/lua_data/templates/heroes/saika_magoichi/`.

The authoritative location for hero 3D assets and metadata SHALL be `scripts/lua_data`. omfx SHALL NOT own or require Saika-specific copies under `omfx/data`, and omfx source SHALL NOT contain Saika-specific asset paths, scale values, offsets, source animation names, or tick ranges.

The metadata SHALL also declare animation source inventory and required animation bindings for `move`, `attack`, `critical`, and `sniper`. Because Saika's base/action FBX files expose animations named `Take 001`, Saika metadata SHALL declare logical animation source keys with source FBX paths, source animation names, duration ticks, and ticks per second. Each required binding SHALL map to a logical source key with an explicit non-empty tick range.

#### Scenario: Saika declares 3D model metadata
- **WHEN** `scripts/lua_data/templates/heroes.lua` is loaded by template codegen
- **THEN** the `saika_magoichi` hero entry contains `render.render_mode = "model_3d"`
- **AND** it references `templates/heroes/saika_magoichi/saika_magoichi.fbx`
- **AND** it references `templates/heroes/saika_magoichi/saika_magoichi_mat.png`
- **AND** it declares a positive model scale
- **AND** it declares logical animation source metadata with source FBX paths, source animation names, positive duration ticks, and positive ticks-per-second
- **AND** it declares animation bindings for `move`, `attack`, `critical`, and `sniper`

#### Scenario: Saika 3D assets exist at the declared location
- **WHEN** the declared `saika_magoichi` 3D asset paths are checked relative to `scripts/lua_data/`
- **THEN** `scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx` exists
- **AND** `scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi_mat.png` exists

#### Scenario: No canonical Saika 3D data exists in omfx
- **WHEN** implementation is complete
- **THEN** Saika model path, texture path, scale, pitch offset, roll offset, yaw offset, z offset, source animation name, and animation tick ranges are declared in scripts metadata or generated data
- **AND** `omfx` source does not contain a Saika-specific hard-coded asset path or animation range table
- **AND** `omfx/data` is not required as a canonical location for Saika 3D model or texture files

#### Scenario: Heroes without 3D metadata remain valid
- **WHEN** a hero template does not declare a `render` table
- **THEN** template codegen still succeeds
- **AND** that hero remains eligible for existing 2D fallback rendering

#### Scenario: Assimp reports Saika animation inventory
- **WHEN** `assimp info scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx` is run against the shipped model
- **THEN** the output reports `Animations: 1`
- **AND** the output lists named animation `Take 001`
- **AND** the output reports 32 bones and 24 animation channels
- **AND** `assimp dump` reports `Take 001` duration `1000` ticks and tick count/rate `30`

#### Scenario: Assimp reports Saika action animation inventory
- **WHEN** `assimp dump` is run against the shipped Saika action FBX files under `scripts/lua_data/templates/heroes/saika_magoichi/`
- **THEN** `b01_ani_attack.fbx` reports a `Take 001` animation with duration `100` ticks and tick count/rate `30`
- **AND** `b01_ani_run.fbx` reports a `Take 001` animation with duration `23` ticks and tick count/rate `30`
- **AND** `b01_ani_stand3.fbx` reports a `Take 001` animation with duration `53` ticks and tick count/rate `30`
- **AND** these action FBX files remain under `scripts/lua_data` as content-owned animation sources

### Requirement: Hero animation bindings describe gameplay actions

For a `model_3d` hero, the content metadata SHALL map gameplay-facing animation actions to logical source animation segments. `saika_magoichi` SHALL provide logical animation source inventory entries and bindings for `move`, `attack`, `critical`, and `sniper`; each source SHALL include source FBX path, source animation name, duration ticks, and ticks-per-second. Each binding SHALL include logical source key, start tick, end tick, and loop behavior. `attack` and `critical` bindings SHALL also include an impact tick between start and end so omfx can align the animation hit frame with the authoritative attack impact event.

#### Scenario: Required Saika animation bindings are present
- **WHEN** template codegen reads `saika_magoichi.render.animations`
- **THEN** it finds `move`, `attack`, `critical`, and `sniper` keys
- **AND** every binding references a logical source key in the declared animation source inventory
- **AND** every referenced source declares source animation `Take 001`
- **AND** every binding has `end_tick > start_tick`
- **AND** every binding range is within the declared source duration that was authored from Assimp inspection
- **AND** `attack` and `critical` bindings have `start_tick < impact_tick < end_tick`

#### Scenario: Animation source inventory is build-time validated
- **WHEN** template codegen reads `saika_magoichi.render.animation_sources`
- **THEN** it finds logical sources referenced by `move`, `attack`, `critical`, and `sniper`
- **AND** every source declares a non-empty source FBX path or explicitly uses the base model source
- **AND** every source declares a non-empty source animation name
- **AND** every source declares positive `duration_ticks`
- **AND** every source declares positive `ticks_per_second`
- **AND** codegen can validate binding ranges without parsing the FBX file or requiring an `assimp` executable

#### Scenario: Move and sniper bindings are loopable
- **WHEN** Saika metadata is generated
- **THEN** the `move` binding is marked loopable
- **AND** the `sniper` binding is marked loopable
- **AND** omfx can use either binding as a sustained state animation

#### Scenario: Attack and critical bindings are one-shot
- **WHEN** Saika metadata is generated
- **THEN** the `attack` binding is marked non-looping
- **AND** the `critical` binding is marked non-looping
- **AND** both bindings include an impact tick for windup/impact/backswing alignment
- **AND** omfx can return to a sustained state animation after either binding completes

#### Scenario: Missing required binding fails validation
- **WHEN** a `model_3d` hero omits any of `move`, `attack`, `critical`, or `sniper`
- **THEN** `omoba-template-ids` build-time codegen fails
- **AND** the error message identifies the hero id and missing action key

### Requirement: Generated hero render metadata is available to runtime code

`omoba-template-ids` SHALL parse optional hero render metadata at build time and expose it through generated Rust lookup APIs. Runtime crates SHALL consume generated data and SHALL NOT load Lua files to discover hero model paths.

#### Scenario: Saika generated metadata returns model data
- **WHEN** runtime code calls `hero_render_metadata(HERO_SAIKA_MAGOICHI)`
- **THEN** the lookup returns metadata with `render_mode = "model_3d"`
- **AND** the metadata contains the Saika FBX path
- **AND** the metadata contains the Saika PNG texture path
- **AND** the metadata contains positive scale data
- **AND** the metadata contains generated logical animation source data, including action FBX paths and source animation names
- **AND** the metadata contains generated animation bindings for `move`, `attack`, `critical`, and `sniper`
- **AND** the `attack` and `critical` bindings contain impact tick metadata

#### Scenario: Hero without render metadata returns none
- **WHEN** runtime code calls `hero_render_metadata` for a hero that has no `render` table
- **THEN** the lookup returns `None`
- **AND** callers can use existing 2D rendering without special-case errors

#### Scenario: Invalid model metadata fails during codegen
- **WHEN** a hero declares `render_mode = "model_3d"` without a non-empty model path or without positive scale
- **THEN** `omoba-template-ids` build-time codegen fails
- **AND** the error message identifies the invalid hero id and invalid field

#### Scenario: Invalid animation source or range fails during codegen
- **WHEN** a hero animation binding references an unknown declared logical source key, has `end_tick <= start_tick`, has an attack impact tick outside `start_tick..end_tick`, exceeds the declared source animation duration, or the source has non-positive ticks-per-second
- **THEN** `omoba-template-ids` build-time codegen fails
- **AND** the error message identifies the invalid hero id, action key, and range field

### Requirement: Simulation snapshot exposes optional hero 3D render data

`SimWorldSnapshot` entity render data SHALL include optional hero 3D render metadata for hero entities whose template declares `model_3d`. Snapshot extraction SHALL resolve hero render metadata from generated template APIs using the hero template id represented by `ScriptUnitTag.unit_id`.

Snapshot data SHALL provide enough render-only state for omfx to choose `move`, `attack`, `critical`, and `sniper` animation actions. These cues SHALL be derived from existing movement, buff, attack, or damage/outcome data and SHALL NOT affect deterministic gameplay state.

#### Scenario: Saika entity snapshot contains hero render data
- **WHEN** a `saika_magoichi` hero entity with `ScriptUnitTag.unit_id = "hero_saika_magoichi"` appears in a snapshot
- **THEN** the corresponding `EntityRenderData` contains optional hero render data
- **AND** that render data contains the Saika FBX path, texture path, scale, pitch offset, roll offset, yaw offset, and z offset
- **AND** that render data contains the four generated animation bindings

#### Scenario: Saika sniper mode snapshot exposes sniper action state
- **WHEN** Saika has an active `sniper_mode` buff in snapshot data
- **THEN** omfx can choose the `sniper` animation action for the hero model
- **AND** the action cue remains render-only

#### Scenario: Saika attack and critical cues are distinguishable
- **WHEN** Saika performs a normal attack
- **THEN** omfx can choose the `attack` animation action
- **AND** when the attack result is critical, omfx can choose the `critical` animation action instead
- **AND** neither cue changes damage, cooldown, target selection, or simulation hash input

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

omfx SHALL use generated logical animation source and binding metadata to play Saika's `move`, `attack`, `critical`, and `sniper` actions from content-owned action FBX tick ranges. omfx SHALL convert content ticks to Fyrox animation seconds using generated ticks-per-second metadata. omfx SHALL NOT hard-code Saika animation tick ranges, Saika asset paths, Saika source animation names, or Saika model scale/offsets in frontend source.

The omfx implementation SHALL be generic: it MAY provide scripts asset path resolution, model loading, texture binding, node lifecycle management, animation segment playback, and fallback behavior, but all hero-specific values SHALL come from generated/snapshot metadata.

#### Scenario: Saika uses a 3D scene node instead of the generic 2D body
- **WHEN** omfx receives a Saika hero snapshot with valid `model_3d` metadata and the model loads successfully
- **THEN** omfx creates or reuses a 3D model node for that entity id
- **AND** omfx positions the model at the hero snapshot position in render space
- **AND** omfx applies the declared scale and facing offset
- **AND** omfx suppresses the generic 2D body quad and facing bar for that hero

#### Scenario: Saika movement plays move binding
- **WHEN** Saika is moving according to snapshot position/state
- **THEN** omfx plays the `move` animation binding as a loop
- **AND** the playback segment comes from the metadata range for the binding's logical source
- **AND** omfx converts the metadata tick range to seconds before calling Fyrox animation playback APIs

#### Scenario: Saika normal attack plays attack binding
- **WHEN** Saika receives a normal attack render cue
- **THEN** omfx plays the `attack` animation binding once
- **AND** omfx retimes the binding so `start_tick..impact_tick` matches the cue windup duration
- **AND** omfx retimes `impact_tick..end_tick` to match the cue backswing duration
- **AND** omfx uses separate Fyrox playback phases before and after impact if one `Animation::set_speed` value cannot satisfy both durations
- **AND** playback returns to `move`, `sniper`, or default state after the one-shot completes

#### Scenario: Fyrox model resource is cached and instantiated once per entity
- **WHEN** omfx receives a hero snapshot with valid `model_3d` metadata for the first time
- **THEN** omfx requests the model through Fyrox `ResourceManager` using the resolved scripts asset path
- **AND** omfx keeps 2D fallback visible while the resource is still loading
- **AND** after the model resource is loaded, omfx instantiates a scene node hierarchy and reuses it for stable snapshots

#### Scenario: Fyrox animation is retargeted to the model instance
- **WHEN** omfx instantiates a model-backed hero
- **THEN** omfx creates or finds an animation player for that model instance
- **AND** omfx retargets generated animation source FBX resources to the instance hierarchy
- **AND** action playback selects animations by generated logical source key rather than by hero-specific frontend code or by non-unique animation name alone

#### Scenario: Manual texture fallback is generic
- **WHEN** the FBX importer does not bind the declared diffuse texture automatically
- **THEN** omfx may load the metadata texture path relative to `scripts/lua_data/`
- **AND** omfx may bind it to the model mesh surfaces through a standard 3D material
- **AND** the fallback remains generic and does not contain a Saika-specific path outside generated/snapshot metadata

#### Scenario: Saika critical attack plays critical binding
- **WHEN** Saika receives a critical attack render cue
- **THEN** omfx plays the `critical` animation binding once
- **AND** the binding impact tick aligns with the authoritative attack impact event
- **AND** this visual choice does not change damage calculation

#### Scenario: Saika sniper mode plays sniper binding
- **WHEN** Saika has active `sniper_mode` state and is not playing a higher-priority one-shot
- **THEN** omfx plays the `sniper` animation binding as a loop
- **AND** the binding remains active until `sniper_mode` ends or a higher-priority action interrupts it

#### Scenario: Existing hero HUD remains driven by snapshot data
- **WHEN** Saika is rendered as a 3D model
- **THEN** the hero panel still displays name, title, level, stats, abilities, buffs, inventory, and gold from snapshot data
- **AND** the HP bar still reflects `EntityRenderData.hp` and `EntityRenderData.max_hp`

#### Scenario: Model node is reused across stable snapshots
- **WHEN** the same Saika entity appears in consecutive snapshots
- **THEN** omfx reuses the existing 3D model node for that entity id
- **AND** it updates transforms instead of reloading or re-instantiating the model every frame

#### Scenario: Animation binding ranges are data-driven
- **WHEN** Saika animation tick ranges are changed in `scripts/lua_data/templates/heroes.lua`
- **THEN** omfx uses the generated metadata after rebuild
- **AND** no omfx source change is required for those range adjustments

#### Scenario: omfx renderer is hero-data agnostic
- **WHEN** a different hero later declares `model_3d` metadata in scripts with its own model path, texture path, scale, offsets, and animation bindings
- **THEN** omfx can load and play that hero through the same generic pipeline
- **AND** no new hero-specific source mapping is required in omfx

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
