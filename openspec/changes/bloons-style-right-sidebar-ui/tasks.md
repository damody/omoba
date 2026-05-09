## 1. 資源載入與面板資料結構

- [x] 1.1 在 `omfx/game/src/lib.rs` 抽出 TD UI 圖片載入 helper，沿用 `TextureResource::load_from_memory`、`CompressionOptions::NoCompression` 與多候選路徑策略。
- [x] 1.2 定義 TD UI asset 命名規則與 fallback 順序，涵蓋塔圖示、升級路線圖示、出售圖示、Start Round、Pause/Play、左右面板背景與卡片背景。
- [x] 1.3 新增右側 shop/control panel handle 結構，封裝買塔格子、Start Round、Pause/Play handles 與 hit-test rect。
- [x] 1.4 重做 selected tower context panel handle 結構，封裝左側錨點與右側商店欄左緣錨點、塔資訊、大塔圖、三路升級卡、出售卡與 hit-test rect。
- [x] 1.5 保留既有 `td_tower_button_rects`、`td_sell_button_rect`、`td_upgrade_button_rects`、`start_round_button_rect` 的行為語意，讓 input handler 不必改變 gameplay routing。

## 2. 右側買塔與回合控制面板

- [x] 2.1 建立右側常駐 shop/control panel 背景節點，支援半透明 PNG 或 fallback 色塊/文字背景。
- [x] 2.2 將動態塔購買清單從文字按鈕改成右側買塔格子網格，卡內只顯示透明塔圖示與價格；快捷鍵、名稱或短名不得疊在買塔卡內。
- [x] 2.3 實作買塔格子的選取、高亮、可購買/不可購買或鎖定狀態視覺。
- [x] 2.4 將 Start Round 控制定位到右側 panel，並保持既有 `StartRound` lockstep input。
- [x] 2.5 新增 Pause/Play 圖示位置；若 gameplay pause action 尚未實作，先顯示 disabled/placeholder 且不送錯誤 input。
- [x] 2.6 以 `1920x1080` 為 primary target 更新右側 layout，根據塔數量、shop viewport、scrollbar 與底部安全區計算 2 欄或降級 1 欄格子位置。
- [x] 2.7 驗證點擊右側買塔格仍設定 `selected_tower_kind`、清空 `selected_tower_entity`，且地圖點擊仍送出 `TowerPlace`。

## 3. 選中塔 context panel、升級與出售面板

- [x] 3.1 建立 selected tower context panel，僅在 `selected_tower_entity` 存在時顯示，未選塔時隱藏並清空可點擊 rect。
- [x] 3.2 建立選中塔資訊卡，顯示大塔圖、塔名稱、三路等級摘要與可用射程資訊。
- [x] 3.3 將三路升級按鈕改成 context panel 內的垂直升級卡片，顯示路線圖示、`P1`/`P2`/`P3`、`Lx->Ly`、下一級名稱與價格。
- [x] 3.4 實作滿級狀態顯示 `MAX`，並避免顯示不存在的下一級價格。
- [x] 3.5 將出售按鈕改成 context panel 底部大型圖文出售卡，顯示透明圖示、既有退款金額與目前 gold 區塊。
- [x] 3.6 驗證點擊 context panel 出售卡仍送出 `TowerSell`，點擊升級卡仍送出 `TowerUpgrade`，且點擊不落到地圖邏輯。

## 4. 圖片 fallback、效能與版面細節

