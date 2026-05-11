## Context

目前 shipped content 的 canonical source 是 `scripts/lua_data` Lua builders，`omoba-template-ids/build.rs` 會在 build time 讀取後產生 Rust lookup。英雄 template 目前已有 id、顯示名稱、portrait、技能與數值，但沒有戰鬥場景用的 model metadata。

omfx 目前透過 `SimWorldSnapshot` 取得 entity render data。英雄、creep、projectile 走 batched quad，tower 則已有 script-owned render metadata 與 composite sprite pipeline。`saika_magoichi` 的 FBX 與 PNG 已存在於 `scripts/lua_data/templates/heroes/saika_magoichi/`，但 runtime 沒有任何路徑會讀取它們。

資料權威邊界：Saika 3D visual 的所有資料都屬於 scripts content。模型檔、貼圖檔、asset path、scale、pitch/roll/yaw offset、z offset、muzzle bone、animation action keys、source animation name、timeline offset 與 tick ranges 都 SHALL 由 `scripts/lua_data` 宣告，再由 `omoba-template-ids` build-time codegen 產生 Rust lookup。`omfx` 只提供 data-driven renderer/loader/player 功能，不擁有 Saika 專屬資料，也不在 source 內維護 Saika 專屬表。

現有 camera 是 orthographic 3D camera，使用 XY 作為畫面平面、Z 作為 draw order depth。這次設計不改整個 camera 或 2D tower/creep pipeline，而是把 hero 3D model 作為單一 Scene node hierarchy 放進既有 XY 畫面平面，並用 metadata 調整 scale、Z layer 與 facing offset。

`assimp info scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx` 的檢查結果顯示 base FBX：4 meshes、1059 vertices、847 faces、32 bones、1 animation、24 animation channels、3 materials、0 embedded textures。base FBX 唯一 animation 名稱是 `Take 001`；`assimp dump` 進一步顯示 `Take 001` 的 duration 是 `1000` ticks、`tick_cnt = 30`，也就是約 33.33 秒。

實作前新增的同目錄 action FBX 提供更精準的 action source：`b01_ani_attack.fbx` duration 100 ticks、`b01_ani_run.fbx` duration 23 ticks、`b01_ani_run2.fbx` duration 32 ticks、`b01_ani_stand.fbx` duration 80 ticks、`b01_ani_stand2.fbx` duration 125 ticks、`b01_ani_stand3.fbx` duration 53 ticks、`b01_ani_cast.fbx` duration 73 ticks、`b01_ani_cast2.fbx` duration 50 ticks、`b01_ani_die.fbx` duration 110 ticks；所有檔案的唯一 animation name 都是 `Take 001`，tick rate 都是 30。這些 action FBX 仍屬於 `scripts/lua_data`，因此 metadata 應以 logical source key 區分 action source，而不是只用 animation name `Take 001`。

`omoba-template-ids/build.rs` 目前只用 `mlua`/`serde` 讀取 Lua content，不應為了驗證 FBX 再新增 assimp/FBX parser dependency，也不應在 build script shell out 到開發機限定的 `assimp` CLI。因此 Assimp 檢查結果會被納入 scripts-owned metadata 的 animation source inventory；codegen 只驗證 binding 是否引用已宣告的 logical source，以及 tick range 是否落在該 source 宣告的 duration 內。`assimp info`/`assimp dump` 保留為 authoring/verification task。

## Goals / Non-Goals

**Goals:**

