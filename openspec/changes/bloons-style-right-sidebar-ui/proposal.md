## Why

目前 TD 模式買塔、選中塔、出售與升級都擠在右側純文字區，和 Bloons TD 6 那種「左側選中塔升級面板、右側買塔與回合控制」的操作節奏不同。這次改版要把 UI 規畫成左右分欄：玩家買塔時看右邊，選塔升級/賣塔時看左邊，降低資訊混在一起的負擔。

## What Changes

- 新增 BTD 風格左右分欄 TD UI：左側為選中塔資訊、三路升級與出售；右側為買塔格子、Start Round、Pause/Play 控制。
- 右側買塔區從文字清單改成可放透明 PNG 的圖示格子，顯示塔圖、價格、可購買/鎖定狀態，並保留數字快捷鍵與既有買塔流程。
- 左側選中塔面板顯示大塔圖、塔名、等級摘要、三路升級卡與出售區，升級卡支援透明 PNG 圖示與價格文字。
- Start Round 與暫停/播放控制移到右側底部，以大型圖示按鈕呈現，避免和左側升級/出售操作混在一起。
- 圖片資源採前端本地載入與 fallback 策略，支援 PNG alpha；缺圖時仍要用文字卡片可操作。
- 保留既有 `TowerPlace`、`TowerSell`、`TowerUpgrade`、`StartRound` lockstep input 流程，不改後端 gameplay 規則。

## Capabilities

### New Capabilities
- `td-rich-sidebar-ui`: 定義 TD 左右分欄圖文 UI、右側買塔/開始暫停控制、左側選中塔升級/出售面板、圖片透明度與互動需求。

### Modified Capabilities

## Impact

- 主要影響 `omfx/game/src/lib.rs` 的 TD UI 建立、每幀定位更新、圖片資源載入、左右面板 hit-test rect 與文字更新。
- 可能新增 `omfx/data/td_ui/` 或相近目錄作為塔圖示、升級路線圖示、出售圖示、開始/暫停圖示、面板與卡片底圖資源位置。
- 不改變 `omoba_core` protocol、`omb` lockstep input、塔升級/出售規則或 snapshot data contract。
- 需注意 stress 場景下 UI 不應每 frame 建立/刪除節點；應沿用既有 create-once、hide-offscreen、update-position/text/texture 的模式。
