## Purpose

定義 Windows 開發與 smoke/stress launcher 的增量建置行為，確保 fresh artifacts 不重複 Cargo build，同時在 artifact missing、source newer 或 freshness check error 時 fail safe rebuild。

## Requirements

### Requirement: `run*.bat` skips fresh build steps

每個 `run*.bat` launcher SHALL 在對應 output artifact 存在，且沒有 configured relevant input 比該 artifact 更新時，skip script DLL、backend 與 frontend Cargo build steps。Debug launchers（`run.bat`、`run_smoke.bat` 與 `run_smoke_long.bat`）SHALL 使用 debug artifacts。`run_stress.bat` SHALL 使用 release artifacts。

Skipped frontend build SHALL 仍會從 repo root launch 已 build 的 `omfx/target/<profile>/executor.exe`。

#### Scenario: source 未變時第二次 launch 會 skip builds
- **WHEN** `run*.bat` launcher 成功執行一次，且沒有 relevant input files 變更
- **THEN** 下一次 invocation 同一個 launcher 時，script DLL、backend 與 frontend build artifacts 會回報 up-to-date
- **AND** launch frontend 前不會對這些 fresh artifacts invoke Cargo build work

#### Scenario: artifact missing 時會 rebuild
- **WHEN** 任一 required matching-profile output artifact missing
- **THEN** launcher 將該 artifact 視為 stale
- **AND** 繼續前會 invoke matching Cargo build step

#### Scenario: stress launcher 使用 release artifacts
- **WHEN** `run_stress.bat` 以 fresh `scripts/target/release/base_content.dll`、`omb/target/release/omobab.exe` 與 `omfx/target/release/executor.exe` 執行
- **THEN** 它回報這些 release artifacts up-to-date
- **AND** stress build pipeline 不使用 debug Cargo build steps

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
- **THEN** it does not set the runtime Lua content mode environment variables
- **AND** stress gameplay initializes from release build-time generated Rust data

#### Scenario: release build can opt into runtime Lua content mode
- **WHEN** a release binary is built with runtime Lua content support and launched with the required runtime Lua content mode environment variables
- **THEN** it can initialize gameplay/story content from `scripts/lua_data`
- **AND** this opt-in does not change `run_stress.bat` default generated-data behavior

### Requirement: staged artifacts 只有 stale 時才 copy

launchers SHALL 只有在 staged DLL missing、older than selected source DLL，或包含不同 selected build-profile artifact 時，才 copy `scripts/target/<profile>/base_content.dll` 到 `omb/scripts/base_content.dll`。no-change launch SHALL NOT rewrite staged DLL。

`run_stress.bat` SHALL 只有在 debug spawn copy missing、older than release executable，或 content 不同時，才 copy `omb/target/release/omobab.exe` 到 `omb/target/debug/omobab.exe`。

#### Scenario: unchanged staged DLL 會保留
- **WHEN** selected `scripts/target/<profile>/base_content.dll` 與 `omb/scripts/base_content.dll` 都存在，staged DLL 至少與 source DLL 一樣新，且兩個檔案 content 相同
- **THEN** launcher skips DLL copy
- **AND** frontend launch 前 `omb/scripts/base_content.dll` 的 `LastWriteTime` 維持不變

#### Scenario: newer source DLL 會 staged
- **WHEN** `scripts/target/debug/base_content.dll` 存在且 newer than `omb/scripts/base_content.dll`
- **THEN** launcher 將 source DLL copy 到 `omb/scripts/base_content.dll`
- **AND** frontend launch 時載入 updated staged DLL path

#### Scenario: stress backend spawn copy unchanged 時會保留
- **WHEN** `omb/target/release/omobab.exe` 與 `omb/target/debug/omobab.exe` 都存在，debug spawn copy 至少與 release executable 一樣新，且兩個檔案 content 相同
- **THEN** `run_stress.bat` skips backend spawn copy
- **AND** frontend launch 前 `omb/target/debug/omobab.exe` 的 `LastWriteTime` 維持不變

### Requirement: relevant source changes 會 rebuild affected artifacts

每個 `run*.bat` launcher SHALL 在該 artifact 的任何 configured relevant input path 比 artifact 更新時，將 artifact 視為 stale。Relevant inputs MUST 包含 Rust source files、Cargo manifests、Cargo lockfiles、`rust-toolchain.toml`、build scripts、shared path dependency sources，以及該 artifact 使用的 protocol files。

Release-profile freshness inputs MUST include generated-data Lua inputs under `scripts/lua_data` because release/stress artifacts use build-time generated Rust data. Debug-profile freshness inputs SHALL NOT treat Lua content-only changes under `scripts/lua_data` as Rust artifact staleness when runtime Lua content mode is enabled by debug launcher; debug-profile freshness MUST still treat `omoba-template-ids` Rust sources, manifests, build scripts, shared dependency sources, protocol files, and script source changes as relevant Rust inputs.

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
- **AND** runtime Lua content mode is responsible for loading the latest Lua values at initialization

#### Scenario: release Lua content change rebuilds generated-data consumers
- **WHEN** a file under `scripts/lua_data` is newer than `scripts/target/release/base_content.dll`, `omb/target/release/omobab.exe`, or `omfx/target/release/executor.exe`
- **THEN** `run_stress.bat` treats affected release artifacts as stale
- **AND** release/stress build steps regenerate and consume build-time generated Rust data before launch

### Requirement: freshness checks 必須 fail safe

如果 freshness helper 無法 inspect input path、比較 timestamps、在需要時比較 content，或判定 artifact state，launcher SHALL 將 artifact 視為 stale 並執行對應 Cargo build step。Build failures MUST 以 non-zero exit code 停止 launcher，且不得啟動 frontend。

#### Scenario: freshness helper error 會 fallback to build
- **WHEN** freshness helper 對 build artifact 以 error 退出
- **THEN** launcher 將該 artifact 視為 stale
- **AND** invoke 對應 Cargo build step

#### Scenario: Cargo build failure 會停止 launch
- **WHEN** required Cargo build step fails
- **THEN** launcher 以 non-zero status exit
- **AND** 不 launch matching-profile frontend executable

### Requirement: launcher-specific runtime behavior 保持不變

incremental freshness checks SHALL NOT 改變 launcher-specific runtime setup。`run_smoke.bat` SHALL 保留 2-second auto-start 與 10-second auto-exit。`run_smoke_long.bat` SHALL 保留 2-second auto-start 與 60-second auto-exit。`run_stress.bat` SHALL 持續 regenerate stress map、launch 前把 `omb/game.toml` swap 到 stress variant，並在完成或失敗後 restore 原本的 `omb/game.toml`。

#### Scenario: smoke launchers 保留 auto-exit settings
- **WHEN** `run_smoke.bat` 或 `run_smoke_long.bat` 以 fresh artifacts 執行
- **THEN** launcher 視情況 skip fresh builds
- **AND** 設定與以往相同的 `OMFX_AUTO_START_AFTER_SEC` 與 `OMFX_AUTO_EXIT_AFTER_SEC` values

#### Scenario: stress launcher 在 skipped builds 後仍 restores game.toml
- **WHEN** `run_stress.bat` 以所有 release artifacts fresh 的狀態執行
- **THEN** launch 前仍會把 `omb/game.toml` swap 到 stress variant
- **AND** frontend exit 後 restore 原本的 `omb/game.toml`
