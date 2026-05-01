# omfx Batched Mesh Sprite Rewrite Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Collapse 5247 per-frame draw calls (1000 entity × ~5 Rectangle nodes each) down to ~250 by routing all repeated entity sprites through a single 3D `Mesh` whose vertex buffer is rewritten each frame, taking omfx 1000-entity stress from 50 fps → 100+ fps.

**Architecture:** New module `omfx/game/src/batched_sprite.rs` exposing `BatchedSpriteMesh` — one Fyrox 3D `Mesh` node per visual class, pre-allocated `4096 * 4` vertex capacity, free-list slot allocator, CPU-mirror buffer written per-frame inside the existing entity-interp loop, single `vertex_buffer.modify()` flush at loop end (avoids `&mut scene.graph` borrow conflict). Per-vertex color (`u8 × 4`, normalized) consumed by Fyrox's built-in `Material::standard_2d()` shader at `layout(location=2)` — no custom shader needed. Phase 1 batches body sprite (verifies API + visuals); Phase 2 batches HP bars; Phase 3 batches facing arrows and consolidates everything into one Mesh = 1 draw call total.

**Tech Stack:** Rust 1.91.0 (locked in `D:/omoba/rust-toolchain.toml`); Fyrox 1.0.1 (glow OpenGL backend); `bytemuck` for `Pod`/`Zeroable` derives on the vertex struct; existing crates `nalgebra`, `serde`, `crossbeam-channel`. **No new dependencies.**

**Build commands (Windows cmd):**
- Compile: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`
- Test: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx`
- Stress benchmark: `D:/omoba/run_stress.bat`

**Submodule:** omfx is a git submodule at `D:/omoba/omfx`. **All commits in this plan go inside the submodule** (`cd D:/omoba/omfx && git commit ...`). Parent-repo pointer bump is a separate manual step at end of each Phase.

**Phase 0 baseline (already measured):**
- `update()` plugin: 0.72ms ✅
- `pure_render_ms`: 17.0
- `draw_calls`: 5247
- `triangles`: 24184
- `fps`: 50

---

## Phase 1 — Body Sprite Batching (PoC)

**Phase goal**: 1000 body Rectangle → 1 batched mesh = save ~1000 draw calls. Stress benchmark expectation: `draw_calls=5247 → ~4247`, `pure_render_ms=17 → ~14`, `fps=50 → ~60`.

**Rollback strategy**: Old `RectangleBuilder` body node is **not removed** in Phase 1 — it gets visibility=false. A `const USE_BATCHED_BODY: bool = true;` toggles the new path. Set false to fall back to old code if anything breaks.

---

### Task 1.1: Create batched_sprite module skeleton

**Files:**
- Create: `D:/omoba/omfx/game/src/batched_sprite.rs`
- Modify: `D:/omoba/omfx/game/src/lib.rs:36` (add `mod batched_sprite;` near other mod declarations)

**Step 1: Write the file with skeleton + types only**

Create `D:/omoba/omfx/game/src/batched_sprite.rs`:

```rust
//! Batched sprite renderer — collapses N per-entity Rectangle nodes into one
//! Fyrox Mesh + manually managed vertex buffer (1 draw call for N quads).
//!
//! Used by omfx update() to render entity body sprites, HP bars, and facing
//! arrows without paying per-node draw-call overhead at 1000+ entity scale.

use fyrox::core::algebra::{Vector2, Vector3};
use fyrox::core::pool::Handle;
use fyrox::scene::{node::Node, Scene};

/// Single vertex pushed to the GPU. Layout matches Fyrox standard2d shader
/// expected `position` (loc=0) + `tex_coord` (loc=1) + `color` (loc=2).
#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct BatchedVertex {
    pub position: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
    pub color: [u8; 4],
}

/// Per-quad parameters supplied by callers each frame.
#[derive(Clone, Debug)]
pub struct QuadParams {
    pub center: Vector2<f32>,
    pub size: Vector2<f32>,
    pub color: [u8; 4],
    pub rotation: f32,
    pub z: f32,
}

impl Default for QuadParams {
    fn default() -> Self {
        Self {
            center: Vector2::new(0.0, 0.0),
            size: Vector2::new(1.0, 1.0),
            color: [255, 255, 255, 255],
            rotation: 0.0,
            z: 0.0,
        }
    }
}

/// Single Mesh node + manually-written vertex buffer holding N quads.
pub struct BatchedSpriteMesh {
    pub mesh_handle: Handle<Node>,
    capacity: u32,
    cpu_mirror: Vec<BatchedVertex>,
    free_list: Vec<u32>,
    next_slot: u32,
    active: Vec<bool>,
    dirty: bool,
}

impl BatchedSpriteMesh {
    /// Stub — real impl in Task 1.5.
    pub fn new(_scene: &mut Scene, capacity: u32) -> Self {
        Self {
            mesh_handle: Handle::NONE,
            capacity,
            cpu_mirror: vec![BatchedVertex::default(); (capacity as usize) * 4],
            free_list: Vec::new(),
            next_slot: 0,
            active: vec![false; capacity as usize],
            dirty: false,
        }
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }

    pub fn live_count(&self) -> u32 {
        self.next_slot - self.free_list.len() as u32
    }
}
```

