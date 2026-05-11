## ADDED Requirements

### Requirement: debug launchers load Lua content without Rust rebuild
Debug launchers（`run.bat`、`run_smoke.bat` 與 `run_smoke_long.bat`）SHALL compile artifacts with runtime Lua content support and explicitly enable runtime Lua content mode before launching the frontend/backend pair. The mode SHALL point runtime content loading at `scripts/lua_data` and SHALL allow Lua content-only changes to be consumed at runtime without rebuilding Rust artifacts.

Release builds MAY compile and use the same runtime Lua content support when explicitly requested by feature/env. `run_stress.bat` SHALL NOT enable runtime Lua content mode and SHALL continue using release artifacts with build-time generated Rust data by default.

#### Scenario: debug Lua template-only change skips Rust builds
- **WHEN** debug artifacts already exist and a file under `scripts/lua_data/templates.lua` or `scripts/lua_data/templates/` is changed without modifying Rust sources, manifests, lockfiles, build scripts, protocol files, or script source
- **THEN** the next debug launcher invocation treats `script-dll-debug`, `backend-debug`, and `frontend-debug` as up-to-date for Rust build purposes
- **AND** gameplay initialization uses the latest Lua-loaded template content for both `omb` and `omfx` `sim_runner`

#### Scenario: debug story-only change skips Rust builds
- **WHEN** debug artifacts already exist and a story Lua file under `scripts/lua_data/MVP_1`, `scripts/lua_data/TD_1`, or another debug-loaded story directory is changed without modifying Rust sources, manifests, lockfiles, build scripts, protocol files, or script source
- **THEN** the next debug launcher invocation does not invoke Cargo solely because of that Lua content change
- **AND** the configured `STORY` is initialized from the latest Lua-loaded story content

#### Scenario: invalid debug Lua fails before gameplay starts
- **WHEN** a debug launcher enables runtime Lua content mode and the selected Lua content is missing, invalid, rejected by path validation, or fails story/template validation
- **THEN** `omb` and `omfx` `sim_runner` fail initialization clearly before gameplay starts
- **AND** the error identifies the rejected path, missing story, missing template reference, or validation failure

#### Scenario: stress launcher does not enable runtime Lua content mode
- **WHEN** `run_stress.bat` launches with fresh release artifacts
- **THEN** it does not set the debug Lua content mode environment variables
- **AND** stress gameplay initializes from release build-time generated Rust data

#### Scenario: release build can opt into runtime Lua content mode
- **WHEN** a release binary is built with runtime Lua content support and launched with the required runtime Lua content mode environment variables
- **THEN** it can initialize gameplay/story content from `scripts/lua_data`
- **AND** this opt-in does not change `run_stress.bat` default generated-data behavior

## MODIFIED Requirements

### Requirement: relevant source changes 會 rebuild affected artifacts

每個 `run*.bat` launcher SHALL 在該 artifact 的任何 configured relevant input path 比 artifact 更新時，將 artifact 視為 stale。Relevant inputs MUST 包含 Rust source files、Cargo manifests、Cargo lockfiles、`rust-toolchain.toml`、build scripts、shared path dependency sources，以及該 artifact 使用的 protocol files。

Release-profile freshness inputs MUST include generated-data Lua inputs under `scripts/lua_data` because release/stress artifacts use build-time generated Rust data. Debug-profile freshness inputs SHALL NOT treat Lua content-only changes under `scripts/lua_data` as Rust artifact staleness when debug runtime Lua content mode is enabled; debug-profile freshness MUST still treat `omoba-template-ids` Rust sources, manifests, build scripts, shared dependency sources, protocol files, and script source changes as relevant Rust inputs.

#### Scenario: script source change 會 rebuild DLL
- **WHEN** `scripts/base_content/src` 底下某檔案 newer than `scripts/target/debug/base_content.dll`
- **THEN** debug launchers invoke `cargo build --manifest-path scripts\Cargo.toml -p base_content`
- **AND** resulting DLL 若 newer than `omb/scripts/base_content.dll` 就會 staged

#### Scenario: stress script source change 會 rebuild release DLL
- **WHEN** `scripts/base_content/src` 底下某檔案 newer than `scripts/target/release/base_content.dll`
- **THEN** `run_stress.bat` invoke `cargo build --release --manifest-path scripts\Cargo.toml -p base_content`
- **AND** resulting release DLL 會視需要在 launch release frontend 前 staged

#### Scenario: shared ABI change 會 rebuild script 與 backend
- **WHEN** `scripts/script-abi/src` 底下某檔案 newer than `scripts/target/debug/base_content.dll` 與 `omb/target/debug/omobab.exe`
- **THEN** debug launchers 在 launch frontend 前 rebuild `base_content` 與 `omobab`

#### Scenario: frontend source change 會 rebuild executor
- **WHEN** `omfx` frontend source tree 底下某檔案 newer than `omfx/target/debug/executor.exe`
- **THEN** launcher 在 launch 前 build matching-profile `executor` package

#### Scenario: debug Lua content-only change does not rebuild Rust artifacts
- **WHEN** only files under `scripts/lua_data` are newer than `scripts/target/debug/base_content.dll`, `omb/target/debug/omobab.exe`, and `omfx/target/debug/executor.exe`
- **THEN** debug launchers do not mark those Rust artifacts stale solely because of the Lua content timestamps
- **AND** debug runtime Lua content mode is responsible for loading the latest Lua values at initialization

#### Scenario: release Lua content change rebuilds generated-data consumers
- **WHEN** a file under `scripts/lua_data` is newer than `scripts/target/release/base_content.dll`, `omb/target/release/omobab.exe`, or `omfx/target/release/executor.exe`
- **THEN** `run_stress.bat` treats affected release artifacts as stale
- **AND** release/stress build steps regenerate and consume build-time generated Rust data before launch
