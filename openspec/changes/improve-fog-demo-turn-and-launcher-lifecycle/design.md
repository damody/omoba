## Context

`run_2player.bat` 現在呼叫 `scripts/run_2player_interactive.lua`，後者已能以 release 產物啟動一個 `omobab`、兩個 `omoba-client-runtime` 與兩個 `executor`，但只依固定秒數存活，沒有跨次啟動的受管理 PID 狀態，也不會以兩個 renderer 是否都關閉作為 session 結束條件。`FOG_2TEAM_DEMO` 則在 Rust 場景初始化中把兩名英雄的 `TurnSpeed` 固定為每秒 3 弧度。

## Goals / Non-Goals

**Goals:**

- 只讓 `FOG_2TEAM_DEMO` 英雄使用每秒 300 弧度的轉向速度。
- 一次執行 `run_2player.bat` 即清理上一個受管理 session，並啟動五個 release 程序。
- 兩個 renderer 都關閉後，自動結束兩個 runtime 與 authoritative server。
- 任何停止操作都先核對 PID 對應的 executable 絕對路徑。

**Non-Goals:**

- 不修改全域英雄模板、移速、尋路、碰撞或戰爭迷霧規則。
- 不以程序名稱掃描並關閉所有同名程序。
- 不改變自動化 evidence runner `scripts/run_2player.lua` 的既有測試模式。
- 不在 `.bat` 放入建置或程序管理邏輯。

## Decisions

### 使用互動式 Lua launcher 作為 BAT 入口

保留 `run_2player.bat` 呼叫 `scripts/run_2player_interactive.lua`。互動式啟動與 evidence runner 的責任不同，直接擴充前者可避免破壞既有 headless、netem 與故障注入流程。替代方案是重寫 `scripts/run_2player.lua` 的預設模式，但會讓測試 runner 與玩家入口再次耦合。

### 使用 active-session JSON 保存精確程序身分

launcher 在 `target/interactive-runs/active-session.json` 記錄 session ID、角色、PID 與 executable 絕對路徑。新啟動只會停止檔案中仍存活且 executable 完全相符的程序；PID 已被重用且路徑不同時不得停止。替代方案是依程序名稱全域清理，但可能關閉使用者另外啟動的遊戲或測試。

### 以兩個 renderer 的聯集生命週期控制後端

launcher 每 100 ms 檢查兩個 renderer。任一 renderer 存活時，server 與 runtime 保持執行；兩者都結束後執行 cleanup stack。若後端在 renderer 尚存活時意外退出，launcher 視為失敗並回收其餘程序，避免留下無法操作的視窗。

### 場景專用轉向數值

在 `create_fog_demo_scene` 建立英雄時寫入 `TurnSpeed(Fixed64::from_i32(300))`。這讓 authoritative world 與 team replica 使用同一份確定性狀態，同時不影響正式內容模板。

## Risks / Trade-offs

- [launcher 被強制終止而來不及 cleanup] → active-session JSON 讓下一次啟動可以安全回收殘留程序。
- [PID 被作業系統重用] → 停止前必須以 Lua host 查詢並比對 executable 絕對路徑。
- [300 rad/s 在 120 Hz 仍不是數學上的瞬間轉向] → 每 tick 可轉 2.5 rad，最慢兩 tick 內跨過移動門檻，符合本次明確要求。
- [只有一個 client 關閉時 session 持續佔用資源] → 這是需求指定行為；第二個 client 關閉後立即統一回收。
