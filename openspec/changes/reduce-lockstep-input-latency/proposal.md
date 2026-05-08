## Why

目前 `Lag` HUD 指標卡在約 55-70ms，即使 CPU/GPU profiling 顯示每 tick 與每 frame 的實際計算成本很低。需要先把 input-to-render 管線拆成可量測的分段，再把 lockstep/server/client cadence 從既有 60Hz/30Hz 混合狀態收斂到 120Hz，避免用猜測調整延遲。

## What Changes

- 新增 per-`input_id` latency trace，記錄 `on_os_event`、`send_lockstep_input`、`lockstep_client submit`、`server receive`、`server drain target_tick`、`client receive TickBatch`、`sim_runner publish snapshot`、`Game pair applied` 等分段 timestamp。
- 將 lockstep tick rate 抽成共享常數 `LOCKSTEP_TPS = 120`，讓 omb broadcaster、omfx sim runner、tick-to-seconds 轉換、log sampling 與 retention windows 使用同一個 cadence。
- 將 server authoritative simulation cadence 從 `TPS = 30` 提升到 120Hz，避免 client mirror 120Hz 但 host authoritative state 仍以 30Hz apply gameplay input。
- 保留 lockstep lookahead 行為，但以 120Hz cadence 重新計算延遲預算；預設仍使用 2 ticks，後續可用 late-input 指標評估是否降到 1 tick。
- 補上 late input 與 phase latency 的 log/metric，以便驗證 120Hz change 是否真正降低 p50/p99，而不是只改常數。

## Capabilities

### New Capabilities
- `lockstep-cadence`: 定義 lockstep/server/client simulation cadence、共享 tick-rate 常數、120Hz timing contract、lookahead 行為與驗證方式。

### Modified Capabilities
- `input-latency-metric`: 將既有總延遲 metric 擴充為 per-phase latency trace，並保留 HUD p50/p99 與 determinism 邊界。

## Impact

- 影響 `omb/src/main.rs`、`omb/src/lockstep/tick_broadcaster.rs`、`omb/src/lockstep/input_buffer.rs`、`omb/src/state/core.rs` 及相關註解/測試中的 30Hz/60Hz 假設。
- 影響 `omfx/game/src/lib.rs`、`omfx/game/src/lockstep_client.rs`、`omfx/game/src/sim_runner.rs` 中的 input submit、TickBatch drain、snapshot publish、HUD latency 顯示與 tick-to-time 轉換。
- 影響 `proto/game.proto` 或 KCP lockstep payload 若需要攜帶 server-side phase timestamps；所有新增 timestamp metadata 必須不進 sim ECS state hash。
- 測試需涵蓋 omb lib tests、omfx input latency tests、lockstep broadcaster tests，以及至少一個 TD_1 smoke/latency log 驗證流程。
