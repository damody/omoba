## 1. Reproduction And Diagnostics

- [x] 1.1 Add or update a focused omb transport / broadcaster test that queues many ordinary events before a `TickBatch` and reproduces lockstep starvation with the current single-FIFO behavior.
- [x] 1.2 Add omfx latency-meter tests for a pending `MoveTo` input whose age exceeds paired p99, verifying HUD-facing latency exposes pending backlog instead of only paired samples.
- [x] 1.3 Add stale / evicted pending input diagnostics tests covering input id, action kind, target tick, base tick, pending age, and last known phase metadata.

## 2. Lockstep Priority Path

- [x] 2.1 Refactor `omb/src/transport/kcp_transport.rs` outbound handling so lockstep frames use a priority path or equivalent queue ordering ahead of ordinary `GameEvent` backlog.
- [x] 2.2 Bound urgent flush behavior so a lockstep frame cannot drain and encode an unbounded number of ordinary events before it is sent.
- [x] 2.3 Preserve ordinary event dedupe / batching for non-lockstep traffic while allowing ordinary events to be delayed, deduped, or bounded under TD_STRESS pressure.
- [x] 2.4 Add queue/backlog debug counters or logs for lockstep-priority sends and ordinary-event backlog without writing those metrics into deterministic gameplay state.

## 3. Latency Metric Correction

- [x] 3.1 Extend `InputLatencyMeter` or adjacent omfx diagnostics to track oldest pending input age, stale pending count, and evicted unpaired count.
- [x] 3.2 Update HUD `Lag:` formatting so paired p50/p99 remains visible, but pending or blocked latency is shown when it exceeds paired samples.
- [x] 3.3 Log stale and evicted pending inputs with enough phase metadata to distinguish client submit, server queue, TickBatch receive-to-forward, sim publish, render pair, and unpaired pending age.
- [x] 3.4 Preserve existing `input_render_latency:` and `input_latency_phase:` paired-sample logs so current grep-based analysis keeps working.

## 4. Stress Validation

- [x] 4.1 Add a synthetic stress regression that simulates 400+ creeps or equivalent ordinary event volume and asserts accepted `MoveTo` input reaches applied input metadata without second-scale non-input backlog delay.
- [x] 4.2 Verify late input rejection remains explicit with player id, input id, target tick, and current tick logs.
- [x] 4.3 Run `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` and relevant omfx tests for latency meter / transport behavior.
- [x] 4.4 Run `run_stress.bat` or a shorter TD_STRESS smoke and inspect `input_render_latency:`, `input_latency_phase:`, stale input logs, and HUD `Lag:` for consistency.

## 5. Determinism Guardrails

- [x] 5.1 Grep deterministic gameplay paths to confirm new latency / queue fields do not appear in ECS components, gameplay tick systems, script ABI, outcomes, or state hash payloads.
- [x] 5.2 Run the existing determinism or metadata guard tests affected by lockstep diagnostics.
