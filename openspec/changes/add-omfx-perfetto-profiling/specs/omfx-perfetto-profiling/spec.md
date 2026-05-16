## ADDED Requirements

### Requirement: opt-in Perfetto trace output
omfx native frontend SHALL provide an opt-in profiling mode that writes a Perfetto-compatible trace using `tracing-perfetto` or an equivalent `tracing` layer that produces Perfetto-readable output. Profiling SHALL be disabled by default and SHALL only be enabled through environment variables, launcher configuration, or an equivalent explicit user action.

#### Scenario: profiling disabled by default
- **WHEN** omfx executor starts without Perfetto profiling environment variables or launcher flags
- **THEN** no Perfetto trace file is created
- **AND** omfx continues using normal logging and gameplay behavior

#### Scenario: profiling enabled creates trace file
- **WHEN** omfx executor starts with profiling enabled and a writable output path
- **THEN** omfx initializes the Perfetto trace output before the game plugin begins normal execution
- **AND** a Perfetto trace file is written at the configured path or documented default path
- **AND** startup log records the trace path and mentions opening it with `ui.perfetto.dev`

#### Scenario: trace path failure does not block startup
- **WHEN** profiling is enabled but the configured output path or parent directory cannot be created
- **THEN** omfx logs a warning or error explaining the path failure
- **AND** the game still starts without Perfetto trace output

### Requirement: profiling configuration is explicit and documented
omfx Perfetto profiling SHALL expose documented controls for enablement, output path, and granularity. The controls SHALL include an enable flag equivalent to `OMFX_PERFETTO_TRACE`, an output path equivalent to `OMFX_PERFETTO_PATH`, and a granularity value equivalent to `OMFX_PERFETTO_DETAIL`.

#### Scenario: user configures output path
- **WHEN** user starts omfx with profiling enabled and a custom trace output path
- **THEN** omfx writes the trace to that path
- **AND** the startup log shows the same resolved path

#### Scenario: user omits output path
- **WHEN** user starts omfx with profiling enabled but no output path
- **THEN** omfx writes the trace to a documented default location under a build or target profile directory
- **AND** the generated file name distinguishes profiling sessions, such as by timestamp or process id

### Requirement: logging remains compatible with profiling
Perfetto profiling SHALL NOT remove or break existing omfx logging. Existing `log` output such as `omfx_app.log`, terminal logs, Fyrox `omfx.log`, `omfx_frame`, `omfx_render`, and `sim_runner_profile` SHALL remain available when profiling is disabled, and SHALL remain usable when profiling is enabled unless an explicit warning explains a degraded logging mode.

#### Scenario: profiling disabled preserves current logs
- **WHEN** omfx starts with profiling disabled
- **THEN** existing terminal and file logging behavior remains available
- **AND** existing frame and sim profile log lines can still be emitted

#### Scenario: profiling enabled does not duplicate or suppress logs
- **WHEN** omfx starts with profiling enabled
- **THEN** regular log messages are not duplicated due to logger/subscriber initialization
- **AND** existing text profile logs remain readable alongside the Perfetto trace

### Requirement: frontend frame spans expose major function boundaries
The trace SHALL include spans for major native frontend frame/update/render sections. At minimum, spans SHALL cover `Plugin::update`, automatic hooks or input submission, lockstep event drain, snapshot consumption, render bridge update, sim batch update, VFX/projectile update, camera update, UI update, and frame statistics recording.

#### Scenario: frame trace shows nested frontend sections
- **WHEN** a profiling-enabled omfx session runs for at least one rendered frame
- **THEN** the Perfetto trace contains nested slices for the major frontend sections within each `Plugin::update` frame
- **AND** those slices include timing durations visible in Perfetto UI

#### Scenario: spans include useful frame fields
- **WHEN** frontend frame spans are emitted
- **THEN** spans include fields such as frame number or lockstep tick, network entity count, projectile count, draw calls, triangle count, or other values that help correlate timing with scene complexity

