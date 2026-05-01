# omfx 3D Scene Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Migrate omfx from `fyrox::scene::dim2::rectangle::RectangleBuilder`-based 2D rendering to native Fyrox 3D `Mesh` + shared `SurfaceResource` per-color, taking 1000-entity stress from 5247 draw calls / 50 fps to ~30 draw calls / 100+ fps.

**Architecture:** All entity sprites become 3D `Mesh` nodes pointing to a single shared 1×1 quad `SurfaceResource` plus a per-color `MaterialResource` (5 entity types + 2 HP-bar colors + 1 facing-arrow color = 9 materials). Camera switches to 3D `OrthographicProjection` looking down -Z, visually identical top-down view. Per-frame transform updates use the existing `local_transform_mut().set_position(...)` pattern — Fyrox's `Mesh::collect_render_data` automatically merges Mesh nodes sharing the same `(SurfaceResource, Material)` into one `RenderDataBundle` with N instances → one draw call per (color, sprite-kind) pair. All low-frequency line/polygon visuals (preview circles, region outlines, debug paths) migrate to `SceneDrawingContext::add_line` (already used for explosion ring), batched into one draw call by Fyrox's debug-line renderer.

**Tech Stack:** Rust 1.91.0; Fyrox 1.0.1 (glow OpenGL backend); existing crates (`nalgebra`, `crossbeam-channel`, `serde_json`); **no new dependencies** (we drop `bytemuck` once batched_sprite.rs is removed).

**Build commands:**
- Compile: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`
- Smoke (regular map): `D:/omoba/run.bat`
- Stress benchmark: `D:/omoba/run_stress.bat`
- Inspect log: `grep "omfx_render\|omfx_frame" D:/omoba/omfx_app.log | tail -10`

**Submodule:** omfx is git submodule at `D:/omoba/omfx`. **All commits go inside the submodule** (`cd D:/omoba/omfx && git commit ...`). Parent-repo pointer bump after each Phase ships.

**Phase 0 baseline (recorded):**
- `update()`: 0.72ms ✅
- `pure_render_ms`: 17.0
- `draw_calls`: 5247
- `triangles`: 24184
- `fps`: 50

**Reference:** Design doc at `D:/omoba/docs/plans/2026-04-30-omfx-3d-migration-design.md`.

---

## Phase 1 — Squash revert batched-mesh attempts

**Phase goal:** omfx submodule HEAD goes from current `7ed97ee` (recovery toggle-off) back to a clean state on top of `1192336` (per-system tick stats commit, before any batched-mesh work). 14 commits replaced by 1 squash-revert commit.

### Task 1.1: Identify revert range

**Files:** None modified — verification only.

**Step 1: List the commits to revert**

Run from `D:/omoba/omfx`:
```bash
git log --oneline 1192336..HEAD
```

Expected output (15 commits, oldest → newest):
- `07b9fb1` perf(visual): explosion ring via SceneDrawingContext (**KEEP** — this works fine)
- `6f95b9e` chore(profile): add omfx per-frame timing (**KEEP** — useful)
- `b2af1e9` fix(profile): events_drained ... (**KEEP**)
- `1192336` chore(profile): include Fyrox renderer stats ... (**KEEP** — already at HEAD position when batched-mesh started; this is the tip we're reverting back TO)

Wait — `1192336` may already be the baseline tip. Re-check by running:
```bash
git log --oneline -20
```

The 14 commits to revert are everything from `fa0e93b` (first batched_sprite skeleton) to `7ed97ee` (toggle-off). Commit `07b9fb1` (explosion SceneDrawingContext) and `1192336` (frame stats) PREDATE the batched-mesh series — keep them.

**Step 2: Note the squash-revert range**

The squash will revert: `fa0e93b..7ed97ee` (inclusive both ends).

**No commit yet — verification only.**

---

### Task 1.2: Squash revert all 14 commits

**Files:**
- Delete: `D:/omoba/omfx/game/src/batched_sprite.rs`
- Modify: `D:/omoba/omfx/game/src/lib.rs` (revert all changes since fa0e93b)
- Modify: `D:/omoba/omfx/game/Cargo.toml` (drop `bytemuck` dep added in Task 1.4 of previous plan)

**Step 1: Use git revert with --no-commit to stage all 14 reverts**

```bash
cd D:/omoba/omfx
git revert --no-commit fa0e93b^..7ed97ee
```

This reverts each commit individually but defers the commit. Conflicts may surface — resolve by accepting the "incoming" (revert) side at each conflict marker.

If conflicts are gnarly, alternative approach:
```bash
cd D:/omoba/omfx
git reset --hard 1192336
```
**WARNING**: `git reset --hard` discards working tree. Only do this if `git status` is clean. This is safer than revert if there are no other commits AFTER the batched-mesh ones (verify with `git log --oneline 1192336..HEAD` matches exactly the 14 commits).

**Step 2: Verify state**

```bash
git status
ls game/src/batched_sprite.rs  # should not exist
grep -n "USE_BATCHED_BODY\|body_batch\|body_slot\|batched_sprite" D:/omoba/omfx/game/src/lib.rs
```

Expected:
- batched_sprite.rs gone
- Zero matches in lib.rs for any batched-mesh symbols
- `Cargo.toml` no longer contains bytemuck

**Step 3: Build clean**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release 2>&1 | tail -5
```

