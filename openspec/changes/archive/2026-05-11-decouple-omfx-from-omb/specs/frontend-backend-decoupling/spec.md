## ADDED Requirements

### Requirement: `omfx` native build has no `omobab` dependency

native `omfx` SHALL build without a Cargo dependency on `omobab`。`omfx/game/Cargo.toml` SHALL NOT declare `omobab` under any native target dependency section，且 `omfx/game/src/**/*.rs` SHALL NOT import or reference `omobab::*`。需要與 backend 共用的 deterministic gameplay runtime SHALL 來自 `omoba-core::runtime`。

#### Scenario: omfx manifest does not reference omobab
- **WHEN** 搜尋 `D:/omoba/omfx/game/Cargo.toml` 中的 `omobab =`
- **THEN** 找不到任何 dependency declaration
- **AND** `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor` 不需要編譯 `D:/omoba/omb` crate

#### Scenario: omfx source does not import backend crate
- **WHEN** 搜尋 `D:/omoba/omfx/game/src/**/*.rs` 中的 `omobab::`
- **THEN** 找不到任何 source reference
- **AND** native `omfx` code 只透過 `omoba-core`、其他 shared crates 與 wire protocol 取得 lockstep simulation 所需 types

### Requirement: backend startup is launcher-owned, not frontend-owned

`omfx` executable SHALL NOT discover `omb/` repo directories、spawn `omobab.exe`、或從 frontend process 內呼叫 `cargo run` 啟動 backend。需要同機啟動 backend 的 dev、smoke、stress flows SHALL 由 launcher scripts 負責建置、啟動與清理 backend process。直接執行 `executor.exe` SHALL 不因為找不到 `omb/game.toml` 或 `omb/target/*/omobab.exe` 而退出。

#### Scenario: omfx does not spawn backend process
- **WHEN** 搜尋 `D:/omoba/omfx/game/src/**/*.rs` 中的 `target/debug/omobab.exe`、`PathBuf::from("omb")`、`Command::new("cargo")` 與 `spawn_backend`
- **THEN** 找不到 frontend-owned backend spawn path
- **AND** backend process lifecycle code 不存在於 `omfx/game`

#### Scenario: launcher starts backend for dev run
- **WHEN** `run.bat` 啟動一般 native dev session
- **THEN** launcher 在啟動 `omfx/target/debug/executor.exe` 前啟動 matching backend executable
- **AND** launcher 在 frontend 結束後清理它啟動的 backend process

#### Scenario: direct executor can start without repo-local backend
- **WHEN** 在沒有可用 `D:/omoba/omb` runtime path 的環境直接啟動 `executor.exe`
- **THEN** frontend process 仍完成初始化
- **AND** 連線狀態透過 `OMB_KCP_ADDR` 或預設位址處理，而不是嘗試尋找或建置 backend

### Requirement: `omoba-core::runtime` provides the mandatory local lockstep replica boundary

前後端共用的 deterministic simulation replica SHALL 位於 `omoba-core::runtime`。`omoba-core::runtime` SHALL 是 `omoba-core` 的 mandatory public contract，而不是 optional feature。它 SHALL expose world initialization、lockstep tick input application、dispatcher/tick execution、script dispatch integration、outcome processing、render-only cue queues 與 snapshot extraction。`omb` SHALL consume this runtime to run authoritative simulation；native `omfx` SHALL consume the same runtime to run local replica for rendering。

#### Scenario: omb and omfx depend on omoba-core runtime
- **WHEN** 檢查 `D:/omoba/omb/Cargo.toml` 與 `D:/omoba/omfx/game/Cargo.toml`
- **THEN** 兩者都依賴含 mandatory `runtime` module 的 `omoba-core`
- **AND** dependency direction 是 `omb -> omoba-core` 與 `omfx -> omoba-core`
- **AND** 不存在 `omfx -> omb` dependency edge
- **AND** 不新增 `omoba-runtime` crate

#### Scenario: sim runner uses omoba-core runtime entrypoints
- **WHEN** 檢查 native `omfx` sim runner implementation
- **THEN** world initialization、tick execution、outcome processing 與 snapshot extraction 都呼叫 `omoba-core::runtime` entrypoints
- **AND** sim runner 不呼叫 backend app crate 的 `state`、`comp`、`scripting` 或 `ability_runtime` modules

### Requirement: lockstep player input uses one shared protocol type

native `omfx` lockstep client 與 `omoba-core::runtime` SHALL 使用同一個 `PlayerInput` Rust type，來源 SHALL 是 `omoba-core` shared protocol module。`omfx` SHALL NOT 在 lockstep client 與 sim runner 邊界透過 prost encode/decode 轉換兩份不同 crate 產生的 `PlayerInput` type。

#### Scenario: convert_player_input prost roundtrip is removed
- **WHEN** 搜尋 `D:/omoba/omfx/game/src/**/*.rs` 中的 `convert_player_input`、`encoded_len()` 與 `PlayerInput::decode`
- **THEN** 不存在用於 `omoba_core` ↔ backend/runtime duplicate `PlayerInput` 的 prost roundtrip bridge
- **AND** `TickBatchInput.input` type 與 lockstep client event input type 相同或來自同一 shared protocol module

### Requirement: decoupling preserves player-visible behavior

解耦 SHALL NOT 改變 lockstep cadence、TD tower placement/sell/upgrade behavior、hero ability UI、snapshot-driven rendering、VFX cue rendering 或 smoke/stress launcher scenario 的玩家可見結果。Dependency boundary 改變後，既有 deterministic tests、backend lib tests 與 native frontend build SHALL 維持通過。

#### Scenario: existing verification commands pass
- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`
- **THEN** backend library tests pass
- **AND** 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features` 時 determinism tests pass
- **AND** 執行 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor` 時 native frontend builds successfully without `omobab`

#### Scenario: stress launcher no longer depends on frontend hard-coded backend path
- **WHEN** `run_stress.bat` 使用 release backend 與 release frontend 執行
- **THEN** launcher 直接啟動 release `omobab.exe`
- **AND** 不需要把 release backend 複製到 `omb/target/debug/omobab.exe` 給 frontend spawn path 使用
