## ADDED Requirements

### Requirement: sim runner 等待對齊 lockstep tick deadlines

`omfx::sim_runner::wait_tick_batch` SHALL 以兩階段策略等待下一個 lockstep tick deadline：只有在剩餘時間大於短精準等待窗口時才 sleep，最後精準等待窗口內改用 yield-based loop。預設精準等待窗口 SHALL 約為 2ms，tick interval SHALL 從 server 宣告的 lockstep cadence 推導，而不是由 `omfx/game.toml` 或環境變數自行決定。

#### Scenario: sleep leaves precision window before deadline
- **WHEN** server 宣告 lockstep step FPS 為 120，frame interval 約為 8.33ms，且該 frame 前段工作已消耗約 3ms
- **THEN** `wait_tick_batch` 計算出的 remaining budget 約為 5.33ms
- **AND** 在進入最後精準等待窗口前，sleep 時間不超過約 3.33ms
- **AND** deadline 前最後約 2ms 由 yield-based wait loop 處理

#### Scenario: wait budget follows server configured FPS
- **WHEN** server 宣告 lockstep step FPS 為 90 或 60
- **THEN** `wait_tick_batch` 使用約 11.11ms 或約 16.67ms 的 tick interval 計算 deadline
- **AND** sleep duration 依該 interval 的 remaining budget 動態計算
- **AND** 最後仍保留約 2ms precision window 給 yield-based wait loop

#### Scenario: available TickBatch is not delayed by pacing
- **WHEN** sim runner input channel 已有可用的 `TickBatchPayload`
- **THEN** `wait_tick_batch` 不再 sleep 另一個 lockstep interval，直接回傳該 payload
- **AND** backlog recovery 只受 tick processing work 限制，不受額外 artificial pacing delay 限制

#### Scenario: cadence source remains shared
- **WHEN** 檢查 `wait_tick_batch` implementation
- **THEN** frame interval 由 server 在 lockstep start metadata 宣告的 cadence 與 shared timing helper 推導
- **AND** implementation 不新增獨立 magic values `60`、`90`、`120`、`16_667`、`11_111` 或 `8_333` 作為 frontend-local cadence source
- **AND** `omfx/game.toml` 不提供 server step FPS override

#### Scenario: long starvation remains observable
- **WHEN** 在一個 starvation logging window 內沒有任何 `TickBatchPayload` 到達
- **THEN** sim runner 仍發出既有 low-volume starvation diagnostic 或等價訊息
- **AND** precision wait loop 不會在每次 yield iteration 或每個 missed deadline 都記錄 log
