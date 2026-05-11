## Purpose

定義 shipped content 以 Lua builder 作為 canonical source，build-time codegen 產生模板與 story 資料，runtime 只消費 generated Rust data 的契約。

## Requirements

### Requirement: All shipped content source files are Lua builders

The canonical content source under `scripts/lua_data` SHALL be Lua builder files. Shipped JSON content files under `scripts/lua_data/**/*.json` SHALL NOT be required for build-time codegen or runtime gameplay after migration.

The expected source files SHALL include `scripts/lua_data/templates.lua` and per-story files such as `entity.lua`, `ability.lua`, `mission.lua`, and `map.lua`.

#### Scenario: No shipped JSON source is required

- **WHEN** the project is built from migrated content
- **THEN** template and story generated Rust data come from Lua builder files
- **AND** removing `scripts/lua_data/**/*.json` does not break runtime story initialization

#### Scenario: Story source is Lua table data

- **WHEN** `scripts/lua_data/TD_STRESS/map.lua` is loaded by codegen
- **THEN** the Lua builder returns a table representing map data
- **AND** that table is converted into generated Rust story data

### Requirement: Lua loading is build-time only in omoba-template-ids

`omoba-template-ids/build.rs` SHALL remain the only production/default code path that loads Lua files, and it SHALL do so at build time. Runtime crates and tools outside `omoba-template-ids` SHALL consume generated Rust data and SHALL NOT load Lua files for gameplay/story initialization unless runtime Lua content support is explicitly compiled and enabled by env.

No crate outside `omoba-template-ids` SHALL add `mlua` as an unconditional production runtime dependency for this change. Any `mlua` dependency outside `omoba-template-ids` MUST be gated behind runtime Lua content support and MUST NOT be required by default release/stress gameplay.

#### Scenario: Codegen loads Lua

- **WHEN** `omoba-template-ids` is built
- **THEN** `build.rs` reads Lua files under `scripts/lua_data`
- **AND** `build.rs` uses `mlua` to call builder functions

#### Scenario: Default runtime uses generated Rust only

- **WHEN** `omb` initializes a shipped story without runtime Lua content mode enabled
- **THEN** it resolves story data through generated Rust APIs
- **AND** it resolves template stats through generated Rust APIs
- **AND** it does not load or execute Lua files
- **AND** it does not parse content JSON files from `scripts/lua_data`

#### Scenario: Runtime Lua mode may load Lua explicitly

- **WHEN** a binary is built with runtime Lua content support and runtime Lua content mode is explicitly enabled
- **THEN** `omb` and `omfx` `sim_runner` may load Lua files under the configured `scripts/lua_data` root for gameplay/story initialization
- **AND** that loader remains unavailable to release/stress default gameplay paths

#### Scenario: Runtime crates do not depend on unconditional mlua

- **WHEN** Cargo manifests outside `omoba-template-ids` are inspected
- **THEN** none of them add `mlua` as an unconditional dependency for production runtime/story loading
- **AND** any `mlua` usage outside `omoba-template-ids` is gated behind runtime Lua content support

### Requirement: Runtime Lua content snapshot preserves generated semantics

When explicitly compiled and enabled for local runtime use, runtime Lua content loading SHALL use the same Lua builder contract, deterministic ordering, path validation, include-cycle detection, and template/story validation as build-time codegen. The Lua-loaded content snapshot SHALL be equivalent to the generated Rust data shape consumed by runtime initialization.

Debug launchers SHALL enable this mode by default. Release builds MAY use the same mode only when the required feature/env is explicitly enabled. `omb` host and `omfx` `sim_runner` SHALL use the same content root and configured story id when runtime Lua content mode is enabled.

#### Scenario: debug loader uses the same builder helpers
- **WHEN** runtime Lua content mode loads `scripts/lua_data/templates.lua` or a story Lua file
- **THEN** it evaluates Lua builders through the same approved helpers including `ctx.include(path)`, `ctx.read_text(path)`, and `ctx.read_toml(path)`
- **AND** it rejects absolute paths, parent-directory traversal, and paths resolving outside `scripts/lua_data`

#### Scenario: debug loader preserves declaration order
- **WHEN** a debug-loaded Lua builder returns array-like template or story declarations
- **THEN** runtime ids and lookup order preserve the same declaration order as build-time generated Rust data

