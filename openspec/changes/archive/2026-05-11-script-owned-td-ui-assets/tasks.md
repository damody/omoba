## 1. Scripts Asset 目錄與預設圖

- [x] 1.1 建立 `scripts/base_content/assets/td_ui/` 目錄。
- [x] 1.2 產生或放入基礎 UI placeholder PNG：`panel_left.png`、`panel_right.png`、`shop_card.png`、`shop_card_selected.png`、`shop_card_locked.png`、`tower_fallback.png`、`sell.png`、`start_round.png`、`pause.png`。
- [x] 1.3 產生或放入塔圖 placeholder PNG：`tower_dart.png`、`tower_bomb.png`、`tower_tack.png`、`tower_ice.png`。
- [x] 1.4 產生或放入升級 fallback placeholder PNG：`upgrade_p1.png`、`upgrade_p2.png`、`upgrade_p3.png`。
- [x] 1.5 產生或放入每塔每路升級 placeholder PNG：`tower_dart_p1.png` 到 `tower_ice_p3.png` 共 12 張。
- [x] 1.6 確認所有 placeholder 是非空 PNG，且圖片本身包含可辨識標籤或圖案與透明區域。

## 2. Asset 文件與替換契約

- [x] 2.1 新增 `scripts/base_content/assets/td_ui/README.md`，說明此目錄是 TD UI 圖片權威來源。
- [x] 2.2 在 README 列出每個 PNG 檔名、用途、建議尺寸與替換注意事項。
- [x] 2.3 在 README 明確說明企劃替換圖片時必須保留檔名、PNG 格式與 alpha。
- [x] 2.4 在 README 標註 `omfx/data/td_ui/` 不是正式替換位置。
- [x] 2.5 新增 `asset-prompts.md`，列出每個 PNG 的手動生圖提示詞。

## 3. omfx 載入路徑調整

- [x] 3.1 修改 `omfx/game/src/lib.rs` 的 TD UI texture loader，使 `scripts/base_content/assets/td_ui/<file>` 優先於 `omfx/data/td_ui/<file>`。
- [x] 3.2 加入 repo root、`omfx` 工作目錄、上一層工作目錄與 `exe_dir` 的 scripts asset 候選路徑。
- [x] 3.3 保留 `CompressionOptions::NoCompression` 與 PNG alpha 支援。
- [x] 3.4 缺圖時保留 fallback 行為並加入可診斷 log 或明確註解，避免 panic。

## 4. 清理前端權威資源假象

- [x] 4.1 移除或停止追蹤目前生成在 `omfx/data/td_ui/` 的 placeholder PNG。
- [x] 4.2 確認沒有文件把 `omfx/data/td_ui/` 描述成企劃替換圖片的位置。
- [x] 4.3 若保留 `omfx/data/td_ui/` fallback，加入註解說明它只是相容 fallback，不是權威來源。

## 5. 驗證

- [x] 5.1 執行 `cargo check --manifest-path omfx/Cargo.toml`。
- [x] 5.2 手動確認 TD UI 在沒有 `omfx/data/td_ui/` 時仍能從 scripts asset 目錄載入圖片。
- [x] 5.3 手動替換一張 scripts asset PNG，確認重新啟動後 UI 顯示替換結果。
- [x] 5.4 手動確認缺少專屬升級圖時會 fallback 到 `upgrade_p*.png` 或 `tower_fallback.png` 且不 panic。
- [x] 5.5 修改完成後執行 `graphify update .`。
