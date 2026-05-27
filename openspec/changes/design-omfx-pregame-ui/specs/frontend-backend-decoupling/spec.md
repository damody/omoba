## MODIFIED Requirements

### Requirement: backend startup is launcher-owned, not frontend-owned

`omfx` executable SHALL NOT discover `omb/` repo source directories、import `omobab::*`、或從 frontend process 內呼叫 `cargo run`/`cargo build` 建置 backend。需要同機啟動 backend 的 native game sessions SHALL 由 `omfx` 透過受控 session launcher 啟動已建置好的 backend executable，並以每場遊戲為 lifecycle 邊界管理該 process。直接執行 `executor.exe` SHALL 先顯示 pregame menu，且 SHALL 不因為 idle menu 階段找不到或連不上 backend 而退出。

#### Scenario: omfx does not depend on backend crate or cargo

- **WHEN** 搜尋 `D:/omoba/omfx/game/Cargo.toml` 與 `D:/omoba/omfx/game/src/**/*.rs`
- **THEN** 找不到 `omobab =` dependency declaration
- **AND** 找不到 `omobab::` source reference
- **AND** 找不到用來從 `omfx` 啟動 `cargo run` 或 `cargo build` 的 backend startup path

#### Scenario: menu startup does not require backend

- **WHEN** 在沒有 active backend process 的環境直接啟動 `executor.exe`
- **THEN** frontend process 完成初始化並顯示 pregame menu
- **AND** `omfx` 不會在玩家選擇地圖與難度前嘗試連線 backend
- **AND** `omfx` 不會因為找不到 `omb/game.toml` 或 repo-local backend source path 而退出

#### Scenario: selected session starts backend executable

- **WHEN** 玩家在 pregame flow 中選擇地圖與難度並確認進入遊戲
- **THEN** `omfx` session launcher 啟動已建置好的 backend executable
- **AND** 傳入與該 session 對應的 story/runtime、difficulty、network address 與 session id config
- **AND** backend ready 後 `omfx` 才啟動 lockstep connection 與 local sim runner

#### Scenario: session teardown closes frontend-owned backend

- **WHEN** active gameplay session 結束、玩家返回 menu、session 啟動失敗需要回復，或 plugin deinit
- **THEN** `omfx` 關閉由該 session launcher 啟動的 backend process
- **AND** 同一個 backend child handle 不會被重複 kill 造成 panic
- **AND** 下一場遊戲必須建立新的 session lifecycle

#### Scenario: external backend mode remains possible for tools

- **WHEN** 開發者以明確 config 指定使用外部 backend address 或停用 session launcher
- **THEN** `omfx` 可以連線到外部已存在 backend
- **AND** `omfx` SHALL NOT 在該模式結束時關閉不是自己啟動的 backend process
- **AND** menu idle 仍不會自動連線或啟動 gameplay runtime
