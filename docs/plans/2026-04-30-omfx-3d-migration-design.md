# omfx 3D Scene Migration Design

## Context

omfx 用 Fyrox 1.0.1 dim2 Rectangle 渲染 1000+ entity，stress 場景 5247 draw calls / 50fps。之前 batched-mesh 嘗試（Phase 1+2+3 共 13 commits）失敗 — 3D Mesh + 自訂 vertex layout 跟 dim2 pipeline + standard_2d shader 不相容。

本設計把 omfx 整個 scene 改成 Fyrox 原生 3D 路徑（保留 UI 不動），以利用 Fyrox renderer 對「N Mesh nodes 共用同一 (SurfaceResource, Material) 自動合併成 1 draw call」的內建支援（`fyrox-impl-1.0.1/src/renderer/bundle.rs:1238-1254`）。

預期：5247 draw calls → ~30，FPS 50 → 100+。

## 設計選擇（已 brainstorm 確認）

- **驗證策略**：B — 直接 commit full migration（不 PoC）
- **Camera 投影**：A — Top-down ortho，視覺上跟現在 100% 一樣
- **遷移範圍**：A — 全部 dim2::rectangle 廢掉，per-frame circles 用 SceneDrawingContext
- **起點**：C — Squash revert 14 個 batched-mesh commits 成單一 commit + 從那做

## 核心架構

### Camera

3D Camera 用 `Projection::Orthographic`，positioned at `(0, 0, 100)` 朝 -Z 方向。`vertical_size = 10.0` 保持目前 zoom 不變。z_near=0.1, z_far=1000.0。

Z 常數從 2D 範圍（0.000-0.005）改成 3D 範圍：
```
Z_BULLET = 0.5,  Z_HP_BAR = 1.0,  Z_RING = 1.5,
Z_ENEMY = 2.0,   Z_TOWER = 3.0,   Z_REGION = 4.0,
Z_GRID_CELL = 5.0,  Z_PATH = 6.0,  Z_BACKGROUND = 8.0
```
數值越大 = 越遠 / 越底層。

### SharedSpriteResources

```rust
pub struct SharedSpriteResources {
    pub quad: SurfaceResource,                // 共用 1×1 quad
    pub mat_hero, mat_creep, mat_tower,       // entity body colors
    pub mat_projectile, mat_default,
    pub mat_hp_bg, mat_hp_fg,                  // HP bar
    pub mat_facing,                            // facing arrow
}
```

`quad` 用 `SurfaceData::make_quad(identity)` + `SurfaceResource::new_embedded`。每個 `mat_*` 用 `Material::standard()` + `bind("diffuseColor", Color)`。

owned by `Game::sprite_resources: Option<SharedSpriteResources>`，lazy init in update() first frame。

### spawn_entity 改動

```rust
let resources = self.sprite_resources.as_ref().expect("...");
let body = MeshBuilder::new(BaseBuilder::new()
    .with_local_transform(TransformBuilder::new()
        .with_local_position(Vector3::new(-x, y, Z_ENEMY))
        .with_local_scale(Vector3::new(size, size, 1.0))
        .build()))
    .with_surfaces(vec![SurfaceBuilder::new(resources.quad.clone())
        .with_material(resources.material_for(entity_type).clone()).build()])
    .build(&mut scene.graph)
    .to_base();  // Handle<Mesh> → Handle<Node>
```

HP bar bg/fg + facing arrow 同 pattern，Material lookup 不同。

### Per-frame interp

跟現有 RectangleBuilder code 結構**完全一樣**：
```rust
scene.graph[entity.node].local_transform_mut()
    .set_position(Vector3::new(-pos.x, pos.y, z))
    .set_scale(Vector3::new(size, size, 1.0));  // unit quad 要顯式縮放
```

facing arrow rotation: `set_rotation(UnitQuaternion::from_axis_angle(&Vector3::z_axis(), facing))`。

