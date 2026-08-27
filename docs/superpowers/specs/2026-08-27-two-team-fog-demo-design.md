# 雙隊戰爭迷霧實機展示設計

日期：2026-08-27  
狀態：已核准，待書面複核

## 目的

建立可由開發者直接執行的 secure selective lockstep 展示場景，使用一個 omb server 與兩個獨立 omfx process 模擬分屬不同隊伍的玩家。兩個視窗必須同時呈現相同 authoritative match，但只收到各自 team 圓形視野允許揭露的單位，讓開發者可肉眼確認戰爭迷霧、reveal、hide、LastKnown 與 server observer replica 是否正常。

此展示必須驗證真正的 server-side information boundary；不得把完整世界送至 client 後僅在 renderer 隱藏。

## 使用者入口

沿用並修正根目錄 `run_2player.bat`，不新增根目錄批次檔。執行後依序：

1. 以既有 freshness 機制確認 script DLL、omb 與 omfx executor。
2. 啟動一個背景 omb process，載入 `FOG_2TEAM_DEMO`。
3. 啟動兩個獨立 omfx executor process。
4. 將 Player 1 綁定 Team 1、Player 2 綁定 Team 2。
5. 將 P1 視窗放在螢幕左側、P2 視窗放在右側，標題清楚標示 player/team。
6. 等待兩個前端結束；兩者都結束後停止本次啟動的 server。若 server 提前失敗，關閉本次啟動的兩個前端並回傳失敗。

啟動流程不得依賴目前不存在的 `run_2player_client.bat`；每個 client 的環境變數與完成狀態由保留腳本或既有 PowerShell helper 明確管理。

## 地圖與單位配置

新增獨立 Lua content package `scripts/lua_data/FOG_2TEAM_DEMO/`，不修改既有 TD 或 MVP 關卡語意。

場景包含 102 個 gameplay 單位：

- 10×10 方格中的 100 個一般單位，固定間距 220 world units。
- 另外兩個可控制玩家英雄，不計入上述 100 個一般單位。
- 100 個一般單位固定分配為 Team 1 共 33 個、Team 2 共 33 個、Neutral 共 34 個。
- 單位以 deterministic 規律交錯排列，不使用 runtime random placement。
- Player 1／Team 1 英雄位於方格左下側，Player 2／Team 2 英雄位於方格右上側。
- 16 個一般單位依固定短路徑巡邏並反覆穿越視野邊界；其餘一般單位保持靜止。
- 地圖約 3200×1800 world units，方格外保留英雄移動空間。

單位必須使用既有可渲染 content definition；展示資料不得依賴不存在的 asset。

## Visibility authority

每個玩家英雄同時是其 team 的 `VisionSource`，使用半徑 700 world units 的純圓形視野。第一版展示不加入地形遮蔽或扇形視野，以免人工驗收時無法區分距離與遮蔽物造成的結果。

Visibility 由 server 的 deterministic team projection 計算：

- 同隊 visibility source 合併為 team-shared view。
- Player viewport、camera 與 renderer 狀態不得參與 gameplay visibility authority。
- 一般單位進入圓形視野時，依既有 scheduled reveal commitment 在 effective tick 擷取 fresh baseline，然後從當下 replica tick 繼續同步。
- 單位離開視野後不再傳送 gameplay state；若使用 `LastKnown`，只留下已去敏感化的 render cache record。
- Team 1 與 Team 2 使用獨立 opaque replica identity；client 不得收到 raw ECS ID、canonical ID、global seed 或另一隊 visibility mask。
- Neutral 單位不代表全域公開；仍需位於該隊視野內才可揭露。

## 前端呈現

兩個 omfx process 使用相同 `SelectiveReplicaRuntime`，但各自只消費所屬 team stream。

每個視窗顯示：

- Team 1 使用藍色標示；Team 2 使用紅色標示。
- 以半透明 fog overlay 表示目前不可見區域。
- 以對應 team 顏色畫出英雄的 700-unit 圓形視野邊界。
- 可見單位使用正常 renderer state。
- `LastKnown` 單位位於獨立 remembered render cache，以低透明度 ghost 呈現，不得進入 targeting、collision、input lookup 或 team hash。
- HUD 顯示 `Demo grid units: 100`、`Player heroes: 2`、目前 team/player、replica tick、`Currently disclosed` 與 `Remembered ghosts`。

