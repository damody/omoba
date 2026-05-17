## 1. Scaffold 與建置骨架

- [x] 1.1 以既有 `<UE_PROJECT_ROOT>` UE 5.7 C++ project 為基底，保留 `om.uproject`、`Source/OmGame`、`OmGame.Target.cs`、`OmGameEditor.Target.cs`、`Config/` 與 `Content/`。
- [x] 1.2 新增 `<UE_PROJECT_ROOT>/bridge/Cargo.toml`，建立 `om_bridge` `cdylib` crate 並接上 `omoba-core` path dependencies。
- [x] 1.3 新增 `<UE_PROJECT_ROOT>/bridge/cbindgen.toml`，定義 `om_bridge.h` 產生規則與 include guard。
- [x] 1.4 新增 `<UE_PROJECT_ROOT>/codegen/Cargo.toml`，建立 `om_codegen` Rust binary。
- [x] 1.5 新增 `<UE_PROJECT_ROOT>/Plugins/OmRuntime/OmRuntime.uplugin`，定義 `OmRuntime`、`OmGenerated` 與 optional `OmEditor` modules。
- [x] 1.6 建立 `OmRuntime`、`OmGenerated`、optional `OmEditor` module source 目錄與 `.Build.cs`。
- [x] 1.7 建立 UE plugin ThirdParty staging：`Source/ThirdParty/OmBridge/include`、`Source/ThirdParty/OmBridge/lib/Win64` 與 `Binaries/Win64`。
- [x] 1.8 更新 `om.uproject` 啟用 `OmRuntime` plugin，同時保留既有 `ModelingToolsEditorMode` plugin entry。
- [x] 1.9 保持 `Source/OmGame` 薄 game host module；只有需要 game-specific bootstrap 時才在 `OmGame.Build.cs` 加入 plugin dependency。
- [x] 1.10 不手動編輯 `.vs/`、`Intermediate/`、`Saved/` 或把 `om.sln` 作為 source；project files 由 UE/UBT 重新產生。
- [x] 1.11 建立最小 `.Build.cs` linking/staging contract，讓 UE module 可 include staged header、compile generated C++ source 並 delay-load/stage `om_bridge.dll`。

## 2. Lua-to-UE Codegen

- [x] 2.1 抽出或複用 `omoba-template-ids` 的 Lua content loader，讓 `om_codegen` 可讀取 `scripts/lua_data/templates.lua` 與 include dependencies。
- [x] 2.2 設計 Lua optional `ue` metadata schema，支援 generated class override、Blueprint soft path、visual event bindings、fallback visual hints、animation binding、buff visual binding 與 editor category。
- [x] 2.3 實作 class name sanitization、duplicate detection、tombstone handling、default Blueprint path derivation 與 invalid path validation。
- [x] 2.4 產生 `OmContentIds.h/cpp`，包含 content id、numeric id、display name、generated class name 與 Blueprint path lookup。
- [x] 2.5 產生 UnitScript hook event metadata，列出所有 mirrored hook、event kind、Blueprint event name 與 payload struct name。
- [x] 2.6 產生 buff registry metadata，包含 buff id、display name、generated visual class、Blueprint soft path、default attach/effect policy、animation overlay mapping 與 lifecycle event flags。
- [x] 2.7 產生 animation registry metadata，包含 idle variants、locomotion variants、buff/modifier overlays、overlay priority、AnimBP variable mapping、state/montage/section soft paths、attack phase mapping、critical attack binding、default play rate 與 fallback policy。
- [x] 2.8 產生 codegen freshness manifest，記錄 Lua input hash、generator version、output file list 與 generated content hash。
- [x] 2.9 在 freshness manifest 中記錄 Lua dependency graph 與 class-surface signature，供 Runtime Lua reload 判斷哪些變更需要 codegen/UE rebuild。
- [x] 2.10 產生 SaikaMagoichi content-specific metadata，包含 hero stats/render/animation source、muzzle bone、ability slot order、四個技能 typed extras 與 generated native handler names。
- [x] 2.11 新增 `om_codegen` tests，覆蓋 deterministic output、identifier validation、default metadata、duplicate class name、UnitScript hook coverage、Saika ability metadata、animation metadata validation、buff visual metadata validation、reload compatibility signature 與 stale output detection。

