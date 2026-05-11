## ADDED Requirements

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

## MODIFIED Requirements

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
