## Context

`omfx` native frontend 目前在 plugin 初始化時直接建立 gameplay runtime：讀取 `game.toml`/env、啟動 `lockstep_client`、spawn `sim_runner`，然後進入既有 render/HUD pipeline。這對開發很快，但玩家視角缺少正式進場流程，也讓 backend lifecycle 只能靠 launcher scripts 預先準備。

現有 `frontend-backend-decoupling` 規格禁止 `omfx` 從 frontend process 啟動 backend，目的是避免 `omfx -> omb` crate dependency、repo path 探測與 `cargo run` 這類開發期耦合。這次需求需要調整邊界：`omfx` 仍不得依賴 `omobab` crate、不得在 runtime 內建置 backend，但可以透過 session launcher 啟動已建置好的 `omobab.exe`，並在每場遊戲結束時清理該 process。

UI 方向參考 BTD 類型的卡通 TD pregame：主畫面有鮮明背景與大型開始按鈕；選地圖畫面以多張地圖卡展示，保留返回、分頁與後續擴充空間；選難度畫面以三個大選項卡呈現簡單/中級/困難。這些 UI 畫面資料、卡片資料、文案、圖片 slot 與按鈕 action 不應硬寫在 `omfx`，而應由 scripts content mod 提供，讓 `scripts/base_content` 或其他 mod 能替換主選單到進遊戲前的流程資料。首版目標是完成可操作流程與 lifecycle boundary，不要求一次做完商店、英雄更換、社群、每日獎勵等周邊入口。

## Goals / Non-Goals

**Goals:**

- `omfx` 啟動後預設停在 pregame menu，不啟動 `lockstep_client`、`sim_runner` 或 backend process。
- 建立清楚的 UI 狀態機：`MainMenu`、`MapSelect`、`DifficultySelect`、`StartingSession`、`InGame`、`SessionEnded`。
- pregame UI 的資料、互動 action、地圖 catalog、難度 catalog 與圖片 slot 由 scripts content mod 擁有，`omfx` 只負責渲染與執行白名單 action。
- 玩家按下開始、選定地圖與難度後，才建立本場 backend session 並進入既有 gameplay 畫面。
- 將 map/difficulty 選擇轉成 deterministic runtime config，例如 `OMB_STORY`、`OMB_SCENE_PATH`、difficulty env/config 或後續可被 backend 讀取的等價設定。
- 每場結束或玩家離開時，停止 lockstep/sim_runner、關閉 session-owned backend，並回到 menu。
- 保持 `omfx` native build 不依賴 `omobab` crate；session launcher 只啟動已存在的 executable。

**Non-Goals:**

- 不重新設計既有 gameplay HUD、TD sidebar、英雄技能 UI 或 snapshot rendering。
- 不在這個 change 完成帳號、商店、英雄養成、多人 lobby、雲端存檔或地圖進度解鎖。
- 不要求 backend 支援同一 process 連續切多場；首版以一場一 process/session 為界線。
- 不把 `omfx` 改成自己編譯 `omb` 或執行 `cargo run`。
- 不在 script ABI 內新增 UI 專用資料模型。
- 不允許 scripts 執行任意前端程式碼；scripts 只宣告資料與受控 action id，`omfx` 執行白名單內的 UI transition/session action。

## Decisions

### Decision: 用 frontend-owned `PregameState` 包住 gameplay runtime 初始化

`Plugin::init` 只建立場景、menu UI 基礎節點與必要的靜態資源，預設 `PregameState::MainMenu`。目前 `init` 中啟動 `lockstep_client` 與 `sim_runner` 的邏輯搬到 `start_game_session(selection)`，並只在 `StartingSession -> InGame` 轉換時呼叫。

替代方案是保留現有初始化、只把 menu 畫在 gameplay 上方。這會讓 menu idle 仍連 backend 且 sim_runner 常駐，無法符合「正式開始才喚醒後端」。

### Decision: 建立 `GameSession` ownership object 管理 backend、lockstep 與 sim_runner

新增 session ownership 結構集中保存：

- session id、本場 map/difficulty selection；
- session-owned backend child handle；
- `lockstep_handle`；
- `sim_runner_handle`；
- teardown 狀態與最近錯誤。

`GameSession::shutdown()` 必須是 idempotent，供正常結束、玩家返回、plugin deinit、啟動失敗 recovery 共用。這比把多個 `Option<_>` 分散在 UI branches 裡安全，尤其後續要處理啟動中取消、backend 啟動失敗與重開一場。

### Decision: session launcher 只啟動 executable，不直接依賴 backend crate

`omfx` 端加入小型 `backend_session` module，透過 config/env 找到已建置的 `omobab.exe`，以 `std::process::Command` 傳入本場環境變數，例如 `OMB_STORY`、`OMB_DIFFICULTY`、`OMB_KCP_ADDR`。禁止從 `omfx` 呼叫 `cargo`、探測 `omb` repo source layout 或 import `omobab::*`。

替代方案是把 backend 啟動繼續交給 `run.bat`。這不符合玩家從 menu 選擇後才喚醒 backend 的產品流程，也會讓不同地圖/難度的 session config 很難由 UI 控制。

### Decision: pregame UI content 由 scripts mod catalog 宣告

新增 script-owned pregame catalog，權威位置放在 `scripts/base_content` 的 content/mod 目錄下，例如 `scripts/base_content/assets/pregame_ui/` 搭配 manifest，或由 Lua/generated story catalog 匯出等價資料。catalog 至少包含 screen id、widget/card id、顯示文字、圖片 slot/path、enabled/locked 狀態、action id、map id、story/runtime identifier、difficulty id 與 reward/description metadata。

