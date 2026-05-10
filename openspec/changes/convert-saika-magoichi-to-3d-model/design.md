## Context

目前 shipped content 的 canonical source 是 `scripts/lua_data` Lua builders，`omoba-template-ids/build.rs` 會在 build time 讀取後產生 Rust lookup。英雄 template 目前已有 id、顯示名稱、portrait、技能與數值，但沒有戰鬥場景用的 model metadata。

omfx 目前透過 `SimWorldSnapshot` 取得 entity render data。英雄、creep、projectile 走 batched quad，tower 則已有 script-owned render metadata 與 composite sprite pipeline。`saika_magoichi` 的 FBX 與 PNG 已存在於 `scripts/lua_data/templates/heroes/saika_magoichi/`，但 runtime 沒有任何路徑會讀取它們。

現有 camera 是 orthographic 3D camera，使用 XY 作為畫面平面、Z 作為 draw order depth。這次設計不改整個 camera 或 2D tower/creep pipeline，而是把 hero 3D model 作為單一 Scene node hierarchy 放進既有 XY 畫面平面，並用 metadata 調整 scale、Z layer 與 facing offset。

## Goals / Non-Goals

**Goals:**

- 讓 `saika_magoichi` 在戰鬥場景中使用 `saika_magoichi.fbx` 與 `saika_magoichi_mat.png` 呈現 3D visual。
- 讓 hero 3D metadata 由 `scripts/lua_data/templates/heroes.lua` 宣告，並透過 `omoba-template-ids` generated lookup 供 runtime 使用。
- 讓 `SimWorldSnapshot` 對 hero entity expose render metadata，omfx 可依 snapshot 建立、更新、移除 hero 3D node。
- 模型載入失敗時保留既有 2D batched quad fallback，不影響選取、移動、攻擊、技能或 lockstep determinism。
- 不新增外部 crate；優先使用 Fyrox 既有 model/resource/scene API。

**Non-Goals:**

- 不重做 camera、地圖座標系、tower composite pipeline 或 creep rendering。
- 不新增 skeletal animation、attack animation、ragdoll、shadow pass 或 hitbox 變更。
- 不搬移使用者已提供的 FBX/PNG 檔案；本 change 以 `scripts/lua_data/templates/heroes/saika_magoichi/` 為 Saika 3D asset canonical location。
- 不改 hero gameplay stats、abilities、portrait、ability icon、network protocol 或 server authoritative simulation。

## Decisions

### Hero render metadata 掛在 hero template

在 `scripts/lua_data/templates/heroes.lua` 的 `saika_magoichi` entry 加入 optional `render` table，例如：

```lua
render = {
  render_mode = "model_3d",
  model = "templates/heroes/saika_magoichi/saika_magoichi.fbx",
  texture = "templates/heroes/saika_magoichi/saika_magoichi_mat.png",
  scale = 0.01,
  yaw_offset_deg = 0.0,
  z_offset = 0.0,
}
```

理由是 hero visual 是 content 屬性，應與 tower render metadata 一樣由 scripts content 宣告，而不是在 omfx 針對 `saika_magoichi` 寫死路徑。`render_mode = "model_3d"` 讓沒有 metadata 的英雄維持現有 2D fallback。

替代方案是直接在 omfx hard-code `hero_saika_magoichi` 到 FBX/PNG 的 mapping。這雖然最短，但會破壞 content-owned asset pattern，之後新增英雄模型時必須改前端 source，因此不採用。

### Generated API 新增 optional hero render lookup

`omoba-template-ids` 新增 `HeroRenderMetadataConst` 與 `hero_render_metadata(HeroId) -> Option<&'static HeroRenderMetadataConst>`。`HeroEntry` 反序列化 `render` table；當 `render_mode = "model_3d"` 時驗證 `model` 非空、`scale > 0`，並保留 texture path、yaw offset 與 z offset。

理由是 omfx 已依賴 `omoba-template-ids`，可以在 sim snapshot extraction 階段用 generated lookup 取得資料，避免 runtime 讀 Lua 或 JSON。這也符合現有「Lua builders build-time only」契約。

替代方案是讓 omfx runtime 直接讀 `heroes.lua` 或掃描 asset directory。這會引入 runtime Lua/data parsing，違反目前 build-time codegen 的架構，因此不採用。

### Snapshot 將 hero render metadata 放在 hero entity 上

`EntityRenderData` 新增 optional boxed `hero_render`，只在 hero entity 且對應 template 有 3D metadata 時填入。`ScriptUnitTag.unit_id` 對 hero 目前是 `hero_<id>`，snapshot extraction 會 strip `hero_` 後用 `hero_by_name` 與 `hero_render_metadata` lookup。

