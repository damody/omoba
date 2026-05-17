## ADDED Requirements

### Requirement: `<UE_PROJECT_ROOT>/` 提供 UE 5.7 前端工程與 runtime plugin
系統 SHALL 使用既有 `<UE_PROJECT_ROOT>` Unreal Engine 5.7 C++ project 作為前端工程，並在其中新增負責載入 Rust bridge DLL 的 `OmRuntime` plugin/modules。UE C++ module SHALL 只透過 generated C header 與動態函式庫呼叫 Rust bridge，不得 include Rust crate source 或連結 Rust internal symbols。

#### Scenario: Existing empty UE project is preserved
- **WHEN** 檢查 `<UE_PROJECT_ROOT>`
- **THEN** 目錄 MUST 保留既有 `om.uproject`
- **AND** MUST 保留既有 `Source/OmGame` game module、`OmGame.Target.cs`、`OmGameEditor.Target.cs`、`Config/` 與 `Content/`
- **AND** implementation MUST NOT replace the project with a newly generated `.uproject`

#### Scenario: OmRuntime plugin is added to existing project
- **WHEN** 檢查 `<UE_PROJECT_ROOT>`
- **THEN** implementation MUST add `Plugins/OmRuntime/OmRuntime.uplugin`
- **AND** plugin MUST 包含可由 UE 5.7 build system 識別的 runtime module source 與 `.Build.cs`
- **AND** `om.uproject` MUST enable `OmRuntime` while preserving existing plugin entries such as `ModelingToolsEditorMode`

#### Scenario: Generated/cache UE artifacts are not source requirements
- **WHEN** 檢查 implementation artifacts
- **THEN** `.vs/`、`Intermediate/`、`Saved/` 與 `om.sln` MUST be treated as generated/cache artifacts
- **AND** build or verification MUST NOT require hand-editing those paths
- **AND** project files MAY be regenerated through `D:/UE5.7/GenerateProjectFiles.bat` or UBT when plugin/source files change

#### Scenario: Game module remains a thin host
- **WHEN** 檢查 `<UE_PROJECT_ROOT>/Source/OmGame`
- **THEN** `Source/OmGame` SHOULD remain the primary game host module with minimal bootstrap logic
- **AND** runtime bridge、generated registry、Blueprint loading、frame consumption 與 reload tooling MUST live in `OmRuntime` plugin modules unless game-specific bootstrap requires otherwise

#### Scenario: UE module consumes generated bridge header
- **WHEN** 檢查 `Om UE frontend` 的 UE C++ module
- **THEN** module MUST include generated `om_bridge.h`
- **AND** module MUST 透過 C ABI function table 或 extern declarations 呼叫 bridge
- **AND** module MUST NOT include Rust source files、`omoba-core` Rust module path 或 `omfx` Rust module path

#### Scenario: Third-party DLL is staged by Build.cs
- **WHEN** UE build system 編譯 `OmRuntime` plugin
- **THEN** `.Build.cs` MUST 設定 bridge include path、import library 或 delay-load DLL
- **AND** `.Build.cs` MUST 宣告 `om_bridge.dll` 的 runtime staging 規則
- **AND** staged header MUST be located under `Plugins/OmRuntime/Source/ThirdParty/OmBridge/include`
- **AND** staged import library MUST be located under `Plugins/OmRuntime/Source/ThirdParty/OmBridge/lib/Win64`
- **AND** staged DLL MUST be located under `Plugins/OmRuntime/Binaries/Win64`

### Requirement: `Om UE frontend` frontend dependency boundary
`Om UE frontend` SHALL 以 Rust bridge 與 `omoba-core::runtime` local replica 作為 gameplay state 來源。`Om UE frontend` UE module SHALL NOT depend on `omb` crate、spawn `omobab.exe`、呼叫 `cargo run` 啟動 backend，或假設 `omb/target/debug/omobab.exe` 存在。需要同機 backend 的 dev run SHALL 由 launcher script 負責。

#### Scenario: UE frontend does not own backend process lifecycle
- **WHEN** 搜尋 `<UE_PROJECT_ROOT>` 中的 backend startup logic
- **THEN** MUST 找不到 `cargo run`、`omobab.exe` hard-coded debug path 或 `omb/target` process spawn
- **AND** UE runtime startup MUST NOT 嘗試建置或啟動 backend
- **AND** Networked mode 在 backend 不存在時 MUST 回報 disconnected/error state
- **AND** SinglePlayer mode 在 backend 不存在時 MUST 啟動本地 lockstep tick source 與 local replica

#### Scenario: Gameplay state comes from local replica bridge
- **WHEN** `Om UE frontend` runtime 成功連線到 backend 並加入 lockstep session
- **THEN** UE scene state MUST 由 `om-bridge` 發布的 local replica render frames 更新
- **AND** UE module MUST NOT 直接向 backend request per-frame full snapshots 來驅動畫面

