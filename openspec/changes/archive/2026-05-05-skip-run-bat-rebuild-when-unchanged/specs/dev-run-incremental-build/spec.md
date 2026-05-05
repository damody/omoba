## ADDED Requirements

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

每個 `run*.bat` launcher SHALL 在該 artifact 的任何 configured relevant input path 比 artifact 更新時，將 artifact 視為 stale。Relevant inputs MUST 包含 Rust source files、Cargo manifests、Cargo lockfiles、`rust-toolchain.toml`、build scripts、generated-data inputs、shared path dependency sources，以及該 artifact 使用的 protocol files。

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