## 3. Generated UE Classes 與 Registry

- [x] 3.1 實作手寫 UE base classes：`AOmUnitActor`、`AOmHeroActor`、`AOmTowerActor`、`UOmAbilityVisual` 與必要 visual component base。
- [x] 3.2 產生 hero/tower/creep/summon generated `UCLASS(Blueprintable)` classes，例如 `AOmHeroSaikaMagoichi`、`AOmTowerDart`。
- [x] 3.3 產生 ability generated visual classes，例如 `UOmAbilitySniperModeVisual`，並接上 ability/cue dispatch metadata。
- [x] 3.4 產生 buff generated visual classes 或 registry entries，例如 `UOmBuffSlowVisual`，並接上 buff lifecycle dispatch metadata。
- [x] 3.5 產生 animation generated metadata/types，例如 `FOmAnimationStatePayload`、generic locomotion/action/attack phase enum、idle variant id 與 AnimBP binding lookup。
- [x] 3.6 產生 `USTRUCT(BlueprintType)` event payloads，對應 frame state、animation state、attack phase、tower fire、ability cue、buff added/removed/refreshed/updated、UnitScript hook events 與 projectile cue。
- [x] 3.7 在 generated classes 宣告 `BlueprintImplementableEvent` 或 `BlueprintNativeEvent`，讓 Blueprint 可覆寫 `OnFrameState`、`OnAnimationState`、`OnAttackPhase`、`OnAbilityCue`、`OnBuffAdded`、`OnBuffRemoved`、`OnScriptAttackStart`、`OnScriptDamageTaken` 等視覺事件。
- [x] 3.8 產生 `OmRegistry`，提供 content id/buff id/animation tag id → generated native class/visual binding/animation metadata → Blueprint soft class path → fallback class lookup。
- [x] 3.9 產生 SaikaMagoichi native C++ API：`AOmHeroSaikaMagoichi` metadata getters、`FSaika*` typed payloads、`HandleSaikaSniperModeChanged`、`HandleSaikaReinforcementsCast`、`HandleSaikaRainIronCannonProc`、`HandleSaikaThreeStageChanged`、`HandleSaikaActionEvent` 等 native/BlueprintNativeEvent handlers。
- [x] 3.10 確認 generated `.generated.h` include order 與 UHT macro 格式可被 UE 5.7 編譯。

## 4. C ABI 基礎

- [x] 4.1 定義 `OM_ABI_VERSION`、status enum、opaque runtime handle、config struct 與 diagnostics struct。
- [x] 4.2 實作 `om_runtime_create`、`om_runtime_start`、`om_runtime_stop`、`om_runtime_destroy` 與 `om_last_error`。
- [x] 4.3 在所有 exported functions 加入 null pointer、struct size、ABI version 與 panic boundary 檢查。
- [x] 4.4 產生 `om_bridge.h`，確認 header 不暴露 `Vec`、`String`、`Arc`、`Box` 或 Rust generic type。
- [x] 4.5 新增 C/C++ smoke source，驗證 generated header 可獨立 include 與 type-check。
- [x] 4.6 新增 Runtime Lua reload C ABI：request reload、query reload state、last reload result、active Lua generation/hash 與 disabled/unavailable status。

## 5. Frame Store 與資料模型

- [x] 5.1 定義 `FrameHeader`、`FrameEntity`、removed ids、animation states、active buff snapshots、buff lifecycle events、UnitScript event cues、FX cue、input metadata 與 generated catalog id DTO。
- [x] 5.2 定義 `RuntimeCatalog`、string table、unit/ability/tower/upgrade/animation metadata DTO 與 generation/hash 欄位。
- [x] 5.3 實作 Rust-owned ring buffer frame slots、sequence、published slot、reader count 與 dropped publish counters。
- [x] 5.4 實作 `om_acquire_latest_frame` / `om_release_frame`，以 acquire/release ordering 管理 frame lease。
- [x] 5.5 實作 catalog acquire/release API，讓 UE 可在 generation 變更時重建 metadata cache。
- [x] 5.6 實作 buff snapshot string-table/payload projection，避免 UE 直接讀 Rust `BuffStore` 或持有 raw JSON pointer。
- [x] 5.7 實作 UnitScript event cue FFI projection，涵蓋 hook kind、entity refs、target refs、amount/damage、skill/state/modifier/order string table refs 與 tick/sequence。
- [x] 5.8 實作 animation state FFI projection，涵蓋 locomotion、locomotion variant、animation overlay/stance、idle variant、action state、attack phase、action instance id、phase elapsed/duration/progress、critical flag、animation tag、target refs 與 play rate。
- [x] 5.9 實作 ability event/cue FFI projection，涵蓋 event kind、caster、ability catalog id、level、target、action/cue instance id、payload schema id、string-table payload 與 Saika typed projection 所需欄位。
- [x] 5.10 在 catalog DTO/diagnostics 中加入 Lua content reload generation/hash、class-surface signature、reload state 與 codegen-required flag。
- [x] 5.11 新增 frame lease tests，覆蓋 slot reuse、slow consumer、destroy with outstanding lease、ability event arrays、animation arrays、buff arrays、UnitScript event arrays、reload metadata 與 empty array safety。

