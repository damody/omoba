## Purpose

定義 omfx render frame pacing 如何與 simulation cadence 對齊，避免 renderer 在 simulation 無法同速前進時 busy-render，同時保留 input、network event 與 sim_runner worker 的反應性。

## Requirements

### Requirement: omfx render pacing follows simulation cadence

omfx native frontend SHALL derive its default render pacing target from the shared lockstep simulation cadence, rather than rendering as fast as the engine can produce frames. The pacing target SHALL use `omoba_core::lockstep_timing` constants or helpers as the source of truth.

#### Scenario: render target derives from lockstep timing
- **WHEN** 檢查 omfx render pacing implementation
- **THEN** frame interval 使用 `LOCKSTEP_TPS`、`LOCKSTEP_TICK_PERIOD_US` 或等價 shared helper 推導
- **AND** render pacing path 不寫死 `60`、`120`、`16_667` 或 `8_333` 作為獨立 FPS/tick interval magic number

#### Scenario: renderer does not exceed sim cadence when healthy
- **WHEN** TD_1 在 backend 與 sim_runner 都健康接收 TickBatch 的狀態下執行至少 5 秒
- **THEN** `omfx_render` diagnostics 顯示 frontend render FPS 約等於 shared sim cadence，允許一般 timer jitter
- **AND** renderer 不會在同一段期間以遠高於 sim cadence 的 FPS busy-render

### Requirement: repeated snapshots do not drive unnecessary render work

omfx SHALL avoid repeatedly running expensive render-facing update work for the same `SimWorldSnapshot.tick` when no new sim tick is available and the render pacing interval has not elapsed. This SHALL reduce duplicate sprite/UI/VFX updates without delaying sim_runner snapshot publication.

#### Scenario: unchanged snapshot can be paced
- **WHEN** `Plugin::update` sees the same latest `SimWorldSnapshot.tick` as the previous rendered frame before the pacing interval has elapsed
- **THEN** omfx skips or defers expensive render-facing work for that update
- **AND** the next eligible frame renders the latest available snapshot, not an older cached snapshot

#### Scenario: new snapshot remains renderable immediately
- **WHEN** sim_runner publishes a `SimWorldSnapshot` with a tick greater than the previous rendered tick
- **THEN** omfx may render that snapshot on the next update even if the previous frame was paced
- **AND** pacing does not require waiting for an additional stale-frame interval before consuming the new tick

### Requirement: render pacing preserves input and network responsiveness

Render pacing SHALL NOT block lockstep event draining, input submission, auto smoke hooks, or sim_runner worker progress. Any sleep, yield, early-return, or engine frame cap SHALL be applied only after frontend has handled time-sensitive input/network work for the update.

#### Scenario: lockstep events still drain while render is paced
- **WHEN** lockstep client emits `TickBatch`, `InputSubmitted`, `Latency`, or `Disconnected` events during a paced render period
- **THEN** omfx drains and handles those events without waiting for the next render frame budget
- **AND** TickBatch payloads are forwarded to sim_runner promptly

#### Scenario: player input submission is not delayed by frame cap
- **WHEN** user input or an auto smoke hook submits a lockstep input while render pacing is active
- **THEN** omfx calculates and sends the input target tick using the latest observed sim tick
- **AND** render pacing does not add an extra full frame interval before submission

### Requirement: render pacing diagnostics are observable

omfx SHALL expose diagnostics that make render pacing behavior visible in existing frame logs or equivalent telemetry. Diagnostics SHALL include the target cadence and enough counters or timings to distinguish a paced renderer from a GPU-bound renderer.

#### Scenario: frame logs include pacing information
- **WHEN** omfx runs for at least one `FrameProfile` window
- **THEN** `omfx_render` or equivalent log includes target render cadence or frame interval
- **AND** the log includes paced/skipped/capped frame information or an equivalent measurement

#### Scenario: diagnostics remain low-volume
- **WHEN** TD_STRESS 或一般 dev run 持續執行
- **THEN** render pacing diagnostics follow the existing frame profile window cadence
- **AND** diagnostics do not log once per entity or once per skipped update
