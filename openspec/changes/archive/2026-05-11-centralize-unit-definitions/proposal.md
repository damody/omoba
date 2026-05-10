## Why

Unit definitions and story data are currently stored as JSON files under `scripts/lua_data`, and map-local creep definitions can still duplicate template stats such as labels, HP, defenses, and movement speed. This keeps content fragmented across large JSON files and lets the same unit id carry different values depending on where it is referenced.

All content source files should become Lua table builders so content authors can compose data in Lua and read supplemental settings through controlled helpers, while the game/runtime crates continue to consume generated Rust data only. `omoba-template-ids` remains the only crate that executes Lua, at build time.

## What Changes

- Replace every shipped JSON content source under `scripts/lua_data` with Lua table builder files: `templates.lua` plus per-story `entity.lua`, `ability.lua`, `mission.lua`, and `map.lua`.
- Add an `mlua`-based builder loader in `omoba-template-ids/build.rs`: load Lua, call a standardized builder function, then read the returned Lua table into Rust codegen structs.
- Provide approved builder context helpers such as `ctx.include(path)`, `ctx.read_text(path)`, and `ctx.read_toml(path)` so Lua content can compose modules or read supplemental settings.
- Extend `omoba-template-ids` codegen beyond template ids so it emits pure Rust data/API for templates and shipped story content.
- Keep `omb`, `omfx`, map editor, scripts, and runtime story loading pure Rust: they must not read JSON source files, read Lua files, or depend on `mlua`.
- Change map creep declarations from full stat records to template references; `map.lua` may define where and when units spawn, but not duplicate unit stats.
- Resolve TD creep emitters by template id at load time through generated `omoba-template-ids` data.
- **BREAKING**: `scripts/lua_data/**/*.json` is no longer canonical content. Content source is Lua table builders, and runtime code consumes generated Rust data.

## Capabilities

### New Capabilities
- `unit-template-references`: Defines how build-time Lua table builders feed generated Rust template/story data and how map unit references resolve through generated template definitions.

### Modified Capabilities

## Impact

- Affected data: all shipped `scripts/lua_data/**/*.json` files and their Lua replacements.
- Affected generated metadata: `omoba-template-ids/build.rs`, its build dependencies, generated `template_ids_gen.rs` or companion generated files, and tests/docs that assume content is read from JSON.
- Affected runtime: story data path/loading code, `omb/src/ue4/import_map.rs`, and `omb/src/state/initialization.rs` must consume generated Rust data only.
- Affected tooling: map editor, docs generation, run/build freshness checks, and import/export paths that still reference JSON content files or write map-local unit stat records.
