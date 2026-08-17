## 1. 建立可觀察 autoplay 核心

- [x] 1.1 在 `omoba-core` 定義 immutable `TdAutoplayFrame`、執行狀態與 observer cancellation 結果
- [x] 1.2 將 1–100 autoplay 抽成 observed runner，保留既有 headless API wrapper 與正式 `PlayerInput` 路徑
- [x] 1.3 實作 100 ms latest-snapshot 發布、round transition／完成／失敗強制發布與取消檢查
- [x] 1.4 新增 headless／observed final report parity、observer 不改變 hash 與 cancellation 單元測試

## 2. 建立 omfx visual autoplay worker

- [x] 2.1 在 `omfx` 新增獨立 visual autoplay worker/handle，於背景執行 observed runner並安全 shutdown
- [x] 2.2 以 `Arc<Mutex<latest frame>>` 發布最新 frame，不累積 render backlog，並在發布後 yield
- [x] 2.3 新增 worker 狀態、latest-frame 覆蓋、完成／失敗與取消行為測試

## 3. 整合 Fyrox 顯示

- [x] 3.1 以 `OMFX_AUTOPLAY_100=1` 選擇 visual autoplay，略過一般 pregame/backend session 並建立 TD scene
- [x] 3.2 將 visual frame 的 `SimWorldSnapshot` 接入既有 snapshot-to-scene reconciliation
- [x] 3.3 新增 `AUTOPLAY 1–100` overlay，顯示 round、進度、cash、lives、tower/enemy 數、tick、狀態與錯誤摘要
- [x] 3.4 完成或失敗後凍結 final frame，plugin shutdown 時取消並 join worker

## 4. 擴充根目錄 launcher

- [x] 4.1 讓 `run.bat` 解析 `--autoplay-100` 並設定 `OMFX_AUTOPLAY_100=1`、`OMB_NO_HEROES=1` 與 TD 環境
- [x] 4.2 保留一般 `run.bat` 行為、既有 freshness build／DLL staging 與非零錯誤傳遞
- [x] 4.3 確認 `run.bat` 維持無 BOM UTF-8 與 CRLF 行尾

## 5. 驗證

- [x] 5.1 執行 `omoba-core`、`omfx` 與既有 TD autoplay 測試，確認 deterministic parity
- [x] 5.2 執行 `run.bat --autoplay-100`，確認約 10 FPS 顯示戰場與 overlay 並完成 round 100
- [x] 5.3 確認一般 `run.bat` 模式未進入 autoplay，且 Git 未包含 DLL、EXE、PDB、`target/`、log、trace 或 failure report