Expected: clean build.

**Step 4: Single squash commit**

```bash
cd D:/omoba/omfx && git add -A && git commit -m "revert: 14 batched-mesh attempts (incompatible with Fyrox 2D pipeline)

These 14 commits attempted to batch sprite rendering via 3D Mesh + custom
vertex layout, but Fyrox 1.0.1's standard_2d shader is registered for
RectangleVertex layout only — custom Mesh + standard_2d material doesn't
render through the dim2 pipeline. Reverting to start fresh with the proper
Fyrox-native approach: 3D camera + 3D Mesh + standard 3D material + shared
SurfaceResource for auto-instancing.

Reverted commits: fa0e93b..7ed97ee
Plan: docs/plans/2026-04-30-omfx-3d-migration-plan.md"
```

**Step 5: Verify final SHA**

```bash
git log --oneline -3
```

Expected: top commit is the new squash revert; below it `1192336` (or whatever was the pre-batched-mesh tip).

---

## Phase 2 — 3D camera + SharedSpriteResources + Z constants

**Phase goal:** Switch camera to 3D ortho top-down. Build the resource pool. **Visuals must look identical to baseline** at end of this phase (RectangleBuilder still in use everywhere; we just changed the camera + added unused resource pool).

### Task 2.1: Update Z constants to 3D depth range

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:59-67`

**Step 1: Edit the constants block**

Find lines 59-67:
```rust
const Z_BULLET: f32 = 0.000;
const Z_HP_BAR: f32 = 0.0005;
const Z_RING: f32 = 0.00075;
const Z_ENEMY: f32 = 0.001;
const Z_TOWER: f32 = 0.002;
const Z_REGION: f32 = 0.00225;
const Z_GRID_CELL: f32 = 0.003;
const Z_PATH: f32 = 0.004;
const Z_BACKGROUND: f32 = 0.005;
```

Replace with:
```rust
// Z layers in 3D camera frustum (camera at z=100 looking down -Z, near=0.1 far=1000).
// Smaller Z = closer to ground (back of view), bigger Z = closer to camera (front).
// We invert here vs old 2D convention: HP bar must render IN FRONT of body, so
// Z_HP_BAR must be GREATER than Z_ENEMY for the depth test to draw HP bar last.
// Wait — actually camera looks down -Z, so vertices with LARGER z are CLOSER to camera.
// To preserve old "2D smaller-Z = on-top" semantics, we keep ordering and reverse range:
// LARGEST z = closest to camera = drawn on top.
const Z_BACKGROUND: f32 = 0.5;
const Z_PATH: f32 = 1.0;
const Z_GRID_CELL: f32 = 1.5;
const Z_REGION: f32 = 2.0;
const Z_TOWER: f32 = 2.5;
const Z_ENEMY: f32 = 3.0;
const Z_RING: f32 = 3.5;
const Z_HP_BAR: f32 = 4.0;
const Z_BULLET: f32 = 4.5;
```

**Step 2: Build**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release 2>&1 | tail -5
```

Expected: clean compile (only constant values changed). Any visual ordering issues will surface in stress test at end of Phase 2.

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): Z constants → 3D depth range (0.5–4.5, larger Z = on top)"
```

---

### Task 2.2: Create sprite_resources.rs module skeleton

**Files:**
- Create: `D:/omoba/omfx/game/src/sprite_resources.rs`
- Modify: `D:/omoba/omfx/game/src/lib.rs:36` (add `mod sprite_resources;`)

**Step 1: Create the new file with skeleton**

`D:/omoba/omfx/game/src/sprite_resources.rs`:

```rust
//! Shared GPU resources for 3D sprite rendering.
//!
//! All entity sprites (body / HP bar bg / HP bar fg / facing arrow) are
//! 3D Mesh nodes pointing to a single shared 1×1 quad SurfaceResource and
//! a per-color MaterialResource. Fyrox's renderer auto-instances Mesh nodes
//! sharing the same (SurfaceResource, Material) → one draw call per pair.
//!
//! Total: 1 surface × 9 materials = 9 draw calls for all entity sprites.

use fyrox::core::color::Color;
use fyrox::material::{Material, MaterialResource};
use fyrox::scene::mesh::surface::{SurfaceData, SurfaceResource};

/// All shared sprite resources, owned by Game and lazily initialized.
pub struct SharedSpriteResources {
    pub quad: SurfaceResource,

