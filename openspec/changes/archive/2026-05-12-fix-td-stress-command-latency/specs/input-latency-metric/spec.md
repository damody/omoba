## ADDED Requirements

### Requirement: HUD Lag exposes pending input backlog

omfx latency diagnostics SHALL expose unpaired pending input backlog in addition to paired input p50/p99. When at least one local input is pending and its age exceeds the currently displayed paired p99, the HUD `Lag:` display SHALL include an explicit pending or blocked latency value derived from the oldest pending input age, so the UI cannot report only a low paired latency while a newer player command is visibly stuck.

#### Scenario: pending input exceeds paired p99
- **WHEN** paired latency samples have `p99 = 46ms`
- **AND** a local `MoveTo` input remains unpaired for at least 1000ms
- **THEN** HUD `Lag:` includes a pending or blocked value of at least 1000ms
- **AND** HUD does not present `46ms` as the only latency signal

#### Scenario: no pending input preserves paired latency display
- **WHEN** `InputLatencyMeter` has paired samples and no pending inputs
- **THEN** HUD `Lag:` continues to show paired `p50` and `p99` values
- **AND** existing `input_render_latency:` paired sample logs remain available

### Requirement: stale and evicted inputs are observable

omfx SHALL log and count local inputs that remain pending past the stale threshold or are evicted without pairing. The diagnostic output SHALL include input id, action kind, target tick, base tick, pending age, and any known phase timestamps. Stale or evicted inputs SHALL NOT be silently dropped from the latency story.

#### Scenario: stale pending input produces diagnostic log
- **WHEN** a local input remains pending past the configured stale threshold
- **THEN** omfx emits a log line identifying the input id, action kind, target tick, base tick, and pending age
- **AND** the stale input count increases

#### Scenario: evicted input is excluded from paired samples but counted
- **WHEN** housekeeping evicts an unpaired input
- **THEN** the input is not added to paired p50/p99 samples
- **AND** the eviction count is updated
- **AND** a diagnostic log explains that the input did not pair

### Requirement: TD_STRESS phase trace identifies backlog source

For paired and stale inputs, phase diagnostics SHALL make the dominant backlog source distinguishable among client submit, server queue, TickBatch receive-to-forward, sim publish, render pair, and pending-unpaired age. TD_STRESS analysis SHALL be possible from logs without requiring cross-machine clock synchronization.

#### Scenario: paired input logs include backlog phases
- **WHEN** a TD_STRESS input pairs successfully
- **THEN** `input_latency_phase:` or equivalent log includes client submit cost, server queue cost, receive-to-forward cost, publish-to-pair cost, server receive tick, server drain tick, base tick, and sim latency ticks
- **AND** phase durations use only client-local deltas or server-computed durations

#### Scenario: unpaired input still identifies missing phase
- **WHEN** an input is stale before it appears in any applied snapshot metadata
- **THEN** diagnostics show the last known completed phase
- **AND** diagnostics identify the input as unpaired rather than reporting a misleading paired latency