HP bar fg 寬度動畫: `set_scale(Vector3::new(0.8 * hp_ratio, 0.06, 1.0))` + position 對齊 left edge。

### 預期 draw call 分布

| 視覺 | 個數 | draw calls |
|---|---|---|
| 5 entity body colors × N instance | ~1000 entity | 5 |
| HP bar bg / fg × N instance | ~1000 health-bearing | 2 |
| facing arrow × N instance | ~1000 | 1 |
| projectile bullet × N instance | ~50 | 1 |
| SceneDrawingContext lines（preview / selection / region / path） | 變動 | 1 |
| UI（HUD / ability / name label） | 30+ | 不變 |
| **scene 總計** | | **~10** |

### Mouse picking

`Camera::make_picking_ray(cursor_uv) → Ray`，跟 z=0 plane 求交：
```rust
let t = -ray.origin.z / ray.dir.z;
let world_x = ray.origin.x + t * ray.dir.x;
let world_y = ray.origin.y + t * ray.dir.y;
self.mouse_world_pos = Vector2::new(-world_x, world_y);  // -x flip 慣例
```

### 低頻視覺改 SceneDrawingContext

- preview circle / selection circle / explosion ring（已是）
- BlockedRegion outlines, TD path debug — 從 init-once Rectangle 改成每 frame push lines（drawing_context.clear_lines() 已在 update 開頭）
- grid cells（如有）— 純色用 lines，textured 用 instance mesh

## 實作 Phase 切分

每 Phase 獨立 commit + 獨立 stress benchmark verify：

1. **Squash revert** — 1 個 commit 還原 14 個 batched-mesh attempts。omfx 回到 `1192336` profile commit 之後的純淨 state。
2. **Camera + SharedSpriteResources** — 換 3D camera、建 9 個 Material，entity 還是 RectangleBuilder。視覺不變。
3. **Body sprite → 3D Mesh** — entity body 改 MeshBuilder。expect entity body draw calls ≤ 5。
4. **HP bar → 3D Mesh** — bg + fg 改 MeshBuilder。
5. **facing arrow → 3D Mesh** — rotation via UnitQuaternion。
6. **Mouse picking → picking_ray** — 改用 Camera::make_picking_ray。
7. **per-frame circles → SceneDrawingContext** — preview / selection / region / path 全 lines。
8. **dim2 imports cleanup** — 刪 RectangleBuilder import 跟相關 helper。

## Verification

每 Phase：
- `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release` clean
- `run.bat` 30s 視覺 smoke（hero 移動、creep、projectile、explosion）
- `run_stress.bat` 60s 看 `omfx_render draw_calls`、`fps`

最終 acceptance：
- `omfx_render draw_calls ≤ 30`（scene）+ UI ≈ 60 total
- `fps ≥ 100`
- 視覺 regression 通過（對照之前的 baseline screenshot 或眼測）
- Mouse picking 準確（preview 圈跟滑鼠走、選塔正確）

## Critical Files

- `D:/omoba/omfx/game/src/lib.rs`（主修改）
  - `:59-67` Z 常數
  - `:626` Game struct 加 `sprite_resources` field
  - `:1059, 2083` CameraBuilder 設置
  - `:2890+` mouse_world_pos 計算
  - `:3525-3690` spawn_entity 全部 RectangleBuilder
  - `:4042-4130` build_line_segment / build_circle_outline / build_polygon_outline 內部改 SceneDrawingContext
- `D:/omoba/omfx/game/src/sprite_resources.rs`（**新檔**） — SharedSpriteResources

## Risks

- Fyrox auto-instance 沒生效（每 Mesh 仍 1 draw call）→ verify SurfaceResource Arc::ptr_eq
- Mesh 不可見 → check material binding / camera frustum
- Z 順序錯 → 微調 Z 常數
- Mouse picking 偏移 → debug ray cast
- -x flip 慣例對不上 → 統一全 codebase