#### Scenario: Default direct launch uses single-player debug mode
- **WHEN** 開發者直接用 UE Editor 開啟 `Om UE frontend`、按 Play、或執行 `run_ue.bat --headless-smoke` 且未指定 networked override
- **THEN** runtime MUST 使用 SinglePlayer mode
- **AND** bridge MUST NOT require a server address or running backend process
- **AND** bridge MUST publish ticked local replica frames from Rust using the configured script DLL and story id
- **AND** diagnostics MUST expose connected state, local player id, lockstep cadence, zero network receive bytes, latest tick, and sim TPS when available

#### Scenario: Direct UE project launch is diagnosable without backend
- **WHEN** 開發者直接用 UE Editor 開啟 `Om UE frontend` 且沒有執行 `omb`
- **THEN** frontend MUST 完成 plugin/module 初始化
- **AND** UI 或 log MUST 顯示 bridge runtime mode 與 connection/local-replica 狀態
- **AND** Editor MUST NOT 因找不到 backend executable 而 crash 或退出

#### Scenario: Networked debug launch remains available
- **WHEN** 開發者執行 `run_ue.bat --networked` 或 `run_ue.bat --with-backend`
- **THEN** launcher script MUST build/start the Rust backend process before launching UE
- **AND** UE runtime MUST pass a networked override to the bridge instead of SinglePlayer mode
- **AND** launcher script MUST stop only the backend process it started when the UE process exits

### Requirement: UE runtime lifecycle is subsystem-owned
`Om UE frontend` SHALL 提供一個 UE runtime subsystem 或等價 singleton，負責 DLL 載入、ABI version check、runtime handle 建立、start/stop/destroy、config 套用、last error 查詢與 diagnostics polling。Rust worker threads SHALL NOT call Unreal APIs；所有 Unreal object 更新 SHALL 在 UE game thread 執行。

#### Scenario: Runtime starts from UE config
- **WHEN** UE runtime subsystem 初始化且 config 指定 runtime mode、server address、player name、script DLL path 與 story/content path
- **THEN** subsystem MUST 呼叫 bridge create/start API
- **AND** bridge MUST 使用該 config 啟動 networked lockstep client 或 single-player local tick source 與 local runtime

#### Scenario: ABI mismatch is rejected
- **WHEN** loaded bridge 回報的 ABI version 與 UE module 編譯時期期待版本不同
- **THEN** subsystem MUST 不啟動 runtime
- **AND** subsystem MUST 記錄可定位的 ABI mismatch error

#### Scenario: Stop and destroy are orderly
- **WHEN** UE world shutdown、PIE 結束或 plugin unload
- **THEN** subsystem MUST 呼叫 bridge stop/destroy
- **AND** bridge worker threads MUST 被要求停止
- **AND** UE MUST 不再持有任何 frame lease 後才釋放 runtime handle

### Requirement: UE scene consumes frame data without per-frame UObject churn
`Om UE frontend` SHALL 以每幀 acquire/release frame 的方式更新 UE scene。Entity identity SHALL use `(entity_id, entity_gen)`。UE SHALL reuse existing actors/components/instances when identity remains alive，並 SHALL process removed ids to despawn or hide stale visuals。高數量 entity 類型 SHALL 使用 instanced/batched representation 或等價機制，避免每幀重建大量 UObject。

#### Scenario: Entity registry updates stable entities
- **WHEN** frame N 與 frame N+1 包含相同 `(entity_id, entity_gen)`
- **THEN** UE bridge actor MUST reuse existing visual instance
- **AND** MUST 更新位置、朝向、HP、owner、kind 與其他 changed scalar state
- **AND** MUST NOT destroy and recreate the UObject solely because a new frame arrived

#### Scenario: Removed entity is cleared
- **WHEN** frame 包含 `removed_entity_ids` 或不再包含先前 alive entity
- **THEN** UE scene MUST remove、hide 或 recycle corresponding visual
- **AND** 後續點擊或 UI selection MUST NOT target stale entity

#### Scenario: Stress path avoids one actor per hot entity
- **WHEN** frame 包含大量 creep、projectile 或其他 high-cardinality dynamic entities
- **THEN** `Om UE frontend` MUST provide instanced/batched rendering path 或等價 pooling strategy
- **AND** frame update MUST NOT allocate a new UObject per entity per frame

### Requirement: UE map route visualization is generated from bridge path data
`Om UE frontend` SHALL expose map creep/path route data from the Rust bridge frame and generate UE spline route actors automatically. Designers SHALL be able to customize the route material or subclass the route actor, without hand-placing route geometry for each map.