- 讓 `saika_magoichi` 在戰鬥場景中使用 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png` 呈現 3D visual。
- 讓 `saika_magoichi` 的 `attack`、`critical`、`move` 與 `sniper` 四個 gameplay-facing 動作都綁定到 FBX animation segment。
- 讓 Saika 攻擊動畫正確對齊 attack windup、impact、backswing，並讓移動/技能取消遵守 authoritative commit point。
- 讓 hero 3D metadata 由 `scripts/lua_data/templates/heroes.lua` 宣告，並透過 `omoba-template-ids` generated lookup 供 runtime 使用。
- 讓 `omfx` 成為 generic data-driven 3D hero renderer：只消費 snapshot/generated metadata，不保存 Saika 專屬 path、scale、animation range 或 action mapping。
- 讓 `SimWorldSnapshot` 對 hero entity expose render metadata，omfx 可依 snapshot 建立、更新、移除 hero 3D node。
- 模型載入失敗時保留既有 2D batched quad fallback，不影響選取、移動、攻擊、技能或 lockstep determinism。
- 不新增外部 crate；優先使用 Fyrox 既有 model/resource/scene API。

**Non-Goals:**

- 不重做 camera、地圖座標系、tower composite pipeline 或 creep rendering。
- 不新增 skeletal animation、attack animation、ragdoll、shadow pass 或 hitbox 變更。
- 不搬移使用者已提供的 FBX/PNG 檔案；本 change 以 `scripts/lua_data/templates/heroes/saika_magoichi/` 為 Saika 3D asset canonical location。
- 不把 `saika_magoichi` 的模型、貼圖、animation ranges、scale 或 offsets 複製到 `omfx/data`，也不在 `omfx` source 寫死 Saika 專屬設定。
- 不改 hero gameplay stats、ability 數值、portrait、ability icon 或 network protocol。

## Decisions

### Hero render metadata 掛在 hero template

在 `scripts/lua_data/templates/heroes.lua` 的 `saika_magoichi` entry 加入 optional `render` table，例如：

```lua
render = {
  render_mode = "model_3d",
  model = "templates/heroes/saika_magoichi/saika_magoichi.fbx",
  texture = "templates/heroes/saika_magoichi/saika_magoichi_mat.png",
  scale = 0.012,
  pitch_offset_deg = -90.0,
  roll_offset_deg = 0.0,
  yaw_offset_deg = -90.0,
  z_offset = 0.0,
  muzzle_bone = "Weapon Ref",
  animation_sources = {
    idle = { model = "templates/heroes/saika_magoichi/b01_ani_stand.fbx", animation = "Take 001", duration_ticks = 80.0, ticks_per_second = 30.0, timeline_offset_ticks = 66.0 },
    idle_2 = { model = "templates/heroes/saika_magoichi/b01_ani_stand2.fbx", animation = "Take 001", duration_ticks = 125.0, ticks_per_second = 30.0, timeline_offset_ticks = 143.0 },
    idle_3 = { model = "templates/heroes/saika_magoichi/b01_ani_stand3.fbx", animation = "Take 001", duration_ticks = 53.0, ticks_per_second = 30.0, timeline_offset_ticks = 747.0 },
    move = { model = "templates/heroes/saika_magoichi/b01_ani_run.fbx", animation = "Take 001", duration_ticks = 23.0, ticks_per_second = 30.0, timeline_offset_ticks = 394.0 },
    attack = { model = "templates/heroes/saika_magoichi/b01_ani_attack.fbx", animation = "Take 001", duration_ticks = 100.0, ticks_per_second = 30.0, timeline_offset_ticks = 268.0 },
    critical = { model = "templates/heroes/saika_magoichi/b01_ani_attack.fbx", animation = "Take 001", duration_ticks = 100.0, ticks_per_second = 30.0, timeline_offset_ticks = 268.0 },
    sniper = { model = "templates/heroes/saika_magoichi/b01_ani_stand3.fbx", animation = "Take 001", duration_ticks = 53.0, ticks_per_second = 30.0, timeline_offset_ticks = 747.0 },
  },
  animations = {
    idle = { source = "idle", start_tick = 0.0, end_tick = 80.0, loop = true },
    idle_2 = { source = "idle_2", start_tick = 0.0, end_tick = 125.0, loop = true },
    idle_3 = { source = "idle_3", start_tick = 0.0, end_tick = 53.0, loop = true },
    move = { source = "move", start_tick = 0.0, end_tick = 23.0, loop = true },
    attack = { source = "attack", start_tick = 0.0, repeat_start_tick = 6.0, impact_tick = 22.0, end_tick = 100.0, loop = false },
    critical = { source = "critical", start_tick = 0.0, repeat_start_tick = 6.0, impact_tick = 22.0, end_tick = 100.0, loop = false },
    sniper = { source = "sniper", start_tick = 0.0, end_tick = 53.0, loop = true },
  },
}
```

理由是 hero visual 是 content 屬性，應與 tower render metadata 一樣由 scripts content 宣告，而不是在 omfx 針對 `saika_magoichi` 寫死路徑、scale 或 animation ranges。`render_mode = "model_3d"` 讓沒有 metadata 的英雄維持現有 2D fallback。

四個 required action keys 是 `move`、`attack`、`critical`、`sniper`。`idle` action family 是 optional loop bindings；普通待機時 omfx 可在 `idle`、`idle_2`、`idle_3` 等 action 中輪替/隨機播放，`sniper` 只在 `sniper_mode` 狀態使用。`move`、`idle*` 與 `sniper` 可 loop；`attack` 與 `critical` 應單次播放，且需要 `start_tick < impact_tick < end_tick`，讓 omfx 能把 animation hit frame 對齊 authoritative impact event。`attack`/`critical` 的 `impact_tick = 22.0` 來自 `b01_ani_attack.fbx` 動作分析：右手/武器主動作集中於 1..22 ticks，之後進入 torso/root recoil 與 recovery。`repeat_start_tick = 6.0` 是連續射擊的視覺起點；第二槍以後用 `repeat_start_tick..impact_tick` retime 到同一個 `cue.windup_ms`，因此只跳過拔槍視覺，不改 backend 前搖時間或 impact commit point。

`animation_sources` 是 build-time 可驗證的 source inventory。每個 source key 是 content-owned logical id，包含 source FBX path、該 FBX 內的 animation name、duration ticks、ticks-per-second 與 timeline offset。所有 Saika action FBX 的 animation name 都是 `Take 001`，所以 codegen/runtime 不得用 animation name 當唯一 key；binding 的 `source` 必須指向 logical source key。Fyrox FBX importer 會保留 action FBX 原始 timeline offset，因此 omfx 播放 Fyrox animation 時以 `seconds = (timeline_offset_ticks + tick) / ticks_per_second` 把 content tick range 轉成 Fyrox `Animation::set_time_slice` 使用的秒數。

替代方案是直接在 omfx hard-code `hero_saika_magoichi` 到 FBX/PNG 的 mapping。這雖然最短，但會破壞 content-owned asset pattern，之後新增英雄模型時必須改前端 source，因此不採用。

### Generated API 新增 optional hero render lookup

`omoba-template-ids` 新增 `HeroRenderMetadataConst` 與 `hero_render_metadata(HeroId) -> Option<&'static HeroRenderMetadataConst>`。`HeroEntry` 反序列化 `render` table；當 `render_mode = "model_3d"` 時驗證 `model` 非空、`scale > 0`，並保留 texture path、pitch/roll/yaw offset、z offset 與 muzzle bone。

