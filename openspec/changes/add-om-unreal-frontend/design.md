## Context

`omfx` 目前以 `omoba-core::runtime` 在 frontend process 內跑本地 lockstep replica：背景 KCP client 收 `TickBatch`，sim runner 推進 ECS，render thread 每幀讀取最新 render-facing state。這個方向已經解掉 `omfx -> omb` crate dependency，也讓前端不需要從 backend 每幀拉完整狀態。

`Om UE frontend` 要走同一條架構線，但 UE 5.7 不能直接消費 Rust `SimWorldSnapshot`。現有 snapshot 型別含有 `Vec`、`String`、`Arc<Vec<_>>`、`Box` 與 Rust enum，這些都不是跨 DLL C ABI contract。UE 端需要的是一個穩定、版本化、可由 `cbindgen` 產生 header 的 C surface，以及每幀只用 pointer + acquire/release 同步讀取的 immutable frame。

UE 官方第三方 library 整合模型以 plugin/module `.Build.cs` 管理 include path、import library、delay-load DLL 與 runtime dependency staging。這與本專案的 Windows-first dev flow 相容，但要明確處理 DLL 搜尋路徑、`base_content.dll` staging、Rust toolchain 固定在 1.91.0，以及 `.bat` CRLF 行尾。

本 change 的 UE 5.7 安裝路徑預設為 `D:\UE5.7`。Windows build/open/package scripts 應先使用這個路徑；若使用者設定 `UE_5_7_ROOT` 或 `UE_ROOT`，則以環境變數覆寫。任何 UE build verification、UHT compile 或 Editor launch 都應在 diagnostic 中印出實際解析到的 engine root。

`<UE_PROJECT_ROOT>` 已經是 UE 5.7 C++ 空專案，不需要由本 change 重建 project。現況是 `om.uproject` 指到 engine association、`Source/OmGame` 只有薄 game module、`OmGame.Target.cs` / `OmGameEditor.Target.cs` 已使用 `BuildSettingsVersion.V6` 與 `EngineIncludeOrderVersion.Unreal5_7`，`Config/` 與 `Content/` 是 Unreal 專案內容。`.vs/`、`Intermediate/`、`Saved/` 與 `om.sln` 是 UE/Visual Studio 生成或快取產物，實作不應手動依賴或改寫它們。

另一個必要邊界是 UE class generation。`omoba-template-ids` 已經用 Rust build-time generator 讀 `scripts/lua_data`，輸出 Rust ids、stats、render metadata、hero abilities 與 tower upgrade lookup。`Om UE frontend` 也應該用同一份 Lua content 產生 UE C++ class surface：每個 hero/tower/ability/summon/creep 都有穩定 generated native class、typed Blueprint events 與 registry entry。設計者在 Editor 裡建立 Blueprint 繼承該 generated C++ class，修改 mesh、Niagara、動畫、材質、音效與 UI 表現；runtime 初始化時依 content id 自動載入對應 Blueprint class。

Buff 需要獨立設計。現有 runtime 的 `BuffStore` 是 `Entity -> buff_id -> BuffEntry { remaining, payload }`；同一 `buff_id` 重複施放通常是 refresh 或 payload update，不是移除再新增。UE 視覺層需要從這個 authoritative buff state 推導 lifecycle events，讓 Blueprint 在 `BuffAdded` 時掛特效，在 `BuffRemoved` 或 entity despawn 時移除特效，而不是每幀猜測要不要重建 Niagara 或材質狀態。

`UnitScript` 的 hook 也需要成為 Blueprint 視覺事件。`scripts/script-abi/src/script.rs` 的 `#[sabi_trait] pub trait UnitScript` 目前包含生成、tick、死亡、傷害、技能、攻擊、資源、狀態、modifier、order、respawn 等 hook。這些 hook 仍由 Rust script/runtime 做 gameplay authority；`Om UE frontend` 只要在同一 hook 發生時產生 render-only event cue，讓 generated C++ class/Blueprint 可以覆寫對應事件播放視覺效果。

動畫狀態需要獨立於事件 cue。`on_attack_start` 和 `on_attack_landed` 適合觸發特效與音效，但 AnimBP state machine 需要每幀可讀的連續狀態：stand 變體、walk、attack、CriticalAttack、攻擊前搖、命中點、後搖、phase progress 與 action instance id。這些 state 必須由 Rust runtime 根據 authoritative movement/combat timing 設定，UE 只把它映射到 AnimBP 變數、state machine 或 montage，不應從速度或事件順序自行推測 gameplay timing。Buff/modifier 也能改變 animation context，例如 `sniper_mode` active 時同樣是 `Walk` locomotion，但 locomotion variant/overlay 應變成 `sniper_walk` 或 `sniper_mode`，讓 AnimBP 切到狙擊模式走路。

SaikaMagoichi 需要成為第一個內容級驗收，而不是只靠 generic hooks。Lua 中 `saika_magoichi` 的技能順序是 `sniper_mode`、`saika_reinforcements`、`rain_iron_cannon`、`three_stage_technique`；script 實作分別對應 toggle buff、召喚鐵炮兵、被動普攻扇形真傷、變身與 multi-shot visual。UE C++ 必須能直接讀取這些技能 metadata 與 action/animation state，並以 native C++ handler 接收 typed events，再讓 Blueprint 選擇性覆寫視覺。

開發期 Lua content 必須能快速迭代。`Om UE frontend` 需要支援 Development/Editor runtime 重新載入 `scripts/lua_data`，更新 runtime content catalog、story/unit/tower/buff metadata 與 UE visual metadata，並把新的 Lua content generation/hash 推給 UE。這個 reload 只能處理不改變 UE C++ class surface 的資料變更；新增 id、改 generated class name、改 Blueprint parent surface 或 UHT-visible event shape 時，仍然需要重新跑 `om-codegen` 與 UE build。

## Goals / Non-Goals

**Goals:**

