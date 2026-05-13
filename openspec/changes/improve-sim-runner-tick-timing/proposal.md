## Why

`omfx::sim_runner::wait_tick_batch` 目前使用一般 sleep 等待下一個 lockstep frame deadline，容易因 Windows timer jitter 錯過目標 cadence；先前以 120 FPS 推導的 8.33ms 也不應成為固定假設。server step FPS 需要可由 `omb/game.toml` 在 120/90/60 間設定，client 則服從 server 宣告的 cadence，避免 frontend 與 backend 在分離打包後各自猜測 simulation timing。

## What Changes

- 新增 server-owned `game.toml` 設定，例如 `omb/game.toml` 的 `[server] STEP_FPS = 120|90|60`，由 backend authoritative loop 與 TickBatch broadcaster 使用；此設定不得改用環境變數作為主要來源。
- server 在 lockstep start metadata 中宣告實際 step FPS 或 tick period，讓 client 以 server cadence 推導 sim dt、deadline 與 diagnostics。
- 調整 `wait_tick_batch` 的 deadline 等待策略：當距離下一幀仍有較長時間時先 sleep，但保留最後約 2ms 給 `yield`/短輪詢迴圈精準貼近 deadline。
- sleep duration 需依 server cadence 與剩餘 frame budget 動態計算，例如 120 FPS 時 tick interval 約 8.33ms、90 FPS 時約 11.11ms、60 FPS 時約 16.67ms，最後都保留約 2ms 做精準等待。
- 新增 `D:/omoba/omfx/game.toml` 作為 frontend 分離打包時的 client-owned 設定檔，但不得複製 server-authoritative 設定，例如 server step FPS。
- `omfx` 讀取 `omfx/game.toml` 中的 frontend-local 設定；simulation step FPS 只能來自 server 宣告，不由 `omfx/game.toml` 或環境變數覆寫。
- 保留 `TickBatch` 到達時的反應性；若資料已可用，不應為了 pacing 額外拖延整個 frame。
- 增加低成本驗證或測試，覆蓋 sleep/yield 分段行為與 deadline 不提早大量醒來的情境。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `lockstep-cadence`: 將固定 120Hz lockstep cadence 改成 server-owned `game.toml` 設定，支援 120/90/60 FPS，並由 server 對 client 宣告實際 cadence。
- `render-sim-cadence`: 補充 `omfx::sim_runner::wait_tick_batch` 必須以 sleep 加最後 yield window 的方式對齊 server 宣告的 lockstep tick deadline，降低 timer jitter 對 simulation cadence 的影響。
- `frontend-backend-decoupling`: 補充 frontend 分離打包時擁有自己的 `omfx/game.toml`，但只存放 client-owned 設定，不複製 server-authoritative simulation 設定。

## Impact

- 主要影響 `omb/game.toml`、`omfx/game.toml`、`proto/game.proto`、server lockstep startup/broadcaster、`omfx` lockstep client 與 `omfx/game/src/**/sim_runner*.rs`。
- 可能需要新增或調整 backend config tests、protocol conversion tests 與 `omfx` 端單元測試，驗證 120/90/60 cadence 與 sleep/yield window 計算。
- 會改變 lockstep start metadata contract，讓 client 能取得 server step FPS；不改變 `TickBatch` 的 gameplay payload 語意。
- 不新增外部依賴；如需常數，應優先使用 `omoba_core::lockstep_timing` 或既有 shared timing helper。
