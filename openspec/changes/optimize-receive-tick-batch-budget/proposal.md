## Why

`run_stress.bat` 顯示 `omfx::sim_runner::tick` 多數只有 1-3ms，理論上足以支撐 120 FPS/TPS，但實際 FPS 仍無法穩定達標；`receive_tick_batch` span 超過 6.33ms 可能只是 tracing 把 blocking wait 標錯，也可能掩蓋真正的 pacing、jitter、queue handoff、snapshot publish/consume 或 render present 問題。這個 change 的目標不是重新命名 span，而是用更細且可對齊的診斷找到真因，並修到 10k units stress 場景也能穩定 120 FPS，1% low >= 119。

## What Changes

- 將 omfx 120 FPS 診斷從單一 `receive_tick_batch`/`tick` span 擴展為完整 frame-to-sim-to-present timeline，涵蓋 lockstep receive、render-thread forwarding、sim_runner wait/receive/tick/publish、main-thread snapshot consume、render bridge update、frame pacing sleep/yield 與 present 間隔。
- 把 blocking wait、OS timer sleep、vsync/present wait、channel backlog、mutex contention、logging/profiling overhead 與真正 CPU work 分開量測，避免把 idle wait 誤判成效能不足，也避免錯過真正造成掉幀的 pacing 問題。
- 新增 120 FPS SLO：10k render-backed units stress 場景在 release、profiling disabled 的驗收條件為平均 FPS 達 120 target，1% low FPS >= 119；短暫 lockstep/network jitter 必須被 buffering/catch-up/pacing 吸收，不應造成可見 dropped frame。
- 若 tracing 顯示 span 標錯，只修 span 不算完成；必須繼續用新增指標定位 FPS/1% low 未達標的實際 bottleneck，並修正 frame pacing、handoff 或 render/sim pipeline 問題。
- 不更動 KCP/lockstep protocol、simulation deterministic state、script ABI 或 gameplay timestep；所有優化不得透過丟 TickBatch、跳 simulation tick、停止 render entity update 來假裝達標。

## Capabilities

### New Capabilities

無。

### Modified Capabilities

- `render-sim-cadence`: 明確要求 omfx 以可驗收的 120 FPS/1% low SLO 驅動 stress 場景，並要求 diagnostics 能把 frame pacing、sim_runner、handoff、snapshot consume/render update 與 present wait 的耗時拆開，找到並修正無法穩定 120 FPS 的真因。

## Impact

- `omfx/game/src/sim_runner.rs`: 調整 wait/receive/tick/publish tracing，加入 backlog、catch-up、idle wait、active work 與 publish handoff 指標。
- `omfx/game/src/native.rs`: 細分 `Plugin::update` 中 lockstep event drain、snapshot consume、render bridge/entity update、UI/VFX、frame pacing/sleep/yield 與 present 前後的時間；修正造成 120 FPS/1% low 未達標的 pacing 或 render pipeline 問題。
- `omfx/executor` / run scripts: 可能需要新增 release stress profiling workflow 或 frame-time summary output，但預設 stress 驗收需 profiling disabled，避免 trace overhead 污染結果。
- `omoba_core::lockstep_timing`: 繼續作為 120 TPS / tick period 的 source of truth。
- 驗證：TD_STRESS_10K 或等價 10k units stress 場景 release run，收集 frame-time histogram / 1% low / sim TPS / queue backlog，確認 1% low >= 119 且沒有靠跳 tick 或停更 render 達成。