    // Body sprite materials (per entity_type)
    pub mat_hero: MaterialResource,         // (50, 180, 50)
    pub mat_creep: MaterialResource,        // (220, 40, 40)
    pub mat_tower: MaterialResource,        // (50, 100, 220)
    pub mat_projectile: MaterialResource,   // (255, 230, 50)
    pub mat_default: MaterialResource,      // (200, 200, 200)

    // HP bar materials
    pub mat_hp_bg: MaterialResource,        // (0, 0, 0)
    pub mat_hp_fg: MaterialResource,        // (0, 220, 0)

    // Facing arrow material
    pub mat_facing: MaterialResource,       // (255, 200, 0)
}

impl SharedSpriteResources {
    pub fn new() -> Self {
        let quad_data = SurfaceData::make_quad(&Default::default());
        let quad = SurfaceResource::new_embedded(quad_data);

        Self {
            quad,
            mat_hero: make_color_material(Color::from_rgba(50, 180, 50, 255)),
            mat_creep: make_color_material(Color::from_rgba(220, 40, 40, 255)),
            mat_tower: make_color_material(Color::from_rgba(50, 100, 220, 255)),
            mat_projectile: make_color_material(Color::from_rgba(255, 230, 50, 255)),
            mat_default: make_color_material(Color::from_rgba(200, 200, 200, 255)),
            mat_hp_bg: make_color_material(Color::from_rgba(0, 0, 0, 255)),
            mat_hp_fg: make_color_material(Color::from_rgba(0, 220, 0, 255)),
            mat_facing: make_color_material(Color::from_rgba(255, 200, 0, 255)),
        }
    }

    /// Lookup body sprite material for an entity_type string.
    pub fn material_for(&self, entity_type: &str) -> &MaterialResource {
        match entity_type {
            "hero" => &self.mat_hero,
            "creep" | "enemy" => &self.mat_creep,
            "unit" | "tower" => &self.mat_tower,
            "bullet" | "projectile" => &self.mat_projectile,
            _ => &self.mat_default,
        }
    }
}

fn make_color_material(color: Color) -> MaterialResource {
    let mut mat = Material::standard();
    mat.bind(
        "diffuseColor",
        fyrox::material::MaterialProperty::Color(color),
    );
    MaterialResource::new_embedded(mat)
}
```

**IMPORTANT — API uncertainty**: The exact paths for `MaterialProperty::Color`, `Material::standard`, and `Material::bind` may differ in Fyrox 1.0.1. Verify against:
- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-material-1.0.1/src/lib.rs` — `Material::standard()` location, `bind` method
- Same crate for `MaterialProperty::Color` enum variant

If `Material::bind` returns Result, use `.expect("...")`. If method is `set_property` instead of `bind`, switch.

**Step 2: Add mod declaration to lib.rs**

After the existing `mod ` declarations (around line 36-43), add:
```rust
mod sprite_resources;
```

**Step 3: Build**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml 2>&1 | tail -10
```

Expected: clean compile (module is unused — `#![allow(warnings)]` covers).

If errors: fix the API references per Step 1's note.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/sprite_resources.rs game/src/lib.rs && git commit -m "feat(sprite_resources): shared quad + 9 colored materials for 3D sprite batching"
```

---

### Task 2.3: Add Game.sprite_resources field + lazy init

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs` — Game struct (~line 626) + update() (~line 1361)

**Step 1: Add field to Game struct**

In `pub struct Game { ... }`, after the network-related fields:

```rust
    /// Shared sprite GPU resources (single quad + 9 materials).
    /// Lazily initialized on first frame; reused for all entity sprite Meshes.
    #[visit(skip)] #[reflect(hidden)]
    sprite_resources: Option<sprite_resources::SharedSpriteResources>,
```

The `#[derive(Default)]` on Game covers `Option::default() = None`.

**Step 2: Lazy init in update()**

In `update()` after `scene.drawing_context.clear_lines()` (around line 1361):

```rust
        // Lazy init shared sprite resources on first frame.
        if self.sprite_resources.is_none() {
            self.sprite_resources = Some(sprite_resources::SharedSpriteResources::new());
        }
```

**Step 3: Build**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml 2>&1 | tail -5
```

Expected: clean. `SharedSpriteResources` may need `#[derive(Debug)]` for Game's Debug derive — add if compile errors.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): Game.sprite_resources field + lazy init"
```

---

### Task 2.4: Replace 2D camera with 3D ortho camera

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:1059-1075` (CameraBuilder), also `:2083-2090` (TD camera setup if separate)

**Step 1: Find both camera setup sites**

```bash
grep -n "CameraBuilder\|set_projection.*Orthographic" D:/omoba/omfx/game/src/lib.rs
```

There are likely 2 sites — one in `init_scene` (line ~1059), one in TD-mode runtime (line ~2083). Both need the same treatment.