- 在既有 `<UE_PROJECT_ROOT>` UE 5.7 C++ 空專案內接入 runtime plugin、OmGenerated module、Rust bridge crate 與 codegen，而不是重建 project。
- 讓 `Om UE frontend` 與 `omfx` 一樣消費 `omoba-core::runtime` local replica，而不是依賴 `omb` crate 或每幀由 backend 傳完整 render state。
- 透過 `om-bridge.dll` 暴露穩定 C ABI，並用 `cbindgen` 產生 `om_bridge.h`。
- 用 Rust-owned immutable frame buffers、opaque pointer leases 與 atomic publication 讓 UE 每幀低拷貝讀取最新 dynamic render state。
- 分離冷資料 catalog 與熱資料 frame：tower/ability/asset metadata 只在初始化或 Lua content generation 改變時更新；entity scalar、removed ids、FX、round/lives、input latency 每 tick 更新。
- 新增 Rust `om-codegen`，像 `omoba-template-ids` 一樣讀取 Lua content，但輸出 UHT-compatible UE C++ generated classes、typed Blueprint event API 與 content registry。
- 讓 Blueprint workflow 成為第一級工作流：開發者可建立 Blueprint 繼承 generated C++ class，覆寫 VFX/animation/visual events，runtime 自動依 content id 載入 Blueprint class。
- 支援開發模式 Runtime Lua content hot reload，讓資料值、story/map、render metadata、buff visual metadata 與 Blueprint path 變更可在 Editor/Development runtime 重新載入並發布新的 catalog generation/hash。
- 發布 Rust-authored per-entity animation state，支援 stand 變體、locomotion、buff/modifier animation overlay、locomotion variant、action state、attack/CriticalAttack、前搖/命中/後搖 phase 與 progress，並讓 UE 套到 AnimBP。
- 讓 UE C++ 可讀取並處理 SaikaMagoichi 四個技能與動作事件；generated classes 需提供 typed native C++ handlers 與 Blueprint override surface。
- 建立 buff lifecycle visual event layer，從 frame buff state 產生 `Added`、`Removed`、`Refreshed`、`Updated` 與 cleanup events，並提供 Blueprint 可覆寫事件來建立/移除特效。
- 將所有 `UnitScript` event hooks 鏡射為 typed generated Blueprint events，包括 spawn/tick/death/damage/skill/attack/resource/state/modifier/order/respawn。
- UE game thread 只呼叫 C ABI、更新 Unreal actors/components/UI；Rust worker threads 不呼叫任何 Unreal API。
- 建立可驗證的 Windows build/stage/smoke pipeline。

**Non-Goals:**

- 不重寫 gameplay rules、script ABI、KCP protocol 或 lockstep cadence。
- 不把 `Om UE frontend` 設計成 backend authoritative renderer；第一版仍以本地 replica + lockstep input 為核心。
- 不把 UE 型別、UObject、Blueprint 型別或 Unreal headers 引入 Rust crate。
- 不把 C ABI 直接加進 `omoba-core::runtime` 的 gameplay 型別；FFI projection 屬於 bridge boundary。
- 不由 Rust generator 產生 `.uasset` Blueprint binary；generator 產生 C++ class 與 expected soft class paths，Blueprint assets 由 UE Editor 建立與編輯。
- 不把 Blueprint visual override 視為 gameplay authority；Blueprint 只能控制視覺、音效、UI 與呈現層副作用，不能改變 lockstep gameplay state。
- 不在 Shipping build 預設允許任意 Lua file hot reload；若未來需要 live-ops content patch，必須另行設計簽章、版本與伺服器相容性策略。

## Decisions

1. `<UE_PROJECT_ROOT>/` 使用既有 UE project 作為 host，新增 plugin 與 Rust bridge/codegen。

   既有檔案/目錄保留：

   - `<UE_PROJECT_ROOT>/om.uproject`
   - `<UE_PROJECT_ROOT>/Source/OmGame/OmGame.Build.cs`
   - `<UE_PROJECT_ROOT>/Source/OmGame/OmGame.cpp`
   - `<UE_PROJECT_ROOT>/Source/OmGame/OmGame.h`
   - `<UE_PROJECT_ROOT>/Source/OmGame.Target.cs`
   - `<UE_PROJECT_ROOT>/Source/OmGameEditor.Target.cs`
   - `<UE_PROJECT_ROOT>/Config/`
   - `<UE_PROJECT_ROOT>/Content/`

   新增結構：

   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/OmRuntime.uplugin`
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmRuntime/`
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmGenerated/`
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmEditor/`（Editor-only reload/validation tooling，可第一版延後但路徑先保留）
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/`
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/lib/Win64/`
   - `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Binaries/Win64/`
   - `<UE_PROJECT_ROOT>/bridge/Cargo.toml`
   - `<UE_PROJECT_ROOT>/bridge/cbindgen.toml`
   - `<UE_PROJECT_ROOT>/codegen/Cargo.toml`

   `om.uproject` 應新增並啟用 `OmRuntime` plugin entry，同時保留現有 `ModelingToolsEditorMode` editor plugin。`Source/OmGame` 保持薄 game host module；只有當 game module 需要直接 reference plugin public symbols 時，才在 `OmGame.Build.cs` 加入 `OmRuntime` dependency。主要 runtime、generated classes、Blueprint registry、DLL load 與 hot reload tooling 都由 plugin modules 擁有。

   不手動修改 `.vs/`、`Intermediate/`、`Saved/` 或把 `om.sln` 視為 source of truth。需要更新 IDE 專案檔時，透過 `D:\UE5.7\GenerateProjectFiles.bat -project=<UE_PROJECT_ROOT>\om.uproject` 或對應 UBT 流程重新產生。

   `OmRuntime.Build.cs` 負責 `Core`、`CoreUObject`、`Engine`、`Projects`、`EnhancedInput`（需要 UI/特效時再加 `UMG`、`Niagara`）以及 ThirdParty bridge include/lib/delay-load/runtime dependency。`OmGenerated.Build.cs` 依賴 `OmRuntime` 並編譯 codegen 輸出的 UHT classes。`OmEditor.Build.cs` 只在 editor target 啟用，依賴 `UnrealEd`、`ToolMenus` 或 file watcher/validation 所需 editor modules。

   Rationale: 使用者已建立 `<UE_PROJECT_ROOT>` 空專案；本 change 應在現有 UE project 上增量接入 runtime plugin。把 bridge crate 與 codegen 放在同一棵 `Om UE frontend` 樹內，讓 staging 路徑固定，且不需要 repo root Cargo workspace。Rust bridge 可用 path dependency 指向 `../../omoba-core`、`../../omoba-sim` 與相關 crates。

   Alternative considered: 在 repo root 新增 `om-bridge/`。拒絕原因是會把 `Om UE frontend` 分散到兩個頂層目錄，UE plugin 的 ThirdParty staging 反而更難追。

