## Context

目前 omfx 的 UI event 會在 `on_os_event()` 中組成 `PlayerInput`，再透過 `send_lockstep_input()` 丟給 `lockstep_client` background thread。background thread 已用約 2ms 的 timeout drain outgoing inputs，因此目前主要瓶頸不像是 input sender 沒有 240Hz polling，而是 lockstep target tick、TickBatch 到主 thread 的 handoff、sim_runner snapshot publish，以及 server/client cadence 不一致。

omb 目前有兩個 cadence：`omb/src/main.rs` 的 authoritative dispatcher 是 `TPS = 30`，`TickBroadcaster` 則以 60Hz 送 `TickBatch`。omfx sim_runner 以 `SIM_DT_S = 1.0 / 60.0` 推進 local mirror，HUD `game_time` 也以 `/ 60.0` 換算。這讓 120Hz client mirror 不能只改一個常數，必須同步調整 server dispatcher、lockstep broadcaster、client sim dt、tick-to-seconds、log sampling 與保留視窗。

現有 `input-latency-metric` 只量到 submit-to-render pairing 的總延遲。它能證明 latency 存在，但不能指出多出的 5-20ms 是卡在 server queue、client TickBatch handoff、sim_runner publish，還是 render-side pairing。

## Goals / Non-Goals

**Goals:**

- 建立 per-`input_id` phase trace，讓每筆 input 可以拆出 client、server 與 sim/render handoff 各段耗時。
- 將 lockstep/client mirror cadence 提升到 120Hz，並以共享 `LOCKSTEP_TPS = 120` 消除散落的 60Hz magic numbers。
- 將 omb authoritative dispatcher 提升到 120Hz，讓 gameplay input apply 與 TickBatch cadence 對齊。
- 保留 deterministic sim 邊界，確保 timestamp、input_id 與 trace metadata 不進 ECS components、resources、outcomes 或 state hash。
- 維持預設 2 tick lookahead，但在 120Hz 下讓固定 input delay 從約 33.3ms 降到約 16.7ms。

**Non-Goals:**

- 不在此 change 實作 client-side prediction 或 rollback reconciliation。
- 不在此 change 調整 Android SurfaceFlinger / swapchain / buffer queue 的低延遲模式。
- 不把 discrete UI input 改成硬性 240Hz polling；目前優先保留 event-driven input，連續控制類 input 可在後續 change 處理。
- 不調整 gameplay balance；若 120Hz 暴露既有 tick-dependent gameplay 行為，應修正為 time-based，而不是重新平衡數值。

## Decisions

### Decision: 用共享 `LOCKSTEP_TPS` 驅動所有 lockstep timing

`LOCKSTEP_TPS` SHALL 定義在 client/server 都能使用的位置，例如 `omoba-core` 的 lockstep timing 模組。衍生常數包含 tick period、seconds per tick 與常用秒數換算 helper。omb、omfx 與測試不得各自寫死 `60`、`16_667`、`1.0 / 60.0` 或 `/ 60.0`。

Alternative considered：只把 `TickBroadcasterConfig.tick_period_us` 改成 `8_333`。這會讓 broadcaster 變快，但 omfx sim dt、HUD time、retention、state hash interval 與 server TPS 仍不一致，容易產生隱性 drift。

### Decision: authoritative server dispatcher 一起提升到 120Hz

`omb/src/main.rs` 的 `TPS` SHALL 從 30 改為 120，並將以 tick 數表示秒數的 intervals 一起縮放，例如 state hash 10 秒、snapshot 30 秒與 visibility diff interval。這讓 `State::tick()` consume host-input sidecar 的頻率跟 broadcaster 相同，避免 server authoritative action 仍以 30Hz apply。

Alternative considered：只把 client mirror 與 broadcaster 提升到 120Hz，server dispatcher 維持 30Hz。這會降低 client visual pairing delay，但 gameplay authority 仍可能延後到下一個 30Hz dispatcher tick，與使用者要求的「server 跟 client 更新頻率也拉高到 120hz」不符。

### Decision: phase trace 使用 client-local durations 與 server queue metadata，不依賴跨機 clock sync