## 6. Runtime Driver

- [x] 6.1 萃取或實作 frontend-agnostic lockstep client/runner，不讓 `om-bridge` 依賴 `omfx` crate。
- [x] 6.2 接上 KCP `join_lockstep`、`GameStart`、`TickBatch`、state hash、network byte counters 與 RTT diagnostics。
- [x] 6.3 接上 `omoba-core::runtime` world initialization、script registry loading、dispatcher、input queue drains 與 tick loop。
- [x] 6.4 將 local replica dynamic state 投影為 `Frame`，包含 entity scalars、removed ids、round/lives、FX 與 applied input metadata。
- [x] 6.5 將 ability/tower/unit/render/animation metadata 投影為 `RuntimeCatalog`，並支援 Lua content generation/hash 變更。
- [x] 6.6 將 `BuffStore` active state 投影為 all-visible-entity buff snapshots，包含 remaining、payload hash、buff id/catalog id 與 visual instance key。
- [x] 6.7 實作 buff lifecycle diff，產生 `BuffAdded`、`BuffRemoved`、`BuffRefreshed`、`BuffUpdated` 與 `OwnerRemoved` cleanup reason。
- [x] 6.8 在 UnitScript hook dispatch 邊界 capture render-only cues，涵蓋所有 event hooks 並排除 `unit_id`/`tower_metadata`。
- [x] 6.9 實作 `on_tick` cue coalescing，輸出 accumulated dt、hook count、first tick 與 latest tick。
- [x] 6.10 實作 animation state derivation，從 authoritative movement/combat/action timing 與 active buff/modifier overlays 產生 stand/walk/sniper_mode walk/attack/CriticalAttack 與 Windup/Impact/Recovery phase/progress。
- [x] 6.11 捕捉 SaikaMagoichi ability cues：`sniper_mode` toggle on/off、`saika_reinforcements` cast/summon links、`rain_iron_cannon` passive proc、`three_stage_technique` transform/multi-shot visual payload。
- [x] 6.12 對齊 bridge catalog ids 與 generated UE registry ids，確保 frame entity/cue/ability event/animation state/buff event/UnitScript event 可找到 generated class、animation metadata、Saika typed handler 或 visual binding。
- [x] 6.13 實作 runtime stop path，確保 network/sim threads 可退出且不會在 DLL unload 後繼續執行。
- [x] 6.14 實作開發模式 Runtime Lua reload transaction：背景載入 Lua、解析 include dependencies、計算 hash/generation、驗證後 atomically publish 新 catalog。
- [x] 6.15 實作 reload compatibility validation，區分 reloadable metadata/story/render/animation/Blueprint path 變更與需要 codegen/UE rebuild 的 id/class/event surface 變更。
- [x] 6.16 實作 reload failure rollback，確保 parse error、invalid metadata 或 incompatible class-surface change 不會取代上一個有效 generation。
- [x] 6.17 實作 optional Lua file watcher debounce，並讓 watcher 只提交與 manual reload 相同的 reload request path。

## 7. UE Runtime Module

