## MODIFIED Requirements

### Requirement: backend startup is launcher-owned, not frontend-owned

`omfx` executable SHALL NOT discover `omb/` repo source directories, import `omobab::*`, or call `cargo run` / `cargo build` from the frontend process to build or run backend code. Native game sessions that need a same-machine backend SHALL use the existing controlled session launcher to start an already-built backend executable, and SHALL manage that process with one game session as the lifecycle boundary. Restoring pregame UI SHALL NOT reintroduce menu-time backend autostart or older launcher-owned assumptions that conflict with the current session-scoped backend flow. Directly running `executor.exe` SHALL first show pregame UI and SHALL not exit merely because no backend is currently running.

#### Scenario: omfx does not depend on backend crate or cargo
- **WHEN** searching `D:/omoba/omfx/game/Cargo.toml` and `D:/omoba/omfx/game/src/**/*.rs`
- **THEN** no `omobab =` dependency declaration exists
- **AND** no `omobab::` source reference exists
- **AND** no backend startup path invokes `cargo run` or `cargo build` from `omfx`

#### Scenario: menu startup does not require backend
- **WHEN** native `executor.exe` starts without an active backend process
- **THEN** frontend initialization completes and the pregame UI is visible
- **AND** `omfx` does not connect to backend before the player selects map and difficulty
- **AND** `omfx` does not exit merely because `omb/game.toml`, `omb/target/*/omobab.exe`, or a repo-local backend source path is unavailable during menu idle

#### Scenario: selected session starts backend executable
- **WHEN** the player selects map and difficulty in the pregame flow
- **THEN** `omfx` session launcher starts an already-built backend executable
- **AND** session config includes the selected story/runtime identifier, difficulty, network address, and session id
- **AND** backend readiness is established before the gameplay lockstep connection and local sim runner become active

#### Scenario: session teardown closes frontend-owned backend
- **WHEN** gameplay ends, the player returns to menu, session startup fails and recovers, or plugin deinitializes
- **THEN** `omfx` closes the backend process started by that session launcher
- **AND** repeated shutdown of the same session is idempotent and does not panic
- **AND** the next game starts a new session lifecycle

#### Scenario: external backend mode remains possible for tools
- **WHEN** developer config explicitly selects an external backend address or disables session launcher ownership
- **THEN** `omfx` can connect to an already-running external backend
- **AND** `omfx` SHALL NOT close a backend process that it did not start
- **AND** menu idle still does not automatically start gameplay runtime
