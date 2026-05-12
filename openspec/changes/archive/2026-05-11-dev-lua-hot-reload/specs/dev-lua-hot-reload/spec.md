## ADDED Requirements

### Requirement: DEV Lua hot reload is explicitly gated

系統 SHALL 只在 runtime Lua content 已啟用且 DEV hot reload 明確啟用時，才偵測並套用 `scripts/lua_data` 的執行中變更。啟用條件 SHALL 至少包含 truthy `OMB_LUA_CONTENT` 與 truthy `OMB_LUA_HOT_RELOAD`；release、stress 與未設定 hot reload 的 runtime SHALL NOT 啟動 watcher、poller 或 live apply pipeline。

#### Scenario: DEV launcher enables hot reload
- **WHEN** `run.bat` 啟動 debug gameplay，且設定 `OMB_LUA_CONTENT=1` 與 `OMB_LUA_HOT_RELOAD=1`
- **THEN** backend 會監看 configured Lua content root 的 relevant file changes
- **AND** Lua content-only 變更可在不 rebuild Rust artifacts 且不重啟 process 的情況下進入 reload pipeline

#### Scenario: Stress launcher does not enable hot reload
- **WHEN** `run_stress.bat` 啟動 release/stress gameplay
- **THEN** hot reload env 不會被設定為 truthy
- **AND** gameplay 繼續使用 stress launcher 的 release generated-data 行為
- **AND** 系統不會啟動 Lua hot reload watcher、poller 或 live apply pipeline

### Requirement: Lua changes produce validated content generations

系統 SHALL 在偵測到 Lua content root 變更後 debounce，重新載入 Lua content，驗證 builder、path safety、template/story data 與目前 live generation 的 structural compatibility，成功後產生 monotonically increasing content generation 與 deterministic content hash。Reload 失敗時系統 MUST 保留上一個有效 generation，且 MUST 輸出可診斷的錯誤原因。

#### Scenario: Numeric template change creates a new generation
- **WHEN** DEV hot reload 已啟用，且開發者修改既有 tower、hero、creep 或 ability id 的 numeric value
- **THEN** 系統會 debounce 該檔案變更並重新載入 Lua content
- **AND** 若新 content 通過 validation 與 structural compatibility check，系統會建立新的 content generation
- **AND** 新 generation 會有 deterministic hash 可供 backend、script DLL 與 omfx replica 比對

#### Scenario: Invalid Lua keeps previous generation active
- **WHEN** DEV hot reload 已啟用，且更新後的 Lua builder 語法錯誤、缺少必要欄位、路徑驗證失敗或 story/template validation 失敗
- **THEN** 系統 SHALL reject 該 reload
- **AND** active content generation SHALL remain unchanged
- **AND** gameplay SHALL continue using the previous valid content values
- **AND** log SHALL identify the rejected path, field, validation failure, or Lua error

#### Scenario: Structural content change requires restart
- **WHEN** DEV hot reload 已啟用，且 Lua 更新新增、刪除、重排 template/story/ability ids，或修改 active story 的 map topology、pathing topology、spawn wave topology
- **THEN** 系統 SHALL reject live apply for that reload
- **AND** active content generation SHALL remain unchanged
- **AND** log SHALL explain that the change is structural and requires restarting gameplay

### Requirement: Reload applies deterministically across simulation participants

backend SHALL be the authoritative scheduler for DEV hot reload application. When backend accepts a new content generation, it SHALL schedule an apply tick carrying the target generation/hash. omfx local replica and loaded script modules MUST apply the same generation/hash before simulating that tick; if any participant cannot load the matching hash, the reload MUST NOT be partially applied.

#### Scenario: Backend and omfx apply the same generation on the same tick
- **WHEN** backend accepts a DEV Lua reload and schedules generation `N` with hash `H` for tick `T`
- **THEN** backend SHALL apply generation `N` before simulating tick `T`
- **AND** omfx sim_runner SHALL reload from the same Lua content root and verify hash `H` before simulating tick `T`
- **AND** both simulations SHALL use generation `N` for script dispatch, active template lookups, and snapshot extraction at tick `T`

#### Scenario: Script module hash mismatch rejects reload
- **WHEN** backend accepts a DEV Lua reload candidate with hash `H`, but a loaded script module reports a different runtime Lua content hash or fails its DEV reload hook
- **THEN** backend SHALL reject the reload candidate
- **AND** active content generation SHALL remain unchanged for host and scripts
- **AND** log SHALL identify the failing script module or hash mismatch