In `D:/omoba/omfx/game/src/lib.rs`, after line 36 (after `use std::collections::...` block), add:

```rust
mod batched_sprite;
```

**Step 2: Run build to verify compile**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: PASS. No warnings about unused module (we'll use it next task).

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/batched_sprite.rs game/src/lib.rs && git commit -m "feat(batched_sprite): skeleton module with BatchedVertex / QuadParams / BatchedSpriteMesh stub"
```

---

### Task 1.2: Slot allocator with TDD tests

**Files:**
- Modify: `D:/omoba/omfx/game/src/batched_sprite.rs` (add `alloc` / `free` / `is_active` methods + `#[cfg(test)] mod tests`)

**Step 1: Write the failing test**

Append to `D:/omoba/omfx/game/src/batched_sprite.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    fn empty_mesh(cap: u32) -> BatchedSpriteMesh {
        // Construct without touching scene — fields are testable in isolation.
        BatchedSpriteMesh {
            mesh_handle: Handle::NONE,
            capacity: cap,
            cpu_mirror: vec![BatchedVertex::default(); (cap as usize) * 4],
            free_list: Vec::new(),
            next_slot: 0,
            active: vec![false; cap as usize],
            dirty: false,
        }
    }

    #[test]
    fn alloc_returns_sequential_slots_until_capacity() {
        let mut m = empty_mesh(4);
        assert_eq!(m.alloc(), Some(0));
        assert_eq!(m.alloc(), Some(1));
        assert_eq!(m.alloc(), Some(2));
        assert_eq!(m.alloc(), Some(3));
        assert_eq!(m.alloc(), None, "5th alloc must fail at cap=4");
    }

    #[test]
    fn free_then_alloc_reuses_freed_slot() {
        let mut m = empty_mesh(4);
        let a = m.alloc().unwrap();
        let b = m.alloc().unwrap();
        m.free(a);
        let c = m.alloc().unwrap();
        assert_eq!(c, a, "freed slot must be reused before bump-pointer advances");
        assert!(m.is_active(b));
        assert!(m.is_active(c));
    }

    #[test]
    fn live_count_tracks_alloc_and_free() {
        let mut m = empty_mesh(8);
        assert_eq!(m.live_count(), 0);
        let a = m.alloc().unwrap();
        let b = m.alloc().unwrap();
        let c = m.alloc().unwrap();
        assert_eq!(m.live_count(), 3);
        m.free(b);
        assert_eq!(m.live_count(), 2);
        let _ = a;
        let _ = c;
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx batched_sprite::tests`
Expected: FAIL with `no method named alloc found for struct BatchedSpriteMesh` (and same for `free` / `is_active`).

**Step 3: Implement alloc / free / is_active**

In `batched_sprite.rs`, inside `impl BatchedSpriteMesh`, add (place these methods just after `live_count`):

```rust
    /// Reserve a new slot. Returns `None` if capacity reached.
    pub fn alloc(&mut self) -> Option<u32> {
        if let Some(reused) = self.free_list.pop() {
            self.active[reused as usize] = true;
            return Some(reused);
        }
        if self.next_slot >= self.capacity {
            return None;
        }
        let slot = self.next_slot;
        self.next_slot += 1;
        self.active[slot as usize] = true;
        Some(slot)
    }

    /// Release a previously-allocated slot. Caller is responsible for writing
    /// a degenerate quad (zero-size + alpha=0) to hide any residual pixels.
    pub fn free(&mut self, slot: u32) {
        if (slot as usize) < self.active.len() && self.active[slot as usize] {
            self.active[slot as usize] = false;
            self.free_list.push(slot);
        }
    }

    pub fn is_active(&self, slot: u32) -> bool {
        (slot as usize) < self.active.len() && self.active[slot as usize]
    }
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx batched_sprite::tests`
Expected: PASS, 3 / 3.

**Step 5: Commit**

```bash
cd D:/omoba/omfx && git add game/src/batched_sprite.rs && git commit -m "feat(batched_sprite): slot allocator (alloc / free / live_count) + unit tests"
```

---

### Task 1.3: write_quad with TDD on corner math

**Files:**
- Modify: `D:/omoba/omfx/game/src/batched_sprite.rs` (add `write_quad` + tests)

**Step 1: Write the failing test**

Append to `mod tests`:

```rust
    #[test]
    fn write_quad_fills_four_vertices_for_unit_quad() {
        let mut m = empty_mesh(4);
        let slot = m.alloc().unwrap();
        m.write_quad(slot, &QuadParams {
            center: Vector2::new(10.0, 20.0),
            size: Vector2::new(2.0, 2.0),
            color: [10, 20, 30, 40],
            rotation: 0.0,
            z: 0.5,
        });
        let base = (slot as usize) * 4;
        let v = &m.cpu_mirror[base..base + 4];
        // Corners CCW: BL, BR, TR, TL (pre-rotated; rot=0 => no change)
        assert_eq!(v[0].position, Vector3::new(9.0, 19.0, 0.5));
        assert_eq!(v[1].position, Vector3::new(11.0, 19.0, 0.5));
        assert_eq!(v[2].position, Vector3::new(11.0, 21.0, 0.5));
        assert_eq!(v[3].position, Vector3::new(9.0, 21.0, 0.5));
        for vert in v {
            assert_eq!(vert.color, [10, 20, 30, 40]);
        }
        assert!(m.dirty);
    }

    #[test]
    fn write_quad_rotation_90_deg_swaps_axes() {
        let mut m = empty_mesh(4);
        let slot = m.alloc().unwrap();
        m.write_quad(slot, &QuadParams {
            center: Vector2::new(0.0, 0.0),
            size: Vector2::new(2.0, 2.0),
            color: [255; 4],
            rotation: std::f32::consts::FRAC_PI_2,
            z: 0.0,
        });
        let base = (slot as usize) * 4;
        let v = &m.cpu_mirror[base..base + 4];
        // BL (-1,-1) rotated 90° CCW = (1, -1). Within float tolerance.
        let approx = |a: f32, b: f32| (a - b).abs() < 1e-4;
        assert!(approx(v[0].position.x, 1.0), "got {}", v[0].position.x);
        assert!(approx(v[0].position.y, -1.0), "got {}", v[0].position.y);
    }

    #[test]
    fn write_quad_collapses_freed_slot_to_zero_size() {
        let mut m = empty_mesh(4);
        let slot = m.alloc().unwrap();
        m.write_quad(slot, &QuadParams {
            center: Vector2::new(5.0, 5.0),
            size: Vector2::new(1.0, 1.0),
            color: [255; 4],
            rotation: 0.0,
            z: 0.1,
        });
        m.free(slot);
        // After free(), caller is expected to write a hidden-quad themselves
        // OR the next allocator user writes new data. Free does NOT clear
        // pixels — verify by re-writing zeros explicitly.
        m.write_quad(slot, &QuadParams::default()); // Default size=1x1 still visible — we must call hide_slot
        // Now hide it
        m.hide_slot(slot);
        let base = (slot as usize) * 4;
        for vert in &m.cpu_mirror[base..base + 4] {
            assert_eq!(vert.position, Vector3::new(0.0, 0.0, 0.0));
            assert_eq!(vert.color[3], 0, "alpha must be 0 for hidden slot");
        }
    }
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx batched_sprite::tests`
Expected: FAIL with `no method named write_quad` and `no method named hide_slot`.

**Step 3: Implement write_quad + hide_slot**

Inside `impl BatchedSpriteMesh`, add:

```rust
    /// Write 4 vertices for the given slot. CCW order: BL, BR, TR, TL.
    /// `rotation` is in radians, applied around `center`.
    pub fn write_quad(&mut self, slot: u32, p: &QuadParams) {
        debug_assert!((slot as usize) < self.active.len(), "slot out of range");
        let half_w = p.size.x * 0.5;
        let half_h = p.size.y * 0.5;
        let (sin, cos) = p.rotation.sin_cos();
        let local_corners = [
            (-half_w, -half_h), // BL
            ( half_w, -half_h), // BR
            ( half_w,  half_h), // TR
            (-half_w,  half_h), // TL
        ];
        let uvs = [
            Vector2::new(0.0, 1.0),
            Vector2::new(1.0, 1.0),
            Vector2::new(1.0, 0.0),
            Vector2::new(0.0, 0.0),
        ];
        let base = (slot as usize) * 4;
        for i in 0..4 {
            let (lx, ly) = local_corners[i];
            let wx = p.center.x + cos * lx - sin * ly;
            let wy = p.center.y + sin * lx + cos * ly;
            self.cpu_mirror[base + i] = BatchedVertex {
                position: Vector3::new(wx, wy, p.z),
                tex_coord: uvs[i],
                color: p.color,
            };
        }
        self.dirty = true;
    }

    /// Collapse a slot's quad to zero-size + alpha=0 so it stops rendering
    /// without removing the slot from the buffer. Use after `free()` if the
    /// slot was previously visible.
    pub fn hide_slot(&mut self, slot: u32) {
        let base = (slot as usize) * 4;
        for i in 0..4 {
            self.cpu_mirror[base + i] = BatchedVertex {
                position: Vector3::new(0.0, 0.0, 0.0),
                tex_coord: Vector2::new(0.0, 0.0),
                color: [0, 0, 0, 0],
            };
        }
        self.dirty = true;
    }
```

**Step 4: Run tests**

Run: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx batched_sprite::tests`
Expected: PASS, 6 / 6.

**Step 5: Commit**

```bash
cd D:/omoba/omfx && git add game/src/batched_sprite.rs && git commit -m "feat(batched_sprite): write_quad + hide_slot with rotation math + unit tests"
```

---

### Task 1.4: Fyrox MeshBuilder + VertexBuffer construction

**Files:**
- Modify: `D:/omoba/omfx/game/src/batched_sprite.rs` (real `new()` impl)

**Step 1: Replace stub `new()` with real impl**

Inside `batched_sprite.rs`, **replace** the existing stub `pub fn new(_scene, capacity)` with:

```rust
use fyrox::material::{Material, MaterialResource};
use fyrox::scene::base::BaseBuilder;
use fyrox::scene::mesh::buffer::{
    TriangleBuffer, VertexAttributeDataType, VertexAttributeDescriptor,
    VertexAttributeUsage, VertexBuffer,
};
use fyrox::scene::mesh::surface::{Surface, SurfaceData, SurfaceResource};
use fyrox::scene::mesh::vertex::AnimatedVertex; // placeholder; we'll define our own
use fyrox::scene::mesh::{MeshBuilder, RenderPath};
use fyrox::scene::transform::TransformBuilder;
use fyrox::asset::manager::ResourceManager;
use bytemuck::{Pod, Zeroable};

unsafe impl Pod for BatchedVertex {}
unsafe impl Zeroable for BatchedVertex {}

impl BatchedSpriteMesh {
    pub fn new(scene: &mut Scene, resource_manager: &ResourceManager, capacity: u32) -> Self {
        let cpu_mirror = vec![BatchedVertex::default(); (capacity as usize) * 4];

        // Vertex layout: position(F32×3, loc=0) + tex_coord(F32×2, loc=1)
        //                + color(U8×4, loc=2, normalized=true).
        let layout = vec![
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::Position,
                data_type: VertexAttributeDataType::F32,
                size: 3,
                divisor: 0,
                shader_location: 0,
                normalized: false,
            },
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::TexCoord0,
                data_type: VertexAttributeDataType::F32,
                size: 2,
                divisor: 0,
                shader_location: 1,
                normalized: false,
            },
            VertexAttributeDescriptor {
                usage: VertexAttributeUsage::Color,
                data_type: VertexAttributeDataType::U8,
                size: 4,
                divisor: 0,
                shader_location: 2,
                normalized: true,
            },
        ];

        let vb = VertexBuffer::new_with_layout(
            layout,
            cpu_mirror.len(),
            bytemuck::cast_slice(&cpu_mirror).to_vec(),
        )
        .expect("BatchedSpriteMesh: vertex layout invalid");

        // Triangle buffer: 2 triangles per quad, indices [0,1,2, 0,2,3] per slot.
        let mut indices = Vec::with_capacity((capacity as usize) * 6);
        for slot in 0..capacity as u32 {
            let base = slot * 4;
            indices.push([base, base + 1, base + 2]);
            indices.push([base, base + 2, base + 3]);
        }
        let tb = TriangleBuffer::new(indices);

        let surface_data = SurfaceData::new(vb, tb);
        let surface_res = SurfaceResource::new_ok(Default::default(), surface_data);

        let material = MaterialResource::new_ok(Default::default(), Material::standard_2d());

        let surface = Surface::new(surface_res).with_material(material);

        let mesh_handle = MeshBuilder::new(
            BaseBuilder::new().with_local_transform(TransformBuilder::new().build()),
        )
        .with_surfaces(vec![surface])
        .with_render_path(RenderPath::Forward)
        .build(&mut scene.graph);

        Self {
            mesh_handle,
            capacity,
            cpu_mirror,
            free_list: Vec::new(),
            next_slot: 0,
            active: vec![false; capacity as usize],
            dirty: false,
        }
    }
}
```

**IMPORTANT — verify imports:** Some of the `fyrox::scene::mesh::*` paths may differ in 1.0.1; if compile fails, grep `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/` for the canonical re-exports. Drop the placeholder `AnimatedVertex` import (it's unused — left only as a hint that vertex types live in that module).

Also: confirm `bytemuck` is a transitive dep. Check `D:/omoba/omfx/game/Cargo.toml`; if missing, add `bytemuck = "1"` to `[dependencies]`.

**Step 2: Run build to verify compile**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: clean compile. If errors, the most likely culprits are:
- Missing `bytemuck` dep → add to Cargo.toml
- `MaterialResource::new_ok` API renamed in 1.0.1 → check `fyrox-material-1.0.1/src/lib.rs`
- `Surface::new` signature differs → check `fyrox-impl-1.0.1/src/scene/mesh/surface.rs:262`
- `MeshBuilder::with_render_path` doesn't exist → drop that line; default is fine

**Step 3: Run existing slot/write_quad tests to verify they still pass**

The tests use a hand-rolled `empty_mesh()` constructor that bypasses `new()`, so they don't actually touch Fyrox APIs. Should still pass.

Run: `cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx batched_sprite::tests`
Expected: PASS, 6 / 6.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/batched_sprite.rs game/Cargo.toml && git commit -m "feat(batched_sprite): MeshBuilder construction with custom vertex layout (position + tex_coord + color)"
```

