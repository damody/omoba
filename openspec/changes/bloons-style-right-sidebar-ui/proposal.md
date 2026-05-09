## Why

目前 TD 模式買塔、選中塔、出售與升級都擠在右側純文字區，和 Bloons TD 6 那種「選中塔 context panel、右側買塔與回合控制」的操作節奏不同。這次改版要把 UI 規畫成右側固定買塔欄與可自動換邊的選中塔面板：玩家買塔時看右邊，選塔升級/賣塔時面板顯示在不遮住塔的位置。

## What Changes

- 新增 BTD 風格 TD UI：選中塔資訊、三路升級與出售使用可自動換邊的 context panel；右側為可捲動買塔格子、Start Round、Pause/Play 控制，版面必須以 `1920x1080` primary reference 對齊 `ui-layout.svg` 與本次右側 scrollbar 修正。
- 右側買塔區從文字清單改成可放透明 PNG 的圖示格子，顯示大塔圖、底部價格、可購買/鎖定狀態，並保留既有買塔流程；右側 shop viewport SHALL 使用緊貼雙欄卡片，支援至少 12 個塔卡內容容量，超出可見高度時以右側 scrollbar 捲動，不得只保留 4 個塔卡的空間，且卡內不得疊名稱/快捷鍵文字或素材內嵌英文造成重疊。
- 選中塔面板顯示大塔圖、塔名、等級摘要、三路升級卡與出售區，升級卡支援透明 PNG 圖示與價格文字；當選中塔在畫面左半邊時，面板 SHALL 顯示在右側商店欄左緣，避免升級 UI 擋住塔與射程圈。
- Start Round 與暫停/播放控制移到右側底部，以大型圖示按鈕呈現，Start 按鈕不額外疊加 `開始 1/5` 類文字，避免和 context panel 的升級/出售操作混在一起。
- `openspec/changes/bloons-style-right-sidebar-ui/ui-layout.svg` 是 `1920x1080` 參考版面契約：選中塔升級必須是三張橫向大卡，並依塔所在半邊選擇左側錨點或右側商店欄左緣錨點；右側買塔必須是 2 欄可捲動卡片網格，Start/Pause 必須固定在右側底部且不得跟著 shop scroll。
- 圖片資源採前端本地載入與 fallback 策略，支援 PNG alpha；缺圖時仍要用文字卡片可操作。
- 保留既有 `TowerPlace`、`TowerSell`、`TowerUpgrade`、`StartRound` lockstep input 流程，不改後端 gameplay 規則。

## Capabilities

### New Capabilities
- `td-rich-sidebar-ui`: 定義 TD 分欄圖文 UI、右側買塔/開始暫停控制、可自動換邊的選中塔升級/出售 context panel、圖片透明度與互動需求。

### Modified Capabilities

## Impact

- 主要影響 `omfx/game/src/lib.rs` 的 TD UI 建立、每幀定位更新、圖片資源載入、左右面板 hit-test rect 與文字更新。
- 可能新增 `omfx/data/td_ui/` 或相近目錄作為塔圖示、升級路線圖示、出售圖示、開始/暫停圖示、面板與卡片底圖資源位置。
- 不改變 `omoba_core` protocol、`omb` lockstep input、塔升級/出售規則或 snapshot data contract。
- 需注意 stress 場景下 UI 不應每 frame 建立/刪除節點；應沿用既有 create-once、hide-offscreen、update-position/text/texture 的模式。