**Step 2: Update primary camera builder (line ~1059)**

Find:
```rust
self.camera = CameraBuilder::new(BaseBuilder::new())
    .with_projection(Projection::Orthographic(OrthographicProjection {
        // ... 2D settings
    }))
    .build(&mut scene.graph);
```

Replace with:
```rust
self.camera = CameraBuilder::new(BaseBuilder::new().with_local_transform(
    TransformBuilder::new()
        .with_local_position(Vector3::new(0.0, 0.0, 100.0))
        .build(),
))
.with_projection(Projection::Orthographic(OrthographicProjection {
    z_near: 0.1,
    z_far: 1000.0,
    vertical_size: 10.0,
}))
.build(&mut scene.graph);
```

Camera at `(0, 0, 100)` looks down -Z by Fyrox default orientation. `vertical_size=10.0` matches existing zoom level.

**Step 3: Update TD-mode camera (line ~2083)**

Find the second `set_projection(Projection::Orthographic(...))` call and update similarly. Verify camera position is set to `(camera_x, camera_y, 100.0)` (z=100 for top-down).

**Step 4: Update viewport_update math (search for `half_height = 10.0 / WORLD_SCALE`)**

The cmd_tx ViewportUpdate computes `half_width / half_height` for backend. Verify these still work — `vertical_size = 10.0` matches the old assumption.

**Step 5: Build + smoke test**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release 2>&1 | tail -5
```

Run: `D:/omoba/run.bat` for 10 seconds.

Expected outcome:
- Game starts, scene renders
- View is top-down identical to before (entities visible at same positions)
- **If view is wrong** (entities black, missing, or rotated): camera orientation is off. Try setting explicit `with_local_rotation(UnitQuaternion::look_at_lh(&Vector3::new(0.0, 0.0, -1.0), &Vector3::y_axis()))` or similar.

If sprites are missing entirely → next task (mouse picking) won't matter; first fix this. Common issue: Fyrox 3D ortho camera with vertices at z<near gets clipped. Verify Z constants in Task 2.1 are within [0.1, 1000].

**Step 6: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): 3D ortho camera at (0,0,100) looking down -Z"
```

---

### Task 2.5: Phase 2 stress benchmark

**Files:** None — verification only.

**Step 1: Build release**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
```

**Step 2: Stress test 60s**

```bash
D:/omoba/run_stress.bat
```

**Step 3: Inspect log**

```bash
grep "omfx_render" D:/omoba/omfx_app.log | tail -5
```

Expected: `draw_calls ≈ 5247` (no change — still RectangleBuilder), `pure_render_ms ≈ 17`, `fps ≈ 50`.

**Acceptance**: numbers match baseline. If draw_calls changed significantly, something is wrong with camera setup (entities being culled / rendered twice / etc.). Debug before proceeding.

---

## Phase 3 — Body sprite → 3D Mesh

### Task 3.1: Add helper function build_sprite_mesh

**Files:** Modify `D:/omoba/omfx/game/src/sprite_resources.rs`

**Step 1: Add helper to SharedSpriteResources impl**

In `D:/omoba/omfx/game/src/sprite_resources.rs`, add to `impl SharedSpriteResources`:

```rust
    /// Build a 3D Mesh node referencing the shared quad + given material.
    /// Caller sets local_transform afterwards (position, scale, rotation).
    pub fn build_mesh(
        &self,
        scene: &mut fyrox::scene::Scene,
        material: fyrox::material::MaterialResource,
    ) -> fyrox::core::pool::Handle<fyrox::scene::node::Node> {
        use fyrox::scene::base::BaseBuilder;
        use fyrox::scene::mesh::surface::SurfaceBuilder;
        use fyrox::scene::mesh::MeshBuilder;

        MeshBuilder::new(BaseBuilder::new())
            .with_surfaces(vec![SurfaceBuilder::new(self.quad.clone())
                .with_material(material)
                .build()])
            .build(&mut scene.graph)
            .to_base()
    }
```

**Step 2: Build**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml 2>&1 | tail -5
```

Expected: clean.

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/sprite_resources.rs && git commit -m "feat(sprite_resources): build_mesh helper for spawn sites"
```

---

### Task 3.2: Migrate spawn_entity body to 3D Mesh

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:3525-3543` (body Rectangle build)

**Step 1: Replace body sprite construction**

Find the existing body Rectangle builder around line 3533-3543:

```rust
let node: Handle<Node> = RectangleBuilder::new(
    BaseBuilder::new().with_local_transform(
        TransformBuilder::new()
            .with_local_position(Vector3::new(-x, y, z))
            .with_local_scale(Vector3::new(size, size, f32::EPSILON))
            .build(),
    ),
)
.with_color(color)
.build(&mut scene.graph)
.transmute();
```

Replace with:

