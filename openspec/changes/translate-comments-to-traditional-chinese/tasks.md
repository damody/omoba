## 1. Scope And Inventory

- [ ] 1.1 Enumerate first-party source paths to include, covering `omb/`, `omfx/`, `omoba-core/`, `eui/`, `scripts/`, `omb-mcp/`, `map_editor/` and project-owned tools/config files.
- [ ] 1.2 Enumerate excluded paths, including generated files, build artifacts, `target/`, vendored code, `specs/`, `log4rs/` and any forked dependency source not explicitly approved for this change.
- [ ] 1.3 Identify comment syntaxes by language (`//`, `///`, `//!`, `/* */`, `--`, `#`, `/** */`) and mark high-risk areas where comments sit near runtime-visible strings or fixtures.

## 2. Comment Localization

- [ ] 2.1 Translate Rust comments and doc comments in shared/core first-party crates while preserving symbol names and API terms.
- [ ] 2.2 Translate Rust comments and doc comments in backend/gameplay areas, including `omb/` and script ABI related first-party code, without changing FFI/API/schema behavior.
- [ ] 2.3 Translate comments in `scripts/` source and Lua content while preserving unit ids, ability ids, story ids and data literals.
- [ ] 2.4 Translate frontend/UI comments in `omfx/`, `eui/` and related JavaScript/TypeScript/WASM glue while preserving UI strings and log/error messages.
- [ ] 2.5 Translate remaining first-party tool/config comments, including MCP, map editor and project scripts, while preserving `.bat` CRLF line endings.
- [ ] 2.6 Remove comments that are obsolete or misleading only when removal does not alter behavior and is easier to verify than translation.

## 3. Review And Verification

- [ ] 3.1 Review the full diff and confirm non-comment code, runtime-visible strings, schemas, test expected values and identifiers were not changed.
- [ ] 3.2 Run formatting for touched Rust workspaces where applicable and verify formatting changes do not introduce behavior edits.
- [ ] 3.3 Run targeted tests or build smoke for touched workspaces, recording any command that cannot be run or any pre-existing failure.
- [ ] 3.4 Run `graphify update .` after code comment changes so the project graph stays current.
- [ ] 3.5 Produce a final implementation summary listing translated areas, excluded paths, verification commands and any remaining comments needing manual follow-up.
