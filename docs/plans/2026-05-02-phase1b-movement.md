# Phase 1b — omb internal movement migration

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development) to implement task-by-task.

**Goal:** 把 omb 內部 movement layer（Pos / Vel / MoveTarget / Facing / TurnSpeed / Scale / Mass / CollisionRadius components + creep_tick / hero_move_tick / projectile_tick systems）從 f32 切到 Fixed32 / Vec2 / Angle，移除 WorldAdapter 9 個 movement-related `// TODO Phase 1[bcd]` boundary conversions。

**Architecture:** ECS Component newtypes 直接 wrap `omoba_sim::{Vec2, Fixed32, Angle}`。Tick systems 內部運算全 Fixed32。Searcher / CollisionIndex spatial index 維持 f32 internal（cache / render hint，rebuilt per tick），在 insert / query 邊界做 lossy conversion — 不影響 sim 確定性因 entity id 是權威 referrer，distance/range 由 sim 自己用 Fixed32 計算 final answer。

**Tech Stack:** Rust 1.91、specs 0.20、omoba-sim Phase 0+1a 鎖死的 6 個 cross-OS pin hashes（Phase 1b 將加 `vec2_normalize_pin_hash` 第 7 個）。

---

## Context

Phase 1a 已 ship 到 master：ABI 邊界 + base_content + WorldAdapter 全切 Fixed32 / Vec2 / Angle，omb internal ECS 仍 f32 由 WorldAdapter 邊界 lossy conversion 過渡。Phase 1a final review 點出 30+ `// TODO Phase 1[bcd]` markers 待 Phase 1b-d 清理。

Phase 1b 處理 movement layer — 是 Phase 1[bcd] 中數量最多的 TODO 群（~9 in WorldAdapter，加上 systems 內部所有 movement math）。完工後 omb 約 50% 的 Phase 1[bcd] 債務消除。

### 受影響檔案（exhaustive）

**Components**:
- `omb/src/comp/phys.rs` — `Pos / Vel / MoveTarget / Scale / Mass / CollisionRadius / PreviousPhysCache + Sticky / Immovable / ForceUpdate`
- `omb/src/comp/facing.rs` — `Facing / TurnSpeed`

**Tick systems**:
- `omb/src/tick/creep_tick.rs` (~40 f32)
- `omb/src/tick/hero_move_tick.rs` (~35 f32, has `advance_with_collision` helper used by WorldAdapter)
- `omb/src/tick/projectile_tick.rs` (~30 f32)

**Spatial structures (邊界處理 only, 不切 internal)**:
- `omb/src/comp/outcome.rs` — `Searcher`
- `omb/src/comp/collision_index.rs` — `CollisionIndex` BVH wrapper
- `omb/src/comp/blocked_region.rs` — map data, **不動**（init-only render hint）

**Adapter**:
- `omb/src/scripting/world_adapter.rs` — 移除 5 個 helper（vek_to_abi / abi_to_vek / rad_to_angle / angle_to_rad）+ 6 個 method site 的 conversion call

**Resources**:
- `omb/src/comp/resources.rs` — `DeltaTime(f32)` → `DeltaTime(Fixed32)`（整 chain 影響大，但 Phase 1a `dispatch.rs` 已用 Fixed32 dt，所以 cascade 已啟動）

**omoba-sim 補強**:
- `Vec2::normalized() -> Vec2` (sqrt + division)
- `Vec2::distance_squared(self, other) -> Fixed32`
- Pin hash for normalize sequence

### 不在 Phase 1b scope

- Hp / Damage / CProperty / Tower / TAttack（Phase 1c 戰鬥）
- BuffStore / UnitStats / Ability runtime（Phase 1d）
- specs single-threaded dispatch（Phase 1e）
- HashMap audit（Phase 1e）
- Searcher / CollisionIndex 內部 f32 → Fixed32 重做（記在 Phase 1e 或拆獨立 phase；目前 boundary lossy 已可接受）

---

## Tasks

### Task 1b.1: omoba-sim Vec2 加 normalize + distance_squared + pin hash

**Files:**
- Modify: `omoba-sim/src/vec2.rs` — 加 normalized / distance_squared / 對應 unit tests
- Modify: `omoba-sim/tests/determinism.rs` — 加 `vec2_normalize_pin_hash`

