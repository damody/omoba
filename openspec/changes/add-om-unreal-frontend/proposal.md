## Why

`omfx` 已經把 native frontend 的 gameplay replica 收斂到 `omoba-core::runtime`，但目前唯一完整前端仍綁在 Fyrox。新增 `Om UE frontend` 可以讓專案以 Unreal Engine 5.7 驗證更完整的 3D/工具鏈路線，同時沿用既有 lockstep runtime、script content 與 snapshot/render data contract。

UE C++/Blueprint 端不應每幀透過 JSON、protobuf 或大量 UObject 建立來同步狀態；需要一個明確的 Rust 動態函式庫 + C ABI header + pointer/atomic 的 render-frame 交換層，讓 UE 像 `omfx` 一樣以每幀低延遲讀取本地 replica 的最新 render state。

除此之外，UE 端不應只拿到通用 entity frame 後再靠手寫 switch 綁視覺。Lua content 已經是單位、技能、塔與 render metadata 的 source of truth；`Om UE frontend` 需要像 `omoba-template-ids` 產生 Rust 一樣，由 Rust codegen 從 Lua 產生 UE C++ class、typed Blueprint events 與 content registry，讓設計者在 Editor 只要建立 Blueprint 繼承對應 generated C++ class，就能覆寫角色、塔、技能與 VFX 表現，初始化時也能依 content id 自動載入正確 Blueprint class。

開發期還需要快速迭代 Lua content。設計者修改 hero/tower/buff/story Lua 後，`Om UE frontend` 應可在 Development/Editor runtime 重新載入 Lua content、發布新的 catalog generation/hash，並讓 UE 端刷新 metadata/asset binding，而不必每次完整重啟 Editor；但新增或改名會影響 UHT C++ class surface 的內容仍必須重新 codegen 與 UE build。

角色動作也不能只靠 UE 從位移推測。Rust runtime/bridge 應發布每個單位的 animation intent/state，例如 idle/stand 變體、walk、attack、CriticalAttack，以及攻擊前搖、命中點、後搖與 normalized progress。這個狀態也要能被 buff/modifier 影響，例如 `sniper_mode` active 時 walk 會切成狙擊模式走路，而不是普通 walk。UE 端將這些 state 套到 AnimBP 變數、state machine 或 montage；Blueprint 仍可覆寫視覺，但不得改變攻擊時序或命中結果。

第一個完整內容驗收目標是 `saika_magoichi`。UE C++ 不應只能從 Blueprint 事件間接取得 Saika 的技能與動作狀態；generator 需要為 `AOmHeroSaikaMagoichi` 與四個技能 `sniper_mode`、`saika_reinforcements`、`rain_iron_cannon`、`three_stage_technique` 產生 C++ 可讀 metadata、typed payload structs、native virtual/`BlueprintNativeEvent` handler 與 runtime dispatch glue，讓 UE C++ 與 Blueprint 都能接收技能 cast/toggle/passive proc/summon/transform/multi-shot/attack phase/animation overlay 事件。

## What Changes

