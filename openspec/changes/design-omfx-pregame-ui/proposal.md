## Why

目前 `omfx` 啟動後會直接進入既有開發中 gameplay 畫面，玩家缺少主畫面、選地圖與選難度的正式進場流程；同時 backend lifecycle 仍偏向 dev launcher 啟動模型，無法做到「玩家確認進入本場遊戲後才啟動 backend、每場結束後關閉」的產品體驗。

這個 change 要讓 `omfx` 先呈現類似參考圖的輕量、明亮、可點選主畫面流程，並把 backend 啟動延後到玩家完成地圖與難度選擇後，避免 idle menu 期間佔用 authoritative server process。

## What Changes

- 新增 `omfx` pregame UI flow：主畫面 → 開始 → 選地圖 → 選難度 → 進入遊戲。
- 主畫面採用卡通 TD lobby 風格：背景場景、底部大型開始按鈕、左右側功能入口與可延伸的角色/資源資訊區，但首版只要求開始流程可用。
- 主畫面、地圖選擇、難度選擇的 UI 資料與互動 action SHALL 由 scripts content mod 宣告，讓 `scripts/base_content` 或其他 mod 可替換流程資料、文案、卡片與按鈕行為。
- 地圖選擇畫面提供由 scripts catalog 定義的多張地圖卡片、左右翻頁/分頁提示、返回主畫面，選定後進入難度畫面。
- 難度畫面提供由 scripts catalog 定義的簡單/中級/困難或 mod 自訂難度選項，顯示推薦或獎勵資訊，選定後才建立本場 game session。
- `omfx` 在 session start 時依選到的 map/difficulty 設定 `STORY` 或等價 runtime config，然後喚醒 backend 並連線進入既有 gameplay 畫面。
- 每場遊戲結束或玩家離開 gameplay 時，`omfx` 關閉該場 backend process/session，回到 pregame UI。
- 既有開發中 gameplay HUD、lockstep client、sim_runner 與 render pipeline 不在 menu 狀態啟動，直到正式進入遊戲才啟動。
- **BREAKING**：修改既有「frontend 不負責 backend lifecycle」的規格邊界；新邊界改成 `omfx` 可透過受控 session launcher 喚醒與關閉 backend，但不得在 menu idle 期間常駐 backend。

## Capabilities

### New Capabilities

- `omfx-pregame-flow`: 定義 `omfx` pregame UI 狀態機、主畫面、地圖選擇、難度選擇，以及進入既有 gameplay 畫面的玩家可見行為。
- `script-owned-pregame-ui-content`: 定義 pregame UI 資料、互動 action、地圖 catalog、難度 catalog 與圖片 slot 如何由 scripts content mod 擁有並由 `omfx` 載入。

### Modified Capabilities

- `frontend-backend-decoupling`: 將 backend startup 從「完全 launcher-owned」調整為「session-scoped frontend-requested lifecycle」，允許 `omfx` 在玩家正式開始一場遊戲時喚醒 backend，並在該場結束時關閉。

## Impact

- 主要影響 `omfx/game/src/native.rs` 的啟動流程、UI 狀態管理、lockstep/sim_runner 初始化與 gameplay teardown。
- 可能新增 `omfx` 端 session launcher/adapter module，用來啟動已建置好的 `omobab.exe`、傳遞 map/difficulty config、追蹤 child process 並清理。
- 需要在 `scripts/base_content` 或 script ABI/shared runtime 中新增 pregame UI content catalog 的載入與匯出合約，讓 UI 像 mod data 一樣替換。
- 需要調整 `run.bat`/dev flow，使啟動 `omfx` 時不預先常駐 backend，或改由 `omfx` session launcher 接管每場 backend lifecycle。
- 可能需要在 shared config 或 command args 中加入 map/difficulty 到 `STORY`、`OMB_STORY`、`game.toml` variant 或等價 launch environment 的映射。
- 驗證需涵蓋 native frontend build、menu-only 啟動不連 backend、進入遊戲後 backend 被啟動、離開/結束後 backend 被關閉，以及既有 gameplay 行為不回歸。