**Implementation:**
```rust
impl Vec2 {
    /// Returns this vector divided by its length. Returns ZERO if length is ZERO
    /// (deterministic, no NaN, matches Fixed32::sqrt(non-positive) → ZERO contract).
    pub fn normalized(self) -> Vec2 {
        let len = self.length();
        if len == Fixed32::ZERO { return Vec2::ZERO; }
        Vec2 { x: self.x / len, y: self.y / len }
    }

    /// Squared distance to `other`. Cheaper than `distance()` for comparisons.
    pub fn distance_squared(self, other: Vec2) -> Fixed32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Distance to `other`. Uses Fixed32::sqrt under the hood.
    pub fn distance(self, other: Vec2) -> Fixed32 {
        self.distance_squared(other).sqrt()
    }
}
```

**Tests** (in `vec2::tests`):
```rust
#[test]
fn normalized_unit_x() {
    let v = Vec2::new(Fixed32::from_i32(3), Fixed32::ZERO);
    let n = v.normalized();
    assert_eq!(n, Vec2::new(Fixed32::ONE, Fixed32::ZERO));
}

#[test]
fn normalized_3_4_5() {
    let v = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
    let n = v.normalized();
    // length 5, so n = (0.6, 0.8) = (615/1024, 819/1024) within ±2 LSB
    assert!((n.x.raw() - 614).abs() <= 2);
    assert!((n.y.raw() - 819).abs() <= 2);
}

#[test]
fn normalized_zero_returns_zero() {
    assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
}

#[test]
fn distance_squared_basic() {
    let a = Vec2::new(Fixed32::from_i32(1), Fixed32::from_i32(2));
    let b = Vec2::new(Fixed32::from_i32(4), Fixed32::from_i32(6));
    assert_eq!(a.distance_squared(b), Fixed32::from_i32(25));
}

#[test]
fn distance_3_4() {
    let a = Vec2::new(Fixed32::ZERO, Fixed32::ZERO);
    let b = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
    assert_eq!(a.distance(b), Fixed32::from_i32(5));
}
```

**Pin hash** in `tests/determinism.rs`:
```rust
#[test]
fn vec2_normalize_pin_hash() {
    use omoba_sim::vec2::Vec2;
    use omoba_sim::fixed::Fixed32;
    let mut h = fxhash::FxHasher64::default();
    for x in (-50..=50).step_by(7) {
        for y in (-50..=50).step_by(7) {
            let v = Vec2::new(Fixed32::from_i32(x), Fixed32::from_i32(y));
            let n = v.normalized();
            n.x.raw().hash(&mut h);
            n.y.raw().hash(&mut h);
        }
    }
    let actual = h.finish();
    println!("VEC2_NORMALIZE PIN HASH = {}", actual);
    assert_eq!(actual, 0u64);  // CAPTURE + LOCK
}
```

**Verify**: `cargo test -p omoba-sim` — expect 46 + 5 unit + 1 pin = 52 PASS。

**Commit**: 單一 commit feat(sim): Vec2 normalized / distance_squared / pin hash。

---

### Task 1b.2: omb components 切型（暫不改 systems）

**Files**:
- Modify: `omb/src/comp/phys.rs`
- Modify: `omb/src/comp/facing.rs`
- Modify: `omb/src/comp/resources.rs` — DeltaTime f32 → Fixed32

**型別變更**:

| Before | After |
|---|---|
| `Pos(pub Vec2<f32>)` | `Pos(pub omoba_sim::Vec2)` |
| `Vel(pub Vec2<f32>)` | `Vel(pub omoba_sim::Vec2)` |
| `MoveTarget(pub Vec2<f32>)` | `MoveTarget(pub omoba_sim::Vec2)` |
| `Scale(pub f32)` | `Scale(pub omoba_sim::Fixed32)` |
| `Mass(pub f32)` | `Mass(pub omoba_sim::Fixed32)` |
| `CollisionRadius(pub f32)` | `CollisionRadius(pub omoba_sim::Fixed32)` |
| `Facing(pub f32)` (radians) | `Facing(pub omoba_sim::Angle)` |
| `TurnSpeed(pub f32)` (rad/s) | `TurnSpeed(pub omoba_sim::Fixed32)` (Fixed32 ticks/s 或 ticks/tick — pick 後者語意更準) |
| `DeltaTime(pub f32)` | `DeltaTime(pub omoba_sim::Fixed32)` (seconds_per_tick 仍 fixed; integer-tick 留 Phase 1d/e) |

`PreviousPhysCache`、`Sticky`、`Immovable`、`ForceUpdate`、`FacingBroadcast(Option<f32>)`：**不動**（cache / render hint）。`FacingBroadcast` 後面 Phase 1d 視野系統可能會處理。

**SystemData 受影響的所有 site**（grep `Pos.0` `Vel.0` 等內部 field 用法）：使用 `Component newtype.0` 自動是新 type — 等下 Task 1b.3-1b.4 同 systems 一起切。本 task 只 fix component 定義 + 受影響的最直接 ctor / Default impl / serialize derives（特別注意：specs 的 Component 還是要保持 derive）。

