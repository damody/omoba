## Why

TD 金錢目前儲存在 `Hero` 元件；開發啟動器停用英雄後，起始金錢、建塔消費、出售退款與回合獎勵也一併消失。為維持 Bloons TD 式玩法，玩家經濟必須成為獨立於場景實體的權威狀態。

## What Changes

- 新增以 `player_id` 索引的權威 `PlayerEconomy` ECS resource，提供查詢、原子扣款與飽和加款。
- TD 初始化不論是否產生英雄，都為既有兩位玩家建立帳戶並套用難度起始金錢或 `OMB_TD_STARTING_GOLD` 覆寫值。
- 建塔、升級、出售、可負擔性判斷、擊破獎勵與回合收入改用玩家帳戶；非 TD 的英雄金錢、道具與經驗流程維持不變。
- `SimWorldSnapshot` 輸出玩家金錢，omfx 依 `local_player_id` 更新 HUD 與商店判斷，零英雄時不建立假英雄狀態。
- 玩家經濟納入 lockstep state hash，並補齊無英雄、失敗不異動、插入順序與前端消費測試。
- 不調整塔價格、回合表、Bloons 收入表，也不加入共享錢包、轉帳、農場或額外收入機制。

## Capabilities

### New Capabilities

- `player-economy`: 規範 TD 玩家帳戶的初始化、消費、退款、擊破與回合收入、失敗語意及 deterministic hashing。

### Modified Capabilities

- `sim-snapshot-rendering`: `SimWorldSnapshot` 必須提供與 Hero render entity 無關的玩家金錢資料。
- `td-rich-sidebar-ui`: TD HUD 與商店可負擔性必須使用本機玩家的 snapshot 金錢，即使場上沒有英雄。

## Impact

- `omoba-core`：新增 economy resource，修改 TD 初始化、塔操作、獎勵、snapshot 與 state hash。
- `omfx/game`：從 snapshot 玩家金錢更新既有 `hero_state.gold` 顯示與購買判斷，不依賴 Hero entity。
- `omb`：既有整合與 deterministic 測試需驗證新權威狀態，不新增外部 protocol migration。
- `run.bat` 與 `run_10000.bat` 的無英雄行為保留；後者仍以 10,000 覆寫起始金錢。
