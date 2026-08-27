## ADDED Requirements

### Requirement: Launcher 啟動一個 server 與兩個獨立 frontend

`run_2player.bat` SHALL 啟動一個載入 `FOG_2TEAM_DEMO` 的 omb process，以及兩個不同 PID 的 omfx executor process。Player 1 SHALL 綁定 Team 1，Player 2 SHALL 綁定 Team 2；每個 frontend SHALL 使用不同 player name、lockstep name、window title 與 log suffix。

#### Scenario: Process topology 正確
- **WHEN** 執行 `run_2player.bat`
- **THEN** launcher 建立一個 omb PID 與兩個互不相同的 executor PID
- **AND** 兩個 client negotiate secure V2 並取得不同 team binding

#### Scenario: 兩個視窗可辨識
- **WHEN** 兩個 executor 完成視窗初始化
- **THEN** P1／Team 1 視窗位於 primary work area 左側且使用藍隊標示
- **AND** P2／Team 2 視窗位於右側且使用紅隊標示
- **AND** 無法定位視窗時只記錄警告，不終止 secure match

### Requirement: Launcher lifecycle 只管理自己建立的 process

Launcher SHALL 保存本次建立的 server 與 frontend PID。正常結束與錯誤清理 SHALL 只針對已驗證 PID，不得以 image-wide kill 作正常 lifecycle；若 server 提前退出，launcher SHALL 關閉本次兩個 frontend 並回傳非零狀態。

#### Scenario: 兩個 frontend 結束後清理 server
- **WHEN** P1 與 P2 frontend 都結束
- **THEN** launcher 停止本次 server PID並回傳 client 結果

#### Scenario: Server 提前失敗
- **WHEN** server 在兩個 frontend 尚未結束前退出
- **THEN** launcher 停止本次持有的 frontend PID並回傳非零狀態

#### Scenario: 缺少 artifact
- **WHEN** freshness/build 後仍缺少 DLL、server binary、frontend binary 或 demo Lua package
- **THEN** launcher 顯示精確缺少項目且不啟動部分 topology

