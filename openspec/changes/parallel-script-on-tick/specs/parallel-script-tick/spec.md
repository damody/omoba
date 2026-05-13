## ADDED Requirements

### Requirement: Script on_tick compute is parallelizable
Runtime SHALL support executing eligible `UnitScript::on_tick` invocations in parallel by using a read-only world view plus per-invocation command buffers. Parallel execution SHALL NOT require mutable ECS storage access during the compute phase.

#### Scenario: scripted towers run without shared mutable ECS borrow
- **WHEN** 1000 tower entities with `ScriptUnitTag` are ready for `on_tick` in the same tick
- **THEN** runtime can evaluate their `UnitScript::on_tick` decisions in parallel
- **AND** no worker directly mutates shared ECS component storage during that compute phase

#### Scenario: compute phase reads a stable tick snapshot
- **WHEN** multiple scripts read position, faction, cooldown, buffs, or target search data during the same tick
- **THEN** each script observes the same tick-start read model for other entities
- **AND** mutation produced by another script in that same compute batch is not visible until deterministic apply

### Requirement: Script mutations are represented as deterministic outcomes
Every gameplay or render-side mutation requested by script `on_tick` SHALL be recorded as deterministic script outcomes or existing `Outcome` variants, then applied by a single ordered apply stage. This includes position, facing, cooldown, projectile spawn, direct damage, splash damage, buff/stat mutation, attack phase cues, fire cues, and explosion cues.

#### Scenario: projectile spawn is deferred to outcome apply
- **WHEN** a script calls `spawn_projectile_ex` during parallel `on_tick`
- **THEN** the call records a projectile spawn outcome with all data needed to spawn the projectile
- **AND** the actual ECS entity creation happens during deterministic outcome apply

#### Scenario: same-script read-your-writes is preserved locally
- **WHEN** a script calls `set_asd_count` and later reads `get_asd_count` in the same `on_tick` invocation
- **THEN** the adapter returns the value from that invocation's local overlay
- **AND** other scripts do not observe that write until outcome apply

### Requirement: Outcome merge order is stable
Runtime SHALL merge per-script command buffers into the global outcome stream using a deterministic order independent of worker scheduling. The order SHALL be derived from the deterministic tagged entity list and per-invocation command order.

#### Scenario: repeated runs produce same outcome order
- **WHEN** two runs process the same TickBatch sequence with the same world state and master seed
- **THEN** parallel `on_tick` produces the same ordered outcome stream in both runs
- **AND** state hash remains stable across runs

#### Scenario: worker scheduling does not affect order
- **WHEN** Rayon schedules ready scripts on different worker threads across runs
- **THEN** the merged outcome order remains based on the deterministic entity order
- **AND** not on the completion order of worker threads

### Requirement: Script RNG is deterministic under parallel execution
Script-facing RNG SHALL NOT depend on shared mutable RNG call order across worker threads. `rand_unit()` and equivalent script RNG APIs SHALL produce deterministic values from stable inputs such as master seed, tick, entity identity, and invocation-local operation index.

#### Scenario: rand_unit is stable across thread schedules
- **WHEN** a script calls `rand_unit()` during parallel `on_tick`
- **THEN** the returned value is identical for the same tick, entity, script invocation, and operation order
- **AND** it does not change when unrelated scripts run earlier or later on different worker threads

### Requirement: Profiling separates script compute and apply costs
Runtime SHALL expose profiling that separates parallel script compute time, deterministic script outcome apply time, ready script count, and per-script-id aggregate cost. omfx sim_runner logs SHALL make it possible to identify whether spikes come from compute or apply.

#### Scenario: TD_STRESS profile shows script phases
- **WHEN** TD_STRESS runs with profiling enabled
- **THEN** logs or trace spans include script compute duration
- **AND** logs or trace spans include script outcome apply duration
- **AND** logs retain per-script-id aggregate timing for the top script costs

### Requirement: Backend and local sim remain lockstep-equivalent
Authoritative `omb` runtime and `omfx` local sim_runner SHALL use equivalent script parallelization semantics and outcome apply ordering. Given the same initial state, master seed, content, and TickBatch sequence, both runtimes SHALL produce matching deterministic gameplay state.

#### Scenario: omb and omfx apply same script outcomes
- **WHEN** backend and local sim_runner process a tick with the same scripted tower decisions
- **THEN** both produce equivalent script outcome data
- **AND** both apply outcomes in the same deterministic order
- **AND** resulting gameplay state remains lockstep-compatible
