## 1. 示範英雄轉向

- [x] 1.1 在 `create_fog_demo_scene` 將兩名玩家英雄的 `TurnSpeed` 從每秒 3 弧度改為每秒 300 弧度
- [x] 1.2 新增小型單元測試，直接確認 `FOG_2TEAM_DEMO` 英雄初始化後的 `TurnSpeed` 是 300

## 2. 受管理 session 狀態

- [x] 2.1 在 `scripts/run_2player_interactive.lua` 定義固定的 active-session JSON 路徑
- [x] 2.2 在 active-session JSON 記錄 session ID、角色、PID 與 executable 絕對路徑
- [x] 2.3 實作讀取舊 active-session JSON 的函式
- [x] 2.4 實作逐筆檢查舊 PID 是否仍存活的函式
- [x] 2.5 實作 executable 路徑相符時才停止舊程序的函式
- [x] 2.6 啟動新程序前呼叫舊 session 清理函式
- [x] 2.7 cleanup 時只在 active-session JSON 仍屬於本次 session 時移除檔案

## 3. 五程序啟動

- [x] 3.1 保留一個 release `omobab.exe` authoritative server 的啟動流程
- [x] 3.2 保留 Team 1 與 Team 2 各一個 release `omoba-client-runtime.exe` 的啟動流程
- [x] 3.3 保留 Team 1 與 Team 2 各一個 release `executor.exe` renderer 的啟動流程
- [x] 3.4 五個程序都通過啟動檢查後寫入 active-session JSON
- [x] 3.5 確認 `run_2player.bat` 仍只呼叫固定 Lua runtime、轉送參數並回傳 exit code

## 4. Client 關閉生命週期

- [x] 4.1 實作每 100 ms 檢查兩個 renderer 存活狀態的監看迴圈
- [x] 4.2 只關閉一個 renderer 時繼續等待另一個 renderer
- [x] 4.3 兩個 renderer 都關閉時離開監看迴圈並執行 cleanup stack
- [x] 4.4 renderer 尚存活但 server 或任一 runtime 意外結束時回報錯誤並 cleanup
- [x] 4.5 launcher 失敗或中斷進入既有 cleanup 路徑時，回收本次已啟動的所有子程序

## 5. 最後統一測試與檢查

- [x] 5.1 執行轉向速度相關 Rust 單元測試
- [x] 5.2 執行 Lua launcher 與 PID executable 身分保護測試
- [x] 5.3 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab`
- [x] 5.4 執行 `cargo test --manifest-path omoba-client-runtime/Cargo.toml`
- [x] 5.5 執行 `cargo test --manifest-path omfx/game/Cargo.toml --features runtime-lua-content`
- [x] 5.6 建置 server、runtime 與 renderer 的 release 產物
- [x] 5.7 啟動第一個 session，確認五個 PID 都存活且 executable 身分正確
- [x] 5.8 在第一個 session 存活時再次啟動，確認舊五程序結束且新五程序啟動
- [x] 5.9 關閉新 session 的一個 renderer，確認另一個 renderer 與三個後端仍存活
- [x] 5.10 關閉第二個 renderer，確認兩個 runtime 與 authoritative server 自動結束
- [x] 5.11 執行 `git diff --check` 並確認沒有修改使用者不相關的檔案