```rust
let node: Handle<Node> = {
    let resources = self.sprite_resources.as_ref()
        .expect("sprite_resources not initialized — should be lazy-init in update()");
    let material = resources.material_for(entity_type).clone();
    let handle = resources.build_mesh(scene, material);
    // Set initial transform after build
    scene.graph[handle]
        .local_transform_mut()
        .set_position(Vector3::new(-x, y, z))
        .set_scale(Vector3::new(size, size, 1.0));
    handle
};
```

Key differences vs old code:
- Uses MeshBuilder via `resources.build_mesh()` instead of RectangleBuilder
- Scale's Z is `1.0` (unit quad's Z is 0; non-zero Z scale is harmless but `1.0` is conventional)
- No `.transmute()` — `build_mesh` already returns `Handle<Node>` via `to_base()`
- The `color` variable used in old `with_color(color)` is no longer needed (color lives in shared material)

**Step 2: Build**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release 2>&1 | tail -10
```

Expected: clean compile.

**Step 3: Smoke test**

Run: `D:/omoba/run.bat` for 15 seconds.

Expected: entity bodies visible at correct positions / colors. **If invisible**: likely `Material::standard()` requires diffuse texture binding. Check Fyrox source `fyrox-material-1.0.1/src/shader/standard/standard.shader` for required uniforms; may need to bind a fallback white texture in `make_color_material`.

If visible but wrong color → diffuse_color binding may use different name (try "albedoColor" or check shader source).

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): entity body sprite → 3D Mesh + shared (quad, material)"
```

---

### Task 3.3: Phase 3 stress benchmark

**Files:** None — verification.

**Step 1: Build + stress**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run_stress.bat
```

**Step 2: Inspect log**

```bash
grep "omfx_render" D:/omoba/omfx_app.log | tail -5
```

Expected: `draw_calls ~ 5247 - 1000 + 5 = ~4252` (entity body now ~5 draw calls instead of 1000+).

**Acceptance**:
- draw_calls drops by ~1000
- entity body visible
- FPS slightly improves (~55-65)

If draw_calls didn't drop → Fyrox auto-instancing isn't working. Verify:
```bash
# Check if the SurfaceResource is being shared (Arc::ptr_eq across spawn sites)
# Add a temporary log in build_mesh: log::info!("quad ptr: {:p}", self.quad.as_ref());
# All spawn sites should print the same pointer.
```

If not shared → `self.quad.clone()` may not preserve Arc identity. Check `SurfaceResource` clone semantics.

---

## Phase 4 — HP bar → 3D Mesh

### Task 4.1: Migrate HP bar bg + fg in spawn_entity

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:3546-3575` (HP bar Rectangles)

**Step 1: Replace HP bar construction**

Find the existing HP bar block:

```rust
let (hp_bar_bg, hp_bar_fg) = if health.is_some() {
    let bar_y = y + size * 0.5 + 0.1;
    let bg = RectangleBuilder::new(...)
        .with_color(Color::from_rgba(0, 0, 0, 255))
        .build(&mut scene.graph)
        .transmute();
    let fg = RectangleBuilder::new(...)
        .with_color(Color::from_rgba(0, 220, 0, 255))
        .build(&mut scene.graph)
        .transmute();
    (Some(bg), Some(fg))
} else {
    (None, None)
};
```

Replace with:

```rust
let (hp_bar_bg, hp_bar_fg) = if health.is_some() {
    let bar_y = y + size * 0.5 + 0.1;
    let resources = self.sprite_resources.as_ref().expect("...");
    let bg = resources.build_mesh(scene, resources.mat_hp_bg.clone());
    scene.graph[bg].local_transform_mut()
        .set_position(Vector3::new(-x, bar_y, Z_HP_BAR))
        .set_scale(Vector3::new(0.8, 0.06, 1.0));
    let fg = resources.build_mesh(scene, resources.mat_hp_fg.clone());
    scene.graph[fg].local_transform_mut()
        .set_position(Vector3::new(-x, bar_y, Z_HP_BAR + 0.001))
        .set_scale(Vector3::new(0.8, 0.06, 1.0));
    (Some(bg), Some(fg))
} else {
    (None, None)
};
```

**Step 2: Build + smoke**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat  # 10 seconds
```

Expected: HP bars visible above entities (black bg + green fg overlapping). If fg covers bg entirely (no black border), the depth difference (0.001) may not be enough → bump to 0.01.

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): HP bar bg + fg → 3D Mesh + shared materials"
```

---