#### Scenario: omfx hash mismatch stops local apply
- **WHEN** omfx sim_runner receives a scheduled DEV reload for hash `H`, but local reload produces a different hash or fails validation
- **THEN** omfx SHALL NOT continue simulating with a mismatched content generation
- **AND** frontend SHALL surface a clear DEV reload mismatch/error state instead of silently diverging

### Requirement: Accepted reload updates gameplay values without restarting

After an accepted DEV Lua reload,系統 SHALL refresh runtime data derived from Lua content so that future spawns, registries, snapshots, and conservative live entity base stats reflect the new generation without restarting the backend or frontend process.

#### Scenario: Future spawns use updated template values
- **WHEN** a DEV Lua reload changes an existing creep template's `hp`, `move_speed`, `armor`, `magic_resistance`, `exp_reward`, or `gold_reward`
- **THEN** the active creep emitters or equivalent spawn sources SHALL be rebuilt from the new generation
- **AND** creeps spawned after the reload SHALL use the updated template values

#### Scenario: Existing live entities refresh base stats conservatively
- **WHEN** a DEV Lua reload changes base stats for an existing hero, tower, or creep id that already has live entities in the world
- **THEN** system SHALL update applicable copied base components such as `CProperty`, `TAttack`, `TurnSpeed`, `CircularVision`, `Hero`, and tower template-derived fields from the new generation
- **AND** current HP SHALL preserve the pre-reload HP ratio relative to max HP when max HP changes
- **AND** runtime state such as buffs, cooldown timers, projectile lifetime, current orders, wave progress, and lockstep input history SHALL NOT be reset by the stats refresh

#### Scenario: Tower and ability registries refresh
- **WHEN** a DEV Lua reload changes existing tower metadata, tower upgrade definitions, ability metadata, ability constants, display text, icon path, or render metadata
- **THEN** `TowerTemplateRegistry`, `TowerUpgradeRegistry`, `AbilityRegistry`, and script-provided metadata snapshots SHALL be rebuilt or invalidated for the new generation
- **AND** future tower placement, upgrades, ability UI metadata, and snapshot metadata SHALL reflect the new generation

### Requirement: Frontend caches observe content generation changes

Snapshots or equivalent sim metadata SHALL expose the active content generation/hash to omfx frontend code. When frontend observes a new generation, it SHALL invalidate cached tower templates, tower upgrade definitions, ability metadata, and Lua-referenced assets that may have changed, then reseed them from the new snapshot/metadata.

#### Scenario: Tower sidebar updates after reload
- **WHEN** a DEV Lua reload changes an existing tower's display name, cost, range, icon, or render metadata and backend/omfx apply the new generation
- **THEN** the next frontend metadata update SHALL include the new content generation
- **AND** TD tower/sidebar caches SHALL be refreshed so the displayed values and icons match the new Lua data

#### Scenario: Asset cache invalidates on generation change
- **WHEN** a DEV Lua reload changes an existing ability icon path, tower texture path, hero model path, or other Lua-referenced asset path
- **THEN** frontend SHALL invalidate the affected cached resource lookup for the new generation
- **AND** subsequent rendering or UI access SHALL attempt to load the asset from the updated path

### Requirement: Non-DEV behavior remains unchanged

When DEV hot reload is disabled,系統 SHALL preserve existing runtime Lua content and generated-data behavior. Runtime Lua content mode without hot reload SHALL continue to load at initialization only, and generated-data release/stress paths SHALL NOT depend on hot reload APIs, watcher loops, or additional runtime Lua reload state.

#### Scenario: Runtime Lua mode without hot reload remains startup-only
- **WHEN** a binary is launched with `OMB_LUA_CONTENT=1` but without truthy `OMB_LUA_HOT_RELOAD`
- **THEN** Lua content SHALL be loaded during initialization as before
- **AND** later file changes under the Lua content root SHALL NOT be applied to the running game until restart

#### Scenario: Default generated-data runtime is unaffected
- **WHEN** a release or default runtime is launched without `OMB_LUA_CONTENT=1`
- **THEN** gameplay SHALL continue using generated Rust content APIs
- **AND** no DEV hot reload watcher, reload scheduler, script reload hook, or frontend cache invalidation path SHALL be required for normal operation