- 使用既有 `<UE_PROJECT_ROOT>` Unreal Engine 5.7 C++ 空專案作為前端工程根目錄；目前專案已包含 `om.uproject`、`Source/OmGame` game module、Game/Editor targets、Config/Content。
- 在既有空專案內新增 `Plugins/OmRuntime` plugin/module，而不是重建 project scaffold；現有 `Source/OmGame` 保持薄 game host module。
- UE 5.7 engine root 預設使用 `D:\UE5.7`；build/open/package scripts MAY allow `UE_5_7_ROOT` 或 `UE_ROOT` override，但不應要求使用者每次手動指定。
- 新增 Rust shared library crate（暫定 `om-bridge`），封裝 `omoba-core::runtime`、KCP lockstep client、sim runner 與 render-frame publication，不讓 UE 直接連結 Rust internal crate 型別。
- 使用 `cbindgen` 從 Rust `extern "C"` API 產生 C header，供 UE 5.7 C++ module 透過動態連結呼叫。
- 定義 C ABI 穩定的 handle、slice、string、entity/frame DTO、input command 與 diagnostics API；跨 DLL 邊界不暴露 Rust `Vec`、`String`、`Arc`、trait object 或泛型型別。
- 建立 pointer + atomic 的雙緩衝或環狀緩衝 render-frame exchange：Rust sim thread 寫入 immutable frame buffer，發布 `sequence`/`published_index` atomic；UE game/render thread 每幀 acquire 最新 frame pointer 並以 snapshot lease 方式讀取。
- 規劃 UE 端 Actor/Subsystem/Component 分層：`UOmRuntimeBridgeSubsystem` 管理 DLL lifecycle，scene actor/component 消費 frame data 並更新 Unreal actors / Niagara / UI，input layer 透過 C ABI 回送 lockstep input。
- 新增 Rust UE code generator（暫定 `om-codegen`），重用 `scripts/lua_data` loader，從 Lua manifest 產生 UHT 可編譯的 C++ base/concrete classes、typed event structs、dispatch glue 與 generated content registry。
- 擴充 Lua authoring schema，讓 hero/tower/ability/summon/creep 可宣告 UE generated class 名稱、預設 Blueprint asset path、可覆寫的 Blueprint events、VFX/animation cue binding 與 fallback visual。
- UE Editor workflow 改為「Blueprint 繼承 generated C++ class」：例如 `BP_SaikaMagoichi` 繼承 `AOmHeroSaikaMagoichi`，設計者在 Blueprint 裡覆寫 `OnAttackPhase`、`OnAbilityCue`、`OnFrameState` 等事件。
- 遊戲初始化時，UE runtime 使用 generated registry 依 content id / generated class / Blueprint path 自動載入對應 Blueprint class；缺失時 fallback 到 generated native C++ class 並記錄診斷。
- 新增開發模式 Runtime Lua content reload：UE 或 bridge 可手動觸發、或由 file watcher 偵測 `scripts/lua_data` 變更後重新載入 Lua content，發布新的 catalog generation/hash，並在 UE 端 invalidates/rebuild catalog-derived caches。
- 新增動畫狀態同步：frame 包含 per-entity `AnimationState`，描述 locomotion、locomotion variant、active animation overlay、idle variant、action state、attack phase、phase progress、action instance id、critical flag 與 animation tag；generated C++/Blueprint/AnimBP adapter 將其映射到 AnimBP state machine。
- 設計 UE buff visual lifecycle：Rust bridge 從 `BuffStore`/frame buff state 對每個 entity 做 diff，發布 `BuffAdded`、`BuffRemoved`、`BuffRefreshed`、`BuffUpdated` typed events；generated C++/Blueprint 事件讓設計者在 buff 加上時建立 Niagara/材質/音效特效，在 buff 移除或 entity despawn 時清理。
- 將 `scripts/script-abi/src/script.rs` 的 `#[sabi_trait] UnitScript` 事件 hook 全部鏡射成 UE generated C++/Blueprint visual events；Blueprint 繼承 generated unit class 後可覆寫這些事件做視覺效果，但不得改變 gameplay state。
- 以 `saika_magoichi` 作為第一個完整 hero integration：產生 `AOmHeroSaikaMagoichi`、Saika ability visual classes、Saika typed event payloads 與 native C++ event handlers，涵蓋四個技能與 action/animation 事件。
- 定義 Windows 開發 build/staging 流程：Rust 1.91.0 建置 bridge DLL，複製 DLL/header/import lib 到 UE plugin 第三方目錄，UE `.Build.cs` 設定 include/library/runtime dependency。
- 加入 smoke/contract tests：header generation、ABI layout checks、DLL load/unload、sim frame publication、UE module link/build，以及不依賴 `omb` crate 的邊界檢查。

## Capabilities

