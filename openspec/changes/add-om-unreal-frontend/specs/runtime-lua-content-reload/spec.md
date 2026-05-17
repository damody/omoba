## ADDED Requirements

### Requirement: Development runtime can reload Lua content
`Om UE frontend` SHALL support Runtime Lua content reload in Editor and Development builds. Reload SHALL allow developers to modify Lua content under `scripts/lua_data`, trigger a reload without restarting UE Editor, and observe updated runtime catalog metadata in subsequent frames. Runtime Lua reload SHALL be disabled by default in Shipping builds unless a separate signed content patch mechanism is explicitly configured.

#### Scenario: Manual reload updates catalog generation
- **WHEN** developer changes a reloadable Lua content file and triggers the documented reload command
- **THEN** bridge MUST parse the Lua content in a background reload transaction
- **AND** bridge MUST publish a new Lua content generation/hash only after validation succeeds
- **AND** subsequent diagnostics and catalog acquire calls MUST expose the new generation/hash

#### Scenario: Shipping build rejects hot reload
- **WHEN** a Shipping build calls the Runtime Lua reload API without an explicitly enabled secure patch mode
- **THEN** bridge MUST reject the request with a stable unavailable/disabled status
- **AND** MUST NOT read arbitrary local Lua files or mutate the active catalog

### Requirement: Reload transaction preserves the last valid generation on failure
Runtime Lua reload SHALL be transactional. Parse errors, invalid metadata, missing required files, incompatible generated registry ids, or UHT class-surface changes SHALL fail the reload without replacing the currently active catalog or corrupting the running local replica.

#### Scenario: Parse error keeps old content active
- **WHEN** Lua reload encounters a syntax error or loader error
- **THEN** bridge MUST report the error with file/path context
- **AND** active Lua generation/hash MUST remain unchanged
- **AND** UE scene MUST continue using the previous valid catalog and frames

#### Scenario: Incompatible class-surface change requires codegen
- **WHEN** Lua reload detects a new content id requiring a generated UE C++ class, a removed active generated id, a changed generated class name, or a changed UHT-visible event payload shape
- **THEN** reload MUST fail with a diagnostic that says codegen and UE rebuild are required
- **AND** bridge MUST NOT publish the incompatible catalog generation

#### Scenario: Reloadable metadata change succeeds
- **WHEN** Lua reload changes numeric stats, story/map data, display metadata, render metadata, Blueprint soft path, animation metadata, buff visual metadata, or asset binding hints without changing generated C++ class surface
- **THEN** reload MUST succeed
- **AND** bridge MUST publish a new catalog generation/hash
- **AND** existing entity identities MUST remain stable when the local replica can continue safely

### Requirement: Bridge exposes reload control and diagnostics through C ABI
`om-bridge` SHALL expose C ABI functions or command DTOs to request Runtime Lua reload, query reload state, inspect the last reload result, and observe active Lua content generation/hash. Reload APIs SHALL use fixed-width status codes and FFI-safe strings or diagnostics snapshots; they SHALL NOT expose Rust Lua loader objects or Rust-owned containers.

#### Scenario: UE requests reload through C ABI
- **WHEN** UE runtime subsystem requests Lua content reload
- **THEN** it MUST call an exported bridge C ABI function
- **AND** the function MUST return whether reload was accepted, already running, disabled, or rejected due to invalid config

#### Scenario: Reload diagnostics are pollable
- **WHEN** reload is running or has completed
- **THEN** diagnostics MUST expose reload state, requested generation, active generation, last successful hash, last error summary, and whether codegen is required
- **AND** UE MUST be able to show this state without parsing bridge log text

### Requirement: Optional file watcher is debounced and development-only
`Om UE frontend` MAY provide an optional development file watcher for Lua dependencies. When enabled, it SHALL watch the configured Lua content root and included dependencies, debounce rapid changes, avoid reading files during partial writes, and submit reload requests through the same bridge reload transaction path used by manual reload.

#### Scenario: File watcher triggers one reload after a save burst
- **WHEN** an editor writes a Lua file multiple times during a save operation
- **THEN** watcher MUST debounce the change events
- **AND** MUST trigger at most one reload after the file state is stable
- **AND** reload MUST still pass through normal transaction validation

#### Scenario: File watcher can be disabled
- **WHEN** runtime config disables Lua file watching
- **THEN** `Om UE frontend` MUST NOT spawn a watcher for the Lua content root
- **AND** manual reload MUST remain available in Editor/Development builds when hot reload is enabled

### Requirement: UE invalidates and rebuilds catalog-derived caches after reload
When bridge publishes a new Lua content generation/hash, UE runtime SHALL invalidate catalog-derived caches and rebuild metadata views. This includes unit/ability/tower/buff metadata, animation metadata, Blueprint soft class path resolution, UI labels/icons, asset binding hints, and generated registry compatibility state. UE SHALL preserve active entity visuals when content id/class compatibility remains valid.

#### Scenario: UE updates Blueprint path cache after reload
- **WHEN** reload changes the Blueprint soft class path for `saika_magoichi`
- **THEN** UE runtime MUST invalidate the old resolved class cache for that content id
- **AND** future spawns of `saika_magoichi` MUST resolve the new Blueprint path
- **AND** existing actor handling MUST be deterministic: either keep current actor until respawn or explicitly rebind according to documented reload policy

#### Scenario: UE refreshes UI metadata
- **WHEN** reload changes tower cost, ability display name, buff display metadata, or icon binding
- **THEN** UE HUD/catalog views MUST use the new catalog generation after reload
- **AND** stale labels or icon bindings from the old generation MUST NOT remain indefinitely

#### Scenario: UE refreshes animation metadata
- **WHEN** reload changes idle variant names, locomotion variant names, buff animation overlay mapping, AnimBP variable mapping, attack montage path, critical attack mapping, or attack phase metadata
- **THEN** UE runtime MUST invalidate animation mapping caches for affected content ids
- **AND** subsequent animation state application MUST use metadata from the new catalog generation

#### Scenario: Missing content after reload uses fallback
- **WHEN** a frame references content that the new catalog cannot resolve
- **THEN** UE runtime MUST log the missing content id and generation
- **AND** MUST use explicit fallback visuals instead of crashing

### Requirement: Runtime Lua reload is verified end-to-end
The implementation SHALL include tests and smoke scenarios for manual reload, failed reload rollback, codegen-required diagnostics, UE cache invalidation, and optional watcher debounce. Verification SHALL distinguish reloadable metadata changes from generated C++ class-surface changes.

#### Scenario: Rust reload tests cover success and rollback
- **WHEN** `om-bridge` reload tests run
- **THEN** tests MUST cover successful metadata reload, syntax-error rollback, incompatible id/class change rejection, generation/hash increment, and diagnostics reporting

#### Scenario: UE reload smoke updates visible metadata
- **WHEN** a dev smoke test changes a reloadable Lua field such as display name, Blueprint path, animation metadata, buff visual metadata, or tower cost and triggers reload
- **THEN** UE runtime MUST observe the new generation
- **AND** relevant UI/actor/cache state MUST update without restarting UE Editor