同時新增 `HeroAnimationSourceConst` 與 `HeroAnimationBindingConst` 或等效結構。source const 包含 logical source key、optional source model path、source animation name、duration ticks、ticks-per-second 與 timeline offset；binding const 包含 action key、logical source key、start/end tick、optional impact tick、loop、priority/interrupt policy。對 `model_3d` hero，codegen 驗證 required bindings：`move`、`attack`、`critical`、`sniper` 都存在、source 存在於 `animation_sources`、tick range 在 `0..=source.duration_ticks` 且 `end_tick > start_tick`。對 `attack` 與 `critical`，額外驗證 `start_tick < impact_tick < end_tick`。codegen 不直接解析 FBX；FBX inventory 由 Assimp authoring task 產生並寫進 scripts metadata。

理由是 omfx 已依賴 `omoba-template-ids`，可以在 sim snapshot extraction 階段用 generated lookup 取得資料，避免 runtime 讀 Lua 或 JSON。這也符合現有「Lua builders build-time only」契約。

替代方案是讓 omfx runtime 直接讀 `heroes.lua` 或掃描 asset directory。這會引入 runtime Lua/data parsing，違反目前 build-time codegen 的架構，因此不採用。

### Snapshot 將 hero render metadata 放在 hero entity 上

`EntityRenderData` 新增 optional boxed `hero_render`，只在 hero entity 且對應 template 有 3D metadata 時填入。`ScriptUnitTag.unit_id` 對 hero 目前是 `hero_<id>`，snapshot extraction 會 strip `hero_` 後用 `hero_by_name` 與 `hero_render_metadata` lookup。

