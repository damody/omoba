## Why

目前 gameplay ownership 仍多處等同於「第一個 Player 英雄」或「任一 Player 塔」。這會讓第二個前端即使成功連線，也無法可靠控制自己的英雄，且可能操作到其他玩家的塔。

此變更要提供一個本機雙人啟動入口 `run_2player.bat`，並把玩家、英雄、塔的 ownership 規則補齊，讓兩個 omfx 前端連到同一個 omb 後端時，各自只控制自己的英雄與自己建造的塔。

## What Changes

- 新增 `run_2player.bat`，沿用 `run.bat` 的 freshness/build/stage 流程，但啟動一個後端與兩個前端。
- 兩個前端 SHALL 使用不同 `OMB_PLAYER_ID`、`OMB_PLAYER_NAME` / `OMB_LOCKSTEP_PLAYER_NAME`，並連到同一個 `127.0.0.1:50061` 後端；client 在連線前就 SHALL 知道自己的 `player_id`。
- 後端 authoritative runtime SHALL 為至少兩個 lockstep player 建立或綁定各自英雄，並以 `player_id` route `MoveTo`、技能施放、技能升級、item use、建塔、賣塔與升級塔。
- 新建塔 SHALL 記錄建造者 `player_id`，後續賣塔與升級 SHALL 驗證 requester 必須是 tower owner。
- 兩個玩家 SHALL 使用相同 combat `team_id`，不得把 `team_id` 設成 `player_id` 或用 `team_id` 當 ownership。
- omfx 本地 replica SHALL 使用同一份 ownership 規則，讓 snapshot/render 與本地預測不分歧。
- UI 操作另一位玩家的塔時 SHOULD 在前端避免送出無效 input；後端仍 MUST 以權威檢查拒絕非法操作。

## Capabilities

### New Capabilities
- `two-player-local-run`: 定義本機雙前端啟動、玩家身份隔離、每個 player 控制自己的英雄，以及塔 ownership 的端到端行為。

### Modified Capabilities
- `player-input-routing`: 將現有 PlayerInput routing 從「單一 Player faction」提升為依 lockstep `player_id` 綁定英雄與 tower owner 的權威路由。

## Impact

- 影響 `run.bat` 相鄰的 Windows launcher 腳本，新增 `run_2player.bat` 並確保 CRLF。
- 影響 omfx 啟動設定與環境變數讀取，包含 `OMB_PLAYER_ID`、`OMB_PLAYER_NAME`、`OMB_LOCKSTEP_PLAYER_NAME`、log/window 識別，以及同機雙前端同時執行時的輸出檔衝突處理。
- 影響 `omoba-core` runtime initialization、`Faction`/ownership component 或等效資料模型、`player_input_tick` queue drain entry points、tower spawn/sell/upgrade 與 hero command routing。
- 影響 omb lockstep join/session 行為與測試，需驗證 server 接受 client-declared `player_id`、拒絕重複或不合法 id，且兩個 player 可同時送 input。
- 影響 snapshot/render 資料，至少需要讓 omfx 可判斷 selected tower 是否屬於本地 player，避免顯示可操作的其他玩家塔升級/出售控制。