2. UE 只連結 `om-bridge` C ABI，不連結 Rust internal crates。

   `om-bridge` crate 產出 `cdylib`，所有跨語言入口都是 `#[no_mangle] extern "C"`。C ABI 使用 opaque handles、fixed-width scalar、`#[repr(C)]` structs、pointer + length slices、caller-owned config 與 Rust-owned frame leases。Header 由 `cbindgen --lang c` 產生，UE C++ 以 `extern "C"` include。

   Alternative considered: 用 `cxx` 或手寫 C++/Rust glue。暫不採用，因為本需求明確要求 cbindgen 與 C 風格指標；C ABI 也較容易被 UE `.Build.cs`、delay-load DLL 與 smoke C compile 驗證。

3. 不暴露 `SimWorldSnapshot`，改在 bridge 內投影成 FFI-safe frame。

   `om-bridge` 內部可以使用 `omoba-core::runtime::SimWorldSnapshot` 或更輕量的 runtime publisher，但 exported DTO 必須是 bridge-owned：

   - `FrameHeader`: ABI version、struct size、sequence、tick、counts、pointers。
   - `FrameEntity`: entity id/gen、kind、position、facing、hp、owner、unit type numeric id、flags。
   - `HeroState`、`TowerState`、`FxCue`、`InputLatency` 等 optional parallel arrays。
   - `RuntimeCatalog`: unit/ability/tower/template/asset metadata，含 UTF-8 string table。

   字串不以 `char*` 指向 Rust `String`。Frame/catalog 內使用 contiguous string table，DTO 欄位使用 `{ offset, len }` 或 numeric ids。UE 只在 lease 有效期間讀取這些指標。

   Alternative considered: cbindgen 直接產生現有 Rust snapshot header。拒絕原因是現有型別不是 FFI-safe，且會把 runtime 內部資料結構鎖死成 public ABI。

4. Frame 同步使用 Rust-owned ring buffer + atomic publication + lease。

   Producer flow:

   - sim thread 將下一個 frame 寫入未被讀取的 slot。
   - 完成所有非 atomic writes 後，以 Release ordering 發布 `published_slot` 與 `published_sequence`。
   - 若所有 slot 都被 UE lease 持有，producer 記錄 dropped/skipped publish，不阻塞 simulation。

   Consumer flow:

   - UE 每幀呼叫 `om_acquire_latest_frame(runtime, &lease)`。
   - Rust 端用 Acquire load 讀最新 sequence/slot，增加 slot reader count，確認 sequence 未變後回傳 frame pointer。
   - UE 在同一幀讀完後呼叫 `om_release_frame(runtime, lease)`。

   重要限制：不把 Rust `AtomicU64` raw layout 當成 C ABI 結構欄位讓 UE 直接 dereference。Rust/C++ atomic 型別 layout 不是這個 contract 的穩定邊界；C ABI 函式負責 acquire/release 記憶體序。若未來需要 UE 直接 poll，可另外設計 C11 `_Atomic` shared control block 並用 C/C++ compile test 固定 layout。

   Alternative considered: `Arc<Mutex<Frame>>` 讓 UE 每幀呼叫 copy 函式。拒絕原因是高實體數時會把 render thread 卡在鎖與拷貝上，且不符合 pointer/atomic 同步目標。

5. Buff 視覺事件由 bridge/frame diff 產生，Blueprint 只處理 add/remove/update。

   `om-bridge` 需要在每個 published frame 中提供全 visible entity 的 active buff snapshot，而不是只提供 hero panel 用的 buff list。每個 buff snapshot 至少包含：

   - target entity id/gen
   - buff id/numeric buff catalog id
   - remaining seconds，永久/toggle 用 `-1.0`
   - payload string-table reference 或 filtered visual payload
   - deterministic `buff_instance_key`

   Bridge 或 UE runtime 保留上一個已處理 frame 的 `(entity_id, entity_gen, buff_id)` map，對新 frame 做 diff：

   - absent → present: `BuffAdded`
   - present → absent: `BuffRemoved`
   - present → present 且 remaining 延長: `BuffRefreshed`
   - present → present 且 payload/visual fields 改變: `BuffUpdated`
   - entity removed/despawn: 對該 entity 所有 active buff 發 `BuffRemoved`，reason = `OwnerRemoved`

   `BuffStore` 目前同 entity 同 buff_id 只有一筆，因此第一版的 instance identity 可用 `(entity_id, entity_gen, buff_id)`，並在 UE side 以 `FName`/`uint64` hash 做 active effect key。若未來支援同 buff_id 多 stack，Lua/bridge 再增加 `stack_key` 或 `source_entity` 到 key；本 change 不假設 stack。

   UE base class 維護 `ActiveBuffEffects` map：key → Blueprint-created effect handle/component references。Generated C++ classes exposes typed events：`OnBuffAdded`、`OnBuffRemoved`、`OnBuffRefreshed`、`OnBuffUpdated`。Blueprint 在 added event 中建立 Niagara、attach component、設定材質參數或播放音效；在 removed event 中依同一 key 清理。若 Blueprint 忘記清理，base class 在 actor end play/despawn 時強制清理 owned components。

   Lua `buffs.lua` 可擴充 optional `ue` metadata：default Blueprint visual class、attach socket、Niagara system path、material parameter names、stacking policy、display category。Generator 會為每個 active buff 產生 `UOmBuff<Id>Visual` 或 registry entry。未宣告時使用 generic buff visual event 但仍產生 typed payload。

   Alternative considered: 在 gameplay systems push explicit visual add/remove events。暫不採用作為唯一來源，因為過期、entity deletion、refresh 與 payload replacement 都可能從不同路徑發生；以 authoritative active buff set diff 可保證 UE 不漏清理。未來可以加 explicit cue 作為補充 metadata，但 lifecycle truth 仍以 buff snapshot diff 為準。

