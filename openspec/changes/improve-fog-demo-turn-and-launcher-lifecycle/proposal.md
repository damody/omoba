## Why

目前 `FOG_2TEAM_DEMO` 英雄每秒只旋轉 3 弧度，反向移動前可能等待近一秒；本機雙玩家啟動器也沒有完整管理舊程序與結束條件，容易連到殘留程序或在視窗關閉後留下 server。需要讓示範操作更接近即時 MOBA 手感，並讓一次按下 `run_2player.bat` 就能可靠取得乾淨、可自動回收的五程序測試環境。

## What Changes

- 將 `FOG_2TEAM_DEMO` 兩名英雄的轉向速度設為每秒 300 弧度，不影響其他場景與英雄模板。
- 將 `run_2player.bat` 維持為薄 wrapper，所有互動式啟動與程序管理邏輯放在 `scripts/run_2player_interactive.lua`。
- 啟動前依受管理 PID 與 executable 路徑關閉舊的雙玩家 session，再啟動一個 authoritative server、兩個 team runtime 與兩個 renderer client。
- 只關閉一個 client 時維持後端運作；兩個 client 都關閉後，自動回收兩個 runtime 與 server。
- 啟動失敗或 launcher 被中斷時，回收本次建立的所有子程序。
- 將完整測試與實際生命週期驗證集中於實作工作最後。

## Capabilities

### New Capabilities

- `fog-demo-local-session`: 規範戰爭迷霧示範英雄的快速轉向，以及本機雙玩家五程序 session 的啟動、舊程序清理與自動結束行為。

### Modified Capabilities

無。

## Impact

- `omoba-core/src/runtime/native/initialization.rs`：示範英雄的 `TurnSpeed` 初始化。
- `scripts/run_2player_interactive.lua` 與既有 Lua process helper：本機 session 的程序生命週期。
- `run_2player.bat`：僅保留 Lua wrapper 契約。
- 測試會涵蓋場景數值、重啟清理、單一 client 關閉與雙 client 關閉後的完整回收。
