## 1. Content Metadata And Codegen

- [x] 1.1 取得可執行的 `assimp` CLI，執行 `assimp info scripts/lua_data/templates/heroes/saika_magoichi/saika_magoichi.fbx`，記錄 Saika FBX 有 4 meshes、32 bones、1 animation、24 animation channels，且唯一 animation name 是 `Take 001`
- [x] 1.2 執行 `assimp dump` 檢查 Saika base/action FBX：base `saika_magoichi.fbx` 的 `Take 001` duration 是 `1000` ticks、`tick_cnt = 30`；action sources 至少包含 `b01_ani_attack.fbx` duration `100`、`b01_ani_run.fbx` duration `23`、`b01_ani_stand3.fbx` duration `53`，tick rate 皆為 `30`；把 source inventory 寫入 scripts metadata，並確認 `attack`/`critical` impact tick
- [x] 1.3 在 `scripts/lua_data/templates/heroes.lua` 為 `saika_magoichi` 加入 `render = { render_mode = "model_3d", model = "templates/heroes/saika_magoichi/saika_magoichi.fbx", texture = "templates/heroes/saika_magoichi/saika_magoichi_mat.png", scale = ..., pitch_offset_deg = ..., roll_offset_deg = ..., yaw_offset_deg = ..., z_offset = ..., muzzle_bone = ..., animation_sources = { idle = ..., idle_2 = ..., idle_3 = ..., move = { model = "templates/heroes/saika_magoichi/b01_ani_run.fbx", animation = "Take 001", duration_ticks = 23.0, ticks_per_second = 30.0, timeline_offset_ticks = ... }, attack = { model = "templates/heroes/saika_magoichi/b01_ani_attack.fbx", animation = "Take 001", duration_ticks = 100.0, ticks_per_second = 30.0, timeline_offset_ticks = ... }, critical = ..., sniper = ... }, animations = { idle = ..., idle_2 = ..., idle_3 = ..., move = { source = "move", start_tick = 0.0, end_tick = 23.0, loop = true }, attack = { source = "attack", start_tick = 0.0, repeat_start_tick = ..., impact_tick = 22.0, end_tick = 100.0 }, critical = ..., sniper = ... } }`，並確認路徑相對於 `scripts/lua_data/` 可解析
- [x] 1.4 在 `omoba-template-ids/src/lib.rs` 新增 hero render metadata const 型別，例如 `HeroRenderModeC`、`HeroRenderMetadataConst`、`HeroAnimationSourceConst` 與 `HeroAnimationBindingConst`，其中 source const 保留 logical key、source model path、source animation name、duration ticks、ticks-per-second 與 timeline offset
- [x] 1.5 在 `omoba-template-ids/build.rs` 擴充 `HeroEntry` 反序列化、logical animation source inventory schema、animation binding schema、impact tick validation、metadata validation、generated const emission 與 `hero_render_metadata(HeroId)` lookup；validation 只依 scripts-declared source inventory 檢查，不解析 FBX、不 shell out 到 `assimp`
- [x] 1.6 新增或更新 `omoba-template-ids/tests/generated.rs`，驗證 `hero_render_metadata(HERO_SAIKA_MAGOICHI)` 回傳 Saika FBX/PNG path、positive scale、logical source `move`/`attack`/`critical`/`sniper` 的 FBX path、`Take 001` source animation、duration/tick rate、四個 binding，以及 `attack`/`critical` impact tick；無 metadata 的 hero 回傳 `None`
- [x] 1.7 確認 Saika 3D 所有資料值只存在於 `scripts/lua_data` source 或 generated output：模型路徑、貼圖路徑、scale、pitch/roll/yaw/z offset、muzzle bone、logical source keys、source animation name、source duration/tick rate、tick ranges、impact ticks 都不得新增到 `omfx/data` 或 omfx source hard-coded table

## 2. Attack Scheduler Cancel Semantics

