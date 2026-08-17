## ADDED Requirements

### Requirement: 可視化 autoplay 使用正式 deterministic 遊戲路徑
系統 SHALL 讓 omfx 可視化 autoplay 與 headless 1–100 runner 共用相同 policy、coarse simulation profile、正式 `PlayerInput`、script、layer combat 與 economy mutation。Observer 發布行為 MUST NOT 修改 simulation state、ledger 或 final state hash。

#### Scenario: Observer 不改變最終結果
- **WHEN** 相同 seed 與內容分別以 headless runner 及啟用 observer 的 runner 完成第 1 至 100 關
- **THEN** 兩次執行的 round end ticks、ledger digest 與 final state hash 相同

### Requirement: omfx 以最新 snapshot 約 10 FPS 顯示快轉戰場
visual autoplay worker SHALL uncapped 推進 coarse simulation，並以 100 ms wall-clock 間隔上限發布最新 `SimWorldSnapshot`。Round transition、完成與失敗 MUST 強制發布；render consumer SHALL 只保留最新 frame，不累積 backlog。

#### Scenario: 長時間 simulation burst 不阻塞畫面
- **WHEN** worker 在兩次 Fyrox render updates 間完成多個 coarse ticks
- **THEN** renderer 取得最近一次已發布 frame
- **AND** 過期中間 frame 不會排隊等待渲染

#### Scenario: Round transition 立即可見
- **WHEN** autoplay 結束一關或開始下一關
- **THEN** worker 不等待下一個 100 ms interval 就發布更新後的 round frame

### Requirement: omfx 顯示 autoplay 狀態與進度
可視化模式 SHALL 顯示 `AUTOPLAY 1–100`、目前 round、完成百分比、cash、lives、tower 數、enemy 數、simulation tick 與 `RUNNING`／`COMPLETED`／`FAILED`／`CANCELLED` 狀態。

#### Scenario: 執行中 overlay 更新
- **WHEN** worker 發布新的 running frame
- **THEN** overlay 與戰場使用同一 frame 的 round、資源及 entity summary 更新

#### Scenario: Round 100 完成
- **WHEN** autoplay 以合法 combat path 完成第 100 關
- **THEN** overlay 顯示 `COMPLETED`
- **AND** omfx 保留最後戰場直到使用者關閉視窗

#### Scenario: Autoplay 失敗
- **WHEN** world 初始化、watchdog 或 simulation invariant 失敗
- **THEN** overlay 顯示 `FAILED` 與簡短原因
- **AND** 提供 `target/td-autoplay/failure.txt` 路徑且不讓 render thread panic

### Requirement: 關閉 omfx 可取消 visual autoplay worker
visual autoplay worker MUST 提供 cancellation 邊界，並在 omfx plugin shutdown 時停止 simulation、釋放 thread 與保留安全的結束順序。

#### Scenario: 執行中關閉視窗
- **WHEN** 使用者在第 100 關完成前關閉 omfx
- **THEN** worker 觀察到取消要求並停止
- **AND** shutdown 不會因 thread panic 或 channel deadlock 卡住

### Requirement: 正式 multiplayer cadence 保持不變
`OMFX_AUTOPLAY_100` 未啟用時，omfx SHALL 使用既有 pregame、backend/KCP 與 sim runner 路徑。可視化 autoplay MUST NOT 修改 server TickBroadcaster 或正式 120 Hz lockstep cadence。

#### Scenario: 一般 run 不進入 autoplay
- **WHEN** 使用者執行不帶 `--autoplay-100` 的 `run.bat`
- **THEN** omfx 不建立 visual autoplay worker
- **AND** 既有一般遊戲啟動流程保持不變