---

### Task 1.5: flush() implementation

**Files:**
- Modify: `D:/omoba/omfx/game/src/batched_sprite.rs` (add `flush` method)

**Step 1: Add flush method**

Inside `impl BatchedSpriteMesh`, append:

```rust
    /// Upload `cpu_mirror` to the GPU vertex buffer if dirty. Call once per
    /// frame after all `write_quad` updates. The caller MUST hold a `&mut Scene`
    /// — we touch the surface data behind the Mesh node.
    pub fn flush(&mut self, scene: &mut Scene) {
        if !self.dirty {
            return;
        }
        self.dirty = false;
        let mesh = scene.graph[self.mesh_handle]
            .as_any_mut()
            .downcast_mut::<fyrox::scene::mesh::Mesh>()
            .expect("BatchedSpriteMesh: handle not a Mesh node");
        let surfaces = mesh.surfaces_mut();
        if surfaces.is_empty() {
            return;
        }
        let surface_data = surfaces[0].data();
        let mut data_ref = surface_data.data_ref();
        let mut vb = data_ref.vertex_buffer.modify();
        let dst: &mut [BatchedVertex] = bytemuck::cast_slice_mut(vb.raw_data_mut());
        debug_assert_eq!(dst.len(), self.cpu_mirror.len());
        dst.copy_from_slice(&self.cpu_mirror);
        // VertexBufferRefMut::Drop triggers GPU re-upload — happens here.
    }
```