- [x] 7.1 實作 `UOmRuntimeBridgeSubsystem`，載入 bridge DLL、解析 function table、檢查 ABI version 並管理 runtime handle。
- [x] 7.2 實作 UE config 來源，支援 server address、player name/player id、script DLL path、story/content path、world scale 與 Blueprint loading mode。
- [x] 7.3 載入 generated registry，檢查 generated content hash 與 bridge/catalog Lua generation/hash 是否相容。
- [x] 7.4 實作 diagnostics polling 與 log/HUD 狀態，顯示 disconnected/joining/connected/error、latest tick、sequence、dropped frames、active leases 與 missing Blueprint count。
- [x] 7.5 實作 active buff effect manager 基礎資料結構，依 visual instance key 追蹤 Blueprint-created effect handles/components。
- [x] 7.6 實作 shutdown/PIE end/plugin unload 清理，確保 frame/catalog lease 釋放後再 destroy runtime，且清除所有 active buff effects。
- [x] 7.7 實作 UE manual Lua reload command/Editor control，透過 C ABI 觸發 reload 並顯示 reload state、last error、active generation/hash。
- [x] 7.8 實作 UE catalog generation change handling：invalidate metadata/UI/Blueprint class caches、重建 catalog-derived views，並保留相容 entity visual identity。
- [x] 7.9 實作 animation mapping cache invalidation，catalog generation 變更時刷新 AnimBP variable/state/montage/phase mapping。
- [x] 7.10 在 Shipping build/config 預設停用 Runtime Lua reload，並對 reload request 回報明確 disabled diagnostic。

## 8. UE Scene、Blueprint Auto-load 與 Rendering Adapter

- [x] 8.1 實作 `AWorldBridgeActor` 或等價 component，每幀 acquire latest frame 並在 scope 結束 release。
- [x] 8.2 建立 `(entity_id, entity_gen)` entity registry，穩定更新位置、朝向、HP、owner、kind、attack range 與 generated content id。
- [x] 8.3 實作 content id → generated registry → Blueprint soft class path → fallback generated native class 的 class resolution。
- [x] 8.4 使用 `FStreamableManager` 或等價方式載入 Blueprint class，並以 content id cache resolved class。
- [x] 8.5 Spawn/update 對應 Blueprint actor 或 generated native fallback actor，並 dispatch typed frame/cue Blueprint events。
- [x] 8.6 Dispatch buff lifecycle events 到 generated actor/Blueprint，支援 add/remove/refresh/update 與 owner removed cleanup。
- [x] 8.7 Dispatch UnitScript events 到 generated actor/Blueprint，支援 spawn/tick/death/damage/skill/attack/resource/state/modifier/order/respawn events。
- [x] 8.8 Dispatch animation state 到 generated actor/AnimBP adapter，支援 stand_1/stand_2/stand_3、normal walk、sniper_mode walk、attack、CriticalAttack、Windup、Impact、Recovery 與 phase progress。
- [x] 8.9 Dispatch Saika ability events 到 `AOmHeroSaikaMagoichi` native C++ handlers，並支援 Blueprint override；涵蓋 sniper mode、reinforcements、rain iron cannon、three stage、multi-shot 與 action events。
- [x] 8.10 實作 action instance id gating，避免同一攻擊 action 每幀重播 montage 或重置 state machine。
- [x] 8.11 實作 removed entity despawn/hide/recycle，避免 stale selection、stale hit-test、殘留 buff effects 或 stale animation state。
- [x] 8.12 實作 backend 2D world units 到 UE `FVector` 的可設定轉換，不把 Fyrox `WORLD_SCALE` 寫入 bridge。
- [x] 8.13 為 high-cardinality creep/projectile/tower body 建立 instanced/batched 或 pooling path，避免每幀 UObject churn。
- [x] 8.14 實作 missing Blueprint/asset/buff visual/animation mapping fallback mesh/material/icon/effect 與明確 log。

## 9. UE Input 與 UI Adapter

- [x] 9.1 定義 C ABI input command DTO 或 per-action submit functions，涵蓋 move、attack、tower、item、start round、cast/upgrade ability。
- [x] 9.2 在 bridge 內將 C ABI input 轉為 shared `PlayerInput`，分配/回報 input id 並套用 target tick policy。
- [x] 9.3 實作 UE viewport hit position 到 backend world units 的轉換與 validation。
- [x] 9.4 實作 UE HUD/control click guard，避免 UI click 同時觸發 world command。
- [x] 9.5 將 applied input metadata 回寫到 frame/diagnostics，供 UE 顯示 latency。

## 10. Scripts 與 Staging