### Task 4.2: Verify per-frame interp HP scale animation

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:1652-1673` (HP bar update in interp loop)

**Step 1: Find existing HP bar update in interp loop**

```bash
grep -n "hp_bar_fg\|hp_ratio\|set_scale" D:/omoba/omfx/game/src/lib.rs | head -20
```

The current code likely sets fg's scale based on hp_ratio. With 3D Mesh + standard material, `set_scale(width, height, 1.0)` should work the same as Rectangle.

**Step 2: Verify the scale animation pattern is compatible**

Read the section. If it does:
```rust
if let Some(fg) = entity.hp_bar_fg {
    scene.graph[fg].local_transform_mut()
        .set_scale(Vector3::new(0.8 * hp_ratio, 0.06, f32::EPSILON));
}
```

Change `f32::EPSILON` → `1.0` (3D mesh wants non-zero Z scale; EPSILON might cause issues):
```rust
        .set_scale(Vector3::new(0.8 * hp_ratio, 0.06, 1.0));
```

Also verify position offset for left-anchored fg (fg shrinks from right):
```rust
let fg_x = -pos.x - (0.8 - 0.8 * hp_ratio) * 0.5;
scene.graph[fg].local_transform_mut()
    .set_position(Vector3::new(fg_x, bar_y, Z_HP_BAR + 0.001))
    .set_scale(Vector3::new(0.8 * hp_ratio, 0.06, 1.0));
```

**Step 3: Build + smoke + commit**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat  # 10 seconds, take damage on hero to see HP shrink
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "fix(omfx): HP bar fg scale Z=1.0 + left-anchored width animation"
```

---

### Task 4.3: Phase 4 stress benchmark

**Files:** None — verification.

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run_stress.bat
grep "omfx_render" D:/omoba/omfx_app.log | tail -5
```

Expected: `draw_calls ~ 4252 - 2000 + 2 = ~2254`. FPS ~70-90.

---

## Phase 5 — facing arrow → 3D Mesh

### Task 5.1: Migrate build_facing_arrow

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs` — `build_facing_arrow` fn (~line 4255)

**Step 1: Find existing build_facing_arrow**

```bash
grep -n "fn build_facing_arrow" D:/omoba/omfx/game/src/lib.rs
```

Read the function — it likely builds a small Rectangle with rotation.

**Step 2: Replace body with 3D Mesh build**

Modify to use `SharedSpriteResources::build_mesh` with `resources.mat_facing.clone()`:

```rust
fn build_facing_arrow(
    scene: &mut Scene,
    resources: &sprite_resources::SharedSpriteResources,
    x: f32,
    y: f32,
    size: f32,
    facing: f32,
) -> Handle<Node> {
    let length = (size * 0.7).max(0.12);
    let thickness = (size * 0.15).max(0.04);
    let render_angle = std::f32::consts::PI - facing;
    let center_offset = length * 0.5;
    let center_x = x + render_angle.cos() * center_offset;
    let center_y = y + render_angle.sin() * center_offset;

    let handle = resources.build_mesh(scene, resources.mat_facing.clone());
    scene.graph[handle].local_transform_mut()
        .set_position(Vector3::new(-center_x, center_y, Z_HP_BAR + 0.0005))
        .set_scale(Vector3::new(length, thickness, 1.0))
        .set_rotation(UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            render_angle,
        ));
    handle
}
```

**Step 3: Update callers**

Find all calls to `build_facing_arrow` and update signature (now takes `&SharedSpriteResources`):

```bash
grep -n "build_facing_arrow(" D:/omoba/omfx/game/src/lib.rs
```

In spawn_entity:
```rust
let facing_arrow = if health.is_some() {
    let resources = self.sprite_resources.as_ref().expect("...");
    Some(build_facing_arrow(scene, resources, x, y, size, initial_facing))
} else {
    None
};
```

**Step 4: Update per-frame facing arrow rotation in interp loop**

```bash
grep -n "facing_arrow\|set_rotation" D:/omoba/omfx/game/src/lib.rs | head -10
```

Find the interp loop's facing arrow update. Replace transform updates with:

```rust
if let Some(arrow) = entity.facing_arrow {
    let length = (size * 0.7).max(0.12);
    let render_angle = std::f32::consts::PI - entity.facing;
    let center_offset = length * 0.5;
    let center_x = pos.x + render_angle.cos() * center_offset;
    let center_y = pos.y + render_angle.sin() * center_offset;
    scene.graph[arrow].local_transform_mut()
        .set_position(Vector3::new(-center_x, center_y, Z_HP_BAR + 0.0005))
        .set_rotation(UnitQuaternion::from_axis_angle(
            &Vector3::z_axis(),
            render_angle,
        ));
}
```

**Step 5: Build + smoke + commit**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat  # 10 seconds, watch facing arrows rotate as hero turns
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): facing arrow → 3D Mesh + shared material with z-axis rotation"
```

---

### Task 5.2: Phase 5 stress benchmark

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run_stress.bat
grep "omfx_render" D:/omoba/omfx_app.log | tail -5
```

Expected: `draw_calls ~ 2254 - 1000 + 1 = ~1255`. FPS ~80-110.

---

## Phase 6 — Mouse picking via picking_ray