`Demo grid units: 100` 與 `Player heroes: 2` 是固定場景說明，不得藉由 client 讀取 authoritative full-world entity count 即時計算。

英雄沿用既有點擊移動輸入。英雄移動時 server 更新其 vision source，讓開發者主動穿越 10×10 方格並比較兩個視窗。

## Server observer replica

omb 在同 process 的另一個 validation thread 為每個 active team 維護一份 observer replica：

- Outbound encoded bytes 先直接進 session send queue。
- 同一份 `Arc<[u8]>` 再以非阻塞 tap 送到該 team observer。
- Observer 只讀該 team 實際 wire stream 與 filtered bootstrap，不得讀 authoritative Specs world 或其他 team state。
- Observer mismatch、lag、coverage gap 與 rebootstrap 狀態寫入 server-only diagnostics；不得阻塞玩家送包。

## 視覺驗收流程

啟動後應能依下列步驟判斷成功：

1. 兩個視窗標示不同 player/team，且各自可控制自己的英雄。
2. 初始畫面中兩隊 disclosed entity count 與單位集合不同。
3. 移動 P1 英雄接近方格中央，P1 視窗出現新單位；P2 不應因 P1 camera 或 viewport 改變而看到相同單位。
4. 讓巡邏單位進入與離開圓形邊界，可觀察 reveal 與 LastKnown ghost。
5. 同一單位同時進入兩隊重疊視野時，兩邊顯示相同 public state。
6. 單位離開其中一隊視野後，該隊不再取得其 gameplay 更新；另一隊若仍可見則繼續同步。
7. Server diagnostics 顯示 Team 1 與 Team 2 observer 各自持續驗算，沒有 mismatch 或未處理 coverage gap。

## 錯誤處理

- 缺少 DLL、binary、Lua package 或地圖資料時，批次檔必須清楚指出缺少項目並以非零狀態結束。
- Player/team binding 不存在、重複登入、V2 negotiation 失敗或 runtime downgrade attempt 時，server 必須 fail closed。
- 任一 client 無法連線時，不得切換到 legacy global snapshot；批次檔需保留 per-player log suffix 供診斷。
- 只終止由本次 launcher 建立且 PID 已確認的 server；不得以廣域 process kill 作正常清理流程。

## 測試策略

實作期間只跑最低限度 compile 與 focused tests；完整展示驗證集中於最後：

- Lua map schema/load test：精確建立 100 個 grid units 與額外 2 個英雄，team 數量為 33／33／34。
- Determinism test：placement、patrol path 與 spawn ordering 在重跑後一致。
- Visibility test：半徑內 reveal、半徑外 hide、team isolation、viewport non-authority。
- Protocol test：兩個 player 綁定不同 team，只取得各隊 filtered bootstrap/frame。
- Client test：fog overlay、circle、disclosed count 與 remembered count 只依 filtered replica/render cache。
- Launcher smoke：一個 server 與兩個獨立 executor process 使用不同 player/team/session/log identity。
- 最終人工驗收：實際執行 `run_2player.bat` 並保留兩視窗供使用者查看。

## 非目標

- 不在此展示加入地形遮蔽、視錐、草叢或高度差。
- 不把 100 個單位改成壓力／效能 benchmark；既有 10,000-entity gate 仍是效能依據。
- 不新增 matchmaking、登入 UI 或可在 active match 中切換 team 的功能。
- 不修改 secure V2 的已核准 disclosure、correction、observer 或 rollback contract。

## 完成門檻

- `run_2player.bat` 可重複啟動一個 server 與兩個獨立 omfx process。
- 場景包含精確 100 個 grid units 加 2 個英雄。
- 兩隊 client 初始與移動後所見集合符合各自 700-unit 圓形 team view。
- Hidden entity 不存在於非授權 client replica、memory-facing snapshot 或 player diagnostics。
- Reveal 後從 server 指定的即時 replica tick 繼續同步；server correction 永遠勝過 client local state。
- 每隊 observer replica 驗算實際送出 bytes，且不位於 outbound critical path。
- 自動測試通過並完成雙視窗人工驗收。
