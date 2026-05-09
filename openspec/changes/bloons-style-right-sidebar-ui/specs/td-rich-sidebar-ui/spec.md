## ADDED Requirements

### Requirement: TD UI 採左右分欄操作模型
TD 模式 SHALL 使用左右分欄 UI：左側為選中塔 context panel，顯示塔資訊、升級路線與出售；右側為常駐 shop/control panel，顯示買塔格子、Start Round 與暫停/播放控制。買塔 SHALL NOT 與選中塔升級/出售混在同一側主要面板。

#### Scenario: TD 模式顯示右側買塔與控制面板
- **WHEN** TD_1 載入且 `td_template_order` 有至少一個塔 template
- **THEN** omfx 顯示右側 shop/control panel
- **AND** 右側 panel 內顯示買塔格子
- **AND** 右側 panel 內顯示 Start Round 或回合控制按鈕

#### Scenario: 選中塔時顯示左側升級與出售面板
- **WHEN** 玩家左鍵點擊一座 snapshot-backed mirror 中的 TD tower
- **THEN** omfx 顯示左側 selected tower panel
- **AND** 左側 panel 顯示該塔資訊、三路升級卡與出售卡
- **AND** 右側買塔 panel 仍保持可見

#### Scenario: 未選塔時隱藏左側 context panel
- **WHEN** `selected_tower_entity` 是 `None`
- **THEN** 左側 selected tower panel SHALL 隱藏或移到螢幕外
- **AND** 右側 shop/control panel SHALL 仍保持可見

### Requirement: 右側買塔格子支援透明圖示與文字資訊
右側 shop panel SHALL 以格子或卡片網格呈現 TD 塔購買項目，每個項目 SHALL 包含塔圖示、快捷鍵序號、塔名稱或短名、價格與可購買/不可購買狀態。塔圖示 SHALL 支援 PNG alpha。缺少圖片資源時，卡片 SHALL 仍顯示可讀文字與價格，且 SHALL NOT 造成 panic。

#### Scenario: 買塔格子顯示圖示與價格
- **WHEN** `td_templates` 已包含 `tower_dart` 且對應圖片存在
- **THEN** 右側買塔格子顯示透明 PNG 塔圖示
- **AND** 同一格子顯示快捷鍵序號與 `$` 價格
- **AND** 圖片透明區域不以不透明底色遮住右側 panel 背景

#### Scenario: 缺少塔圖示時 fallback 成文字格子
- **WHEN** 某個 tower kind 的圖片不存在或解碼失敗
- **THEN** omfx 仍顯示該塔的格子背景、名稱、快捷鍵序號與價格
- **AND** 點擊該格子仍選擇對應 tower kind
- **AND** omfx SHALL NOT panic 或讓整個 shop panel 消失

#### Scenario: 點擊買塔格子沿用既有選塔行為
- **WHEN** 玩家左鍵點擊任一右側買塔格子
- **THEN** `selected_tower_kind` 設為該格子對應的 tower kind
- **AND** `selected_tower_entity` 清空
- **AND** 下一次地圖點擊仍送出既有 `TowerPlace` lockstep input

### Requirement: 右側提供 Start Round 與暫停播放控制
右側 shop/control panel SHALL 包含大型 Start Round 控制。當 round 正在執行或未執行時，控制 SHALL 反映目前 round state。若暫停/播放 gameplay action 尚未實作，暫停/播放按鈕 SHALL 顯示為 disabled 或只作為視覺 placeholder，且 SHALL NOT 送出錯誤 gameplay input。

#### Scenario: Start Round 位於右側 panel
- **WHEN** TD_1 round 尚未開始且 `round_is_running == false`
- **THEN** 右側 panel 顯示可點擊的 Start Round 控制
- **AND** 點擊該控制時 omfx 送出既有 `StartRound` lockstep input

#### Scenario: Round running 時控制狀態更新
- **WHEN** snapshot 顯示 `round_is_running == true`
- **THEN** 右側 Start Round 控制 SHALL 顯示為 running、pause 或不可重複開始狀態
- **AND** 玩家點擊時 SHALL NOT 重複送出無效 StartRound input

#### Scenario: 暫停播放未實作時不送錯誤 input
- **WHEN** UI 顯示暫停或播放圖示但 gameplay pause action 尚未實作
- **THEN** 該圖示 SHALL 顯示 disabled 或 placeholder 狀態
- **AND** 點擊該圖示 SHALL NOT panic 或送出錯誤的 lockstep input variant

### Requirement: 左側選中塔資訊與出售操作以圖文面板呈現
當玩家選中已蓋塔時，左側 selected tower panel SHALL 顯示塔圖示、塔名稱、目前三路升級等級與可用的射程資訊。出售卡 SHALL 顯示退款金額，並 SHALL 支援透明圖片或半透明裝飾底圖。未選中塔時，出售與升級互動 rect SHALL 移出可點擊區域。