### Requirement: sim runner spans expose tick pipeline timing
The trace SHALL include spans from the `omfx-sim-runner` thread for the local simulation tick pipeline. At minimum, spans SHALL cover `TickBatch` receive, input push/apply, dispatcher execution, pending queue drains, pre-script outcome processing, script dispatch, post-script outcome processing, snapshot extraction, render FX retention, and snapshot publish.

#### Scenario: sim runner trace shows tick pipeline
- **WHEN** profiling is enabled and lockstep `TickBatch` payloads are received
- **THEN** the Perfetto trace contains `omfx-sim-runner` thread slices for each major tick pipeline section
- **AND** each slice duration is visible in Perfetto UI

#### Scenario: sim spans include tick fields
- **WHEN** sim runner spans are emitted
- **THEN** spans include the lockstep tick and relevant counts such as pending input queue length, processed input count, runtime publish flag, entity count, or snapshot entity count where available

### Requirement: omoba-core spans expose shared runtime hot paths used by omfx
The trace SHALL include `omoba-core` spans for shared runtime and client paths that omfx native frontend directly uses. At minimum, spans SHALL cover pending queue drains, `process_outcomes`, `run_script_dispatch`, selected script `GameWorld` adapter work, runtime snapshot or metadata extraction used by `sim_runner`, and KCP lockstep client receive/send paths where those paths run in the omfx process.

#### Scenario: sim runner trace enters omoba-core runtime spans
- **WHEN** profiling is enabled and `omfx-sim-runner` processes a tick
- **THEN** the Perfetto trace contains nested `omoba-core` runtime spans under or adjacent to the sim runner tick pipeline
- **AND** those spans distinguish drain, outcome processing, script dispatch, and snapshot or metadata extraction work instead of showing only one opaque omfx call site

#### Scenario: omoba-core spans do not own Perfetto output
- **WHEN** `omoba-core` code emits tracing spans
- **THEN** `omoba-core` does not create Perfetto files, parse `OMFX_PERFETTO_*`, or install the Perfetto layer itself
- **AND** trace output is controlled by the omfx executor subscriber/layer initialization

#### Scenario: core instrumentation avoids default per-entity spans
- **WHEN** profiling is enabled with default granularity
- **THEN** `omoba-core` runtime instrumentation uses coarse hot path spans
- **AND** it does not emit one span per entity, script unit, projectile, or world adapter call by default

### Requirement: profiling granularity controls overhead
Perfetto profiling SHALL provide at least two practical granularity levels: a default low-overhead level for frame/tick sections, and a deeper level for selected inner-loop or per-entity diagnostics. Deep profiling SHALL require explicit opt-in separate from simply enabling trace output.

#### Scenario: default granularity avoids per-entity spans
- **WHEN** profiling is enabled with default granularity in TD_STRESS
- **THEN** trace output includes frame/tick hot path sections
- **AND** it does not emit one span per entity by default

#### Scenario: deep granularity is explicit
- **WHEN** user enables deep granularity through the documented control
- **THEN** omfx may emit selected inner-loop or per-entity spans
- **AND** documentation warns that deep granularity can perturb performance and produce large traces

### Requirement: profiling workflow is documented and verifiable
The project SHALL document how to run omfx with Perfetto profiling enabled, where the trace file is written, how to open it in Perfetto UI, and which trace tracks/spans correspond to frontend render work versus `omfx-sim-runner` work.

#### Scenario: user can follow documented command
- **WHEN** user follows the documented profiling command or launcher
- **THEN** a trace file is produced without manually editing source code
- **AND** the documented path points to the generated file

#### Scenario: normal verification covers profiling-disabled build
- **WHEN** executing the normal frontend build command
- **THEN** the build succeeds without requiring profiling environment variables
- **AND** profiling dependencies do not break wasm/android target declarations

#### Scenario: enabled verification confirms trace content
- **WHEN** executing a short profiling-enabled native run
- **THEN** the generated trace can be opened in Perfetto UI
- **AND** the trace contains frontend main thread spans, `omfx-sim-runner` spans, and `omoba-core` runtime/client spans when those paths are exercised