### Task 6.1: Replace mouse_world_pos calculation

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:2890+` (search for `mouse_world_pos`)

**Step 1: Find mouse_world_pos calculation**

```bash
grep -n "mouse_world_pos\|cursor.*world\|screen_to_world" D:/omoba/omfx/game/src/lib.rs | head -10
```

The current code computes `mouse_world_pos` from cursor position via direct ortho math. This needs to become a 3D `Camera::make_picking_ray` cast.

**Step 2: Replace with picking_ray**

```rust
let camera_node = &scene.graph[self.camera];
let camera = camera_node.cast::<fyrox::scene::camera::Camera>()
    .expect("self.camera not a Camera node");
let cursor_uv = Vector2::new(
    cursor.x / window_size.x,
    1.0 - cursor.y / window_size.y,  // Y-flip if Fyrox cursor is top-down
);
let ray = camera.make_picking_ray(cursor_uv);
// Intersect ray with z=0 plane (ground)
let t = -ray.origin.z / ray.dir.z;
let world_x = ray.origin.x + t * ray.dir.x;
let world_y = ray.origin.y + t * ray.dir.y;
self.mouse_world_pos = Vector2::new(-world_x, world_y);  // -x flip preserved
```

**IMPORTANT**: Cursor Y-axis convention may be top-down (winit / window) or bottom-up. Test by hovering top-left corner — if `mouse_world_pos` shows top-right of world, adjust the `1.0 - cursor.y` math.

**Step 3: Build + smoke**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat
```

Test in TD mode:
- Click "Tower" button to enter placement mode
- Move mouse — preview circle should follow cursor exactly
- Click empty grid — tower should spawn at clicked position
- Click existing tower — selection circle should appear on it

If cursor offset / wrong direction → adjust Y-flip / `-x` flip.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): mouse picking via Camera::make_picking_ray + z=0 plane intersection"
```

---

## Phase 7 — Per-frame circles → SceneDrawingContext

### Task 7.1: Rewrite build_circle_outline

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:4098-4130` (`build_circle_outline`)

**Step 1: Replace fn body**

Old signature: `fn build_circle_outline(scene: &mut Scene, center, radius, segments, thickness, color, z) -> Vec<Handle<Node>>`

Change to push lines to drawing_context (returns empty Vec to keep signature compatibility):

```rust
fn build_circle_outline(
    scene: &mut Scene,
    center: Vector2<f32>,
    radius: f32,
    segments: usize,
    _thickness: f32,  // unused (lines are 1px screen-space)
    color: Color,
    z: f32,
) -> Vec<Handle<Node>> {
    use fyrox::scene::debug::Line;
    use fyrox::core::algebra::Vector3;

    let mut prev = Vector3::new(-(center.x + radius), center.y, z);
    for k in 1..=segments {
        let theta = (k as f32) * std::f32::consts::TAU / (segments as f32);
        let (s, c) = theta.sin_cos();
        let next = Vector3::new(
            -(center.x + radius * c),
            center.y + radius * s,
            z,
        );
        scene.drawing_context.add_line(Line { begin: prev, end: next, color });
        prev = next;
    }
    Vec::new()  // No handles — caller can drop the empty Vec
}
```

**Step 2: Update callers**

Callers store `Vec<Handle<Node>>` and use `scene.graph.remove_node(h)` to clean up. With drawing_context (immediate-mode), there's no cleanup needed. Callers may still drop the empty Vec safely.

`grep -n "build_circle_outline\|drain.*remove_node" D:/omoba/omfx/game/src/lib.rs` — find callers.

For per-frame rebuilders (preview, selection, explosion already done): the existing `for h in vec.drain(..) { scene.graph.remove_node(h); }` becomes a no-op (Vec is empty) — leave it for now, will clean up in Phase 8.

For init-once (BlockedRegion / TD path): these need to be **moved into per-frame update** since drawing_context is cleared every frame.

**Step 3: Build + smoke**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat  # 10 seconds, click tower to see selection ring (now thin lines)
```

Expected: tower selection circle visible as thin outline.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "perf(omfx): build_circle_outline → SceneDrawingContext lines"
```

---

### Task 7.2: Rewrite build_polygon_outline + build_path_segment

Same pattern as Task 7.1 — replace `RectangleBuilder` line segments with `drawing_context.add_line` calls.

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs` — both helpers (search for fn definitions)

**Step 1: Locate**

```bash
grep -n "fn build_polygon_outline\|fn build_path_segment" D:/omoba/omfx/game/src/lib.rs
```

**Step 2: Replace each** with drawing_context.add_line analog.

**Step 3: Build + commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "perf(omfx): build_polygon_outline + build_path_segment → SceneDrawingContext lines"
```

---