6. Bridge 擁有 frontend runtime driver，並盡量從 `omfx` 萃取共用邏輯。

   第一版不讓 `Om UE frontend` 依賴 `omfx` crate。可行路徑是把 `omfx/game/src/lockstep_client.rs` 與 `sim_runner.rs` 裡與 Fyrox 無關的部分萃取到 `omoba-core::runtime::client` 或 bridge-local module，再讓 `omfx` 後續也可回用。`om-bridge` 啟動自己的 KCP client thread、sim worker thread、input queue 與 frame publisher。

   Alternative considered: UE 直接呼叫 `omfx` 的 sim runner。拒絕原因是會把 Fyrox crate、UI/render assumptions 與 UE frontend 綁在一起。

7. `UnitScript` hook 鏡射成 render-only Blueprint event cue。

   Runtime/bridge 需要在 `UnitScript` hook dispatch 邊界產生 `UnitScriptEventCue`。事件 cue 只記錄 visual payload，不參與 gameplay hashing，不回寫 ECS，也不允許 Blueprint 影響 script return value。`unit_id()` 與 `tower_metadata()` 是 metadata/accessor，不是事件 hook；metadata 由 catalog/registry 處理。其餘 hook 都必須有對應 event kind：

   - `on_spawn`
   - `on_tick`
   - `on_death`
   - `on_damage_taken`
   - `on_damage_dealt`
   - `on_skill_cast`
   - `on_attack_hit`
   - `on_attack_start`
   - `on_attack_landed`
   - `on_attack_fail`
   - `on_attacked`
   - `on_health_gained`
   - `on_mana_gained`
   - `on_spent_mana`
   - `on_heal_received`
   - `on_state_changed`
   - `on_modifier_added`
   - `on_modifier_removed`
   - `on_order`
   - `on_respawn`

   C ABI frame 加入 `unit_script_events` array，每筆包含 event kind、tick、sequence、primary entity id/gen、secondary entity id/gen（若有）、target payload、amount/damage/mana/heal values、ability/state/modifier/order string-table refs，以及 optional flags/reason。對 `on_damage_taken` 這類 hook 會修改 `DamageInfo` 的事件，visual payload 應記錄 hook dispatch 後 gameplay 接受的 final value；若 runtime 容易取得 original value，可額外提供 original/final，但 Blueprint 不依賴它做判定。

   `on_tick` 是高頻 hook。契約仍要求它有 Blueprint event surface；但 bridge 可以在同一 UE frame 內 coalesce 多個 sim tick 的 `on_tick` cue，payload 提供 accumulated `dt`、tick count 與 latest tick，避免 120 TPS × 大量單位直接打爆 UE event graph。其他離散 hook 不應被 coalesce，除非多個完全相同 tick/event key 明確標記為 count。

   Generated C++ class 宣告對應 Blueprint events，例如 `OnScriptSpawn`、`OnScriptTick`、`OnScriptDeath`、`OnScriptDamageTaken`、`OnScriptAttackStart`、`OnScriptModifierAdded`。Blueprint 繼承 generated unit class 後可以覆寫這些事件做 Niagara、Animation Montage、材質、音效與 UI feedback。事件命名以 `Script` 前綴區分 gameplay script hook 與純 UE actor lifecycle。

   Alternative considered: 只從 frame state 推導泛用 visual events，不 mirror `UnitScript` hook。拒絕原因是使用者要 Blueprint 能繼承 script event surface，且許多視覺效果（attack fail、spent mana、order、modifier added/removed）無法可靠從最終 state 推導。

8. Animation state 由 Rust runtime 發布，UE AnimBP 只消費狀態。

   `om-bridge` 在每個 frame 為需要骨架動畫的 entity 投影 `AnimationState`。這是連續狀態，不取代 `UnitScript` 離散 event cue。最小欄位包含：

   - target entity id/gen
   - locomotion state：`Stand`、`Walk`、`Run`、`Dead` 等 generic category
   - locomotion variant id：例如 `normal_walk`、`sniper_walk`
   - animation overlay/stance id：例如 `none`、`sniper_mode`
   - idle/stand variant id：例如 `stand_1`、`stand_2`、`stand_3`
   - action state：`None`、`Attack`、`CriticalAttack`、`Cast`、`HitReact` 等
   - attack phase：`None`、`Windup`、`Impact`、`Recovery`
   - action instance id，用來讓 UE 判斷新攻擊/施法開始，而不是每幀重播 montage
   - phase elapsed/duration、phase normalized progress、action elapsed/duration
   - animation tag/catalog id、critical flag、optional target entity id/gen、play rate

   Rust runtime 依 authoritative combat timing 決定 phase。攻擊指令被接受後進入 `Windup`；真正 hit/critical 結果由 gameplay 決定並在命中 tick 附近進入 `Impact` 或發布對應 event；命中後到下一次可行動前維持 `Recovery`。UE 可以使用 progress 驅動 AnimBP transition、AnimNotify 對齊、montage section 或 state machine blend，但不能把 AnimNotify 回寫成命中判定。

   Lua `ue.animation` metadata 提供 content-specific mapping，例如 idle variants、AnimBP variable names、state machine state names、montage/section soft paths、attack phase names、critical attack visual tag 與 default play rate。Buff 或 modifier metadata 也可宣告 animation overlay，例如 `sniper_mode` 可宣告 `overlay = "sniper_mode"`、`locomotion_overrides.walk = "sniper_walk"`、`priority = 100`。Rust bridge 在投影 animation state 時讀取 active buff/modifier set，依 deterministic priority 與 content id tie-break 選出生效 overlay/variant；UE 不需要同時讀 buff list 再自己決定走路動作。

   為了讓開發期 hot reload 有用，第一版的 runtime frame 使用 generic enum + catalog/string ids；新增 `stand_4`、新增 `sniper_walk` variant 或改 montage path 可 reload，只要不改 UHT-visible payload shape。若選擇產生 UENUM value 或 Blueprint-exposed typed class surface，該變更就需要 codegen + UE build。

   UE 端由 `UAnimStateComponent` 或 generated actor base 將 frame state 複製成 AnimInstance 可讀的 Blueprint 變數，例如 `LocomotionState`、`LocomotionVariant`、`AnimationOverlay`、`IdleVariant`、`ActionState`、`AttackPhase`、`AttackPhaseProgress`、`ActionInstanceId`、`bCriticalAttack`。Blueprint/AnimBP 可以用這些值驅動 state machine：stand 1/2/3、walk、sniper-mode walk、attack、CriticalAttack、windup/recovery blend。

   Alternative considered: UE 從 velocity、attack events 或 AnimNotify 自行推導 attack phase。拒絕原因是前搖/命中/後搖是 gameplay timing，UE 推測會造成視覺與命中不同步，尤其在 latency、dropped frame 或 critical attack 分支時更容易錯。

