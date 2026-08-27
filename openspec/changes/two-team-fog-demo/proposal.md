## Why

Selective lockstep 已具備 team-scoped information boundary，但目前缺少可由開發者直接啟動、以兩個獨立 omfx 視窗肉眼比較的固定展示場景。新增 deterministic 雙隊地圖與 launcher，可驗證不同 team 確實收到不同單位集合，而不是只在 renderer 隱藏完整世界。

## What Changes

- 新增 `FOG_2TEAM_DEMO` Lua content package，建立 10×10 的 100 個一般單位與額外 2 個玩家英雄。
- 將 Player 1／Team 1 與 Player 2／Team 2 綁定到兩個獨立 omfx process，各自使用英雄中心、半徑 700 的 team-shared 圓形視野。
- 加入固定交錯 team 配置與 16 個 deterministic 巡邏單位，持續產生可觀察的 reveal、hide 與 LastKnown transition。
- 在 omfx 顯示 team 色彩、fog overlay、視野圓、disclosed/remembered 計數與 demo 固定場景資訊。
- 修正 `run_2player.bat`，不再依賴不存在的 helper，並以一個 server 加兩個獨立 frontend process 執行展示。
- 保留每隊同 process observer replica，驗算實際送出的 team wire bytes。
- 新增 map、projection、client presentation、launcher 與雙 session 的 focused/最終驗收證據。

## Capabilities

### New Capabilities

- `two-team-fog-demo`: 規範 100 個 grid units、2 個玩家英雄、圓形 team visibility、巡邏 transition、fog／LastKnown 呈現與人工驗收結果。
- `two-process-demo-launcher`: 規範一個 omb server、兩個獨立 omfx process、固定 player/team identity、左右視窗與 PID-scoped lifecycle。

### Modified Capabilities

- `frontend-backend-decoupling`: 明確要求雙玩家展示仍由 launcher 擁有唯一 backend lifecycle，兩個 frontend 不得各自建立 backend。
- `sim-snapshot-rendering`: 增加 secure filtered snapshot 上的 fog overlay、vision boundary、disclosed count 與獨立 LastKnown render cache 呈現契約。

## Impact

- Content：`scripts/lua_data/FOG_2TEAM_DEMO/` 與必要的 Lua/import schema。
- Backend：omb demo scene initialization、team ownership/vision source、deterministic patrol 與 existing selective projection integration。
- Shared runtime/protocol：只在現有 V2 contract 足以承載時重用；若缺少 render-safe demo metadata，僅新增 filtered/team-safe 欄位。
- Frontend：omfx fog demo presentation、HUD、視野圓、window identity 與位置參數。
- Launcher：修改根目錄既有 `run_2player.bat`，維持 CRLF，不新增根目錄 `.bat`。
- Compatibility：既有 TD/MVP story、一般 `run.bat` 與 secure V2 disclosure contract 不變；展示是 opt-in 開發入口。
