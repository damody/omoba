## 1. Content Metadata And Codegen

- [ ] 1.1 在 `scripts/lua_data/templates/heroes.lua` 為 `saika_magoichi` 加入 `render = { render_mode = "model_3d", model = "templates/heroes/saika_magoichi/saika_magoichi.fbx", texture = "templates/heroes/saika_magoichi/saika_magoichi_mat.png", scale = ..., yaw_offset_deg = ..., z_offset = ... }`，並確認路徑相對於 `scripts/lua_data/` 可解析
- [ ] 1.2 在 `omoba-template-ids/src/lib.rs` 新增 hero render metadata const 型別，例如 `HeroRenderModeC` 與 `HeroRenderMetadataConst`
- [ ] 1.3 在 `omoba-template-ids/build.rs` 擴充 `HeroEntry` 反序列化、metadata validation、generated const emission 與 `hero_render_metadata(HeroId)` lookup
- [ ] 1.4 新增或更新 `omoba-template-ids/tests/generated.rs`，驗證 `hero_render_metadata(HERO_SAIKA_MAGOICHI)` 回傳 Saika FBX/PNG path 與 positive scale，且無 metadata 的 hero 回傳 `None`

## 2. Snapshot Projection

- [ ] 2.1 在 `omfx/game/src/sim_runner.rs` 新增 hero render snapshot 型別與 `EntityRenderData.hero_render: Option<Box<...>>` 欄位，預設非 hero 為 `None`
- [ ] 2.2 在 `extract_snapshot` 中針對 `ScriptUnitTag.unit_id = "hero_<id>"` strip prefix，透過 `hero_by_name` 與 `hero_render_metadata` 填入 Saika hero render data
- [ ] 2.3 新增 snapshot 單元測試，覆蓋 Saika hero entity 帶出 render data、非 hero entity 不帶 render data，以及 runtime 不讀 Lua 的路徑

## 3. omfx Hero Model Rendering

- [ ] 3.1 確認 Fyrox 1.0 在本 workspace 可用的 FBX/model instantiate API，實作不新增外部 crate 的 `load_hero_model` 或等效 helper
- [ ] 3.2 新增 `scripts/lua_data` asset path resolver，支援 repo root、相對路徑與 executable ancestor 搜尋，並提供 FBX 與 PNG 載入診斷
- [ ] 3.3 在 `Game` state 新增 hero model resource/status cache 與 `hero_model_nodes: HashMap<u32, ...>`，避免穩定 snapshot 每幀重新載入或 instantiate
- [ ] 3.4 在 `update_sim_batches` 中建立或更新 Saika 3D node transform，套用 snapshot position、facing、scale、yaw offset 與 z offset
- [ ] 3.5 對成功使用 3D node 的 hero suppress generic 2D body quad 與 facing bar，同時保留 HP bar 與 hero panel snapshot 更新
- [ ] 3.6 在 `removed_entity_ids` 與 stale entity cleanup 路徑移除 hero 3D node，釋放 per-entity cache entry
- [ ] 3.7 若 FBX 或 texture 載入失敗，log diagnostic 一次並回到既有 2D fallback，不 panic、不影響 gameplay

## 4. Verification

- [ ] 4.1 執行 `cargo test --manifest-path omoba-template-ids/Cargo.toml`，確認 template codegen 與 hero render metadata tests 通過
- [ ] 4.2 執行 `cargo test --manifest-path omfx/Cargo.toml -p omfx` 或最小可行 omfx/sim_runner test，確認 snapshot 與 render helper tests 通過
- [ ] 4.3 執行 `cargo build --manifest-path omfx/Cargo.toml -p omfx`，確認前端可編譯
- [ ] 4.4 以 `run.bat` 或 TD_1 dev run 目視確認 `saika_magoichi` 使用 3D model，HP bar、hero panel、移動與技能仍正常
- [ ] 4.5 完成 code changes 後執行 `graphify update .` 更新 knowledge graph