`omfx` 只載入 catalog、渲染資料、命中測試與執行受控 action，例如 `Navigate(screen_id)`、`SelectMap(map_id)`、`SelectDifficulty(difficulty_id)`、`StartSession`、`Back`、`NoOp`。這讓 UI 行為像 mod data 一樣替換，但避免 script 端直接呼叫任意 frontend/runtime function。

替代方案是把 pregame map/difficulty tables hard-code 在 `omfx`。這會讓新增地圖、替換文案或做不同 mod 主畫面都需要改 frontend code，不符合「scripts 像 mod 一樣」的內容 ownership。

### Decision: difficulty 先走資料驅動設定，避免硬寫 gameplay 分支

難度由 scripts catalog 宣告 `difficulty_id`、顯示名稱、倍率/獎勵文字與 backend config 值。進入遊戲時只保證把 difficulty 傳給 backend；若 backend 目前尚未套用難度倍率，implementation tasks 需要補最小可測的 mapping 或保留 no-op 但有 log/metadata。這避免 UI 先被 gameplay balance 綁死。

### Decision: UI 以 Fyrox/eui 現有 immediate-mode patterns 實作首版

首版應優先使用 repo 既有 UI/scene asset pipeline，不引入新的 UI framework。卡片、按鈕與文字可先用 texture atlas 或簡化 vector/colored panel 佔位，但 layout 必須符合參考圖的視覺節奏：寬螢幕滿版背景、中心內容區、左上返回、底部大型 CTA/難度列。

替代方案是先做外部 web menu 或全新 renderer overlay；這會增加工具鏈與 event routing 複雜度，對當前 native `omfx` 目標不划算。

## Risks / Trade-offs

- [Risk] 直接由 `omfx` 啟動 backend 可能重新引入 frontend/backend 耦合 → Mitigation: 規格限定 executable-level session launcher，不允許 crate dependency、`cargo run`、source repo path probing。
- [Risk] Windows child process 清理不完整會留下 `omobab.exe` → Mitigation: `GameSession::shutdown()` 統一 kill/wait，plugin `on_deinit` 與返回 menu 都必須呼叫；測試用 process list 或 log 驗證。
- [Risk] backend 啟動到可連線有時間差 → Mitigation: `StartingSession` 狀態顯示 loading，lockstep client 連線需有 timeout/retry 與失敗返回難度畫面。
- [Risk] script-owned UI action 太自由會讓 mod 破壞 frontend 狀態機 → Mitigation: manifest 只允許白名單 action enum，所有未知 action fallback 為 disabled/no-op 並 log。
- [Risk] map/difficulty 與 backend `STORY`/game config 不一致會造成 sim_runner/backend 世界分歧 → Mitigation: session config 從同一份 scripts catalog 產生並同時餵給 backend 與 local sim_runner，log 輸出 story/difficulty/session id。
- [Risk] UI first screen 需要美術資源，但目前 repo 可能沒有完整 menu assets → Mitigation: 首版允許可替換 placeholder textures，但 layout、狀態與點擊流程必須完成；後續可由 asset change 補精緻圖。
- [Risk] 現有 dev scripts 預先啟動 backend，與新 lifecycle 重疊 → Mitigation: 調整 `run.bat` native flow 或新增 env flag 讓 dev 可選舊模式；預設 product path 由 `omfx` session launcher 管理。

## Migration Plan

1. 新增 scripts-owned pregame UI catalog/manifest 與 base_content 預設資料，包含主畫面、地圖、難度與 action。
2. 新增 pregame state 與 catalog loader，讓 `omfx` 啟動後從 scripts content 載入主畫面資料並停在主畫面。
3. 將現有 gameplay startup 拆成 `start_game_session(selection)`，保留原本 lockstep/sim_runner 建立邏輯但延後執行。
4. 新增 backend session launcher，先支援 Windows native executable path/env config 與 idempotent shutdown。
5. 實作主畫面、地圖選擇、難度選擇的 render、input hit testing 與白名單 action dispatch。
6. 串接 `StartingSession -> InGame`，成功後顯示既有 gameplay 畫面；失敗時回到可重試狀態。
7. 串接遊戲結束/返回 menu teardown，確保 backend、lockstep 與 sim_runner 都停止。
8. 調整 dev launcher scripts，避免 native `run.bat` 與 `omfx` 同時擁有同一場 backend lifecycle。
9. 補上 build/test/smoke 驗證，確認 menu-only、script catalog、start session、end session 都符合規格。

Rollback 策略：保留 env flag，例如 `OMFX_SKIP_PREGAME=1` 或 `OMFX_LEGACY_AUTOSTART=1`，讓開發者可暫時直接進入舊 gameplay startup；若 session launcher 出問題，可用該 flag 回退到既有 dev 操作。

## Open Questions

- difficulty 是否首版就要影響 backend wave/gold/HP，還是先只傳遞並顯示於 session metadata？
- pregame catalog 應該優先用 JSON/RON/TOML manifest、Lua generated data，或 script ABI export？首版建議選最容易讓 content mod 替換且 `omfx` 可安全讀取的 manifest。
- game over 的權威訊號目前是否足以讓 `omfx` 判斷「本場結束」並自動關 backend，或需要新增明確 session-end event？
- release packaging 時 `omobab.exe` 的相對路徑與 asset staging 規則要放在哪個 manifest？