**Step 2: Run build**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: clean compile.

If `as_any_mut().downcast_mut::<Mesh>()` doesn't work, alternative pattern (Fyrox-specific):
```rust
if let Some(mesh) = scene.graph[self.mesh_handle].cast_mut::<fyrox::scene::mesh::Mesh>() {
```

If `vb.raw_data_mut()` doesn't exist, try `vb.cast_data_mut::<BatchedVertex>()` (returns `Result<&mut [BatchedVertex], _>`).

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/batched_sprite.rs && git commit -m "feat(batched_sprite): flush() — single per-frame vertex_buffer.modify() upload"
```

---

### Task 1.6: Add USE_BATCHED_BODY const + Game field

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — Game struct (~line 626), constants block (~line 76)

**Step 1: Add const flag**

Find the constants block in `lib.rs` (around line 73 where `COLLISION_RING_ENABLED` lives). Add **before or after** that line:

```rust
/// Phase 1 toggle: set to false to fall back to per-entity RectangleBuilder.
/// Used to A/B compare draw-call count and visual fidelity.
const USE_BATCHED_BODY: bool = true;
```

**Step 2: Add field to Game struct**

In `D:/omoba/omfx/game/src/lib.rs`, find the `Game` struct definition (line 626 area). Add after the existing `network` / `event_buffer` field block:

```rust
    /// Phase 1 batched-mesh body sprite renderer (1 draw call for N entities).
    /// Lazily initialized on first frame after the Fyrox graphics context becomes
    /// `Initialized` (we need a `&mut Scene` AND a `ResourceManager` ref to build
    /// the mesh; both are only available inside `Plugin::update()`).
    #[visit(skip)] #[reflect(hidden)]
    body_batch: Option<batched_sprite::BatchedSpriteMesh>,