`EntityRenderData` 也需要 render-only hero animation cues 或 state hints。最低限度應提供：是否移動中、是否處於 `sniper_mode` buff、最近 attack phase cue、attack cancel cue，以及若能從 damage result/outcome 得知是否 critical，就提供 critical cue。若 backend 目前沒有「hero attack critical」或「windup canceled」render cue，實作需新增 render-only queue 或擴充既有 attack/damage render cue；這些 cue 不得參與 simulation hash，也不得改變 gameplay outcome。

理由是 hero 數量低，把少量靜態 metadata clone 到 hero entity snapshot 比新增 `hero_templates: Arc<Vec<_>>` 與 omfx template cache 更簡單，也避免新的 static lifecycle 邏輯。非 hero row 只增加一個 `None` 指標，與現有 `hero_ext` 模式一致。

替代方案是仿照 tower template snapshot 建立 shared `Arc<Vec<HeroRenderTemplateSnapshot>>`。這對大量 templates 比較好，但目前需求只針對少量英雄，會增加更多 cache 與同步程式碼，因此先不採用。

### omfx 以 per-entity node cache 呈現 3D hero

omfx 新增 `hero_model_nodes: HashMap<u32, HeroModelRender>` 與 asset load status cache。`update_sim_batches` 遇到 hero 且 `hero_render.render_mode == "model_3d"` 時，嘗試載入/instantiate model，將 root node 放到 `world_to_render(e)` 的 XY 位置與 hero Z layer，並用 render-space facing 加上 metadata pitch/roll/yaw offset 更新旋轉。若 metadata 提供 muzzle bone，omfx 會在模型 hierarchy 中查找同名 node，並用該 node 的 global position 作為該 hero projectile 的 render-only 起點。

這個 pipeline 必須是 generic。`omfx` 可以知道「如何」載入 scripts asset、建立 Fyrox node、套用 scale/offset、播放 segment、fallback，但不能知道「Saika 的資料值」。具體 path、`Take 001`、tick ranges、action binding 都來自 snapshot/generated metadata。

有 3D node 的 hero 會把原本 body quad 寫成透明或極小 quad，並跳過 generic facing bar，避免同一英雄同時出現 2D placeholder 與模型。HP bar、hero panel 與 gameplay input 仍讀 snapshot entity，不依賴 3D node。

替代方案是把 hero 3D model 也塞進 batched mesh。FBX 是 scene/model hierarchy，不適合現有 quad batch；用 Fyrox scene nodes 可最小化變更。

### Fyrox 1.0.1 model lifecycle 使用方式

omfx 透過 `context.resource_manager.request::<Model>(resolved_model_path)` 請求 FBX；`request` 是 non-blocking，回傳 `ModelResource` 後載入在 resource system 背景執行。`HeroModelAsset` cache 應保存 `ModelResource`、resolved model path、optional texture path 與 failure/logged status。當 `model_resource.is_loading()` 時維持 2D fallback；當 `model_resource.is_failed_to_load()` 時 log 一次並固定 fallback；只有 `model_resource.is_ok()` 才 instantiate。

實例化使用 `fyrox::resource::model::ModelResourceExtension`：

```rust
let root = model_resource
    .begin_instantiation(scene)
    .with_position(Vector3::new(pos.x, pos.y, z + z_offset))
    .with_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), facing + yaw_offset))
    .with_scale(Vector3::new(scale, scale, scale))
    .finish();
```

