# omfx 可視化 TD 1–100 快轉 Autoplay 設計

## 目標

讓使用者從 repository 根目錄執行 `run.bat --autoplay-100`，直接開啟 Fyrox `omfx` 視窗觀看自動玩家完成 TD 第 1 至 100 關。模擬應盡可能快，畫面約每秒更新十次，並沿用 headless autoplay 的正式 `PlayerInput`、塔、技能、script、layer 與 economy 邏輯。

## 非目標

- 不新增第五個根目錄 `.bat`。
- 不修改正式 KCP 120 Hz cadence 或網路 protocol。
- 不要求每個 simulation tick 都渲染，也不保證動畫連續無跳幀。
- 不取代 `scripts/test_td_1_to_100.bat` 的自動 pass/fail 測試用途。
- 不在可視化模式自動關閉視窗；完成或失敗後保留最終畫面供檢查。

## 啟動介面

`run.bat` 新增 `--autoplay-100` 參數。啟用時批次檔設定 `OMFX_AUTOPLAY_100=1` 與無英雄 TD 環境，沿用既有 freshness build、script DLL staging 與 `omfx` executor 啟動流程。一般不帶參數的 `run.bat` 行為保持不變。

可視化模式不啟動後端連線 session；`omfx` 偵測環境變數後略過一般選單／pregame，自行建立 local autoplay worker。這可避免為快轉修改 server tick broadcaster，也不會把 coarse cadence 帶入正式 multiplayer。

## Autoplay 執行核心

在 `omoba-core` 將現有 `run_td_autoplay_1_to_100` 迴圈抽出可觀察入口，保留現有函式作 headless wrapper。新的 observed runner 仍使用 `TdAutoplayRunConfig::coarse_1_to_100` 與相同 policy，只增加兩個邊界：

- observer：收到只讀 `TdAutoplayFrame`，包含 `SimWorldSnapshot`、round、tick、cash、lives、tower/enemy 數、完成率與執行狀態。
- cancellation：observer 可回傳停止要求，讓關閉 `omfx` 時 worker 能安全離開。

observer 不得修改 ECS、插入額外輸入或改變 deterministic ledger。發布 snapshot 的 wall-clock 時機不進入 simulation state、hash 或 replay 判定。

## omfx Worker 與資料流

`omfx` 新增獨立 visual autoplay handle，在背景執行緒呼叫 observed runner。背景執行緒以 uncapped coarse ticks 推進，最多每 100 ms wall time擷取並發布一次 frame；回合切換、完成與失敗時強制發布，不等待下一個週期。

```text
run.bat --autoplay-100
        ↓ OMFX_AUTOPLAY_100=1
omfx visual autoplay worker
        ↓ formal PlayerInput + coarse simulation
TdAutoplayFrame (Arc<Mutex<latest>>)
        ↓ 約 10 Hz
既有 omfx snapshot-to-scene 同步
        ↓
Fyrox 戰場與狀態 overlay
```

render thread 只複製最新 frame，不排隊保存舊 frame；若模擬在兩次渲染間前進很多 ticks，中間狀態可被覆蓋。發布後 worker 呼叫 `yield_now`，降低對 Fyrox render thread 的 CPU 飢餓。

## 畫面與狀態

可視化模式重用既有 TD 戰場、塔、creep、projectile、血條與效果同步。額外 overlay 顯示：

- `AUTOPLAY 1–100` 模式標示；
- 目前 round／100 與完成百分比；
- cash、lives、tower 數、enemy 數與 simulation tick；
- `RUNNING`、`COMPLETED`、`FAILED` 或 `CANCELLED` 狀態；
- 失敗時的簡短原因與既有 failure report 路徑。

完成或失敗後 worker 停止推進，omfx 保留最後 snapshot，使用者關閉視窗才結束程序。

## 錯誤處理

- script DLL、scene 或 world 初始化失敗時發布 `FAILED` frame，畫面顯示原因並寫入 log。
- simulation invariant 或 watchdog 失敗時沿用 `target/td-autoplay/failure.txt`，並將摘要送到 overlay。
- render thread 不得因 worker panic 一起 panic；join 時將 panic 轉成 log 與失敗狀態。
- `run.bat` 建置失敗時不得啟動 executor，並傳回原始非零 exit code。

## 測試與驗收

- `omoba-core`：確認 observed 與 headless runner 使用同一 policy、輸入與最終 ledger/state hash；observer cadence 不影響 deterministic 結果；cancellation 可停止 worker。
- `omfx`：確認環境模式選擇、10 Hz frame 節流、latest-frame 覆蓋、完成／失敗 overlay 與關閉取消。
- 批次檔：確認 `run.bat --autoplay-100` 可解析且一般 `run.bat` 行為不變，並維持 CRLF。
- 整合驗收：視窗實際顯示塔與敵人變化、狀態 overlay 持續更新，最後到達 round 100 並停留在 `COMPLETED`。
- Git 檢查：DLL、EXE、PDB、`target/`、log、trace 與 failure report 皆不納入提交。

## 風險與取捨

- 10 FPS snapshot 會讓高速敵人跳動；這是換取接近 headless 執行速度的刻意取捨。
- snapshot extraction 與 scene reconciliation 會降低 autoplay throughput，但不作為 pass/fail 條件。
- `omfx/game/src/native.rs` 體積已大；新增模式選擇與 overlay glue 可放在該檔，但 worker、frame 與節流邏輯應留在獨立模組，避免擴大既有熱路徑。