#### Scenario: Frame path data creates spline route actors
- **WHEN** the bridge publishes a frame containing map route point arrays from Lua/world path checkpoints
- **THEN** the UE world bridge MUST create or update one route actor per route
- **AND** each route actor MUST build a `USplineComponent` plus spline mesh segments from the published points
- **AND** the points MUST use the same backend 2D world unit to UE `FVector` conversion as entity positions

#### Scenario: Route visuals are designer-customizable
- **WHEN** a designer changes the route actor class, mesh, width, vertical offset, or material in UE settings/Blueprint defaults
- **THEN** generated route geometry MUST keep using the authoritative path points
- **AND** the designer MUST NOT need to edit generated C++ or hand-place spline control points to change the route appearance

#### Scenario: Route data updates without per-frame component churn
- **WHEN** subsequent frames publish the same route points
- **THEN** UE MUST reuse existing route actors and components
- **AND** route spline meshes MUST rebuild only when route point data or route count changes

### Requirement: UE custom RTS camera navigation
`Om UE frontend` SHALL provide a local RTS camera navigation layer for map inspection. The camera SHALL support mouse-edge panning, mouse-wheel zoom, designer-configurable limits/smoothing, and optional map-bound clamping. Camera navigation SHALL be presentation-only and SHALL NOT submit gameplay input events through the bridge.

#### Scenario: Edge scroll moves the camera
- **WHEN** the mouse cursor is inside a configured edge band near the viewport border
- **THEN** the camera MUST pan in the corresponding map direction
- **AND** panning MUST stop when UI captures the pointer, the cursor leaves the viewport, or the window loses focus

#### Scenario: Wheel zoom changes camera distance
- **WHEN** the player scrolls the mouse wheel
- **THEN** the camera MUST zoom in or out within configured min/max limits
- **AND** the zoom operation MUST keep camera transforms finite and bounded

#### Scenario: Camera input is isolated from gameplay commands
- **WHEN** the player edge-scrolls or wheel-zooms the camera
- **THEN** UE MUST NOT submit move、attack、ability、tower 或 start-round gameplay commands solely because the camera moved
- **AND** gameplay commands MUST still flow through the gameplay input event generator and bridge receiver

### Requirement: UE input adapter submits lockstep commands through bridge
`Om UE frontend` SHALL 將 UE mouse/keyboard/UI input 轉成 C ABI command，透過 bridge 提交 lockstep input。UE module SHALL NOT construct protobuf bytes directly for gameplay input。Bridge SHALL assign or return input ids and apply the same low-latency target tick policy as existing native frontend path。

#### Scenario: UE event generator routes gameplay intent to bridge
- **WHEN** UE receives gameplay intent from viewport click、hotkey、HUD、Blueprint 或 C++ call
- **THEN** a gameplay input event generator MUST produce a typed event for move、Shift/queued move、attack move、attack target、ability cast、ability upgrade、item use、tower placement、tower upgrade/sell、tower target priority 或 start-round
- **AND** the event MUST flow through the bridge input receiver instead of mutating UE scene state as authority

#### Scenario: Move command is submitted through C ABI
- **WHEN** 玩家在 UE viewport 觸發 hero move command
- **THEN** UE input adapter MUST convert hit position to backend 2D world units
- **AND** MUST call bridge input API
- **AND** bridge MUST convert command to shared `PlayerInput` and submit it through lockstep client

#### Scenario: Shift move queues instead of replacing orders
- **WHEN** 玩家按住 Shift 或等價 queue modifier 觸發 move command
- **THEN** UE input adapter MUST submit the command with an explicit queued/append flag
- **AND** bridge/Rust receiver MUST preserve the flag when converting to shared `PlayerInput`
- **AND** non-queued move MUST retain the normal replacement semantics

#### Scenario: Tower actions use authoritative ids
- **WHEN** 玩家透過 UE UI 放塔、賣塔或升級 tower
- **THEN** UE input adapter MUST submit tower kind/entity/path values through bridge C ABI
- **AND** backend/local replica result MUST later appear through published frames
- **AND** UE MUST NOT optimistically mutate gameplay state outside the local replica result

#### Scenario: Ability, item, and tower priority actions use typed events
- **WHEN** 玩家透過 UE hotkey、HUD、Blueprint 或 C++ call 觸發 ability upgrade、item use、attack move 或 tower target priority
- **THEN** UE input adapter MUST submit the corresponding typed event through bridge C ABI
- **AND** bridge/Rust receiver MUST preserve the action kind instead of coercing it into generic move、attack 或 ability cast

#### Scenario: UI click does not double-submit map input
- **WHEN** 玩家點擊 UE HUD button 或 ability/tower UI control
- **THEN** `Om UE frontend` MUST submit only the intended command
- **AND** the same click MUST NOT also trigger world move、tower selection 或 ability cast path