理由是 hero 數量低，把少量靜態 metadata clone 到 hero entity snapshot 比新增 `hero_templates: Arc<Vec<_>>` 與 omfx template cache 更簡單，也避免新的 static lifecycle 邏輯。非 hero row 只增加一個 `None` 指標，與現有 `hero_ext` 模式一致。

替代方案是仿照 tower template snapshot 建立 shared `Arc<Vec<HeroRenderTemplateSnapshot>>`。這對大量 templates 比較好，但目前需求只針對少量英雄，會增加更多 cache 與同步程式碼，因此先不採用。

### omfx 以 per-entity node cache 呈現 3D hero

omfx 新增 `hero_model_nodes: HashMap<u32, HeroModelRender>` 與 asset load status cache。`update_sim_batches` 遇到 hero 且 `hero_render.render_mode == "model_3d"` 時，嘗試載入/instantiate model，將 root node 放到 `world_to_render(e)` 的 XY 位置與 hero Z layer，並用 `facing_rad + yaw_offset_deg` 更新旋轉。

有 3D node 的 hero 會把原本 body quad 寫成透明或極小 quad，並跳過 generic facing bar，避免同一英雄同時出現 2D placeholder 與模型。HP bar、hero panel 與 gameplay input 仍讀 snapshot entity，不依賴 3D node。

替代方案是把 hero 3D model 也塞進 batched mesh。FBX 是 scene/model hierarchy，不適合現有 quad batch；用 Fyrox scene nodes 可最小化變更。

### Asset path resolver 支援 `scripts/lua_data`

新增 hero 3D asset resolver，將 metadata path 視為相對於 `scripts/lua_data/` 的路徑，並比照 tower texture loader 搜尋 repo root、`../`、`../../` 與 executable ancestors。texture 會先依 FBX 內部 material reference 嘗試載入；若需要手動指定 diffuse texture，則使用 metadata 的 `texture` path 套用到模型 mesh material。

替代方案是把 model 複製到 `scripts/base_content/assets/heroes/`。長期也許更一致，但使用者已明確提供檔案位置；本 change 避免搬檔與破壞既有路徑。

## Risks / Trade-offs

- [Risk] FBX import 的實際座標軸、比例或材質綁定可能與 omfx camera 不一致 → Mitigation：metadata 保留 `scale`、`yaw_offset_deg`、`z_offset`；implementation 以 TD_1/dev run 實機畫面微調預設值。
- [Risk] Fyrox 1.0 model loading API 可能是 async/resource based，直接在 render loop 載入會卡幀 → Mitigation：使用 resource cache/status cache，只在第一次看到 asset 時 request/load，失敗結果也 cache 並 fallback。
- [Risk] 模型檔缺失或 PNG 解碼失敗會讓英雄不可見 → Mitigation：載入失敗時 log 一次並走既有 2D body/facing rendering。
- [Risk] 3D scene nodes 比 batched quad 貴 → Mitigation：只對有 3D metadata 的 hero 建立 node；entity 移除時清理 node；tower/creep stress hot path 不變。
- [Risk] 變更 snapshot struct 可能影響 tests → Mitigation：新增 default/empty metadata 行為，既有沒有 hero render metadata 的測試仍維持 2D fallback。

## Migration Plan

1. 擴充 `omoba-template-ids` hero metadata model 與 generated lookup，新增測試確認 `saika_magoichi` render metadata 生成正確。
2. 在 `scripts/lua_data/templates/heroes.lua` 為 `saika_magoichi` 宣告 3D metadata，路徑指向現有 FBX/PNG。
3. 擴充 `SimWorldSnapshot` hero entity render projection，讓 omfx 可從 snapshot 取得 optional `hero_render`。
4. 在 omfx 新增 hero model loader、node cache、transform update、fallback 與 cleanup。
5. 執行 template tests、omfx build；用 `run.bat` 或 TD_1 dev run 目視確認 Saika 使用 3D model。

Rollback 很簡單：移除或停用 `saika_magoichi.render` metadata，omfx 會回到既有 2D fallback；不需要資料遷移。

## Open Questions

- `scale`、`yaw_offset_deg` 與 `z_offset` 的最終值需要在載入實際 FBX 後目視微調。
- 若 FBX 已內嵌或引用材質，implementation 需確認 Fyrox 是否自動解析同目錄 PNG；若沒有，才套用 metadata 指定 texture。