### Task 7.3: Move BlockedRegion + TD path to per-frame redraw

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs` — region/path init sites + update() loop

**Step 1: Cache region/path data on Game struct**

If not already, add to Game:
```rust
#[visit(skip)] #[reflect(hidden)]
cached_blocked_regions: Vec<Vec<Vector2<f32>>>,  // polygon vertices
#[visit(skip)] #[reflect(hidden)]
cached_td_paths: Vec<Vec<Vector2<f32>>>,
```

**Step 2: When backend sends region/path data**, update cache instead of building scene nodes.

**Step 3: In update(), push lines from cache after clear_lines()**:

```rust
for poly in &self.cached_blocked_regions {
    // push edges as Line into drawing_context
}
for path in &self.cached_td_paths {
    // push segments
}
```

**Step 4: Build + smoke + commit**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run.bat  # check region outlines + path lines visible
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "perf(omfx): BlockedRegion + TD path debug → per-frame drawing_context lines"
```

---

### Task 7.4: Phase 7 stress benchmark

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run_stress.bat
grep "omfx_render" D:/omoba/omfx_app.log | tail -5
```

Expected: `draw_calls ≤ 30`. FPS ~100+.

---

## Phase 8 — dim2 imports cleanup + final benchmark

### Task 8.1: Remove dim2 import + unused helpers

**Files:** Modify `D:/omoba/omfx/game/src/lib.rs:29` (use statement) + various dead code

**Step 1: Remove `use fyrox::scene::dim2::rectangle::RectangleBuilder`**

```bash
grep -n "RectangleBuilder\|dim2::rectangle" D:/omoba/omfx/game/src/lib.rs
```

If grep returns only the `use` line and zero call sites → safe to remove the import.

If any call sites remain → those are missed by Phases 3-7. Migrate them.

**Step 2: Remove now-empty Vec returns from rebuilders**

The `build_circle_outline` etc. return `Vec::new()`. Callers do `for h in vec.drain(..) { scene.graph.remove_node(h); }` which is a no-op. Clean up these patterns:

```rust
// Before
let segs = build_circle_outline(...);
for (h, _) in segs { self.preview_nodes.push(h); }
// ... later
for h in self.preview_nodes.drain(..) { scene.graph.remove_node(h); }

// After
build_circle_outline(...);  // pushes lines to drawing_context directly
// preview_nodes Vec can be removed entirely
```

Remove `td_preview_nodes`, `td_selected_range_nodes`, `region_line_nodes`, `region_blocker_nodes`, `td_path_nodes` from Game struct if unused.

**Step 3: Build + commit**

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "refactor(omfx): drop dim2::rectangle import + handle Vec fields no longer needed"
```

---

### Task 8.2: Final stress benchmark + record metrics

**Files:** None — verification.

```bash
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release
D:/omoba/run_stress.bat
grep "omfx_render\|omfx_frame" D:/omoba/omfx_app.log | tail -10
```

**Acceptance criteria:**
- `draw_calls ≤ 30`
- `pure_render_ms ≤ 8`
- `fps ≥ 100`
- Visual smoke: hero / creep / tower / projectile / HP bar / facing arrow / preview / selection / region / path all visible
- Mouse picking accurate (TD placement works)

**If acceptance fails**: bisect by toggling Phase commits — `git revert` the most recent Phase, re-bench, find which step broke.

**Optional commit recording metrics**:
```bash
cd D:/omoba/omfx && git commit --allow-empty -m "chore(perf): final 3D migration benchmarks (draw_calls X / fps Y)"
```

---

## Verification Summary

After all 8 Phases ship, expected:

| Metric | Phase 0 | Phase 8 |
|---|---|---|
| `draw_calls` | 5247 | ≤ 30 |
| `triangles` | 24184 | ~24K (similar) |
| `pure_render_ms` | 17 | ≤ 8 |
| `fps` | 50 | ≥ 100 |

Plus zero usage of `fyrox::scene::dim2` in omfx codebase.

### Submodule pointer bump

After full sequence ships and benchmarks pass:

```bash
cd D:/omoba && git add omfx && git commit -m "chore: bump omfx for full 3D migration (1000-entity stress: 5247 → 30 draw calls, 50 → 100+ fps)"
```

---

## Critical Files

- **Modified throughout:** `D:/omoba/omfx/game/src/lib.rs`
- **New file:** `D:/omoba/omfx/game/src/sprite_resources.rs`
- **Reference (read-only):** `D:/omoba/docs/plans/2026-04-30-omfx-3d-migration-design.md`
- **Fyrox internals (read-only):**
  - `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/renderer/bundle.rs:1238-1254` (auto-instancing)
  - `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/surface.rs` (SurfaceData::make_quad, SurfaceResource)
  - `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/mod.rs` (MeshBuilder)
  - `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-material-1.0.1/src/lib.rs` (Material::standard, MaterialResource)
  - `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/debug.rs` (SceneDrawingContext, add_line)