client 端 phase 使用同一個 process 的 `wall_clock_us()` 或 monotonic timestamp 來源記錄：`on_os_event`、`send_lockstep_input`、`lockstep_client submit`、`client receive TickBatch`、`Game forward to sim`、`sim_runner publish snapshot`、`Game pair applied`。server 端不回傳絕對時間給 client 做跨機相減，而是回傳 `server_receive_tick`、`server_drain_tick` 與 `server_queue_us` 這種 server-local duration/metadata。

Alternative considered：直接把 server wall clock timestamp echo 給 client。這在 localhost 看似方便，但跨機 clock offset/skew 會污染 phase duration，容易讓 metric 誤導。

### Decision: trace metadata 只存在 wire-edge 與 omfx pending book

`input_id`、phase timestamps、server queue metadata SHALL 只存在 `InputSubmit` / `InputForPlayer` wire-edge、`InputBuffer` edge metadata、omfx `PendingInput` / `InputLatencyMeter` 與 log/HUD。任何 gameplay ECS system 不得讀它們，任何 state hash payload 不得包含它們。

Alternative considered：把 trace 寫入 sim ECS resource 方便 snapshot extraction。這會擴大 determinism surface，也讓 metadata lifecycle 跟 gameplay state 混在一起。

### Decision: 先保守使用 2 tick lookahead，再用 late metric 評估 1 tick

120Hz 下 2 ticks 約 16.7ms，已比 60Hz 的 2 ticks 約 33.3ms 少一半。實作後先保留 `INPUT_LOOKAHEAD_TICKS = 2`，並新增 late input counter/log；若 localhost 與目標裝置 late rate 足夠低，再另行調整到 1 tick。

Alternative considered：同時改成 1 tick lookahead。這會直接追求最低延遲，但如果 late input 上升，玩家會感覺操作偶發失效，debug 難度也更高。

## Risks / Trade-offs

- [Risk] 120Hz server dispatcher 使 CPU 成本約增加到 4 倍於 30Hz。→ Mitigation：先用 TD_1 與 TD_STRESS profile 驗證 `tick_profile` headroom，必要時保持 broadcaster 120Hz 但讓 heavy systems 節流。
- [Risk] 既有 gameplay code 可能隱含每 tick 固定加減，而非使用 `DeltaTime`。→ Mitigation：重點檢查 movement、buff、projectile、wave timing、attack cooldown 與 scripts 的 dt 使用，測試 120Hz 下遊戲速度不變。
- [Risk] proto 加 phase metadata 會增加 wire bytes。→ Mitigation：只在 `InputForPlayer` 加小型 numeric metadata，且只對 submitted inputs 產生，不影響空 TickBatch 大小太多。
- [Risk] timestamp log 量過大。→ Mitigation：保留 HUD p50/p99，每筆詳細 phase trace 只在 debug level 或 explicit env flag 下輸出。
- [Risk] Windows timer / tokio interval 精度造成 120Hz jitter。→ Mitigation：使用 derived duration，並以 runtime log 監控 TickBatch frames per 5 seconds 是否約 600。

## Migration Plan

- 第一步先加共享 cadence constants 與 phase trace data model，但保持現有頻率，確認 metric 正確。
- 第二步切換 lockstep broadcaster 與 omfx sim_runner 到 `LOCKSTEP_TPS = 120`，修正測試與 log sampling。
- 第三步切換 omb authoritative `TPS = 120`，同步調整 second-based intervals。
- 第四步跑 TD_1 latency smoke，確認 p50/p99 下降且 late input 沒有明顯增加。
- 若 120Hz regression 明顯，rollback 策略是將 `LOCKSTEP_TPS` 與 `TPS` 回到原值，phase trace 可保留作為診斷工具。

## Open Questions

- phase trace 詳細 log 是否預設只在 `RUST_LOG=debug` 顯示，或需要獨立環境變數例如 `OMFX_INPUT_TRACE=1`。
- `LOCKSTEP_TPS` 最終放在 `omoba-core` 還是 omb/omfx 共用的其他 crate；優先選擇不引入循環依賴的位置。