後續更新不重新 instantiate，只用 `scene.graph.try_get_mut(root)` 取得 root node 並更新 `local_transform_mut().set_position(...)`、`set_rotation(...)`、`set_scale(...)`。entity 移除或 stale cleanup 時用 `scene.graph.remove_node(root)`；這會遞迴移除 model hierarchy。`update_sim_batches` 目前只拿 `scene` 與 `dt`，實作可把 `context.resource_manager` 也傳入該 helper，避免在 `Game` state 另存 resource manager。

### Fyrox animation player 使用方式

FBX model resource 內的 animation track target 指向 resource graph node；實例 node 要播放時必須 retarget。實作不依賴 importer 是否已建立 animation player，而是在 model root 下建立專用 player。base model 只負責 mesh/skeleton；action source FBX 透過 metadata path 載入後 retarget 到同一個 instance root：

```rust
let player = AnimationPlayerBuilder::new(BaseBuilder::new().with_name("Hero Animation Player"))
    .build(&mut scene.graph);
scene.graph.link_nodes(player, root);
let handles = action_model_resource.retarget_animations_to_player(root, player, &mut scene.graph);
```

`HeroModelRender` 應保存 `root_node`、`animation_player`、`animations_by_source: HashMap<String, Handle<Animation>>` 與 active playback state。retarget 後使用 metadata logical source key 建立 source key 到 handle 的 mapping；不得只用 `Animation::name()`，因為多個 action FBX 都叫 `Take 001`。播放 action 時先停用同 player 的其他 animation，再對 active source 設定：`set_time_slice(start_sec..end_sec)`、`set_time_position(start_sec)`、`set_loop(loop_flag)`、`set_enabled(true)`、`set_speed(speed)`。

因為 Fyrox `Animation::set_speed` 對整個 current time slice 只有一個倍率，`attack`/`critical` one-shot 不能用單一 slice 同時精準對齊 windup 與 backswing。實作應把一個 attack action 當成兩個 visual sub-phase：先播放 `start_tick..impact_tick`，`speed = source_windup_seconds / cue.windup_seconds`；到 authoritative impact time 後切成 `impact_tick..end_tick`，`speed = source_backswing_seconds / cue.backswing_seconds`。windup cancel cue 抵達時停用該 sequence animation 並回到 sustained state；backswing interrupt cue 抵達時可停用後搖，但不回收已 impact 的 hit/projectile/critical visual。

### Fyrox material/texture 使用方式

Fyrox FBX importer 的 `ModelImportOptions` 預設 `MaterialSearchOptions::RecursiveUp`，會以 FBX material reference 的 filename 從 model path 往上尋找同名 texture，並把 diffuse 類 reference 綁到 standard shader 的 `diffuseTexture`。因此第一步是讓 model 以 resolved filesystem path 載入，讓 importer 有機會自動找到與 FBX 同目錄的 `saika_magoichi_mat.png`。

若 importer 沒有綁到 diffuse texture，omfx 才用 metadata 的 `texture` path 做 generic manual fallback。fallback 以既有 `TextureResource::load_from_memory` pattern 載入 PNG，建立 3D material：

```rust
let mut material = Material::standard();
material.bind("diffuseTexture", Some(texture));
let material = MaterialResource::new_embedded(material);
```

接著收集 `scene.graph.traverse_handle_iter(root)` 的 handles，對其中 `Mesh` node 的 `surfaces_mut()` 呼叫 `surface.set_material(material.clone())`。這個流程只使用 metadata texture path，不加入 Saika 專屬 fallback。

### Animation binding 由 content metadata 驅動

Saika base/action FBX 都只有單一 `Take 001`，因此 omfx 不應寫死 frame ranges，也不能只靠 animation name 區分 action。Content metadata 需要提供 `animation_sources` 與 `animations` table，把 gameplay-facing action key 對到 logical source key 與 tick segment。Generated metadata 會把 source path、source animation name 與 tick segment 交給 omfx，omfx 播放時只依 action key 選 segment。

