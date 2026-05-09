## ADDED Requirements

### Requirement: TD UI assets owned by scripts content mod
TD UI 圖片資源 SHALL 由 scripts content mod 擁有，權威目錄 SHALL 是 `scripts/base_content/assets/td_ui/`。前端 `omfx` SHALL NOT 以 `omfx/data/td_ui/` 作為正式 TD UI 圖片的權威來源。主程式與前端 SHALL 從 scripts asset 目錄讀取圖片或從該目錄 staging 後的位置讀取。

#### Scenario: scripts asset directory is canonical
- **WHEN** repo 中存在 `scripts/base_content/assets/td_ui/panel_right.png`
- **THEN** omfx TD UI loader 優先從 scripts asset 目錄載入該檔
- **AND** `omfx/data/td_ui/panel_right.png` 不會覆蓋 scripts 版本

#### Scenario: frontend-only asset path is not required
- **WHEN** `omfx/data/td_ui/` 不存在
- **THEN** TD UI 仍可從 `scripts/base_content/assets/td_ui/` 載入預設圖片
- **AND** omfx 不會因缺少前端本地 `td_ui` 目錄而 panic

### Requirement: every TD UI image slot has a unique default PNG
每個 TD UI 可替換圖片位置 SHALL 有唯一檔名與預設 PNG。不同用途 SHALL NOT 只共用同一個 `default.png`。預設 PNG SHALL 帶有可辨識標籤或圖案，讓企劃在畫面上能看出該圖對應哪個 UI 位置。預設圖與生圖提示詞 SHALL 採「甜點戰爭」主題。每張圖片 SHALL 有清楚可見的外框、徽章底座或厚描邊輪廓，讓替換前後都能看出圖片範圍。

#### Scenario: required base assets exist
- **WHEN** 檢查 `scripts/base_content/assets/td_ui/`
- **THEN** 目錄包含 `panel_left.png`、`panel_right.png`、`shop_card.png`、`shop_card_selected.png`、`shop_card_locked.png`、`tower_fallback.png`、`sell.png`、`start_round.png` 與 `pause.png`
- **AND** 每個檔案都是非空 PNG
- **AND** 每張圖片都有明確外框或徽章底座

#### Scenario: required tower assets exist
- **WHEN** 檢查 TD tower image assets
- **THEN** 目錄包含 `tower_dart.png`、`tower_bomb.png`、`tower_tack.png` 與 `tower_ice.png`
- **AND** 每個檔案都是非空 PNG

#### Scenario: required upgrade assets exist
- **WHEN** 檢查 TD upgrade image assets
- **THEN** 目錄包含 `upgrade_p1.png`、`upgrade_p2.png`、`upgrade_p3.png`
- **AND** 目錄包含 `tower_dart_p1.png`、`tower_dart_p2.png`、`tower_dart_p3.png`
- **AND** 目錄包含 `tower_bomb_p1.png`、`tower_bomb_p2.png`、`tower_bomb_p3.png`
- **AND** 目錄包含 `tower_tack_p1.png`、`tower_tack_p2.png`、`tower_tack_p3.png`
- **AND** 目錄包含 `tower_ice_p1.png`、`tower_ice_p2.png`、`tower_ice_p3.png`

### Requirement: asset manifest documents every replaceable file
`scripts/base_content/assets/td_ui/` SHALL include human-readable documentation that lists every replaceable file, its UI usage, and recommended dimensions. The documentation SHALL tell企劃 to replace files in scripts assets, not in frontend assets.

#### Scenario: README lists canonical replacement location
- **WHEN** 企劃打開 `scripts/base_content/assets/td_ui/README.md`
- **THEN** README 明確說明此目錄是 TD UI 圖片權威來源
- **AND** README 列出每個 PNG 檔名與用途
- **AND** README 說明替換圖片需保留檔名與 PNG alpha

### Requirement: omfx loader prioritizes scripts assets
omfx TD UI texture loader SHALL search scripts asset paths before any frontend-local fallback path. Loader SHALL keep `CompressionOptions::NoCompression` or equivalent behavior so UI PNG alpha and decode remain reliable.

#### Scenario: scripts asset wins over frontend fallback
- **WHEN** `scripts/base_content/assets/td_ui/tower_dart.png` 與 `omfx/data/td_ui/tower_dart.png` 同時存在
- **THEN** omfx 載入 scripts asset 版本
- **AND** 前端 fallback 版本不會被使用

#### Scenario: missing asset falls back without panic
- **WHEN** 某個專屬 tower path 圖例如 `tower_dart_p2.png` 缺失
- **THEN** omfx 可 fallback 到 `upgrade_p2.png` 或 `tower_fallback.png`
- **AND** omfx SHALL log 或保留可診斷的缺圖狀態
- **AND** omfx SHALL NOT panic

### Requirement: frontend generated td_ui placeholders are removed or demoted
若目前存在 `omfx/data/td_ui/` placeholder，實作 SHALL 移除該目錄或明確降級為非權威 fallback。企劃替換流程 SHALL NOT 指向 `omfx/data/td_ui/`。

#### Scenario: no conflicting canonical frontend placeholders
- **WHEN** 實作完成後搜尋 TD UI placeholder 目錄
- **THEN** 權威 placeholder PNG 位於 `scripts/base_content/assets/td_ui/`
- **AND** `omfx/data/td_ui/` 不會被文件描述為替換位置
