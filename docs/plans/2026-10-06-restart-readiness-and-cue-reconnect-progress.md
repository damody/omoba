# restart 專案綁定與六類事件 TCP 重連

## 決定與實作

- `omfue/restart` 的 `wait-mcp` 不再掃預設 HTTP 埠。固定 Lua `scripts/wait_ue_mcp.lua` 共用 `ue_mcp_endpoint`：完整專案路徑、唯一 live Editor、registry 生命周期與 OS listener PID 都必須相符；HTTP health 後再驗身分。沒有本專案 Editor 就失敗，不接受其他專案的服務。
- `scripts/ue_mcp_readiness.lua` 提供可注入依賴的有界等待與嚴格 health decoder。Rust 只做固定 Lua 呼叫、結構化結果、獨占暫存目錄與 watchdog；不新增工作流 fallback。
- `auto-resume` 同樣先驗專案，再核對設定的 legacy TCP 埠由同一 Editor PID 持有，才送命令。不嘗試埠範圍；保留舊 `--mcp-port-attempts` 參數但明示已停用掃埠。沒有實際送出 auto-resume 命令。
- 後續send邊界補強：已驗明的是IPv4 listener，命令連線固定127.0.0.1；不讓localhost解析至另一個未驗明IPv6程序。僅私有send參數移除與編譯確認，沒有送任何MCP命令。
- 新 Rust 測試共用同一份 CueRetention、watch/critical 通道及 renderer 連線狀態，實際建立兩段 TCP 連線。六種 typed cue 同時存在且 ID ordinal 各異；初始舊 cue、已 ACK cue、離線累積 cue 不能重播，兩次連線的 fresh 六種 cue 正常送達且 ACK 後退休。不更動 production 准入或去重規則。

## 當前功能確認

- 固定 Lua readiness：10 個 endpoint/readiness、2 個 legacy owner、7 個 health decode 案例通過，共 19；共用 endpoint 的既有 12 案例仍通過。health 前後不只比 PID/port，也比 executable 與 process birth，拒絕同 PID 被重用；legacy owner 同樣核對 process lifetime。
- restart Rust 單元測試 9/9 通過；實際從 omfue 工作目錄執行 `wait-mcp --project D:/code/omoba/omfue/om.uproject --mcp-timeout-seconds 1 --output json`，本專案無 live Editor 時回傳 `success=false`、結構化 `exit_code=6` 與 found 0／never fallback 診斷（程序非零 exit 1）。沒有啟動 Editor 或請求其他專案。
- `all_retained_cue_kinds_survive_real_tcp_reconnect_without_history_replay`：1 passed、0 failed；既有 `retained_cues_survive_watch_overwrite_and_ack_filter_already_prepared_frames`：1 passed、0 failed。Cargo 另顯示 binary filter 為 0 tests，不計成功案例。既有 td_rounds 三項 dead-code 警告仍存在。
- 本輪未執行模擬場次、全套 Unreal 或完整遊戲驗收；未 commit/push、未修改 omfx 或共享引擎。

## 剩餘界線

OpenSpec 24/31 不因局部測試自動勾整項。4.3 尚須整合核對全部正式效果/audio 的投影及 native 呈現證據；6.2 尚須正式選角到自然結算；4.4/6.4 的兩台 LAN 缺第二台實機；6.5 client max 仍超固定 50ms，不能放寬門檻或挑樣本。最後才完整驗收，未來每次場次模擬最多 10 場、不得拆批規避。