```

`Option<>` because we can't construct `BatchedSpriteMesh` until update() is called — `Game::default()` doesn't have access to a `&mut Scene`.

**Step 3: Run build**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: clean compile (field is unused yet — `#![allow(warnings)]` covers).

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): Game.body_batch field + USE_BATCHED_BODY toggle"
```

---

### Task 1.7: Lazy init body_batch in update()

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — `update()` near `clear_lines()` (~line 1361)

**Step 1: Add init block**

In `update()`, find the existing `scene.drawing_context.clear_lines();` line near the top (added in earlier task; ~line 1362). Insert **right after** it:

```rust
        // Phase 1 batched body mesh — lazy init on first frame (needs &mut Scene).
        if USE_BATCHED_BODY && self.body_batch.is_none() {
            self.body_batch = Some(batched_sprite::BatchedSpriteMesh::new(
                scene,
                &context.resource_manager,
                4096,
            ));
        }
```

If `context.resource_manager` is wrong path, check `fyrox-impl-1.0.1/src/plugin/mod.rs` for the correct field name (might be `context.resource_manager` or accessed via `context.resource_manager()`).

**Step 2: Run build**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`
Expected: clean compile. The `body_batch` field is now initialized on first frame but not yet used.

**Step 3: Smoke test — start game and verify mesh node is created**