9. Lua-to-UE codegen 產生 C++ class，不在 runtime 用字串 switch 拼視覺邏輯。

   新增 `om-codegen` Rust binary，重用或抽出 `omoba-template-ids::lua_content` loader，讀取 `scripts/lua_data/templates.lua` 與 story content。它輸出到 `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmGenerated/`：

   - `OmContentIds.h/cpp`: content id、numeric id、display name、class name 與 Blueprint path lookup。
   - `OmContentClasses.h/cpp`: `AOmHeroSaikaMagoichi`、`AOmTowerDart`、`AOmCreep...` 等 `UCLASS(Blueprintable)` concrete base。
   - `OmContentClasses.h/cpp`: `UOmAbilitySniperModeVisual` 等 ability/VFX binding classes。
   - `OmContentClasses.h/cpp`: `UOmBuffSlowVisual`、`UOmBuffSniperModeVisual` 等 buff visual binding classes 或 generated registry entries。
   - `OmVisualRegistry.h/cpp`: animation tag ids、default AnimBP variable mapping、idle variants、attack phase metadata 與 montage/section soft path lookup。
   - `OmEventTypes.h`: UHT 可見的 `USTRUCT(BlueprintType)` event payload，例如 frame state、animation state、attack phase、tower fire、ability cue、buff cue。
   - `OmRegistry.cpp`: content id → generated native class → expected Blueprint soft class path → fallback native class 的 registry。

   手寫 C++ 只提供穩定抽象基底：`AOmUnitActor`、`AOmHeroActor`、`AOmTowerActor`、`UOmAbilityVisual`、`UOmUnitVisualComponent`。Generated concrete classes 繼承這些基底並宣告 content-specific native C++ virtual handler / `BlueprintNativeEvent`，例如 `HandleFrameState`、`HandleAnimationState`、`HandleAttackPhase`、`HandleAbilityCue`、`HandleBuffCue`、`HandleTowerFire`，並可再提供 Blueprint override event。C++ subclass 與 Blueprint 都能處理同一份 typed payload；Blueprint 覆寫仍只做視覺，不碰 lockstep state。

   `saika_magoichi` 是第一個完整 generated content target。Generator 需要產生 `AOmHeroSaikaMagoichi` 與 Saika-specific payload/header，例如：

   - `FSaikaMagoichiMetadata`: hero id、display name、title、base stats、render/animation sources、muzzle bone、ability slot list。
   - `FSaikaSniperModeEvent`: toggle on/off、level、range/damage/attack-speed/move-speed/accuracy values、buff visual instance key。
   - `FSaikaReinforcementsEvent`: cast accepted、formation rows/cols/count、duration、spawned `saika_gunner` entity refs、summon positions。
   - `FSaikaRainIronCannonEvent`: passive learned/proc、attacker/victim、aoe radius、arc half angle、true-damage percent、affected entity refs/damage cue ids when available。
   - `FSaikaThreeStageEvent`: transform started/ended/refreshed、duration、attack bonus、multi-shot count、multi-shot visual bullet index/count。
   - `FSaikaActionEvent`: stand variant、normal/sniper walk variant、attack/CriticalAttack、Windup/Impact/Recovery、action instance id、animation overlay。

   Generated class exposes C++ readable getters such as `GetSaikaMagoichiMetadata()` and `GetSaikaAbilityMetadata(AbilityId)`, plus C++ handlers such as `HandleSaikaSniperModeChanged`, `HandleSaikaReinforcementsCast`, `HandleSaikaRainIronCannonProc`, `HandleSaikaThreeStageChanged`, and `HandleSaikaActionEvent`. Generic ability/animation events still exist for all heroes; Saika-specific handlers are typed projections for the first complete content path.

   Lua schema 增加 optional `ue` section，例如 generated class override、Blueprint soft path、visual events、cue bindings、fallback asset 與 `ue.animation` mapping。未宣告 `ue` 時 generator 依 id 產生合法 class name 與 deterministic fallback path，例如 `/Game/Generated/Heroes/BP_SaikaMagoichi.BP_SaikaMagoichi_C`。

   因為 UHT 需要 build 前看到 C++ source，這不能只靠 Cargo `OUT_DIR`。`om-codegen` 是 UE build 前的明確步驟，會寫入 repo 內 generated source，並產生 freshness manifest。UBT 前置腳本或 `build_om.bat` 必須先跑 codegen，再跑 UE build。

   Alternative considered: 只在 frame catalog 裡傳 Blueprint path，由 UE 用一個 generic actor switch 處理所有單位。拒絕原因是會把型別與事件契約藏在 runtime string dispatch，Editor 也看不到每個內容項目的可繼承 C++ class。