### New Capabilities
- `om-unreal-frontend`: 定義 `<UE_PROJECT_ROOT>/` UE 5.7 前端工程、module/plugin 分層、啟動流程、輸入提交、render state 消費與與既有 backend/runtime 的依賴邊界。
- `c-abi-render-frame-bridge`: 定義 Rust 動態函式庫、`cbindgen` header、C ABI 型別、pointer lease、atomic sequence publication、記憶體所有權與跨語言同步規則。
- `lua-generated-ue-classes`: 定義 Rust 從 Lua content 產生 UE C++ classes、Blueprint inheritance surface、typed visual events、content registry 與 runtime Blueprint auto-load 的需求。
- `ue-buff-visual-events`: 定義 buff lifecycle diff、C ABI buff event payload、generated Blueprint buff events、active buff effect handle 管理與清理規則。
- `unit-script-blueprint-events`: 定義 `UnitScript` lifecycle/combat/resource/order/modifier hooks 到 C ABI render events 與 generated Blueprint events 的對應、payload 與視覺-only 限制。
- `runtime-lua-content-reload`: 定義開發期 Runtime Lua content hot reload、reload transaction、generation/hash、UE cache invalidation、與需要重新 codegen/UE build 的邊界。
- `ue-animation-state-machine`: 定義 Rust-authored animation state、buff/modifier animation overlays、攻擊前搖/命中/後搖 phase、AnimBP 變數映射、Lua animation metadata 與 Blueprint override surface。
- `saika-magoichi-ue-integration`: 定義 SaikaMagoichi hero-specific UE C++ metadata、技能事件、被動/召喚/變身/multi-shot cues、動作事件與驗收 smoke。

### Modified Capabilities
- 無。

## Impact

- `<UE_PROJECT_ROOT>/`: 使用既有 UE 5.7 C++ project；保留 `Source/OmGame` game module，新增 `Plugins/OmRuntime` runtime/generated/editor modules、`.Build.cs`、runtime subsystem、actor/component、input/UI/render adapters、UnitScript event dispatcher、buff visual effect manager 與 Blueprint loading registry。`.vs/`、`Intermediate/`、`Saved/`、`.sln` 視為 UE/VS 產物。
- `omoba-core`: `om-bridge` 會消費既有 `omoba-core::runtime`、KCP protocol type、lockstep timing 與 render-facing snapshot/delta data；若現有 Rust snapshot 型別不適合 C ABI，新增 bridge-local projection，而不是把 C ABI 汙染到 gameplay runtime。
- `om-bridge` / Rust workspace: 新增 `cdylib`/`staticlib` 目標、C ABI surface、FFI-safe DTO、atomic frame store、UnitScript event cue queue、input queue、diagnostics 與 `cbindgen` 設定。
- `om-codegen`: 新增 Rust codegen 工具，讀取 `scripts/lua_data`，輸出 UE UHT-compatible C++ headers/sources、registry、UnitScript hook event declarations、buff visual classes/events、manifest 與 generated-code freshness metadata。
- Runtime Lua reload: 新增 dev-only bridge reload API、Lua dependency watcher/manual reload command、reload diagnostics、content compatibility checks、catalog republish 與 UE cache invalidation flow；Shipping build 預設停用任意 Lua hot reload。
- Animation state sync: 新增 `AnimationState` frame DTO、animation catalog metadata、buff/modifier animation overlay resolution、AnimBP adapter/base component、attack phase transition handling、critical attack flag 與 animation reload/cache validation。
- SaikaMagoichi integration: 新增 content-specific generated C++ native API、四個技能的 readable metadata 與 event payload projection、ability event frame/cue array、C++/Blueprint dispatch tests。
- `scripts/script-abi`: `UnitScript` trait 本身仍維持 ABI；`Om UE frontend` 需要在 runtime hook dispatch 邊界鏡射事件，不把 UE/Blueprint 型別加入 script ABI。
- `scripts/lua_data`: 擴充可選 UE visual/codegen metadata 欄位；未宣告時 generator 產生安全 fallback class、fallback Blueprint path 與 generic buff visual binding。
- `scripts/base_content`: 沿用既有 script DLL/content；`Om UE frontend` 不重新實作 gameplay rules。
- `omb`: 不新增 `Om UE frontend -> omb` crate dependency；backend 仍只透過 existing KCP/lockstep protocol 與本地 replica 同步。
- Build scripts: 新增或擴充 Windows `.bat`/PowerShell pipeline 來建置 Rust bridge、產生 header、stage DLL/import lib 到 UE plugin，並保持 `.bat` CRLF 行尾。
- UE engine path: Windows scripts 預設使用 `D:\UE5.7`，並在該路徑缺失時提供清楚 diagnostic；`UE_5_7_ROOT`/`UE_ROOT` 只作為 override。
- 測試/CI: 新增 ABI/header smoke、Rust FFI contract tests、UE module compile check 與 stress-oriented frame publication diagnostics。