**Verify**: `cargo check --manifest-path /d/omoba/omb/Cargo.toml 2>&1 | tail -10` — 預期 FAIL with cascade errors in tick systems / world_adapter。捕捉錯誤數。

**Commit**:
```
refactor(omb): movement Component newtypes Pos/Vel/MoveTarget/Facing/Scale/
Mass/CollisionRadius/TurnSpeed/DeltaTime → Fixed32 / Vec2 / Angle

Component 定義切型；systems / world_adapter / dispatch 暫不動 — 預期
cargo check 失敗 (cascade)，Tasks 1b.3 / 1b.4 / 1b.5 修正。
```

---

### Task 1b.3: hero_move_tick + advance_with_collision Fixed32

**Files**:
- Modify: `omb/src/tick/hero_move_tick.rs`
- Modify: `omb/src/scripting/world_adapter.rs` — `advance_with_collision` 邊界 cleanup（drop step / radius f32 conversion）

**Migration**:
- `pos.0 - target.0`：直接 Vec2 算術（已 Add/Sub）
- `.magnitude()` → `.length()` 或 `.length_squared() + sqrt`
- `diff / distance`：先確認 distance != ZERO，用 `Vec2::normalized()` 或手動 `diff * (Fixed32::ONE / distance)`（cheaper）
- `direction * step`：`Vec2 * Fixed32` 已實作
- `dt`: `DeltaTime` 已切 Fixed32（Task 1b.2）
- `radius` from `CollisionRadius`：直接 Fixed32
- ParJoin **保留**（Phase 1e 才砍）

`advance_with_collision(pos: vek::Vec2<f32>, target: vek::Vec2<f32>, step: f32, radius: f32) -> (vek::Vec2<f32>, bool)` 改成 `advance_with_collision(pos: Vec2, target: Vec2, step: Fixed32, radius: Fixed32) -> (Vec2, bool)`。

**WorldAdapter cleanup**：line 364 移除 step.to_f32_for_render() / radius f32 conversion，直接傳 Fixed32。

Searcher 邊界（hero_move_tick 用 `searcher.creep.search_xy(...)`）：`search_xy` 簽名仍是 `f32` (because Searcher internal 仍 Vec2<f32>)。在 caller 邊界做 `pos.0.x.to_f32_for_render()` lossy conversion，標 `// TODO Phase 1e: Searcher 切 Fixed32 後刪`。

**Verify**: `cargo check --manifest-path /d/omoba/omb/Cargo.toml 2>&1 | tail -20` — hero_move_tick 應 clean，creep_tick / projectile_tick 仍 fail（Task 1b.4）。

**Commit**: refactor(omb): hero_move_tick + advance_with_collision Fixed32 / Vec2

---

### Task 1b.4: creep_tick + projectile_tick Fixed32 + Facing rotate_toward Angle

**Files**:
- Modify: `omb/src/tick/creep_tick.rs`
- Modify: `omb/src/tick/projectile_tick.rs`
- Modify: `omb/src/comp/facing.rs` — `rotate_toward` helper 切 Angle 演算法

**creep_tick migrations**:
- `path.check_points` movement: 用 Vec2 算術 + `length_squared` 比 `< threshold_squared` 避免 sqrt
- `rotate_toward(facing.0, desired, turn_rate * dt)`: 改為 Angle-based。新 helper：
```rust
// in omoba-sim/src/trig.rs (or facing.rs)
pub fn angle_rotate_toward(current: Angle, target: Angle, max_step_ticks: i32) -> Angle {
    let diff = (target.ticks() - current.ticks()).rem_euclid(TAU_TICKS);
    let signed_diff = if diff > TAU_TICKS / 2 { diff - TAU_TICKS } else { diff };
    let clamped = signed_diff.clamp(-max_step_ticks, max_step_ticks);
    Angle::from_ticks(current.ticks() + clamped)
}
```
加在 `omoba-sim/src/trig.rs` + 對應 unit test + 新 pin hash `angle_rotate_toward_pin_hash`。

**projectile_tick migrations**:
- Homing: `proj.tpos - pos.0` Vec2 sub
- Swept-segment: `a + (delta / dist) * step` → `a + delta.normalized() * step` 或 inline 等價
- `time_left -= dt`: Fixed32 -= Fixed32
- `msd * dt`: Fixed32 * Fixed32
- Fixed32 `<=` comparison for time_left, hit_radius