Run: `D:/omoba/run.bat` (regular map, NOT stress).

Expected: game starts normally, no crashes, normal map renders correctly. (No visual change — body_batch is created but no quads written yet.)

Press Esc / close window after ~5 seconds.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): lazy-init body_batch on first update()"
```

---

### Task 1.8: NetworkEntity gains body_slot field

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — `NetworkEntity` struct (~line 3653)

**Step 1: Add field**

Find `struct NetworkEntity { ... }` (around line 3653). Add after the existing `node: Handle<Node>` field:

```rust
    /// Phase 1 — slot index in body_batch when USE_BATCHED_BODY is true.
    /// `None` for entities created before batch was initialized OR when toggle is off.
    body_slot: Option<u32>,
```

**Step 2: Update construction sites**

Find the `NetworkEntities::insert(id, NetworkEntity { ... });` call inside `spawn_entity` (around line 3653). Add `body_slot: None,` to the struct literal (just before the `extrap_velocity: 0.0,` line).

Search for any OTHER `NetworkEntity { ... }` struct-literal construction in the file. Likely only one (in `spawn_entity`), but verify with:

Run: `grep -n "NetworkEntity {" D:/omoba/omfx/game/src/lib.rs`

Update each such literal to include `body_slot: None,`.

**Step 3: Run build**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: clean compile.

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): NetworkEntity gains body_slot field"
```

---

### Task 1.9: spawn_entity allocates slot + writes initial quad

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — `spawn_entity` body sprite section (~line 3525-3543)

**Step 1: Conditionally hide old node + alloc + write**

Find the existing body Rectangle construction in `spawn_entity` (around line 3533):

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

Add **immediately after** the `.transmute();`:

```rust
        // Phase 1 — also register in body_batch and hide the legacy node.
        // Old `node` stays in the scene graph for rollback safety; we just
        // collapse its scale so it draws nothing.
        let body_slot = if USE_BATCHED_BODY {
            if let Some(batch) = self.body_batch.as_mut() {
                let s = batch.alloc();
                if let Some(s) = s {
                    batch.write_quad(s, &batched_sprite::QuadParams {
                        center: Vector2::new(-x, y),  // omfx convention: x flipped
                        size: Vector2::new(size, size),
                        color: [color.r, color.g, color.b, color.a],
                        rotation: 0.0,
                        z,
                    });
                    // Hide legacy node so we don't double-draw.
                    scene.graph[node].local_transform_mut()
                        .set_scale(Vector3::new(0.0, 0.0, f32::EPSILON));
                }
                s
            } else {
                None
            }
        } else {
            None
        };
```

Then in the `NetworkEntity { ... }` literal below (around line 3653), set:

```rust
            body_slot,
```

(Replace the `body_slot: None,` placeholder added in Task 1.8.)

**Step 2: Run build**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`
Expected: clean compile.

**Step 3: Smoke test**

Run: `D:/omoba/run.bat` for 10 seconds.

Expected: game runs, entities spawn. The new batched body sprites should render at the same positions as the old (now-hidden) Rectangles. Visually identical to before.

If sprites are MISSING: legacy node hide is working but batch render isn't visible — likely `flush()` not called yet (Task 1.10).
If sprites are DOUBLE: legacy hide didn't work — verify scale was actually set to 0.

Don't worry about per-frame movement yet (Task 1.10).

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): spawn_entity registers body in batch + hides legacy Rectangle"
```

---

### Task 1.10: Per-frame interp loop writes batch + flushes

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — entity interp loop (~line 1645-1694), end of interp phase (~line 1647)

**Step 1: Update interp loop body**

Find the entity interp loop (the section that reads `entity.target_position`, `entity.prev_position`, computes `pos`, and calls `scene.graph[entity.node].local_transform_mut().set_position(...)` for each entity, around line 1500-1571).

Right before the `set_position` line (where `pos` is computed for the entity body), insert:

```rust
        // Phase 1 — also write to batch buffer.
        if USE_BATCHED_BODY {
            if let (Some(batch), Some(slot)) =
                (self.body_batch.as_mut(), entity.body_slot)
            {
                let (color_arr, size_render, z_render) = match entity.entity_type.as_str() {
                    "hero" => ([50, 180, 50, 255], 0.4_f32, Z_ENEMY),
                    "creep" | "enemy" => ([220, 40, 40, 255], 0.3, Z_ENEMY),
                    "unit" | "tower" => ([50, 100, 220, 255], 0.4, Z_TOWER),
                    "bullet" | "projectile" => ([255, 230, 50, 255], 0.1, Z_BULLET),
                    _ => ([200, 200, 200, 255], 0.3, Z_ENEMY),
                };
                batch.write_quad(slot, &batched_sprite::QuadParams {
                    center: Vector2::new(-pos.x, pos.y),
                    size: Vector2::new(size_render, size_render),
                    color: color_arr,
                    rotation: 0.0,
                    z: z_render,
                });
            }
        }
```

