## 1. Lua Content Migration

- [x] 1.1 Inventory every shipped `scripts/lua_data/**/*.json` file and every `Creep[].Name` in shipped maps.
- [x] 1.2 Define the Lua source layout: `scripts/lua_data/templates.lua`, template modules, and per-story `entity.lua`, `ability.lua`, `mission.lua`, and `map.lua` builder files.
- [x] 1.3 Convert `scripts/lua_data/templates.json` into Lua builder modules while preserving category declaration order and keeping `td_stress.hp = 10000.0` authoritative.
- [x] 1.4 Convert all shipped story `entity.json`, `ability.json`, `mission.json`, and `map.json` files into Lua builder tables.
- [x] 1.5 Convert map `Creep[]` entries to template references and remove duplicated unit fields such as `Label`, `HP`, `DefendPhysic`, `DefendMagic`, and `MoveSpeed`.

## 2. Build-Time Lua Codegen

- [x] 2.1 Add the Rust Lua dependency only to `omoba-template-ids` build dependencies, using a vendored Lua runtime on Windows.
- [x] 2.2 Implement an `mlua` builder loader in `omoba-template-ids/build.rs` that loads Lua files, calls their builder function with a context table, and converts returned tables into codegen structs.
- [x] 2.3 Implement codegen context helpers `ctx.include(path)`, `ctx.read_text(path)`, and `ctx.read_toml(path)`, including relative-path validation, include-cycle detection, deterministic order, and `cargo:rerun-if-changed` tracking for every read file.
- [x] 2.4 Update `omoba-template-ids/build.rs` parsing flow to generate template Rust data from Lua builder output without an intermediate JSON file.
- [x] 2.5 Add generated story data structs/accessors for shipped stories without making `omoba-template-ids` depend on `omb` crate types.
- [x] 2.6 Verify the existing generated template API remains compatible for users of `*_by_name`, `*_display`, `tower_stats`, `hero_stats`, `creep_stats`, `summon_stats`, ability metadata, and projectile kind lookups.
- [x] 2.7 Verify no crate outside `omoba-template-ids` depends on `mlua` or loads Lua files for runtime/story initialization.

## 3. Pure Rust Runtime Loading

- [x] 3.1 Remove runtime dependency on `scripts/lua_data/**/*.json`; `omb` should initialize shipped stories from generated Rust data.
- [x] 3.2 Add or update adapters from generated story data into existing runtime structures such as `CampaignData` and `CreepWaveData`, or refactor initialization to consume generated story data directly.
- [x] 3.3 Slim map creep data to the reference contract and reject duplicated unit definition fields during Lua codegen or generated data validation.
- [x] 3.4 Update `StateInitializer::setup_creep_emiters` to resolve `CreepJD.Name` through `omoba_template_ids::creep_by_name`, `creep_display`, and `creep_stats`.
- [x] 3.5 Build emitter label, HP, max HP, move speed, physical defense, and magic defense from generated template stats instead of map-local fields.
- [x] 3.6 Fail fast with a clear missing-id error when a map references a creep template that does not exist or has no stats.

## 4. Tooling And References

- [x] 4.1 Update map editor, docs generation, and any import/export tooling to read/write Lua table source or generated Rust data instead of JSON content files.
- [x] 4.2 Update repository docs, comments, and build freshness checks that reference `templates.json`, story JSON files, `omb/Story`, or hard-coded story paths to use Lua source inputs and generated Rust data.
- [x] 4.3 Remove or deprecate obsolete shipped JSON content files after Lua codegen and tests are in place.

## 5. Verification

- [x] 5.1 Add a regression test that loads `templates.lua`, calls its builder function, expands includes, and verifies deterministic generated ordering for important entries such as `td_stress`.
- [x] 5.2 Add regression tests that load per-story Lua builders and verify generated TD_1 and TD_STRESS story data is available without JSON files.
- [x] 5.3 Add regression tests that confirm `ctx.read_text` / `ctx.read_toml` can read approved in-root config files and reject absolute or parent-directory paths.
- [x] 5.4 Add a regression test that scans generated map data and verifies every `Creep[].Name` resolves through `omoba_template_ids::creep_by_name` and `creep_stats`.
- [x] 5.5 Add a regression test that rejects or flags forbidden map-local creep definition fields in Lua-generated map data.
- [x] 5.6 Add a TD_STRESS-specific check that initialized `td_stress` emitter stats use generated template values, especially HP `10000.0` and move speed `100.0`.
- [x] 5.7 Add a guard that fails if runtime/tests/tools still depend on old `omb/Story` paths or shipped JSON story files.
- [x] 5.8 Add a dependency/grep guard that fails if runtime crates outside `omoba-template-ids` add `mlua` or load Lua story/template files.
- [x] 5.9 Run the relevant Rust tests for `omoba-template-ids` and `omobab` after migration and loader changes.