10. UE module 分成 runtime subsystem、world bridge actor、render adapters 與 input adapters。

   `UOmRuntimeBridgeSubsystem` 管理 DLL load、function table、runtime handle、start/stop、last error 與 diagnostics。`AWorldBridgeActor` 在 game thread tick acquire 最新 frame，更新 entity registry。MVP 可用簡單 mesh/actor 顯示 hero/tower/creep/projectile；stress path 應優先支援 `UInstancedStaticMeshComponent` 或 Niagara/batched component，避免 1000+ entity 變成 1000+ UObject per-frame churn。

   Input adapter 從 UE mouse/keyboard 轉成 backend world coordinates 與 C ABI command，呼叫 `om_submit_*`。Rust bridge 負責轉成 shared `PlayerInput`、分配 input id、套用 lookahead tick 並交給 lockstep client。

   Alternative considered: 在 Rust bridge 內做 UE actor lifecycle 決策。拒絕原因是 Rust 不應知道 Unreal object model，也不能從 worker thread 操作 UE API。

11. UE runtime 預設使用 SinglePlayer debug mode，Networked mode 仍由 launcher 明確開啟。

   `OmRuntimeConfig.flags` 包含 SinglePlayer bit。SinglePlayer mode 不要求 server address，也不連 KCP backend；bridge 會在本地 worker thread 產生固定 cadence tick batch，直接把 UE 提交的 `PlayerInput` 依 target tick 併入 local replica。這條路徑仍使用相同 `omoba-core::runtime` ECS、script DLL、Lua story id、frame projection、input ack 與 diagnostics，因此 Editor/PIE 可以不啟 `omb` 快速 debug 視覺、Blueprint event、camera、HUD 與輸入。

   Networked mode 保留既有 KCP lockstep client，`run_ue.bat --networked` 或 `--with-backend` 才會建置/啟動 backend，並以 `-om-networked` override UE runtime mode。預設 `run_ue.bat` 走 `-om-single-player`，跳過 backend build/start，headless smoke 會清掉舊 log 並確認本次 log 出現 runtime startup marker。

   Alternative considered: 讓 UE frontend 永遠要求同機 backend。拒絕原因是 content/Blueprint/debug loop 太慢，且使用者要求 Unreal 前端預設能進單機模式方便快速 debug。

12. Runtime 初始化透過 generated registry 自動載入 Blueprint class。

   `UOmRuntimeBridgeSubsystem` 啟動後載入 generated registry。`AWorldBridgeActor` 第一次看到 `(entity_id, entity_gen, unit_id)` 時，先從 registry 查該 content id 的 Blueprint soft class path，透過 `FStreamableManager` 或等價 async/sync load 取得 Blueprint generated class；成功時 spawn Blueprint class，失敗時 fallback 到 generated native C++ class。已載入 class cache by content id，避免每個 entity 重複 load。

   對 ability/tower fire/attack phase 這類 cue，registry 也提供 content-specific visual class 或 component class。Frame/cue 只帶 numeric content ids 與 event payload；UE dispatch 到對應 generated class/Blueprint event。

   Alternative considered: 在 Lua 裡只放 mesh/asset path，不產生 class registry。拒絕原因是設計者需要 Blueprint 繼承點來調整 Niagara、Timeline、Animation Blueprint、材質參數與音效邏輯，而不是只能換資產路徑。

13. Coordinate contract 使用 backend 2D world units，UE side 負責轉換到 `FVector`。

   C ABI frame fields 使用 backend logical units：`pos_x`、`pos_y`、`facing_rad`。UE config 提供 `world_units_to_cm`、axis mapping 與 optional handedness flip。這避免把 Fyrox `WORLD_SCALE` 或 UE-specific centimeters 寫進 Rust runtime。

   Alternative considered: Rust 直接輸出 UE centimeters。拒絕原因是會讓 bridge 對 UE scene scale 做過早承諾，未來地圖、camera 或 3D presentation 調整時需要重建 Rust DLL。

14. Build/stage 由明確腳本完成，不依賴 UE Editor 手動步驟。

   新增 `build_om_bridge.bat` 或 `<UE_PROJECT_ROOT>/build_bridge.bat`，流程為：

   - `cargo run --manifest-path <UE_PROJECT_ROOT>/codegen/Cargo.toml -p om_codegen -- --content-root scripts/lua_data --out <UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/OmGenerated`
   - `cargo build --manifest-path <UE_PROJECT_ROOT>/bridge/Cargo.toml -p om_bridge`
   - `cbindgen --config <UE_PROJECT_ROOT>/bridge/cbindgen.toml --crate om_bridge --lang c --output <UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/include/om_bridge.h`
   - copy `om_bridge.lib` 到 `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Source/ThirdParty/OmBridge/lib/Win64/`
   - copy `om_bridge.dll` 與 `.pdb` 到 `<UE_PROJECT_ROOT>/Plugins/OmRuntime/Binaries/Win64/`
   - copy/stage `scripts/base_content.dll` 與必要 Lua/generated content path

   `.Build.cs` 使用 include path、import library、`PublicDelayLoadDLLs` 與 `RuntimeDependencies`，並以 plugin-relative path stage `om_bridge.dll`。腳本解析 engine root 的順序是 `UE_5_7_ROOT`、`UE_ROOT`、預設 `D:\UE5.7`。若解析後的 UE 5.7 path 不存在，Rust/header smoke 仍可執行；UE module build 則清楚報告 skipped 或 missing engine path，並顯示期望的 `D:\UE5.7`。plugin/source 變更後應透過 `D:\UE5.7\GenerateProjectFiles.bat -project=<UE_PROJECT_ROOT>\om.uproject` 或 UBT 重新產生 project files；`.sln`、`.vs/`、`Intermediate/`、`Saved/` 不作為手寫 source。

