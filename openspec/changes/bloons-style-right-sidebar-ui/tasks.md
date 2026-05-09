## 1. 資源載入與面板資料結構

- [x] 1.1 在 `omfx/game/src/lib.rs` 抽出 TD UI 圖片載入 helper，沿用 `TextureResource::load_from_memory`、`CompressionOptions::NoCompression` 與多候選路徑策略。
- [x] 1.2 定義 TD UI asset 命名規則與 fallback 順序，涵蓋塔圖示、升級路線圖示、出售圖示、Start Round、Pause/Play、左右面板背景與卡片背景。
- [x] 1.3 新增右側 shop/control panel handle 結構，封裝買塔格子、Start Round、Pause/Play handles 與 hit-test rect。
- [x] 1.4 新增左側 selected tower panel handle 結構，封裝塔資訊、大塔圖、三路升級卡、出售卡與 hit-test rect。
- [x] 1.5 保留既有 `td_tower_button_rects`、`td_sell_button_rect`、`td_upgrade_button_rects`、`start_round_button_rect` 的行為語意，讓 input handler 不必改變 gameplay routing。

## 2. 右側買塔與回合控制面板

- [x] 2.1 建立右側常駐 shop/control panel 背景節點，支援半透明 PNG 或 fallback 色塊/文字背景。
- [x] 2.2 將動態塔購買清單從文字按鈕改成右側買塔格子網格，顯示透明塔圖示、快捷鍵、名稱或短名與價格。
- [x] 2.3 實作買塔格子的選取、高亮、可購買/不可購買或鎖定狀態視覺。
- [x] 2.4 將 Start Round 控制定位到右側 panel，並保持既有 `StartRound` lockstep input。
- [x] 2.5 新增 Pause/Play 圖示位置；若 gameplay pause action 尚未實作，先顯示 disabled/placeholder 且不送錯誤 input。
- [x] 2.6 更新右側 layout，根據 `window_size`、塔數量與底部安全區計算 2 欄或降級 1 欄格子位置。
- [x] 2.7 驗證點擊右側買塔格仍設定 `selected_tower_kind`、清空 `selected_tower_entity`，且地圖點擊仍送出 `TowerPlace`。

## 3. 左側選中塔、升級與出售面板

- [x] 3.1 建立左側 selected tower panel，僅在 `selected_tower_entity` 存在時顯示，未選塔時隱藏並清空可點擊 rect。
- [x] 3.2 建立選中塔資訊卡，顯示大塔圖、塔名稱、三路等級摘要與可用射程資訊。
- [x] 3.3 將三路升級按鈕改成左側垂直升級卡片，顯示路線圖示、`P1`/`P2`/`P3`、`Lx->Ly`、下一級名稱與價格。
- [x] 3.4 實作滿級狀態顯示 `MAX`，並避免顯示不存在的下一級價格。
- [x] 3.5 將出售按鈕改成左側底部大型圖文出售卡，顯示透明圖示、既有退款金額與目前 gold 區塊。
- [x] 3.6 驗證點擊左側出售卡仍送出 `TowerSell`，點擊左側升級卡仍送出 `TowerUpgrade`，且點擊不落到地圖邏輯。

## 4. 圖片 fallback、效能與版面細節

- [x] 4.1 加入缺圖與解碼失敗 fallback，確保任一圖片缺失時 UI 仍顯示文字、價格與可點擊卡片。
- [x] 4.2 確認透明 PNG 的 alpha 在塔圖示、升級圖示、出售圖示、Start/Pause 圖示與左右面板背景都正常保留。
- [x] 4.3 確認穩定 frame 中不會每 frame 建立或刪除買塔格、出售卡、升級卡、Start/Pause UI nodes。
- [x] 4.4 調整字級、間距與顏色，讓 1920x1080 與較窄視窗下都可讀，且左右面板不遮住底部技能列主要互動。
- [x] 4.5 如需要 placeholder，新增可替換的 `omfx/data/td_ui/` 透明 PNG 資源並避免使用受版權保護素材。

## 5. 驗證

- [x] 5.1 執行 `cargo check --manifest-path omfx/Cargo.toml` 或等效 omfx build 檢查，修正編譯錯誤。
- [ ] 5.2 執行 `run.bat` 手動驗證 TD_1：右側買塔、左側選塔資訊、出售、三路升級與右側 Start Round 仍可用。
- [ ] 5.3 手動驗證未選塔狀態：左側 panel 隱藏且出售/升級 rect 不可點擊，右側買塔仍可用。
- [ ] 5.4 手動驗證圖片缺失情境：暫時移除或改名其中一張 TD UI PNG，確認 UI fallback 不 panic。
- [ ] 5.5 手動驗證透明圖片：替換一張含 alpha 的 PNG，確認透明區域不遮住面板背景。
- [x] 5.6 修改完成後執行 `graphify update .` 更新 knowledge graph。