- [x] 10.1 新增 `Om UE frontend` build/stage `.bat`，依序執行 `om_codegen`、cargo build、cbindgen、copy DLL/import lib/header/generated manifest 到既有 `<UE_PROJECT_ROOT>` project plugin paths。
- [x] 10.2 確保新增 `.bat` 檔案使用 CRLF 行尾。
- [x] 10.3 stage `scripts/base_content.dll` 與必要 content/config path，並在 runtime config 中明確傳入。
- [x] 10.4 新增 `run_om.bat` 或文件化 UE open/build command，預設 engine root 為 `D:\UE5.7`，並支援 `UE_5_7_ROOT`/`UE_ROOT` override。
- [x] 10.5 在缺少 `D:\UE5.7` 且無有效 override 時提供清楚 diagnostic，Rust/codegen/header smoke 仍可獨立執行。
- [x] 10.6 文件化開發期 Lua reload 流程：manual reload command、file watcher config、哪些變更可 hot reload、哪些變更需要重新 codegen/UE build。
- [x] 10.7 文件化 animation authoring flow：Lua `ue.animation`、buff `ue.animation_overlay`、AnimBP 變數命名、stand variant、normal/sniper walk variant、attack phase、critical attack、montage/section mapping 與 reload 邊界。
- [x] 10.8 在 plugin/Source/OmGenerated C++ 變更後使用 `D:\UE5.7\GenerateProjectFiles.bat -project=<UE_PROJECT_ROOT>\om.uproject` 或等價 UBT 流程更新 project files，但不要求 hand-edit 或 check in `.sln`、`.vs/`、`Intermediate/`、`Saved/`。

## 11. Verification

- [x] 11.1 執行 `cargo test --manifest-path <UE_PROJECT_ROOT>/codegen/Cargo.toml`，驗證 Lua-to-UE codegen、class names、Blueprint paths、animation metadata、UnitScript hook coverage、buff visual metadata 與 freshness。
- [x] 11.2 執行 `cargo test --manifest-path <UE_PROJECT_ROOT>/bridge/Cargo.toml`，驗證 ABI、lifecycle、input validation、animation state projection、buff lifecycle diff、UnitScript event cue projection 與 frame lease。
- [x] 11.3 執行 build/stage script，確認 generated C++、`om_bridge.dll`、`.lib`、`.pdb` 與 `om_bridge.h` 產出並位於 plugin 預期路徑。
- [x] 11.4 執行 generated header freshness check，Rust ABI 變更但 header 未更新時必須失敗。
- [x] 11.5 執行 generated C++ freshness check，Lua content 變更但 UE generated source 未更新時必須失敗。
- [x] 11.6 使用預設 `D:\UE5.7` 或 configured override 對 `<UE_PROJECT_ROOT>/om.uproject` 執行 UE module build，包含 UHT compile generated classes；path missing 時明確標示 skipped 並印出 resolved engine root。
- [x] 11.7 建立或驗證至少一個 Blueprint 繼承 generated hero class，確認 Editor 可見 typed visual events 與 UnitScript hook events。
- [x] 11.8 建立或驗證至少一個 Blueprint 覆寫 generated buff added/removed events，確認 buff 加上會建立 tracked effect、移除會清理。
- [x] 11.9 建立 synthetic 或 TD_1 smoke 驗證 `on_attack_start`、`on_attack_landed`、`on_modifier_added`、`on_modifier_removed` 至少各觸發一次 Blueprint event。
- [x] 11.10 執行 TD_1 smoke：backend + `Om UE frontend` 可連線、join lockstep、發布 frame、自動載入 hero/tower Blueprint 或 fallback class，buff add/remove 與 UnitScript hook 事件可觸發，並提交至少一種 input。
- [x] 11.11 執行 animation state smoke：synthetic 或 TD_1 中讓 `saika_magoichi` 經過 stand_1/2/3、normal walk、sniper_mode walk、attack、CriticalAttack、Windup、Impact、Recovery，確認 AnimBP adapter 收到正確變數與 monotonic progress。
- [x] 11.12 執行 Saika native C++ smoke：UE C++ test class 讀取 Saika hero/ability metadata，並收到 `sniper_mode` toggle、`saika_reinforcements` summon、`rain_iron_cannon` proc、`three_stage_technique` transform/multi-shot、attack phase/action events。
- [x] 11.13 執行 stress-oriented smoke 或 synthetic frame test，確認 dropped frame、active leases、sim TPS、Blueprint load cache、animation mapping cache、ability event throughput、buff effect cleanup、UnitScript event throughput 與 UE frame update diagnostics 可觀測。
- [x] 11.14 執行 Runtime Lua reload Rust tests，覆蓋成功 reload、syntax error rollback、incompatible id/class change rejection、generation/hash 變更與 diagnostics。
- [x] 11.15 執行 UE reload smoke：修改 reloadable Lua metadata、animation metadata 或 Blueprint path，manual reload 後 UE 觀察到新 generation 且 metadata/cache 更新，不需重啟 Editor。
- [x] 11.16 執行 watcher debounce smoke，確認保存 burst 只觸發一次 reload，且 partial write 不會取代 active generation。
- [x] 11.17 執行 codegen-required smoke，新增/改名 generated class surface 時 reload 失敗並提示重新跑 `om-codegen` 與 UE build。
- [x] 11.18 驗證既有空專案邊界：`om.uproject` 保留原有 project identity、`Source/OmGame` 仍是薄 game module、`OmRuntime` plugin 已啟用，且 `.vs/`、`Intermediate/`、`Saved/`、`om.sln` 未被當成手寫 source。