- [x] 2.1 在 backend attack scheduler 中明確建立 attack sequence state，讓 windup、impact、backswing 具備可判定的 phase 與 sequence id
- [x] 2.2 讓已接受的移動指令在 attack windup、impact 前取消該 attack sequence，並確認不會產生 damage、projectile、hit outcome、critical result 或 impact-side effect
- [x] 2.3 讓已接受的技能指令在 attack windup、impact 前取消該 attack sequence，並確認技能照既有 validation 執行，取消的攻擊不造成傷害
- [x] 2.4 讓已接受的移動或技能指令在 impact 後、backswing 期間只取消剩餘後搖或後搖 lockout，不 rollback 已 committed 的 damage、projectile、hit outcome、critical result 或 cooldown accounting
- [x] 2.5 新增 backend 測試覆蓋 windup move cancel 無傷害、windup skill cancel 無傷害、backswing move cancel 保留傷害、backswing skill cancel 保留傷害

## 3. Snapshot Projection

- [x] 3.1 在 `omfx/game/src/sim_runner.rs` 新增 hero render snapshot 型別與 `EntityRenderData.hero_render: Option<Box<...>>` 欄位，預設非 hero 為 `None`
- [x] 3.2 在 `extract_snapshot` 中針對 `ScriptUnitTag.unit_id = "hero_<id>"` strip prefix，透過 `hero_by_name` 與 `hero_render_metadata` 填入 Saika hero render data
- [x] 3.3 在 snapshot projection 中提供 render-only animation state/cues：移動狀態、`sniper_mode` buff 狀態、attack phase cue、attack cancel cue，以及 critical attack cue 或可區分 normal/critical 的 render cue
- [x] 3.4 若現有 outcome/render queue 無法表示 critical attack 或 windup cancel，新增 render-only cue 或擴充既有 attack/damage cue；不得影響 gameplay state hash、damage calculation 或 protocol gameplay semantics
- [x] 3.5 新增 snapshot 單元測試，覆蓋 Saika hero entity 帶出 render data 與四個 animation bindings、`attack`/`critical` impact tick、非 hero entity 不帶 render data、`sniper_mode` 會產生 sniper state、windup cancel cue 會進 snapshot，以及 runtime 不讀 Lua 的路徑

## 4. omfx Generic Hero Model Rendering

