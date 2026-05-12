## ADDED Requirements

### Requirement: lockstep input path is not starved by high-volume events

omb transport SHALL prioritize lockstep frames and player input handling over high-volume non-input game event broadcasts. Under TD_STRESS-scale ordinary event traffic, `TickBatch` delivery and `InputSubmit` processing SHALL NOT wait behind an unbounded backlog of `creep.M`, `Creep.H`, `entity.F`, heartbeat, or equivalent render-only / legacy events.

#### Scenario: TickBatch bypasses ordinary event backlog
- **WHEN** the outbound transport has a large backlog of ordinary game events
- **AND** a `TickBatch` is queued for lockstep clients
- **THEN** the `TickBatch` is sent through a priority path or otherwise processed before the ordinary backlog can delay it by seconds
- **AND** ordinary events may be deduped, delayed, or bounded without blocking the lockstep frame

#### Scenario: urgent flush does not carry unbounded ordinary events
- **WHEN** a lockstep frame or urgent input-related event triggers an immediate flush
- **THEN** the flush does not first process an unbounded number of ordinary events
- **AND** the lockstep frame remains within the intended low-latency path

### Requirement: TD_STRESS move input remains responsive at 400+ creeps

In TD_STRESS or an equivalent synthetic stress test with at least 400 active creeps and high-frequency ordinary event production, a local `MoveTo` input SHALL be accepted, drained into its target tick, delivered to the client sim, and reflected in applied input metadata without second-scale delay caused by transport or broadcaster backlog.

#### Scenario: MoveTo under stress avoids second-scale delay
- **WHEN** TD_STRESS has at least 400 active creeps
- **AND** the player sends a `MoveTo` input
- **THEN** omb accepts or explicitly rejects the input according to target tick rules within the normal lockstep budget
- **AND** if accepted, the corresponding `input_id` appears in `TickBatch.inputs[]` and omfx applied input metadata without waiting more than 1000ms because of non-input event backlog

#### Scenario: late input remains explicit
- **WHEN** a stress input is rejected because `target_tick <= current_tick`
- **THEN** omb logs the late rejection with player id, input id, target tick, and current tick
- **AND** the rejection is distinguishable from transport starvation or client-side pending backlog

### Requirement: priority diagnostics preserve determinism boundary

Any new metrics, queue age measurements, stale input counters, priority queue counters, or backlog logs added for this change SHALL remain outside deterministic gameplay state. They SHALL NOT be read by ECS gameplay systems, script ABI, outcome processing, or state hashing.

#### Scenario: diagnostics do not enter gameplay state
- **WHEN** searching gameplay component, tick, script ABI, and state hash paths for new latency or queue metric field names
- **THEN** matches are absent from deterministic gameplay state
- **AND** allowed matches are limited to transport, lockstep wire-edge metadata, omfx pending input diagnostics, tests, and logs
