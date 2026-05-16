## ADDED Requirements

### Requirement: frontend package owns client-only game.toml

Native `omfx` SHALL include `omfx/game.toml` for separated frontend packaging。此檔案 SHALL 只包含 frontend package 需要的 client-owned 設定，例如 connection endpoint、player/display preferences、local asset paths 或 frontend render preferences。它 SHALL NOT 包含 `STEP_FPS` 這類 server-authoritative simulation settings；client simulation cadence SHALL 來自 server lockstep metadata。

#### Scenario: omfx package has local game.toml
- **WHEN** checking the native frontend package layout
- **THEN** `D:/omoba/omfx/game.toml` exists
- **AND** native `omfx` can load frontend-local defaults from that file when launched separately from backend

#### Scenario: frontend config does not duplicate server step FPS
- **WHEN** inspecting `D:/omoba/omfx/game.toml`
- **THEN** it does not define `STEP_FPS` or an equivalent server authoritative simulation step FPS key
- **AND** changing server step FPS is done in `D:/omoba/omb/game.toml`, not in frontend config

#### Scenario: client obeys server cadence
- **WHEN** server starts with `omb/game.toml [server] STEP_FPS = 90` and omfx connects with its own `omfx/game.toml`
- **THEN** omfx sim_runner uses the server-declared 90 FPS cadence for local replica dt and wait deadlines
- **AND** omfx does not require a matching local FPS setting in `omfx/game.toml`

#### Scenario: FPS settings are not environment-variable driven
- **WHEN** inspecting server and frontend FPS configuration paths
- **THEN** server step FPS is read from `omb/game.toml`
- **AND** frontend does not introduce `OMFX_*_FPS` or `OMB_*_FPS` environment variables as the primary configuration mechanism for this change