- [x] 4.1 加入缺圖與解碼失敗 fallback，確保任一圖片缺失時 UI 仍顯示文字、價格與可點擊卡片。
- [x] 4.2 確認透明 PNG 的 alpha 在塔圖示、升級圖示、出售圖示、Start/Pause 圖示與左右面板背景都正常保留。
- [x] 4.3 確認穩定 frame 中不會每 frame 建立或刪除買塔格、出售卡、升級卡、Start/Pause UI nodes。
- [x] 4.4 以 `1920x1080` 為 primary target 調整字級、間距與顏色，且 context panel、右側商店與底部控制不遮住主要地圖互動。
- [x] 4.5 如需要 placeholder，新增可替換的 `omfx/data/td_ui/` 透明 PNG 資源並避免使用受版權保護素材。
- [x] 4.6 將 layout source of truth 改為 `ui-layout.svg` 的 1920x1080 reference bounds，並在 16:9 視窗等比例縮放。
- [x] 4.7 修正 selected tower context panel：以 `1920x1080` bounds 對齊左側錨點 `x=24 y=45 w=426 h=990`，右側錨點貼齊右側 shop panel 左緣 `x=1053 y=45 w=426 h=990`，未選塔時整組 hidden。
- [x] 4.8 修正三路升級 UI：改為三張 context panel 內橫向大卡，1920 左錨點 reference bounds 為 `57,480 357x117`、`57,615 357x117`、`57,750 357x117`，右錨點時加上 anchor delta，不得只顯示小 icon 與漂浮價格。
- [x] 4.9 修正 context panel 底部出售/金額區：退款或目前 gold 以 1920 左錨點 reference `57,915 177x78`，SELL 以 `255,900 159x99`，右錨點時加上 anchor delta，hit-test 使用按鈕背景 bounds。
- [x] 4.10 修正右側 shop/control panel：面板對齊 1920 reference `1479,0 405x1080`，買塔格使用 2 欄可捲動網格；scrollbar 與 12 塔容量修正在 6.x 追蹤。
- [x] 4.11 修正右側 Start/Pause/Play：以 `1920x1080` 固定在右側底部，不得放到 context panel、中央地圖底部技能列，也不得跟著 shop viewport 捲動。
- [x] 4.12 確認所有買塔、升級、出售、Start/Pause 的 hit-test rect 以 1920 reference 卡片 bounds 為準，而不是文字或 icon bounds；買塔卡需裁切到 shop viewport。

## 5. 驗證

- [x] 5.1 執行 `cargo check --manifest-path omfx/Cargo.toml` 或等效 omfx build 檢查，修正編譯錯誤。
- [ ] 5.2 執行 `run.bat` 手動驗證 TD_1：右側買塔、context panel 選塔資訊、出售、三路升級與右側 Start Round 仍可用。
- [ ] 5.3 手動驗證未選塔狀態：context panel 隱藏且出售/升級 rect 不可點擊，右側買塔仍可用。
- [ ] 5.4 手動驗證圖片缺失情境：暫時移除或改名其中一張 TD UI PNG，確認 UI fallback 不 panic。
- [ ] 5.5 手動驗證透明圖片：替換一張含 alpha 的 PNG，確認透明區域不遮住面板背景。
- [x] 5.6 修改完成後執行 `graphify update .` 更新 knowledge graph。
- [ ] 5.7 以 1920x1080 截圖對照 `ui-layout.svg` 與 Image 1：context panel 自動換邊、三張升級大卡、右側可捲動 2 欄買塔 viewport、scrollbar 與右側底部 Start/Pause 位置都符合版面。

## 6. 右側塔商店 scrollbar 與 12 塔容量修正

- [x] 6.1 將右側買塔區拆成固定 shop/control panel 與可捲動 shop viewport；Start/Pause/Play 固定在右側底部，不跟著買塔清單捲動。
- [x] 6.2 在右側 shop viewport 右緣新增 scrollbar track + thumb，支援滑鼠拖曳 thumb、滑鼠滾輪與點擊 track 捲動。
- [x] 6.3 將買塔卡片改為 content-space 2 欄網格，內容容量至少 12 個塔卡（2 欄 x 6 列）；塔數超過可見高度時只裁切顯示，不刪節點。
- [x] 6.4 將 `td_tower_button_rects` 改為 viewport-clipped rect：卡片捲出 viewport 後不可點擊，卡片在 viewport 內時 hit-test rect 必須對齊視覺卡片 bounds。
- [x] 6.5 更新小視窗布局：可退化為 1 欄，但仍保留至少 12 張塔卡的可捲動內容容量與 scrollbar。
- [ ] 6.6 手動驗證 12 個 tower templates：第 1 到第 12 張卡都可透過 scrollbar 看到並點選，且 Start/Pause 位置不被商店內容擠走。

## 7. 1920x1080 與選中塔面板自動換邊

- [x] 7.1 將 TD UI layout primary target 設為 `1920x1080`；若實際視窗不是 1920x1080，才以此基準縮放或退化。
- [x] 7.2 計算選中塔的 screen-space x；當塔中心位於 1920 視窗左半邊（`x < 960`）時，context panel 顯示在右側 shop/control panel 左緣。
- [x] 7.3 當塔中心位於 1920 視窗右半邊（`x >= 960`）時，context panel 使用左側錨點，避免遮住右半邊的塔。
- [x] 7.4 確認 context panel 在右側錨點時，不覆蓋右側 shop/control panel，不跟 shop viewport scroll，且升級/出售 hit-test 仍準確。
- [ ] 7.5 以 1920x1080 手動驗證：選左半邊塔時升級 UI 靠右側欄左緣，選右半邊塔時升級 UI 靠左側，兩種情況都不擋住選中塔本體。
