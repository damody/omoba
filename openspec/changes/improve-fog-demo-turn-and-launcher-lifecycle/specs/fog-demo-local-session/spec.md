## ADDED Requirements

### Requirement: 示範英雄快速轉向
系統 SHALL 讓 `FOG_2TEAM_DEMO` 的 Team 1 與 Team 2 玩家英雄使用每秒 300 弧度的確定性轉向速度，且不得改變其他場景的英雄轉向設定。

#### Scenario: 反向移動迅速跨過門檻
- **WHEN** 示範英雄以 120 Hz 從目前朝向轉向正後方的移動目標
- **THEN** 英雄 SHALL 在最多兩個 authoritative tick 內進入 30 度移動門檻

### Requirement: 本機五程序 session
`run_2player.bat` SHALL 透過固定 Lua runtime 啟動一個 authoritative server、兩個 team runtime 與兩個 renderer client，且 `.bat` 本身不得包含建置或程序管理邏輯。

#### Scenario: 從乾淨狀態啟動
- **WHEN** 使用者執行 `run_2player.bat` 且沒有受管理的既有 session
- **THEN** launcher SHALL 啟動五個 release 程序並讓兩個 renderer 分別連接 Team 1 與 Team 2 runtime

### Requirement: 重啟前安全清理舊 session
launcher SHALL 在啟動新 session 前讀取受管理 PID 狀態，且只有在 PID 仍存活並且 executable 絕對路徑相符時才能停止舊程序。

#### Scenario: 再次按下 BAT
- **WHEN** 前一個受管理 session 仍在執行時再次執行 `run_2player.bat`
- **THEN** launcher SHALL 回收舊 session 的五個程序後再建立新的五個程序

#### Scenario: PID 已被重用
- **WHEN** 狀態檔中的 PID 指向不同 executable
- **THEN** launcher MUST NOT 停止該程序

### Requirement: 依兩個 client 結束 session
launcher SHALL 在至少一個 renderer client 存活時維持後端，並在兩個 renderer 都結束後自動回收兩個 team runtime 與 authoritative server。

#### Scenario: 只關閉一個 client
- **WHEN** 使用者只關閉 Team 1 或 Team 2 renderer
- **THEN** launcher SHALL 保持另一個 renderer、兩個 runtime 與 authoritative server 運作

#### Scenario: 兩個 client 都關閉
- **WHEN** 兩個 renderer 都已結束
- **THEN** launcher SHALL 自動停止兩個 runtime 與 authoritative server，且不得留下本 session 的受管理子程序

#### Scenario: 啟動失敗或 launcher 中斷
- **WHEN** 任一程序啟動失敗或 launcher 正常接收到中斷並進入 cleanup
- **THEN** launcher SHALL 回收本次已建立的所有子程序