15. Runtime Lua content hot reload 是開發模式功能，並以 transaction 套用。

   `om-bridge` 暴露 `om_request_lua_reload` 或等價 C ABI。UE Editor/Development build 可用 console command、Editor utility button 或 optional file watcher 觸發 reload。Bridge 會在背景載入新的 Lua manifest、解析 include dependencies、計算 Lua content hash/generation，並先驗證與目前 generated registry 的相容性；驗證成功才 atomically publish 新 catalog generation，失敗則保留舊 catalog/runtime state 並回報 diagnostic。

   Reloadable change 包含數值、story/map builder output、tower/hero/buff/ability metadata、render metadata、Blueprint soft path、buff visual metadata 與 asset binding hints。Non-reloadable change 包含新增或移除會產生 UE C++ class 的 content id、改 generated class name、改 Blueprint parent module/class、改 UHT-visible event payload shape、改 `UnitScript` hook surface，或任何使 generated registry ids 與 runtime catalog 不相容的變更。這類變更必須 fail reload，提示開發者重新跑 `om-codegen` 與 UE build。

   UE runtime 收到新的 catalog generation 後，會 invalidate catalog-derived caches、重新解析 Blueprint soft class path、更新 UI metadata、對已存在 entity 重新套用 visual metadata，並保留 entity identity 與 active buff effect ownership。若某個已存在 entity 的 content id 在新 catalog 無法解析，UE 使用 fallback visual 並記錄 missing content diagnostic，不 crash。

   Alternative considered: 每次 Lua 變更都完整重啟 UE runtime。拒絕原因是會讓 content/visual iteration 太慢，且使用者明確要求開發時可動態載入 Runtime Lua content。

## Risks / Trade-offs

- [Risk] Cross-language memory lifetime bug 造成 UE 讀到已重用 frame slot → Mitigation: 所有 frame pointer 必須透過 lease 取得與釋放；producer 不重用 reader count > 0 的 slot；debug build 加入 poison/generation assert。
- [Risk] UE tick 持有 lease 太久導致 producer 無 slot 可 publish → Mitigation: ring buffer 至少 3 slots；UE wrapper 使用 RAII guard；producer 不阻塞 sim，只記錄 dropped publish diagnostics。
- [Risk] FFI DTO 與 Rust runtime snapshot drift → Mitigation: bridge projection 有 unit tests，header compile smoke 驗證 struct size，ABI version/struct_size 每次 acquire 都可檢查。
- [Risk] 直接 expose atomic layout 跨 Rust/C++ 不穩 → Mitigation: 第一版以 C ABI function 執行 acquire/release；未來若需要 shared atomic control block，必須用 C11/C++ compile test 與 per-platform layout guard。
- [Risk] UE plugin DLL staging 找不到 `om_bridge.dll` 或 `base_content.dll` → Mitigation: `.Build.cs` 加 `RuntimeDependencies`，runtime subsystem 啟動時 log resolved path，build script 一次 stage 所有 DLL。
- [Risk] Generated C++ class 與 Lua content drift，導致 runtime frame 找不到 class → Mitigation: `om-codegen` 產生 freshness manifest；UE build 或 smoke test 比對 Lua content hash/codegen hash，不一致即失敗。
- [Risk] UHT 不接受 generated code 的 include/order/macro 格式 → Mitigation: generator 只輸出少量固定模板，所有 `UCLASS`/`USTRUCT` 都有 deterministic names、`.generated.h` include 在最後，並用 UE module compile smoke 驗證。
- [Risk] Blueprint asset 路徑不存在或設計者尚未建立 Blueprint → Mitigation: generated registry 對每個 content id 都有 fallback native C++ class；runtime log missing soft path 並仍可顯示 debug visual。
- [Risk] Blueprint 覆寫事件修改 gameplay state 破壞 lockstep → Mitigation: generated Blueprint events 只暴露 render payload 與 visual helper API；gameplay input 只能走 bridge input API，不能直接改 runtime frame。
- [Risk] Buff remove event 遺漏導致特效殘留 → Mitigation: lifecycle 以 active buff set diff 產生，entity removal 強制對 active buff 發 `OwnerRemoved` cleanup，actor end play 也清空 `ActiveBuffEffects`。
- [Risk] Refresh 被誤判為 remove/add 造成特效閃爍 → Mitigation: same `(entity_id, entity_gen, buff_id)` 保持同一 `buff_instance_key`，remaining 延長只發 `BuffRefreshed`，payload 改變只發 `BuffUpdated`。
- [Risk] Buff payload 任意 JSON 不適合直接給 Blueprint → Mitigation: C ABI 保留 string-table payload 供診斷，generated typed payload 只投影 known visual fields；未知 payload 不影響 add/remove lifecycle。
- [Risk] Mirroring `on_tick` for every unit can overwhelm UE event graphs → Mitigation: `on_tick` supports coalescing per entity per publication window with accumulated dt/count；discrete hooks remain ordered and distinct。
- [Risk] UnitScript visual cue capture accidentally affects deterministic state → Mitigation: cue queues are render-only, excluded from gameplay hash, and Blueprint receives copies/projections only。
- [Risk] UE AnimBP 從 velocity/event 推測攻擊 phase 導致前搖/命中/後搖與 gameplay 不一致 → Mitigation: Rust frame 發布 authoritative `AnimationState`、attack phase、progress 與 action instance id；UE 只消費狀態。
- [Risk] 每幀 animation state 重播 montage 或重置 state machine → Mitigation: payload 帶 action instance id 與 phase transition，UE 只在 instance/phase 改變時觸發一次性 montage/section，連續 progress 用於 blend。
- [Risk] Lua hot reload 改 animation tag/variant 後 UE AnimBP cache stale → Mitigation: animation metadata 納入 catalog generation，UE generation 變更時 invalidate animation mapping cache；generic enum + FName/string id 變更可 reload。
- [Risk] 多個 buff 同時宣告 animation overlay，造成 walk/attack variant 選擇不穩定 → Mitigation: Lua metadata 必須有 deterministic priority 與 tie-break；bridge 在 Rust 側解析最終 overlay/variant，UE 只套用 resolved state。
- [Risk] `sniper_mode` buff 移除後 AnimBP 還停在 sniper walk → Mitigation: 每幀 animation state 都帶 resolved overlay/variant；buff removed 後下一個 compatible frame 必須回到 normal overlay/variant，UE cache 不保留舊 overlay 作為 authority。
- [Risk] 重複 `omfx` sim runner 邏輯造成維護成本 → Mitigation: 實作前先萃取 frontend-agnostic lockstep/runtime driver 到 `omoba-core` 或 bridge-local module，並以 `omfx` 後續回用為準則切邊界。
- [Risk] 每個 entity 對應一個 UE Actor 無法撐 stress 場景 → Mitigation: MVP 允許少量 actor；大量 creep/projectile/tower body 使用 instanced/batched component，spec 要求 frame 更新不得每幀重建 UObject。
- [Risk] Runtime Lua reload 套用到不相容的 generated C++/Blueprint surface 導致 Editor crash 或 class mismatch → Mitigation: reload 先做 compatibility validation，class-surface/id 變更 fail fast，要求 codegen + UE build；成功 reload 以 transaction atomically publish new generation。
- [Risk] File watcher 在保存中途讀到半寫入 Lua file → Mitigation: watcher debounce，多次 hash 穩定後才 reload；manual reload API 回報 parse error 並保留上一個有效 generation。
- [Risk] 開發期 hot reload 被誤開到 packaged Shipping build → Mitigation: reload API 由 build config/runtime config gate，Shipping 預設 disabled，diagnostics 明確顯示 reload unavailable。
- [Risk] Unreal 5.7 安裝路徑、plugin template 或 UBT 行為與文件版本差異 → Mitigation: 腳本預設 `D:\UE5.7`，並接受 `UE_5_7_ROOT`/`UE_ROOT` override；UE build step 可獨立於 Rust smoke；`.Build.cs` 遵循官方 third-party library pattern。

