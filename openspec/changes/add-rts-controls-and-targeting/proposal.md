## Why

目前英雄操作仍偏向單點移動與既有技能輸入，缺少 RTS/MOBA 常見的攻擊移動、右鍵指定攻擊與可靠尋路，導致玩家無法用熟悉的方式控制英雄進退與交戰。TD 塔也缺少可配置的目標優先策略，使塔群在不同路線、不同敵人血量與不同戰術意圖下無法被玩家精細調整。

## What Changes

- 新增 RTS/MOBA 風格英雄操作：
  - `A` 點地 attack-move：英雄移動到指定地點，途中自動攻擊偵測範圍內的合法敵人。
  - 右鍵點敵人：英雄鎖定指定敵方 entity 作為攻擊目標，目標失效時依規則回退。
  - 右鍵點地或既有移動輸入：英雄使用自動尋路前往地點，避開 blocked regions 與不可穿越物件。
  - 按住 `Shift` 下達移動、attack-move 或指定攻擊時，將命令追加到英雄動作 queue，queue 上限 16；超過上限拒絕 append。
  - 任一非 append gameplay action 會覆蓋目前命令並清空既有 hero command queue。
- 新增塔的目標優先策略設定：
  - 支援 first、last、nearest、farthest、highest-health、lowest-health。
  - 玩家可對單座塔設定策略，後端 authoritative 儲存並套用於塔的攻擊選目標。
  - omfx 顯示並送出選中塔的策略變更，後續 snapshot 反映權威值。
- 擴充 lockstep `PlayerInput` routing，使英雄控制與塔策略設定都走現有 deterministic input pipeline。
- 不移除既有移動、放塔、升級、出售、技能與 item 行為；新行為需與既有 UI hit-test 優先序相容。

## Capabilities

### New Capabilities
- `rts-unit-control`: 定義英雄 attack-move、指定攻擊、右鍵移動與自動尋路的玩家操作契約。
- `tower-target-priority`: 定義塔目標優先策略的資料契約、玩家設定流程與攻擊選目標規則。

### Modified Capabilities
- `player-input-routing`: 新增英雄控制與塔策略設定的 `PlayerInput` 端到端 routing requirements。
- `sim-snapshot-rendering`: snapshot 需要 expose 英雄尋路/命令可視化所需狀態與塔目前目標策略，供 omfx UI 顯示與 render feedback。

## Impact

- `proto/game.proto` / `omoba-core` lockstep schema：新增或擴充 `PlayerInputAction` variants 與 snapshot-facing fields。
- `omb/src/tick/player_input_tick.rs`：新增 input arms，路由到 ECS entry points，失敗只記錄 warning。
- `omb` ECS gameplay systems：新增 hero command state、attack-move acquisition、指定攻擊 validation、路徑計算/跟隨，以及塔 target priority component/selection logic。
- `omoba-core::runtime::SimWorldSnapshot`：新增塔策略與必要的 hero command/path render state。
- `omfx` UI/input/render：新增 A 鍵點地模式、右鍵點地/點敵處理、選中塔策略控制與 snapshot-backed 顯示。
- 測試：新增 lockstep routing、pathfinding command、attack target acquisition、塔策略排序與 snapshot projection 的單元/整合測試。