**facing.rs**: `rotate_toward(current_rad: f32, target_rad: f32, max_delta: f32) -> f32` 重寫為 `rotate_toward(current: Angle, target: Angle, max_step: Fixed32) -> Angle`，把 max_step Fixed32 (rad/tick equivalent) 映射到 ticks 數：`max_step_ticks = (max_step.raw() as i64 * TAU_TICKS as i64 / (TWO_PI_FIXED.raw() as i64)) as i32`。或更直接：TurnSpeed 在 Task 1b.2 已 Fixed32（rad/tick），這邊直接用 omoba_sim helper。

**Verify**: `cargo check --manifest-path /d/omoba/omb/Cargo.toml` — 應 clean except WorldAdapter 還剩幾 site（Task 1b.5）。

**Commit**: refactor(omb): creep_tick + projectile_tick Fixed32 + Angle rotate_toward

---

### Task 1b.5: WorldAdapter movement TODO cleanup + verify

**Files**:
- Modify: `omb/src/scripting/world_adapter.rs` — 移除 5 個 helper + 6 個 method site conversion
- Modify: `omb/src/state/initialization.rs` — Vf32::new(...) sites 切（hero / creep stats Pos init）
- Modify: 任何剩餘 omb 內 Pos/Vel/Facing 用 site

**WorldAdapter cleanup**:
- 移除 `vek_to_abi` / `abi_to_vek` / `rad_to_angle` / `angle_to_rad` 4 個 helper。`f32_to_fixed` 保留（Phase 1c/d 還會用，標 `// TODO Phase 1c/d: drop after Hp/CProperty migrate`）。
- `get_pos` / `set_pos`: 直接讀 `pos.0` Vec2、寫 `pos.0 = p`
- `get_facing` / `set_facing`: 直接讀寫 `facing.0` Angle
- `advance_with_collision`: 直接傳 Fixed32 step / radius
- `query_enemies_in_range` / `query_nearest_enemy`: Searcher 邊界仍 lossy（保留 `// TODO Phase 1e: Searcher Fixed32`）
- `spawn_projectile_ex`: `spec.from` 直接 Vec2 給 omb internal Projectile（Projectile 內部 fields 仍 f32 — Phase 1d 才切，但 from 位置可以 cascade，因為 Pos 已 Vec2）— 實際更精確：保留 `// TODO Phase 1d: Projectile.tpos / msd / time_left Fixed32`

**搜剩餘 cascade**: `cargo build --manifest-path /d/omoba/omb/Cargo.toml 2>&1 | grep error` — fix until clean。常見 site：
- `state/initialization.rs` Vf32::new spots
- `comp/creep.rs` 內部運算
- `tick/tower_tick.rs` 等其他 tick 讀 Pos
- `comp/creep_move_broadcast.rs` 用 Pos 做 wire encoding
- `aoi.rs` / `vision/*` 視野系統用 Pos（保留 lossy 邊界因 Phase 1d/e 才動）

**Verify**:
1. `cargo build` 整 chain clean
2. `cargo test -p omoba-sim` 52 PASS（含新 vec2_normalize + angle_rotate_toward pin hashes）
3. `cargo test --manifest-path /d/omoba/omb/Cargo.toml -p omobab` PASS
4. `cargo run -p omobab --bin gen-docs --features gen-docs --release` 渲染 catalog
5. 數一下剩餘 `// TODO Phase 1[bcd]` markers — 預期從 30+ 降到 ~15 (剩下的是 Hp / Damage / Buff / Searcher / Projectile 等待 1c/d/e)

**Final code review**: 派 superpowers:code-reviewer review 整 Phase 1b。

**Commit**: refactor(omb): WorldAdapter movement TODO cleanup + Phase 1b close

---

## Verification end-to-end

完成 5 task 後：
- omoba-sim 52 tests pass，8 個 pin hashes locked（5 phase 0 + atan2 + composite + snapshot + vec2_normalize + angle_rotate_toward — 等等，verify 數量）
- omb cargo build clean
- gen-docs 渲染 9 units + 8 abilities
- 剩餘 Phase 1[bcd] TODO markers grep 結果 < 20（Phase 1c/d/e 處理）
- omb master fast-forward to phase1b-movement tip

## 開放問題

1. **Searcher / CollisionIndex Fixed32 化**：Phase 1e 拆獨立 task；目前 boundary lossy 可接受
2. **Mass 是否該 Fixed32**：Mass 在現有 code 幾乎不用，切 Fixed32 是 trivial 一致性；按 plan 切
3. **Time(f64) 不切**：render hint，保留
4. **DeltaTime(Fixed32) 跨 Phase 連動**：Phase 1c/d/e systems 用 DeltaTime 仍會繼續切，本 phase 只是 type 變更，bottom 用 site 是 cascading
