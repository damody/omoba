## MODIFIED Requirements

### Requirement: backend startup is launcher-owned, not frontend-owned

`omfx` executable SHALL NOT discover `omb/` repo directories、spawn `omobab.exe`、或從 frontend process 內呼叫 `cargo run` 啟動 backend。需要同機啟動 backend 的 dev、smoke、stress 與雙隊 fog demo flows SHALL 由 launcher scripts 負責建置、啟動與清理 backend process。直接執行 `executor.exe` SHALL 不因為找不到 `omb/game.toml` 或 `omb/target/*/omobab.exe` 而退出。雙隊 fog demo 的兩個 frontend MUST 共用 launcher 建立的唯一 backend，frontend 間不得互相擁有 lifecycle。

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

#### Scenario: 雙隊 demo 共用唯一 backend
- **WHEN** `run_2player.bat` 啟動兩個獨立 executor
- **THEN** 兩個 frontend 都連線到 launcher 建立的同一個 omb PID
- **AND** 任一 frontend 都不建立、重啟或停止 backend