(Keep the existing `scene.graph[entity.node].local_transform_mut().set_position(...)` line — it's a no-op since the legacy node has zero scale, but removing it is Phase 3 cleanup.)

**Step 2: Add flush() call after interp loop**

After the entire entity interp loop ends (after the `}` that closes the `for entity in self.network_entities.values_mut()` block, around line 1571), add:

```rust
        let interp_ns = t_interp.elapsed().as_nanos();  // <-- this line already exists; INSERT BELOW it

        // Phase 1 flush batched body mesh once per frame.
        if let Some(batch) = self.body_batch.as_mut() {
            batch.flush(scene);
        }
```

**Step 3: Run build + smoke test**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`
Expected: clean compile.

Run: `D:/omoba/run.bat` for 30 seconds. Move hero around, watch creeps move.

Expected: entities visibly move on screen. Body sprite color/size matches previous build. **If there's flicker / lag / corruption, set `USE_BATCHED_BODY=false` and rebuild — that's the rollback.**

**Step 4: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): entity interp writes body_batch + flush per frame"
```

---

### Task 1.11: entity_remove frees slot

**Files:**
- Modify: `D:/omoba/omfx/game/src/lib.rs` — `entity_remove` (~line 3866-3872)

**Step 1: Add free + hide_slot before legacy removal**

Find the entity removal logic (around line 3866). The current code does something like:

```rust
        if let Some(entity) = self.network_entities.remove(&id) {
            scene.graph.remove_node(entity.node);
            // ... and hp_bar nodes, etc.
        }
```

Before the `scene.graph.remove_node(entity.node);` line, insert:

```rust
            if let (Some(batch), Some(slot)) = (self.body_batch.as_mut(), entity.body_slot) {
                batch.free(slot);
                batch.hide_slot(slot);
            }
```

**Step 2: Run build + smoke test**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml`

Run: `D:/omoba/run.bat` for 60 seconds. Let creeps die / despawn.

Expected: dead creeps disappear from the scene cleanly (no leftover ghosts at origin). The hidden slot should not be visible anywhere.

**Step 3: Commit**

```bash
cd D:/omoba/omfx && git add game/src/lib.rs && git commit -m "feat(omfx): entity_remove releases body_batch slot"
```

---

### Task 1.12: Phase 1 stress benchmark

**Files:** None modified — verification only.

**Step 1: Build release**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`

**Step 2: Run stress 60s**

Run: `D:/omoba/run_stress.bat`. Wait until `omfx_app.log` shows ~30+ `omfx_frame` entries (~60 seconds of runtime), then exit.

**Step 3: Inspect log**

Run: `grep "omfx_render" D:/omoba/omfx_app.log | tail -10`

Expected output line shape (key field is `draw_calls`):

```
omfx_render window=60 avg(ms) pure=14.X capped=14.X fps=N draw_calls=42XX triangles=24K
```

Acceptance:
- `draw_calls`: should be `~4000-4400` (was 5247 in Phase 0).
- `pure_render_ms`: should be `~14ms` (was 17ms).
- `fps`: should be `≥58` (was 50).

If values don't move at all → batched mesh isn't actually rendering OR legacy nodes aren't hidden. Set `USE_BATCHED_BODY=false`, rebuild, re-run. Confirm Phase 0 numbers come back. Then debug.

**Step 4: Optional commit if you want to record the metrics**

```bash
cd D:/omoba/omfx && git commit --allow-empty -m "chore(perf): record Phase 1 stress benchmark (draw_calls 5247 → ~42XX)"
```

---

### Task 1.13: Phase 1 rollback flag verification

**Files:** None modified — verification only.

**Step 1: Toggle off**

Edit `D:/omoba/omfx/game/src/lib.rs`, change `const USE_BATCHED_BODY: bool = true;` to `false`.

**Step 2: Build + stress**

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`
Run: `D:/omoba/run_stress.bat` for 60s.

**Step 3: Verify Phase 0 numbers return**

Run: `grep "omfx_render" D:/omoba/omfx_app.log | tail -5`

Expected: `draw_calls=52XX`, `pure_render_ms=17`, `fps=50`. Same as Phase 0 baseline.

**Step 4: Toggle back on**

Edit lib.rs, change `false` back to `true`. Rebuild.

Run: `cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release`

(No commit — file is back to its committed state.)

---

## Phase 2 — HP Bar Batching

**Phase goal**: 2000 HP-bar Rectangles → 1-2 batched meshes. Stress: `draw_calls ~4200 → ~2200`, `pure_render_ms ~14 → ~9`, `fps 60 → ~100`.

**Approach**: Add `ui_batch: Option<BatchedSpriteMesh>` field with capacity 8192. Allocate 2 slots per entity in `spawn_entity` health branch. Replace HP bar `RectangleBuilder` calls with batch writes. Add `ui_batch.flush(scene)` after the per-frame interp loop.

**Tasks (to be expanded after Phase 1 ships):**
- 2.1: Add ui_batch field + lazy init
- 2.2: NetworkEntity gains `hp_bg_slot`, `hp_fg_slot` (Option<u32> each)
- 2.3: spawn_entity allocates HP slots when health.is_some()
- 2.4: Per-frame interp writes both bars (different size for fg based on hp%)
- 2.5: entity_remove frees slots
- 2.6: ui_batch.flush() after loop
- 2.7: Hide legacy hp_bar_bg / hp_bar_fg with scale=0
- 2.8: Stress benchmark + commit

---

## Phase 3 — Facing Arrow + Cleanup

**Phase goal**: facing arrow → batched, draw calls < 250, optionally merge body_batch + ui_batch into a single mesh = 1 draw call total.

**Tasks (to be expanded after Phase 2 ships):**
- 3.1: facing_slot in ui_batch (3rd slot per entity, rotation handled)
- 3.2: spawn_entity allocates facing slot
- 3.3: Per-frame interp writes rotated quad to facing_slot
- 3.4: Remove legacy `facing_arrow: Option<Handle<Node>>` field
- 3.5: Remove legacy `node` field, `hp_bar_bg`, `hp_bar_fg` (all RectangleBuilder calls deleted)
- 3.6: Optional — collapse body_batch + ui_batch into single mesh with capacity 16384
- 3.7: Final stress benchmark — expect `fps ≥ 200`

---

## Verification

### Acceptance criteria (final)

After Phase 3, stress (1000 entity) `omfx_render` log line should show:

```
omfx_render window=60 avg(ms) pure=3-5 capped=3-5 fps=200+ draw_calls<300 triangles=24K
```

Plus correctness:
- 1000 creeps moving smoothly with HP bars + facing arrows
- Spawn / die behavior correct (no ghost sprites, slot reuse working)
- `USE_BATCHED_BODY=false` rollback path still works
- `cargo build --release` clean
- `cargo test -p omfx batched_sprite::tests` 6 / 6 passing
- `run.bat` (regular non-stress map) plays normally for 30+ seconds

### Stress benchmark commands

```cmd
:: Build
cargo build --manifest-path D:/omoba/omfx/Cargo.toml --release

:: Run stress (terminate via window close after 60+ seconds)
D:/omoba/run_stress.bat
```

```bash
# Inspect log
grep "omfx_render\|omfx_frame" D:/omoba/omfx_app.log | tail -20
```

### Submodule pointer bump (after each Phase ships)

Phase commits stay inside `D:/omoba/omfx`. After Phase 1 / Phase 2 / Phase 3 is verified, bump the parent:

```bash
cd D:/omoba && git add omfx && git commit -m "chore: bump omfx for batched-mesh sprite Phase N"
```

---

## Critical Files

- `D:/omoba/omfx/game/src/batched_sprite.rs` (**new file** — module entry, ~250 LOC)
- `D:/omoba/omfx/game/src/lib.rs` — modifications:
  - `:36` — `mod batched_sprite;` declaration
  - `:73` area — `USE_BATCHED_BODY` const
  - `:626` area — `Game.body_batch: Option<BatchedSpriteMesh>` field
  - `:1361` area — lazy init in `update()` after `clear_lines()`
  - `:1500-1571` — entity interp loop (write_quad calls)
  - `:1647` area — `body_batch.flush(scene)` after interp
  - `:3525-3543` — body Rectangle in `spawn_entity` (alloc + hide)
  - `:3653-3684` — `NetworkEntity` struct (add `body_slot`)
  - `:3866-3872` — `entity_remove` (free + hide_slot)

### Read-only references (Fyrox internals)

- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/buffer.rs` — VertexBuffer / TriangleBuffer / VertexAttributeDescriptor
- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/surface.rs:228, 262` — SurfaceData::new
- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-impl-1.0.1/src/scene/mesh/mod.rs:285-316, 427-441` — MeshBuilder + Mesh::surfaces_mut
- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-material-1.0.1/src/lib.rs:721` — `Material::standard_2d()`
- `C:/Users/damod/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/fyrox-material-1.0.1/src/shader/standard/standard2d.shader:71-86` — vertex layout: position(loc=0) + tex_coord(loc=1) + color(loc=2)