### Requirement: UE TD/HUD/overlay event surfaces
`Om UE frontend` SHALL expose C++/Blueprint event surfaces for TD control presentation, HUD state, entity overlays, and runtime diagnostics. These events SHALL mirror the useful native-frontend UI states without making UE the gameplay authority. Any event that represents player intent to change gameplay state SHALL route through the gameplay input event generator.

#### Scenario: TD UI emits presentation and request events
- **WHEN** tower shop selection、placement preview、selected tower change、upgrade/sell/target-priority request 或 start-round request occurs
- **THEN** UE MUST emit typed C++/Blueprint events with catalog ids、entity refs、source、validity、cost/range/upgrade data、and consumed/guard state where applicable
- **AND** only request events may route to the bridge input receiver

#### Scenario: HUD and overlay events follow frame/catalog data
- **WHEN** hero stats、ability cooldown/level、buff list、entity health/name、selected range overlay 或 diagnostics change
- **THEN** UE MUST emit typed C++/Blueprint state events that are safe to consume after frame release
- **AND** removed/recycled entities MUST clear stale overlay and selection events before any new management request can be emitted

### Requirement: UE content catalog and asset mapping
`Om UE frontend` SHALL consume bridge-published catalog data for unit ids、ability definitions、tower templates、upgrade definitions、hero render metadata、animation metadata 與 asset paths。Catalog SHALL update when Lua content generation/hash changes。UE asset binding MAY map catalog paths to Unreal assets, but missing assets SHALL be diagnosable and use explicit fallback visuals instead of crashing.

#### Scenario: Initial catalog populates UE metadata cache
- **WHEN** bridge runtime publishes initial catalog
- **THEN** UE runtime MUST cache ability、tower、unit、render 與 animation metadata by stable ids
- **AND** scene/HUD rendering MUST use that cache for labels、icons、tower costs、upgrade names 與 asset bindings

#### Scenario: Lua content generation changes
- **WHEN** bridge reports a new Lua content generation/hash
- **THEN** UE runtime MUST invalidate stale catalog-derived caches
- **AND** subsequent frames MUST use the updated catalog metadata

#### Scenario: Missing asset uses fallback
- **WHEN** catalog references an asset path that has no UE binding
- **THEN** UE renderer MUST log the missing id/path
- **AND** MUST render an explicit fallback mesh/material/icon instead of crashing

### Requirement: Windows build, staging, and run workflow
系統 SHALL provide repeatable Windows commands to build the Rust bridge, generate the C header, stage bridge artifacts into the UE plugin, and run or open the `Om UE frontend` frontend. The default Unreal Engine 5.7 root SHALL be `D:\UE5.7`, with `UE_5_7_ROOT` or `UE_ROOT` allowed as overrides. Any new `.bat` files SHALL use CRLF line endings.

#### Scenario: Bridge build stages artifacts
- **WHEN** 開發者從 `D:/omoba` 執行 documented om bridge build command
- **THEN** command MUST build `om_bridge.dll`
- **AND** MUST generate `om_bridge.h`
- **AND** MUST stage DLL/import library/header to the UE plugin paths expected by `.Build.cs`
- **AND** header path MUST be `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h`
- **AND** import library path MUST be under `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/lib/Win64`
- **AND** DLL path MUST be under `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Binaries/Win64`

#### Scenario: UE build command is documented
- **WHEN** UE 5.7 is installed at `D:\UE5.7` or configured through documented environment variable overrides
- **THEN** 開發者 MUST be able to invoke a documented build/open command for `Om UE frontend`
- **AND** missing UE installation MUST produce a clear diagnostic rather than a partial silent failure

#### Scenario: Existing project is built through D drive UE install
- **WHEN** no engine override is set and `D:\UE5.7` exists
- **THEN** build/open/package commands MUST target `<UE_PROJECT_ROOT>/om.uproject`
- **AND** commands MUST use the resolved `D:\UE5.7` UBT/UHT/Editor tooling
- **AND** generated project-file refresh MUST use the existing `.uproject` rather than creating a second UE project

#### Scenario: Default UE engine root is D drive path
- **WHEN** no `UE_5_7_ROOT` or `UE_ROOT` override is set
- **THEN** `Om UE frontend` build/open/package scripts MUST resolve the UE engine root to `D:\UE5.7`
- **AND** diagnostics MUST print the resolved engine root before invoking UBT、UHT、Editor、or package commands

#### Scenario: UE engine root override is honored
- **WHEN** `UE_5_7_ROOT` or `UE_ROOT` is set
- **THEN** scripts MUST use the override instead of `D:\UE5.7`
- **AND** the diagnostic MUST identify that an override was used

#### Scenario: Batch files keep CRLF
- **WHEN** 新增或更新 `Om UE frontend` related `.bat` scripts
- **THEN** their line endings MUST be CRLF
- **AND** Windows cmd MUST NOT fail with truncated first-character command errors caused by LF-only files
