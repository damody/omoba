# 戰爭迷霧示範英雄轉向速度與本機雙玩家啟動設計

## 目的

讓 `FOG_2TEAM_DEMO` 內的兩名玩家英雄在收到移動指令後幾乎立即完成轉向，排除目前每秒 3 弧度造成的最長約 0.87 秒起步等待。

## 範圍

- 將戰爭迷霧示範場景建立英雄時使用的 `TurnSpeed` 從每秒 3 弧度改為每秒 300 弧度。
- Team 1 與 Team 2 使用相同設定。
- 不修改一般英雄模板、其他故事或其他遊戲模式。
- 不修改英雄移速、30 度移動門檻、尋路或碰撞演算法。

## 行為

Server authoritative world 與兩隊 replica 都從相同場景初始化資料取得每秒 300 弧度的確定性轉向速度。以 120 Hz 執行時，每 tick 最大可旋轉 2.5 弧度；最慢在兩個 tick 內即可進入 30 度移動門檻。

## 驗證

最後統一執行：

1. 單元測試確認示範英雄的 `TurnSpeed` 是每秒 300 弧度。
2. 建置 release server。
3. 重新啟動一個 server、兩個 team runtime 與兩個 renderer，供實際右鍵操作確認。

## 本機雙玩家啟動器

根目錄 `run_2player.bat` 維持薄 wrapper，只呼叫固定 Lua 執行環境執行 `scripts/run_2player.lua`。Lua 啟動器負責完整程序生命週期：

1. 啟動前只清理由本機雙玩家啟動器管理的既有 `omobab.exe`、兩個 `omoba-client-runtime.exe` 與兩個 `executor.exe`，避免連接舊 server 或占用舊連接埠。
2. 使用 release 產物啟動一個 authoritative server、Team 1／Team 2 各一個獨立 runtime，以及兩個 renderer client。
3. launcher 持續監看兩個 renderer；只關閉其中一個時遊戲仍繼續，兩個 renderer 都關閉後才依序結束兩個 runtime 與 authoritative server。
4. launcher 被中斷或啟動失敗時，也必須透過 cleanup stack 回收本次啟動的所有子程序。
5. PID 檔需記錄精確程序，清理前確認 executable 路徑，避免終止不屬於這個啟動器的同名程序。