行為選擇規則：移動速度或 position delta 高於小閾值時播放 `move` loop；`sniper_mode` buff 存在時播放 `sniper` loop；普通待機時從 `idle` action family 中選擇一個 loop binding 輪替/隨機播放；收到 attack phase cue 時播放 `attack` one-shot；收到 critical cue 時用 `critical` one-shot 覆蓋一般 attack。one-shot 播放時，第一槍把 `start_tick..impact_tick` retime 到 cue 的 windup duration；連續攻擊若 binding 宣告 `repeat_start_tick`，第二槍以後把 `repeat_start_tick..impact_tick` retime 到同一個 cue windup duration；impact 後把 `impact_tick..end_tick` retime 到 cue 的 backswing duration，使 hit frame 對齊 authoritative impact event。one-shot 播完後回到 `move`、`sniper` 或普通 idle 狀態。

### Attack cancel semantics use impact as the commit point

既有 `unit-attack-phase-timing` 已定義 windup、impact、backswing 三階段，但這次要補上取消語意。後端 attack scheduler SHALL treat impact as the commit point。已接受的移動或技能指令若在 attack windup 期間抵達，SHALL cancel 該次攻擊，清除 pending impact outcome，且不產生 damage、projectile 或 critical result。若指令在 impact 已觸發後、backswing 期間抵達，MAY cancel backswing animation/lockout，但 SHALL NOT rollback 已 committed 的 damage/projectile/outcome。

這個語意必須由後端 authoritative 處理，omfx 只消費 render cue。前端可以在 windup cancel cue 到達後停止 attack animation 並切到 move/skill animation；若 backswing 被取消，前端可以中斷後搖動畫，但 hit event 已發生，因此 damage/projectile VFX 不回收。

替代方案是讓 omfx 自行判斷取消點。這會造成 visual 與 gameplay 可能分歧，尤其 lockstep input 延遲下前端無法權威知道指令是否在 impact 前被 accepted，因此不採用。

替代方案是把每個動作拆成多個 FBX 檔或重新匯出 named clips。這會讓 runtime 簡單，但目前使用者提供的是單一 FBX，且 `assimp info` 證實只有 `Take 001`，所以本 change 先用 segment binding 支援現有素材。

### Asset path resolver 支援 `scripts/lua_data`

新增 hero 3D asset resolver，將 metadata path 視為相對於 `scripts/lua_data/` 的路徑，並比照 tower texture loader 搜尋 repo root、`../`、`../../` 與 executable ancestors。model resolver 必須回傳實際 filesystem path 給 `ResourceManager::request::<Model>`，讓 Fyrox FBX importer 的 `RecursiveUp` material search 能以 FBX 位置搜尋同目錄 texture。texture 會先依 FBX 內部 material reference 嘗試載入；若需要手動指定 diffuse texture，則使用 metadata 的 `texture` path 套用到模型 mesh material。

Resolver 可以有 scripts-root 搜尋策略，但不應有 Saika 專屬 fallback path。若 metadata path 缺失或檔案不存在，omfx fallback 到既有 2D rendering 並 log 診斷，不改用 `omfx/data` 的替代 Saika asset。

替代方案是把 model 複製到 `scripts/base_content/assets/heroes/`。長期也許更一致，但使用者已明確提供檔案位置；本 change 避免搬檔與破壞既有路徑。

## Risks / Trade-offs

