## ADDED Requirements

### Requirement: stress render cadence meets 120 FPS 1% low SLO

omfx native frontend SHALL sustain the shared lockstep/render cadence in a 10k render-backed units stress scenario. The release, profiling-disabled acceptance run SHALL report average FPS at the shared 120 FPS target and 1% low FPS >= 119 over the fixed measurement window.

#### Scenario: 10k units stress reaches 120 FPS target
- **WHEN** `TD_STRESS_10K` 或等價 10k render-backed units stress scenario runs in release mode with Perfetto/deep tracing disabled for the configured measurement window
- **THEN** frame-time summary reports average FPS >= 120 target minus normal measurement tolerance
- **AND** 1% low FPS is >= 119
- **AND** sim_runner processed TPS remains near shared `LOCKSTEP_TPS`
- **AND** queue backlog does not grow unbounded

#### Scenario: passing result does not fake correctness
- **WHEN** the stress SLO passes
- **THEN** implementation has not lowered `LOCKSTEP_TPS`, skipped simulation ticks, dropped TickBatch payloads, stopped updating render-backed entities, or reduced the configured unit count to satisfy the metric
- **AND** render-visible entity movement/HP/UI state continues to update from current simulation snapshots or equivalent runtime state

### Requirement: frame-time diagnostics expose 1% low and pacing causes

omfx SHALL provide low-overhead frame-time diagnostics suitable for release stress acceptance. The diagnostics SHALL include enough data to identify whether missed 120 FPS cadence comes from active CPU work, frame pacing sleep/yield overshoot, present/vsync wait, sim-to-render handoff, lockstep backlog, or logging/profiling overhead.

#### Scenario: release stress summary includes low-overhead SLO metrics
- **WHEN** omfx runs a stress measurement window with profiling disabled
- **THEN** diagnostics include frame count, average FPS, p50/p95/p99 frame time, 1% low FPS, max frame time, sim TPS, latest sim tick, and queue backlog summary
- **AND** diagnostics are emitted at low frequency, not once per entity or once per frame unless explicitly requested for debugging

#### Scenario: pacing wait is distinguishable from active work
- **WHEN** a frame misses the 8.33ms budget while active render/sim work appears below budget
- **THEN** diagnostics expose frame pacing sleep/yield requested duration versus actual elapsed duration or equivalent overshoot data
- **AND** diagnostics expose present/vsync wait or engine frame-cap wait when available
- **AND** these waits are not reported as active render or sim CPU work

### Requirement: sim runner receive diagnostics separate idle wait from active work

omfx sim_runner SHALL distinguish blocking channel wait time from active TickBatch receive and tick processing time. `omfx::sim_runner::receive_tick_batch` or its equivalent active receive span SHALL NOT include time spent waiting for `tick_input_rx.recv_timeout` or any other blocking wait for the next lockstep TickBatch.

#### Scenario: healthy cadence wait is not reported as active receive work
- **WHEN** TD_1 或 stress scenario 在 lockstep 120 TPS 健康送出 TickBatch，且 sim_runner 等待下一個 batch
- **THEN** blocking wait time is reported under a wait/idle/starvation span or counter, not under `omfx::sim_runner::receive_tick_batch`
- **AND** `receive_tick_batch` active timing represents only already-received batch bookkeeping
- **AND** a normal 8.33ms cadence wait is not treated as a 6.33ms+ active work budget violation

#### Scenario: starvation remains observable
- **WHEN** sim_runner waits for TickBatch longer than the existing starvation threshold
- **THEN** omfx still emits the low-volume `sim_runner: no TickBatch in 1.0s` style diagnostic or an equivalent starvation signal
- **AND** the starvation wait is not included in per tick active receive or tick work averages

### Requirement: sim runner profile exposes backlog and catch-up state

omfx sim_runner SHALL expose low-volume profile diagnostics that make queue backlog and catch-up behavior visible. The diagnostics SHALL include enough information to distinguish healthy idle waiting, upstream TickBatch starvation, sim_runner falling behind active tick processing, and downstream publish/consume contention.

#### Scenario: profile window identifies healthy wait versus backlog
- **WHEN** omfx runs for at least one sim_runner profile window
- **THEN** `sim_runner_profile` or equivalent diagnostics include processed tick count and latest tick
- **AND** diagnostics include queue length or max queue length
- **AND** diagnostics include active receive/tick/publish timings separately from idle wait or wait count
- **AND** diagnostics include catch-up/backlog counters when queued TickBatches are processed without blocking wait

#### Scenario: queued TickBatches are processed without extra blocking wait
- **WHEN** `tick_input_rx` already contains more than one queued TickBatch
- **THEN** sim_runner processes the available queued batches in tick order without entering a blocking wait between those queued batches
- **AND** diagnostics report catch-up or backlog processing through low-volume counters
- **AND** no TickBatch is skipped, coalesced, or reordered

#### Scenario: diagnostics remain low-volume under stress
- **WHEN** stress scenario 或 `run_stress.bat` 持續執行
- **THEN** receive/backlog/frame diagnostics follow profile-window cadence
- **AND** diagnostics do not log once per tick, once per entity, or once per queued TickBatch except through opt-in tracing spans intended for profiler capture

### Requirement: optimization continues until missed 120 FPS root cause is fixed

If diagnostics show that the original `receive_tick_batch` measurement was primarily idle wait, implementation SHALL continue to identify and fix the next largest contributor preventing the stress SLO from passing. A change that only renames or moves spans SHALL NOT be considered complete unless the stress SLO passes or a documented external blocker prevents validation.

#### Scenario: span correction alone is insufficient when FPS still misses target
- **WHEN** `receive_tick_batch` active timing no longer includes idle wait but release stress still reports 1% low FPS < 119
- **THEN** implementation continues to use frame/sim/pacing diagnostics to identify the next bottleneck
- **AND** tasks are not marked complete solely because the receive span is corrected

#### Scenario: blocker is explicitly documented
- **WHEN** the SLO cannot be validated in the current environment because required stress assets, runtime duration, GPU/driver access, or toolchain constraints are unavailable
- **THEN** the implementation records the blocker, the best available partial metrics, and the next concrete measurement step
- **AND** build/test tasks still verify that instrumentation and code changes compile