#### Scenario: host and sim_runner use the same debug content source
- **WHEN** a debug launcher enables runtime Lua content mode for a configured `STORY`
- **THEN** `omb` initializes that story from the debug Lua-loaded content root
- **AND** `omfx` `sim_runner` initializes its replica world from the same content root and story id

#### Scenario: release opt-in uses the same content source contract
- **WHEN** a release binary is built with runtime Lua content support and launched with runtime Lua content mode enabled for a configured `STORY`
- **THEN** `omb` initializes that story from the configured Lua content root
- **AND** `omfx` `sim_runner` initializes its replica world from the same content root and story id

#### Scenario: debug loader validates creep template references
- **WHEN** a debug-loaded map declares `Creep[].Name: "missing_creep_template"`
- **THEN** story initialization fails before gameplay starts
- **AND** the error message includes `missing_creep_template`

### Requirement: Lua builders return data through mlua

Lua content files SHALL be loaded with `mlua`. Each Lua file SHALL produce a builder function. The loader SHALL call that function with a context table and SHALL convert the returned Lua table into the corresponding Rust codegen data structures.

The context table SHALL expose project-approved helpers including `ctx.include(path)`, `ctx.read_text(path)`, and `ctx.read_toml(path)`. Path-based helpers SHALL read relative to `scripts/lua_data/`, MUST NOT accept absolute paths, and MUST NOT allow parent-directory traversal.

The build script SHALL track every Lua/helper/config file read and emit `cargo:rerun-if-changed` for each one.

#### Scenario: Builder function returns template data

- **WHEN** `scripts/lua_data/templates.lua` is loaded
- **THEN** the Lua chunk produces a builder function
- **AND** `omoba-template-ids/build.rs` calls the builder through `mlua`
- **AND** the returned table converts into the template codegen model

#### Scenario: Supplemental config can be read through context

- **WHEN** a Lua builder calls `ctx.read_toml("templates/creeps_extra.toml")`
- **THEN** the helper reads the file relative to `scripts/lua_data/`
- **AND** the returned Lua table can be used to compute the final generated data
- **AND** `build.rs` emits `cargo:rerun-if-changed` for the TOML file

#### Scenario: Unsafe read paths are rejected

- **WHEN** a Lua builder calls `ctx.read_text("../secret.txt")` or `ctx.read_toml("C:/secret.toml")`
- **THEN** codegen fails
- **AND** the error message identifies the rejected path

### Requirement: Lua includes preserve generated order

Lua builders SHALL support `ctx.include(path)` for splitting data across multiple Lua files. Include paths SHALL be relative to `scripts/lua_data/`, MUST NOT be absolute paths, and MUST NOT escape the data root with parent-directory traversal.

Included builders SHALL be evaluated through the same `mlua` loader. Array-like declaration order in returned tables SHALL be preserved and SHALL be the order used by generated Rust data.

#### Scenario: Included modules load in sequence order

- **WHEN** a builder calls `ctx.include("templates/towers.lua")` and `ctx.include("templates/creeps.lua")`
- **THEN** entries are loaded from each included file in sequence order
- **AND** generated ids within each category follow the loaded declaration order

#### Scenario: Include cycles fail clearly

- **WHEN** a Lua include cycle is encountered
- **THEN** codegen fails before emitting generated Rust data
- **AND** the error message includes the include cycle path

### Requirement: Template id codegen preserves public Rust API

`omoba-template-ids/build.rs` SHALL load the Lua template catalog with `mlua`, call the template builder function, and emit generated Rust code with the same public lookup API currently used by runtime and scripts. This includes generated ids, constants, `*_by_name`, `*_id_str`, `*_display`, `tower_stats`, `hero_stats`, `creep_stats`, `summon_stats`, ability metadata lookups, and projectile kind lookups.

The build script SHALL use returned Lua tables as input and SHALL NOT require an intermediate generated JSON file.

#### Scenario: Existing Rust lookups keep working

- **WHEN** code calls `omoba_template_ids::creep_by_name("td_stress")`
- **THEN** it returns the generated `td_stress` creep id
- **AND** `omoba_template_ids::creep_stats(id)` returns stats from the Lua template catalog

### Requirement: Generated story data is available to pure Rust runtime

`omoba-template-ids` SHALL emit dependency-light generated Rust data or accessors for shipped stories. In production/default runtime mode, `omb` SHALL initialize shipped stories from generated Rust data rather than reading JSON or Lua source files at runtime.