## 12. Map Route / Path Visualization

- [x] 12.1 在 bridge frame ABI 加入 map route 與 route point arrays，從 `SimWorldSnapshot.paths` 發佈 Lua/world checkpoint route geometry。
- [x] 12.2 新增 Blueprintable UE route spline actor，能用程式化 route points 建立 `USplineComponent` 與 spline mesh segments，並暴露材質、mesh、寬度與高度偏移設定。
- [x] 12.3 讓 world bridge actor 每幀同步 route data，只有 route hash 或數量變更時才 spawn/update/destroy route actors。
- [x] 12.4 重新產生 cbindgen header、執行 bridge tests/build，並用 `run_ue.bat` smoke 驗證 UE 前端能載入 route ABI 與 spline route actor。

## 13. UE Gameplay Input Event Generator / Rust Receiver

- [x] 13.1 設計 UE gameplay input event payload 與 event generator surface，涵蓋 viewport、hotkey、HUD、Blueprint、C++ call 來源。
- [x] 13.2 定義 move、Shift/queued move、attack move、attack target、ability cast、ability upgrade、item use、place tower、upgrade/sell tower、tower target priority、start round 的 typed event 欄位與 validation rule。
- [x] 13.3 實作 UE input event generator 到 bridge 的 routing，確保 UI guard/consumed flag 可阻止同一次 click 同時送出 world event。
- [x] 13.4 擴充 C ABI input DTO/flags 或 per-action submit functions，支援 queued/Shift append、attack move、point/entity/no-target ability casts、ability upgrade、item use、tower placement metadata、tower management actions、target priority 與 clear status 回報。
- [x] 13.5 實作 bridge/Rust gameplay input event receiver，驗證 event shape、轉換成 shared `PlayerInput`、保留 queue flag、分配 input id 並 enqueue 到 lockstep/local replica。
- [x] 13.6 實作 receiver lifecycle/error handling，覆蓋 not-started、disconnected、busy/full queue、invalid ids、stale entity gen、non-finite coordinates 與 unsupported event kind。
- [x] 13.7 將 accepted/rejected/applied input metadata 暴露到 UE diagnostics 或 frame，讓 UE UI 可關聯 input id、target tick、observed tick、queue length 與 latency。
- [x] 13.8 新增 Rust receiver tests，覆蓋 move、Shift move、attack move、attack target、point/entity/no-target ability、ability upgrade、item use、place/upgrade/sell tower、target priority、start round、invalid payload、queue flag preservation 與 no-mutation rejection。
- [x] 13.9 新增 UE/synthetic smoke，確認 UE event generator 送出的 move、Shift move、attack move、attack、ability cast/upgrade、item use、place/upgrade/sell tower、target priority、start round 事件能抵達 Rust receiver 並取得 ack/status。

## 14. Custom RTS Camera

