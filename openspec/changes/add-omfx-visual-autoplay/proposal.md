## Why

現有 TD 1–100 autoplay 只能在 headless integration test 中執行，使用者無法觀看塔、敵人與經濟狀態如何演進。需要一個由根目錄 launcher 啟動的 omfx 可視化快轉模式，在保留正式遊戲邏輯的同時提供約 10 FPS 戰場觀察。

## What Changes

- 為 `run.bat` 新增 `--autoplay-100` 模式，不新增根目錄批次檔。
- 新增可觀察且可取消的 TD autoplay runner，與既有 headless runner 共用 policy、正式 `PlayerInput` 與 deterministic simulation。
- 在 omfx 背景 worker 以 uncapped coarse profile 推進模擬，每 100 ms 發布最新 render snapshot。
- 讓 Fyrox 前端直接顯示 autoplay 戰場，以及 round、cash、lives、tower/enemy 數、tick 與完成狀態 overlay。
- Round 100 完成或失敗後保留最終畫面，直到使用者關閉視窗。
- 保持一般 `run.bat`、正式 KCP 120 Hz cadence 與 headless pass/fail 測試行為不變。

## Capabilities

### New Capabilities

- `omfx-visual-autoplay`: 規範 omfx 可視化 TD 1–100 快轉 worker、10 Hz snapshot、戰場呈現、狀態 overlay、取消與完成行為。

### Modified Capabilities

- `dev-run-incremental-build`: 擴充 `run.bat` launcher-specific runtime behavior，加入 `--autoplay-100` 模式，同時保留既有 freshness build 與一般啟動語意。

## Impact

- 主 repo：`run.bat`、`omoba-core` autoplay API 與測試。
- `omfx` submodule：visual autoplay worker、模式啟動、snapshot consumption 與 overlay。
- 不修改 KCP protocol、正式 server cadence、遊戲內容或第三方相依套件。
- build／測試產物仍限制於已忽略的 `target/`、DLL 與 log 路徑。
