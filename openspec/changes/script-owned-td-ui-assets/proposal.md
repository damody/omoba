## Why

TD UI 圖片如果放在 `omfx/data`，企劃需要進前端專案才知道能換哪些圖，也無法和 `base_content` 的塔、升級與模式內容一起維護。圖片資源應由 scripts content mod 擁有，且每個 UI 圖片位置都要有一張可替換的預設 PNG 與唯一檔名，讓企劃直接照檔名換圖。

## What Changes

- 將 TD UI 圖片資源的權威來源移到 scripts content mod，例如 `scripts/base_content/assets/td_ui/`，並以「甜點戰爭」作為 TD UI 美術主題。
- 每個可放圖的 UI 位置都提供唯一檔名與預設 PNG，不再只有程式內 fallback 或空白圖片節點。
- 新增資源 manifest 或目錄規範，列出右側買塔格、左側選中塔、升級路線、出售、Start/Pause、面板背景與卡片背景的檔名用途。
- 前端 `omfx` 改為從 scripts asset 目錄讀取 TD UI 圖片；`omfx/data/td_ui` 不再是權威來源，可只作為相容 fallback 或移除。
- 主程式/文件需要能清楚指出 scripts content mod 提供的圖片清單，方便企劃與工具鏈檢查缺圖。
- 保留 PNG alpha 支援與缺圖不 panic 的 fallback 行為；但正式內容必須內建預設圖，缺圖應視為可診斷問題。

## Capabilities

### New Capabilities
- `script-owned-td-ui-assets`: 定義 TD UI 圖片由 scripts content mod 擁有、每個用途都有唯一檔名與預設 PNG、前端與主程式從 scripts assets 載入的需求。

### Modified Capabilities

## Impact

- 影響 `scripts/base_content/`：新增 TD UI asset 目錄、預設 PNG、README/manifest，並成為企劃替換圖片的位置。
- 影響 `omfx/game/src/lib.rs`：TD UI texture loader 的候選路徑要優先讀取 scripts asset 目錄，而不是前端 `omfx/data/td_ui`。
- 可能影響 `run.bat` 或 build/staging 流程：若執行目錄不同，仍要能找到 scripts assets，或在 dev run 時明確 stage 到可讀位置。
- 不改變 lockstep protocol、塔價格、升級規則、出售規則或 backend gameplay。