- [x] 14.1 設計 UE RTS camera controller/pawn/component surface，支援本地 presentation-only camera state，且不送 gameplay input 到 bridge。
- [x] 14.2 實作滑鼠靠近 viewport 邊緣時的 edge-scroll panning，支援上下左右與角落斜向移動。
- [x] 14.3 實作滑鼠滾輪 zoom in/out，支援 configurable min/max distance 或 FOV、zoom speed、smoothing 與穩定 focus/look-at 行為。
- [x] 14.4 實作 camera bounds，支援手動 map bounds 或從 map route/path data 推導 bounds，並在缺少 bounds 時使用安全預設。
- [x] 14.5 將 edge band、pan speed、zoom limits、zoom speed、pitch/yaw、smoothing、bounds behavior 暴露到 UE settings 或 Blueprint defaults。
- [x] 14.6 整合 UI capture/window focus guard，確保 UI 捕捉滑鼠、視窗失焦或 cursor 離開 viewport 時不觸發 edge scroll。
- [x] 14.7 確認 RTS camera input 與 gameplay input event generator 隔離，edge scroll/wheel zoom 不會誤送 move、attack、ability、tower 或 start-round command。
- [x] 14.8 新增 synthetic/UE smoke，覆蓋四邊與角落 edge scroll、wheel zoom、bounds clamp、UI capture suppression、finite transform 與 no-gameplay-submit。

## 15. UE TD/HUD/Overlay Event Surfaces

- [x] 15.1 設計 C++/Blueprint payload、delegate/event surface 與 source enum，涵蓋 TD control、HUD、entity overlay、diagnostics，並明確區分 presentation event 與 gameplay input event。
- [x] 15.2 補 tower shop selection 與 start-round request 事件，包含 tower catalog id、content id/name、display label、cost、current gold、affordability、input source 與 UI consumed state。
- [x] 15.3 補 tower placement preview/confirmation 事件，包含 backend-world point、footprint radius、attack range、validity、invalid reason、tower catalog id、player id 與 route-to-input-generator contract。
- [x] 15.4 補 selected tower changed 與 tower management request 事件，包含 previous/new entity ref、owner、upgrade levels、range、target priority、clear reason、upgrade path/level、sell request 與 target priority request。
- [x] 15.5 補 hero HUD、ability HUD、item hotbar 與 buff list state events，讓 Blueprint UI 可讀取 health、lives、gold、XP、skill points、cooldown、ability level、upgrade availability、tooltip data、buff remaining 與 payload summary。
- [x] 15.6 補 entity overlay state events，涵蓋 name/HP label、selection state、range overlay、upgrade level label、visibility/throttling reason，並在 removed/recycled entity 時清除 stale overlay/selection。
- [x] 15.7 補 runtime diagnostics changed event，涵蓋 connection state、latest tick/sequence、sim TPS、published/dropped frames、active leases、input queue、network bytes、RTT、Lua generation/hash、missing Blueprint count 與 last error。
- [x] 15.8 新增 synthetic/UE automation，驗證 TD control events、HUD state events、entity overlay cleanup、diagnostics updates、UI guard no double-submit，以及 presentation events 不會直接 mutate gameplay authority。

## 16. Single-Player Unreal Debug Mode

- [x] 16.1 在 C ABI runtime config 增加 SinglePlayer flag，並讓 runtime-driver 在 SinglePlayer mode 不要求 server address。
- [x] 16.2 實作 Rust 本地 lockstep tick source，使用相同 script DLL、story id、local replica、frame projection 與 input receiver，不連 backend 也能發布 ticked frames。
- [x] 16.3 在 UE runtime settings 增加 RuntimeMode，預設 SinglePlayer，並支援 `-om-single-player`、`-om-networked` 與 `OM_RUNTIME_MODE` override。
- [x] 16.4 更新 `run_ue.bat`：預設 SinglePlayer、不建置/啟動 backend；`--networked`/`--with-backend` 才建置與啟動 backend；headless/safe smoke 必須檢查本次 runtime startup marker。
- [x] 16.5 新增 bridge single-player integration smoke，驗證無 backend 時 runtime connected、local cadence、ticked frame、catalog 與 zero network rx bytes。
- [x] 16.6 執行 Rust tests、bridge build/header freshness、UE 增量編譯與 `run_ue.bat --headless-smoke --no-build`，確認 Unreal 前端預設能單機啟動。
