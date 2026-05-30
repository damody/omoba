## Why

先前完成的 `omfx` pregame UI 在後續合併中被還原或部分覆蓋，導致玩家進入遊戲前的主選單、地圖選擇與難度選擇體驗退回較陽春狀態。現在需要把先前的 pregame UI 體驗加回來，同時不破壞目前已修好的 backend session launcher、lockstep timeout、join timeout、TD sidebar 與既有 gameplay HUD。

## What Changes

- 重新導入先前 pregame UI 的玩家可見流程與版面：主畫面、開始按鈕、地圖選擇、難度選擇、返回與 loading/error 狀態。
- 保留現有功能作為硬邊界：不得倒退 `backend_session` lifecycle、不得移除 lockstep client 30s stall timeout/10s join timeout、不得破壞 TD sidebar/tooltip/擊破數顯示。
- 以目前 `omfx` 最新 `master` 為基底，選擇性移植舊 UI commit 中仍有效的 layout、render 與 hit-test 邏輯；不得整檔回退 `native.rs`、`pregame.rs` 或 `backend_session.rs`。
- 讓 pregame UI 繼續使用 scripts-owned catalog 作為內容來源，保留 invalid catalog 安全 fallback 與 diagnostic log。
- 加入 regression 驗證，確認 pregame UI 回來後，menu idle 不啟動 backend，選圖/選難度後才開始 session，遊戲內既有 input/HUD 行為不回歸。
- 不引入 breaking change；這是功能恢復與相容性修復。

## Capabilities

### New Capabilities

- `omfx-pregame-ui-restoration`: 定義恢復 pregame UI 時必須保留的玩家可見流程、現有 session 行為邊界，以及避免整檔回退造成 regression 的需求。

### Modified Capabilities

- `frontend-backend-decoupling`: 補強 pregame UI 恢復時不得破壞 session-scoped backend lifecycle、external backend mode 與 menu idle 行為的要求。

## Impact

- 主要影響 `omfx/game/src/native.rs`、`omfx/game/src/pregame.rs`、`omfx/game/src/backend_session.rs`，但 implementation 必須以局部移植與相容調整為原則。
- 可能影響 `scripts/base_content/assets/pregame_ui/catalog.json` 與相關 pregame UI asset path，但不應改變 gameplay script ABI。
- 需要比對舊 pregame UI commit，例如 `9f88f55`、`8fe49fd`、`06baa16`，擷取仍需要的 UI layout/flow，而不是把舊版整體覆蓋到現有程式。
- 驗證需涵蓋 `omfx` build、pregame catalog/action tests、session launcher ownership tests，以及至少一次手動或可腳本化 smoke path：啟動 menu → 開始 → 選圖 → 選難度 → 進入 gameplay。