Generated story structs SHALL avoid depending on `omb` crate types. `omb` MAY convert generated data into existing runtime structures or consume generated data directly. Explicit runtime Lua content mode MAY construct equivalent runtime structures from Lua-loaded content for local debug launchers and release opt-in runs.

#### Scenario: omb loads TD_1 through generated data

- **WHEN** `STORY = "TD_1"` is configured and runtime Lua content mode is not enabled
- **THEN** `omb` obtains TD_1 entity, ability, mission, and map data through generated Rust APIs
- **AND** `omb` does not require `scripts/lua_data/TD_1/*.json`
- **AND** `omb` does not execute `scripts/lua_data/TD_1/*.lua`

#### Scenario: debug omb loads TD_1 through Lua-loaded content

- **WHEN** `STORY = "TD_1"` is configured and a debug launcher explicitly enables runtime Lua content mode
- **THEN** `omb` obtains TD_1 entity, ability, mission, and map data from Lua builders under `scripts/lua_data/TD_1`
- **AND** the resulting runtime structures pass the same campaign validation as generated story data

#### Scenario: release opt-in omb loads TD_1 through Lua-loaded content

- **WHEN** `STORY = "TD_1"` is configured and a release binary is built with runtime Lua content support and launched with runtime Lua content mode enabled
- **THEN** `omb` obtains TD_1 entity, ability, mission, and map data from Lua builders under the configured Lua content root
- **AND** default release/stress execution without that opt-in still obtains TD_1 through generated Rust APIs

### Requirement: Creep templates are the canonical unit definitions

Generated Rust template data SHALL contain the authoritative creep definition for every creep id referenced by shipped story maps. Each active creep template SHALL define `id`, `display_name`, `hp`, `armor`, `magic_resistance`, `damage`, `attack_range`, `move_speed`, `enemy_type`, `ai_type`, `exp_reward`, and `gold_reward`.

Story maps SHALL NOT override these fields with map-local creep stats or labels.

#### Scenario: Every shipped map creep resolves to a generated template

- **WHEN** all generated map data is scanned for `Creep[].Name` values
- **THEN** every value resolves through `omoba_template_ids::creep_by_name`
- **AND** every resolved id has `omoba_template_ids::creep_stats(id)` data

#### Scenario: TD_STRESS uses the generated catalog value

- **WHEN** the `td_stress` creep template is loaded from generated Rust data
- **THEN** its `display_name` is `壓測怪`
- **AND** its `hp` is `10000.0`
- **AND** its `move_speed` is `100.0`

### Requirement: Story maps declare creep template references

Generated map data SHALL represent entries under `Creep[]` as template references. `Creep[].Name` SHALL equal a generated creep template id and SHALL be the name used by `CreepWave[].Detail[].Creeps[].Creep`.

Map-local creep entries MUST NOT contain duplicated unit definition fields, including `Label`, `HP`, `DefendPhysic`, `DefendMagic`, `MoveSpeed`, `damage`, `attack_range`, `enemy_type`, `ai_type`, `exp_reward`, or `gold_reward`.

#### Scenario: TD_STRESS creep entry is reference-only

- **WHEN** generated TD_STRESS map data is inspected
- **THEN** the `Creep[]` entry for `td_stress` contains `Name: "td_stress"`
- **AND** the entry does not contain `Label`, `HP`, `DefendPhysic`, `DefendMagic`, or `MoveSpeed`

### Requirement: Creep emitters resolve stats from generated templates

`omb/src/state/initialization.rs::setup_creep_emiters` SHALL build each `CreepEmiter` by resolving `CreepJD.Name` through `omoba_template_ids::creep_by_name`, `creep_display`, and `creep_stats`. The emitter's label, HP, max HP, move speed, physical defense, and magic defense SHALL come from the resolved generated template data.

If a map references a missing or stat-less creep template, initialization MUST fail fast with an error message that includes the missing creep id.

#### Scenario: TD_STRESS emitter uses generated template stats

- **WHEN** `TD_STRESS` initializes creep emitters from generated story data
- **THEN** the `td_stress` emitter has label `壓測怪`
- **AND** its HP and max HP are built from generated template `hp = 10000.0`
- **AND** its move speed is built from generated template `move_speed = 100.0`

#### Scenario: Missing template reference fails clearly

- **WHEN** a map declares `Creep[].Name: "missing_creep_template"`
- **THEN** story initialization fails before gameplay starts
- **AND** the error message includes `missing_creep_template`