- [x] 4.1 在 omfx 匯入 Fyrox 1.0.1 model/animation API：`Model`、`ModelResource`、`ModelResourceExtension`、`AnimationPlayerBuilder`、`Animation`、`Mesh`；將 `context.resource_manager` 傳入 `update_sim_batches` 或等效 helper，不新增外部 crate
- [x] 4.2 新增 generic `scripts/lua_data` asset path resolver，支援 repo root、相對路徑與 executable ancestor 搜尋，並提供 FBX 與 PNG 載入診斷；resolver 回傳實際 filesystem path 給 `ResourceManager::request::<Model>`，且不得加入 Saika 專屬 fallback path
- [x] 4.3 在 `Game` state 新增 hero model resource/status cache，保存 `ModelResource`、resolved paths、loading/failed/logged 狀態，避免穩定 snapshot 每幀重新 request 或重複 log
- [x] 4.4 在 `Game` state 新增 `hero_model_nodes: HashMap<u32, HeroModelRender>`，保存 root node、animation player、source animation handles 與 playback state，避免穩定 snapshot 每幀重新 instantiate
- [x] 4.5 模型載入完成後使用 `model_resource.begin_instantiation(scene).with_position(...).with_rotation(...).with_scale(...).finish()` 建立 node hierarchy；載入中或失敗時維持既有 2D fallback
- [x] 4.6 為每個 model instance 建立或使用專用 `AnimationPlayer`，針對每個 generated logical animation source 載入/取得 action `ModelResource`，呼叫 `action_model_resource.retarget_animations_to_player(root, player, &mut scene.graph)`，並以 logical source key 建立 animation handle mapping；不得只用非唯一的 `Animation::name()`
- [x] 4.7 實作 generic manual texture fallback：若 importer 未綁定 diffuse texture，使用 metadata texture path 載入 PNG，建立 `Material::standard()`、`bind("diffuseTexture", Some(texture))`、`MaterialResource::new_embedded(material)`，並套到 root hierarchy 下 `Mesh::surfaces_mut()` 的 surfaces
- [x] 4.8 在 `update_sim_batches` 中建立或更新 Saika 3D node transform，套用 snapshot position、facing、scale、pitch/roll/yaw offset 與 z offset；更新時只改 root transform，不重新 instantiate
- [x] 4.9 在 `update_sim_batches` 中依 snapshot/cue 選擇動畫：移動播放 `move` loop、一般攻擊播放 `attack` one-shot、爆擊播放 `critical` one-shot、`sniper_mode` 播放 `sniper` loop
- [x] 4.10 將 generated `start_tick`/`impact_tick`/`end_tick` 加上該 logical source 的 generated `timeline_offset_ticks` 後，再透過 `ticks_per_second` 轉成 Fyrox seconds，並用 `Animation::set_time_slice`、`set_time_position`、`set_loop`、`set_enabled`、`set_speed` 播放 data-driven segment
- [x] 4.11 讓 `attack`/`critical` animation retime 分成兩個 playback phase：`start_tick..impact_tick` 對齊 cue windup duration，impact 後切到 `impact_tick..end_tick` 對齊 cue backswing duration，hit frame 對齊 authoritative impact event
- [x] 4.12 收到 windup cancel cue 時停止或 blend out 同 sequence attack animation，且不顯示 hit frame、critical animation、projectile fire 或 damage-only impact effect
- [x] 4.13 收到 backswing interrupt 時可停止後搖動畫，但保留已經對應 committed impact 的 hit、projectile、recoil、critical 或 impact visual
- [x] 4.14 對成功使用 3D node 的 hero suppress generic 2D body quad 與 facing bar，同時保留 HP bar 與 hero panel snapshot 更新
- [x] 4.15 在 `removed_entity_ids` 與 stale entity cleanup 路徑移除 hero 3D node，釋放 per-entity cache entry 與 animation playback state
- [x] 4.16 若 FBX、texture 或 animation binding 載入失敗，log diagnostic 一次並回到既有 2D fallback，不 panic、不影響 gameplay
- [x] 4.17 檢查 omfx 實作保持 data-agnostic：不新增 Saika 專屬 path、scale、offset、`Take 001` range、action mapping 或 `match "saika_magoichi"` 之類 frontend-only mapping

## 5. Verification

- [x] 5.1 執行 `cargo test --manifest-path omoba-template-ids/Cargo.toml`，確認 template codegen 與 hero render metadata tests 通過
- [x] 5.2 執行 backend attack scheduler/cancel tests，確認前搖取消不造成傷害、後搖取消保留已擊中的攻擊結果
- [x] 5.3 執行 `cargo test --manifest-path omfx/Cargo.toml -p omfx` 或最小可行 omfx/sim_runner test，確認 snapshot 與 render helper tests 通過
- [x] 5.4 執行 `cargo build --manifest-path omfx/Cargo.toml -p omfx`，確認前端可編譯
- [x] 5.5 以 `run.bat` 或 TD_1 dev run 目視確認 `saika_magoichi` 使用 3D model，且移動、一般攻擊、爆擊攻擊、`sniper_mode` 都會切到對應 animation binding
- [x] 5.6 目視確認攻擊動畫前搖、hit frame、後搖與 backend timing 對齊；前搖中移動/技能取消不出傷害，後搖中取消仍保留已擊中的傷害或 projectile
- [x] 5.7 目視確認 HP bar、hero panel、移動、攻擊、技能與 `sniper_mode` gameplay 效果仍正常，且缺 model/texture/binding 時會 fallback 到 2D
- [x] 5.8 搜尋 omfx source，確認沒有 Saika 專屬 hard-code data；允許通用欄位名稱與 generated metadata 型別，禁止 `saika_magoichi` 專屬資料表或 `omfx/data` canonical asset path
- [x] 5.9 完成 code changes 後執行 `graphify update .` 更新 knowledge graph