#### Scenario: 點擊已蓋塔後左側顯示選中塔資訊
- **WHEN** 玩家左鍵點擊一座 snapshot-backed mirror 中的 TD tower
- **THEN** `selected_tower_entity` 設為該 tower entity id
- **AND** 左側 panel 顯示該塔的圖示、名稱與三路等級
- **AND** 若 `attack_range_backend` 大於 0，地圖上仍顯示該塔射程圈

#### Scenario: 左側出售卡顯示退款並送出 TowerSell
- **WHEN** 玩家選中塔且左側出售卡可見
- **THEN** 出售卡顯示依既有公式計算的退款金額
- **AND** 玩家點擊出售卡時 omfx 送出既有 `TowerSell` lockstep input
- **AND** 送出後 `selected_tower_entity` 清空

#### Scenario: 未選塔時隱藏出售與升級操作
- **WHEN** `selected_tower_entity` 是 `None`
- **THEN** 左側出售卡與升級卡 SHALL 不可點擊
- **AND** `td_sell_button_rect` 與 `td_upgrade_button_rects` SHALL 位於螢幕外或尺寸為 0

### Requirement: 左側三路升級卡片顯示路線圖示、等級、名稱與價格
選中塔後，左側 selected tower panel SHALL 垂直顯示三個升級路線卡片。每張卡片 SHALL 顯示路線圖示、路線序號、目前等級、下一級名稱與價格。升級路線圖示 SHALL 支援 PNG alpha。達到最大等級時，該路線卡片 SHALL 顯示 `MAX`，且不應顯示錯誤的下一級價格。

#### Scenario: 可升級路線顯示下一級資訊
- **WHEN** 選中塔的 path 0 目前等級為 0，且 `td_upgrade_defs` 有 path 0 level 1 definition
- **THEN** 左側 path 0 升級卡顯示 `P1`、`L0->L1`、下一級名稱與價格
- **AND** 卡片可顯示透明 PNG 路線圖示

#### Scenario: 滿級路線顯示 MAX
- **WHEN** 選中塔的任一路線等級大於或等於最大等級
- **THEN** 該路線升級卡顯示 `MAX`
- **AND** 該卡片 SHALL NOT 顯示不存在的下一級價格

#### Scenario: 點擊左側升級卡送出 TowerUpgrade
- **WHEN** 玩家點擊選中塔的任一路線升級卡
- **THEN** omfx 以該路線 index 與目前等級加一組成既有 `TowerUpgrade` lockstep input
- **AND** 該次點擊不會落到地圖放塔、選塔或取消選取邏輯

### Requirement: TD 左右面板圖片資源可替換且具 fallback
TD UI 圖片資源 SHALL 從前端本地資料夾載入，並 SHALL 支援 repo root、`omfx` 工作目錄與 executable 同層 `data` 的常見路徑。圖片用途 SHALL 包含塔圖示、升級路線圖示、出售圖示、Start Round 圖示、暫停/播放圖示、左右 panel 背景與卡片背景。所有圖片 SHALL 可被替換成帶 alpha 的 PNG，而不需要修改 gameplay code。

#### Scenario: 從常見路徑載入 TD UI 圖片
- **WHEN** `omfx/data/td_ui/tower_dart.png` 或等效候選路徑存在
- **THEN** omfx 載入該圖片作為 `tower_dart` 的塔圖示
- **AND** 使用 `CompressionOptions::NoCompression` 或等效方式避免 UI texture 顯示為空白

#### Scenario: 替換透明 PNG 後 UI 保留 alpha
- **WHEN** 使用者以含透明區域的 PNG 替換塔圖示、升級圖示、出售圖示或 Start Round 圖示
- **THEN** 下一次啟動 omfx 時該圖片透明區域仍保持透明
- **AND** 不需要修改 tower template、snapshot 或 lockstep protocol

### Requirement: 左右面板更新避免每幀重建 UI 節點
TD 左右面板實作 SHALL 避免在每 frame 建立或刪除 buy、sell、upgrade、start/pause 卡片節點。當塔 template 數量增加時 MAY 建立缺少的買塔格子節點；當數量減少或區塊不可見時 SHALL 隱藏既有節點並更新 hit-test rect。每幀更新 SHOULD 限制在位置、文字、可見狀態與必要 texture 變更。

#### Scenario: 塔清單穩定時不重建買塔格子
- **WHEN** TD_1 連續渲染多個 frames 且 `td_template_order` 未改變
- **THEN** omfx SHALL NOT 每 frame 建立新的右側買塔格子 UI nodes
- **AND** 既有格子 SHALL 只更新位置、文字或選取狀態

#### Scenario: 未選塔時重用左側升級卡節點
- **WHEN** 玩家取消選中塔
- **THEN** 左側升級卡與出售卡 SHALL 被隱藏或移到螢幕外
- **AND** 下一次選中塔時 SHOULD 重用同一批 UI handles