- [Risk] FBX import 的實際座標軸、比例或材質綁定可能與 omfx camera 不一致 → Mitigation：metadata 保留 `scale`、`pitch_offset_deg`、`roll_offset_deg`、`yaw_offset_deg`、`z_offset`；implementation 以 TD_1/dev run 實機畫面微調預設值。
- [Risk] FBX 只有 `Take 001`，action segment ranges 需要人工校準 → Mitigation：tasks 要求用 `assimp dump`/實機預覽填入明確 tick ranges，並在 metadata validation 阻擋缺漏或空 range。
- [Risk] build-time codegen 直接解析 FBX 會引入新 dependency 或開發機工具耦合 → Mitigation：Assimp 結果寫入 scripts metadata 的 `animation_sources`，codegen 驗證 declared source/duration/range，不解析 FBX。
- [Risk] 實作時為了快速驗證而把 Saika path/ranges 寫進 omfx → Mitigation：specs/tasks 加入禁止 hard-code 與 grep 驗證，所有 Saika data 必須只在 scripts metadata 或 generated output 中出現。
- [Risk] attack windup cancel 與 impact commit 判定若依 render frame 而非 backend tick，會造成傷害與動畫不一致 → Mitigation：取消與 commit point 只由 backend attack scheduler 判定；omfx 只消費 authoritative phase/cancel cues。
- [Risk] Fyrox 1.0 model loading API 可能是 async/resource based，直接在 render loop 載入會卡幀 → Mitigation：使用 resource cache/status cache，只在第一次看到 asset 時 request/load，失敗結果也 cache 並 fallback。
- [Risk] 模型檔缺失或 PNG 解碼失敗會讓英雄不可見 → Mitigation：載入失敗時 log 一次並走既有 2D body/facing rendering。
- [Risk] 3D scene nodes 比 batched quad 貴 → Mitigation：只對有 3D metadata 的 hero 建立 node；entity 移除時清理 node；tower/creep stress hot path 不變。
- [Risk] 變更 snapshot struct 可能影響 tests → Mitigation：新增 default/empty metadata 行為，既有沒有 hero render metadata 的測試仍維持 2D fallback。

## Migration Plan

1. 用 `assimp info`/`assimp dump` 確認 Saika base/action FBX animation inventory，記錄每個 source 的 `Take 001` duration 與 tick rate，並找出 `move`、`attack`、`critical`、`sniper` 的實際 tick ranges。
2. 擴充 `omoba-template-ids` hero metadata model 與 generated lookup，新增 animation binding schema 與 validation。
3. 在 `scripts/lua_data/templates/heroes.lua` 為 `saika_magoichi` 宣告 3D metadata、`animation_sources` 與四個 required animation bindings，路徑指向現有 FBX/PNG。
4. 擴充 backend attack scheduler，讓已接受的移動/技能指令在 windup 取消攻擊且不出傷害，在 backswing 只取消後搖不 rollback impact outcome。
5. 擴充 `SimWorldSnapshot` hero entity render projection，讓 omfx 可從 snapshot 取得 optional `hero_render`、render-only animation cues 與 attack cancel/interrupt cues。
6. 在 omfx 新增 generic hero model loader、node cache、transform update、animation segment playback、impact tick retiming、cancel handling、fallback 與 cleanup，所有資料值都從 snapshot/generated metadata 進入。
7. 執行 template tests、attack cancel tests、omfx build；用 `run.bat` 或 TD_1 dev run 目視確認 Saika 使用 3D model，會切換 move/attack/critical/sniper 動作，且前搖取消不出傷害、後搖取消保留已擊中結果。

Rollback 很簡單：移除或停用 `saika_magoichi.render` metadata，omfx 會回到既有 2D fallback；不需要資料遷移。

## Open Questions

- `scale`、`pitch_offset_deg`、`roll_offset_deg`、`yaw_offset_deg`、`z_offset`、`muzzle_bone` 與四個 animation segment tick ranges 的最終值仍需要在載入實際 FBX 後目視微調。
- 仍需確認 Saika FBX 內部 material reference 的 filename 是否能讓 Fyrox `RecursiveUp` 自動找到 `saika_magoichi_mat.png`；若不能，走 metadata texture manual fallback。
- 現有 damage/attack render cues 是否已能標示 hero critical hit 需要實作階段確認；若不能，需要新增 render-only critical cue。
- 現有 input/attack scheduler 是否已保存 pending impact outcome 需要實作階段確認；若沒有，需要新增 attack sequence state 來支援 windup cancel 與 backswing interrupt。