## Migration Plan

1. 以既有 `<UE_PROJECT_ROOT>` UE 5.7 C++ 空專案為基底，保留 `om.uproject`、`Source/OmGame`、targets、`Config/`、`Content/`，新增 `OmRuntime` plugin skeleton、Rust `om_bridge` crate、Rust `om_codegen` crate 與 `cbindgen.toml`。
2. 抽出或複用 `scripts/lua_data` loader，先讓 `om-codegen` 能讀 manifest 並輸出 deterministic generated C++ source。
3. 建立手寫 UE base classes 與 generated classes/registry，先用 synthetic content 驗證 Blueprint 可繼承 generated class。
4. 定義最小 C ABI：version、config、create/start/stop/destroy、last_error、diagnostics、acquire/release frame。
5. 建立 bridge frame DTO 與 ring buffer publisher，先用 synthetic frames 驗證 UE 每幀 acquire/release。
6. 接上 `omoba-core::runtime` local replica driver：KCP join、GameStart、TickBatch、input queue、sim tick、metadata catalog。
7. 將 runtime dynamic state 投影到 `Frame`，將 tower/ability/unit/buff metadata 投影到 `RuntimeCatalog`，並以 catalog ids 對齊 generated registry。
8. 增加 all-visible-entity buff snapshot 與 buff lifecycle diff，發布 `BuffEvent`。
9. 增加 per-entity `AnimationState` 投影，發布 locomotion、buff/modifier overlay、locomotion variant、action/attack phase/progress/action instance id，並在 catalog 中加入 animation metadata。
10. 增加 `UnitScript` hook cue capture，發布 `UnitScriptEvent`。
11. UE 端實作 `UOmRuntimeBridgeSubsystem` 與 function table loading，處理 DLL load failure、ABI mismatch、generated registry load 與 lifecycle。
12. UE 端實作 `AWorldBridgeActor`，以 entity id/gen registry 更新 generated/Blueprint actor 或 instanced mesh，並處理 removed ids、FX cues、buff events、animation states 與 UnitScript events。
13. 接上 Blueprint auto-load：content id/buff id → Blueprint soft class path → fallback generated native class。
14. 接上 AnimBP adapter：將 `AnimationState` 寫入 AnimInstance 變數或 generated Blueprint events，驗證 stand/walk/attack/CriticalAttack 與 windup/recovery。
15. 接上 input adapter，提供 move、attack、tower place/sell/upgrade、start round、ability cast/upgrade 的 C ABI submit 函式。
16. 接上 Runtime Lua content reload：manual reload API、optional file watcher、compatibility validation、catalog generation publish、UE cache invalidation 與 diagnostics。
17. 新增 Windows build/stage scripts，確保 `.bat` CRLF，並讓 codegen、Rust bridge/header、UE plugin staging、`D:\UE5.7` project-file regeneration 與 UE build 可重複執行。
18. 加入 verification：codegen freshness、Rust tests、cbindgen header generation、C/C++ header compile smoke、UE module build、Blueprint inheritance smoke、AnimBP state smoke、Runtime Lua reload smoke、buff add/remove smoke、UnitScript event smoke、TD_1 run smoke、frame publish/drop diagnostics。

Rollback strategy: 如果 UE integration 阻塞，保留 `<UE_PROJECT_ROOT>/bridge` 的 Rust C ABI smoke 與 synthetic-frame UE scaffold，暫停 gameplay runtime 接線；不要把 `Om UE frontend` 改成直接依賴 `omb` 或用 per-frame backend snapshot response 代替。

## Open Questions

- 第一版 `Source/OmGame` game module 是否需要直接 reference `OmRuntime` public API？建議先讓 plugin 自己完成 subsystem/actor/component 註冊，game module 保持薄 host，只有實作 game-specific bootstrap 時才加依賴。
- `omfx` 的 lockstep/sim runner 共用化要在本 change 內完成，還是先在 `om-bridge` bridge-local 實作後再回收？建議本 change 至少切出 frontend-agnostic Rust module，避免一開始就複製大段程式。
- 第一版高量 entity rendering 要直接使用 instanced mesh，還是先用 debug actors 再優化？建議 hero/tower 用 actor，creep/projectile 從第一版就走 instanced/batched path。
- Lua `ue` schema 第一版要覆蓋到 ability 級別的 Niagara cue 與 Animation Blueprint binding，還是先只產生 class/Blueprint path/events？建議第一版先產生 class、registry、typed events 與 Blueprint path，具體 asset binding 可由 Blueprint 實作。
- Runtime Lua hot reload 第一版是否只支援 manual reload，還是同時提供 file watcher？建議第一版兩者都設計進 API，但實作可先完成 manual reload，再加 watcher debounce。
- AnimBP integration 第一版要只寫 generic Blueprint variables，還是同時產生 content-specific AnimBP interface？建議先用 generic variables + generated metadata，避免新增 stand variant、sniper walk variant 或 montage path 每次都需要 UHT rebuild。
