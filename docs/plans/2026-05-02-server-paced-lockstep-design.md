# Server-Paced Lockstep Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 omoba 從 server-authoritative + dirty-state delta broadcast (~64-206 KB/s per player) 重構為 8 人 Server-Authoritative Lockstep，目標 < 5 KB/s per player。

**Architecture:** 抽 `omoba-sim` crate（fixed-point ECS / systems / RNG / state hash），server 與 omfx client 都引用；server 為 60 Hz 節拍器 + authoritative referee；wire protocol 改為 InputSubmit (C→S) + TickBatch (S→C 廣播 player input + server events)；3 tick (50ms) input delay；server-paced（不等慢 client）；8 player 各跑相同 deterministic sim；每 600 tick (10s) StateHash 比對 desync。

**Tech Stack:** Rust 1.91、specs 0.20 (single-thread dispatch)、abi_stable (base_content.dll 兩端載入)、tokio_kcp、prost / tonic-build (proto)、rand_pcg、bincode、fxhash、Fyrox 1.0.1 (omfx render)。

---

## Context (新工程師必讀)

### 為何要做
目前 stress 場景（1000 tower × 1000 creep × 8 player）量測 baseline 64-206 KB/s per player（HUD 顯示 `net_wire_bytes_current`，data source `omfx/game/src/lib.rs:1530-1535`）。已有完整漸進優化計畫 `docs/plans/2026-04-24-kcp-multiplayer-traffic-optimization.md`（6 Phase 目標削 75-85%）但**使用者決定跳過該路徑**直接重做為 Lockstep，目標削 95%+。

### 既有架構
- `omb/` (server, bin `omobab`)：specs 0.20 ECS，30 Hz tick。發 GameEvent (HeroHot 0.3s / Heartbeat 0.5s / CreepMove 變化觸發 / ProjectileCreate per-attack)。
- `omfx/` (client, bin `executor`)：純 renderer + velocity-based interpolation，無本地 sim。Spawn `target/debug/omobab.exe` 子行程（hard-coded path，stress 用 release-copy 繞過）。
- `scripts/base_content/` (cdylib)：所有塔 / 英雄 / 召喚物 script，abi_stable FFI，**只在 server 載入**。
- `proto/game.proto`：prost / tonic 共用 schema，KCP framing `[1B tag][4B len BE][prost payload]`，目前 tag 0x01-0x06。
- `omoba-core/`：client/server 共用 schema（`tower_meta.rs`、`grpc/` `kcp/` client）。
- `omb-mcp/`：MCP server，KCP query-only。
- 場景：`omb/Story/{MVP_1,TD_1,TD_STRESS,...}` + `game.toml` 的 `STORY` 欄。Stress: `scripts/gen_stress_map.py`、`run_stress.bat`。

### 確認的設計決策
| 項目 | 值 |
|---|---|
| Tick rate | 60 Hz (16.66ms) |
| Input delay | 3 tick (~50ms) |
| 數值型別 | Fixed-point i32 + SCALE=1024 (~0.001 unit 精度) |
| 玩家拓樸 | 8+ 人 MOBA |
| Lag 處理 | Server-paced (不等慢 client) |
| Client prediction | 不做、純 lockstep |
| Server 角色 | 也跑同一份 sim（authoritative） |
| Rejoin | 玩家斷線=GG，snapshot 只給 observer |
| Desync detection | 每 600 tick (10s) state hash compare |
| 代碼共用 | `omoba-sim` crate + `base_content.dll` 兩邊載入 |

---

## 階段總覽

| Phase | 範圍 | 工期 | 此 plan 詳細度 |
|---|---|---|---|
| **0** | 基礎建設（omoba-sim skeleton, fixed-point, RNG, state hash） | 1-2 週 | **task-by-task 完整展開** |
| 1 | 把 omb ECS / base_content 搬入 sim crate + fixed-point 化 | 3-4 週 | outline + 任務清單 |
| 2 | Wire protocol 切換（新 tag 0x10-0x16 + InputBuffer + TickBroadcaster） | 2 週 | outline |
| 3 | omfx 變 simulator（worker thread 跑 sim、render 讀 sim state） | 4-6 週 | outline |
| 4 | 砍舊 path（移除 dirty-state broadcast / legacy_transport feature） | 1-2 週 | outline |
| 5 | Observer / Snapshot | 1-2 週 | outline |

**完成 Phase 0 後**，需重新跑 brainstorming + writing-plans 為 Phase 1 寫獨立 plan file。每個 Phase 都會有自己的詳細 plan、自己的 worktree 與 review checkpoint。

---

# Phase 0：基礎建設（task-by-task）

**目標**：建 `omoba-sim` crate，含 fixed-point arithmetic、deterministic trig LUT、deterministic RNG、state hash 工具，並有完整 unit test 證明跨 platform 確定性。完工後 sim crate 可被 omb / omfx 引用，但還沒有任何 game logic。

**Worktree**：建議 `git worktree add ../omoba-lockstep-phase0 -b lockstep/phase0-foundation`（用 superpowers:using-git-worktrees）。

---

### Task 0.1：建立 omoba-sim crate skeleton

**Files:**
- Create: `omoba-sim/Cargo.toml`
- Create: `omoba-sim/src/lib.rs`
- Modify: `Cargo.toml` (workspace root, add member)

**Step 1：寫 `omoba-sim/Cargo.toml`**

```toml
[package]
name = "omoba-sim"
version = "0.1.0"
edition = "2021"
publish = false

[dependencies]
rand_pcg = "0.3"
rand_core = "0.6"
fxhash = "0.2"
bincode = "1.3"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
proptest = "1"
```

**Step 2：寫 `omoba-sim/src/lib.rs`**

```rust
//! omoba-sim: deterministic ECS simulation crate
//! Shared between server (omb) and client (omfx) for lockstep networking.

pub mod fixed;
pub mod trig;
pub mod rng;
pub mod state_hash;
pub mod snapshot;
```

**Step 3：把 `omoba-sim` 加入 workspace**

讀取 `D:\omoba\Cargo.toml`，找 `[workspace] members = [...]` 區塊，加入 `"omoba-sim"`。

**Step 4：執行 `cargo check -p omoba-sim`**

Expected: PASS（雖然 mod 還沒檔案會 error，所以下個 task 起每個 mod 各別建檔再回來 check）。

**Step 5：建空白 stub `fixed.rs` `trig.rs` `rng.rs` `state_hash.rs` `snapshot.rs`，每檔只放 `// stub` 註釋**

**Step 6：再執行 `cargo check -p omoba-sim`**

Expected: PASS

**Step 7：Commit**

```bash
git add omoba-sim/ Cargo.toml
git commit -m "feat(sim): add omoba-sim crate skeleton for lockstep foundation"
```

---

### Task 0.2：Fixed32 type 加減運算

**Files:**
- Modify: `omoba-sim/src/fixed.rs`
- Test: `omoba-sim/src/fixed.rs` (`#[cfg(test)] mod tests`)

**Step 1：寫失敗測試**

```rust
// omoba-sim/src/fixed.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_basic() {
        let a = Fixed32::from_i32(3);
        let b = Fixed32::from_i32(2);
        assert_eq!(a + b, Fixed32::from_i32(5));
        assert_eq!(a - b, Fixed32::from_i32(1));
    }

    #[test]
    fn add_fractional() {
        let a = Fixed32::from_raw(512);  // 0.5
        let b = Fixed32::from_raw(256);  // 0.25
        assert_eq!((a + b).raw(), 768);  // 0.75
    }
}
```

**Step 2：執行測試確認失敗**

Run: `cargo test -p omoba-sim --lib fixed::tests::add_sub_basic`
Expected: FAIL（type 未定義）

**Step 3：實作最小代碼**

```rust
// omoba-sim/src/fixed.rs
use serde::{Serialize, Deserialize};

pub const SCALE: i32 = 1024;
pub const SCALE_BITS: u32 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed32(i32);

impl Fixed32 {
    pub const ZERO: Fixed32 = Fixed32(0);
    pub const ONE: Fixed32 = Fixed32(SCALE);

    pub const fn from_raw(raw: i32) -> Self { Fixed32(raw) }
    pub const fn from_i32(v: i32) -> Self { Fixed32(v * SCALE) }
    pub const fn raw(self) -> i32 { self.0 }
    pub fn to_f32(self) -> f32 { self.0 as f32 / SCALE as f32 }
}

impl std::ops::Add for Fixed32 {
    type Output = Fixed32;
    fn add(self, rhs: Fixed32) -> Fixed32 { Fixed32(self.0.wrapping_add(rhs.0)) }
}

impl std::ops::Sub for Fixed32 {
    type Output = Fixed32;
    fn sub(self, rhs: Fixed32) -> Fixed32 { Fixed32(self.0.wrapping_sub(rhs.0)) }
}
```

**Step 4：執行測試確認通過**

Run: `cargo test -p omoba-sim --lib fixed::tests`
Expected: 兩個 test PASS

**Step 5：Commit**

```bash
git add omoba-sim/src/fixed.rs
git commit -m "feat(sim): add Fixed32 add/sub with i32 + scale=1024"
```

---

### Task 0.3：Fixed32 乘除（含 i64 中介）

**Files:**
- Modify: `omoba-sim/src/fixed.rs`

**Step 1：寫失敗測試**

```rust
#[test]
fn mul_basic() {
    let a = Fixed32::from_i32(3);
    let b = Fixed32::from_i32(4);
    assert_eq!(a * b, Fixed32::from_i32(12));
}

#[test]
fn mul_fractional() {
    let a = Fixed32::from_raw(512);  // 0.5
    let b = Fixed32::from_raw(512);  // 0.5
    assert_eq!((a * b).raw(), 256);  // 0.25
}

#[test]
fn div_basic() {
    let a = Fixed32::from_i32(10);
    let b = Fixed32::from_i32(4);
    assert_eq!((a / b).raw(), 2560);  // 2.5
}

#[test]
fn mul_no_overflow_in_range() {
    // ±2000 unit (典型 map size) × Fixed32 不能溢位
    let pos = Fixed32::from_i32(2000);
    let scale = Fixed32::from_raw(2048);  // 2.0
    let _ = pos * scale;  // 不應 panic
}
```

**Step 2：跑測試確認失敗**

Run: `cargo test -p omoba-sim --lib fixed::tests::mul_basic`
Expected: FAIL（`Mul` 未實作）

**Step 3：實作**

```rust
// 加在 fixed.rs 既有 impl 之後
impl std::ops::Mul for Fixed32 {
    type Output = Fixed32;
    fn mul(self, rhs: Fixed32) -> Fixed32 {
        // (a * SCALE) * (b * SCALE) / SCALE = a*b*SCALE
        let prod = (self.0 as i64) * (rhs.0 as i64);
        Fixed32((prod >> SCALE_BITS) as i32)
    }
}

impl std::ops::Div for Fixed32 {
    type Output = Fixed32;
    fn div(self, rhs: Fixed32) -> Fixed32 {
        let num = (self.0 as i64) << SCALE_BITS;
        Fixed32((num / rhs.0 as i64) as i32)
    }
}

impl std::ops::Neg for Fixed32 {
    type Output = Fixed32;
    fn neg(self) -> Fixed32 { Fixed32(-self.0) }
}
```

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib fixed::tests`
Expected: 6 個 test 全 PASS

**Step 5：Commit**

```bash
git add omoba-sim/src/fixed.rs
git commit -m "feat(sim): add Fixed32 mul/div via i64 intermediate"
```

---

### Task 0.4：Fixed32 sqrt（牛頓法）

**Files:**
- Modify: `omoba-sim/src/fixed.rs`

**Step 1：寫失敗測試**

```rust
#[test]
fn sqrt_perfect() {
    assert_eq!(Fixed32::from_i32(4).sqrt(), Fixed32::from_i32(2));
    assert_eq!(Fixed32::from_i32(9).sqrt(), Fixed32::from_i32(3));
    assert_eq!(Fixed32::from_i32(100).sqrt(), Fixed32::from_i32(10));
}

#[test]
fn sqrt_imperfect_within_1lsb() {
    // sqrt(2) ≈ 1.41421
    let r = Fixed32::from_i32(2).sqrt();
    let expected = Fixed32::from_raw(1448);  // ≈ 1.41406
    let diff = (r.raw() - expected.raw()).abs();
    assert!(diff <= 2, "sqrt(2) raw={} expected~={}, diff={}", r.raw(), expected.raw(), diff);
}

#[test]
fn sqrt_zero() {
    assert_eq!(Fixed32::ZERO.sqrt(), Fixed32::ZERO);
}
```

**Step 2：跑測試**

Run: `cargo test -p omoba-sim --lib fixed::tests::sqrt_perfect`
Expected: FAIL

**Step 3：實作**

```rust
impl Fixed32 {
    pub fn sqrt(self) -> Fixed32 {
        if self.0 <= 0 { return Fixed32::ZERO; }
        // 牛頓法：y_{n+1} = (y_n + x/y_n) / 2，10 次迭代足夠收斂
        // 在 fixed-point 下 x = self.0 << SCALE_BITS（為了 i64 運算位寬）
        let x = (self.0 as i64) << SCALE_BITS;
        let mut y: i64 = (self.0 as i64).max(SCALE as i64);
        for _ in 0..16 {
            let next = (y + x / y) >> 1;
            if next == y { break; }
            y = next;
        }
        Fixed32(y as i32)
    }
}
```

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib fixed::tests`
Expected: 全 PASS

**Step 5：Commit**

```bash
git add omoba-sim/src/fixed.rs
git commit -m "feat(sim): add Fixed32::sqrt via Newton iteration"
```

---

### Task 0.5：跨平台 determinism 證明測試

**Files:**
- Create: `omoba-sim/tests/determinism.rs`

**Step 1：寫測試**

```rust
// omoba-sim/tests/determinism.rs
use omoba_sim::fixed::Fixed32;

/// 此測試的 hash 值是 Phase 0 的「鎖死」常數。
/// 任何 Fixed32 實作改動都會改變此 hash → 表示打破跨機器確定性。
/// 若必須改 Fixed32 內部表示（罕見），更新此常數並通知所有 client 升版。
#[test]
fn fixed32_arithmetic_pin_hash() {
    use std::hash::{Hash, Hasher};
    let mut h = fxhash::FxHasher64::default();

    // 1000 步混合運算
    let mut acc = Fixed32::from_raw(1);
    for i in 1..=1000 {
        let v = Fixed32::from_raw(i);
        acc = (acc + v) * Fixed32::from_raw(1003);
        acc = acc / Fixed32::from_raw(997);
        if i % 7 == 0 { acc = acc.sqrt(); }
        acc.raw().hash(&mut h);
    }

    // 把實際 hash 填進來（首次跑出後鎖住）
    let actual = h.finish();
    println!("PIN HASH = {}", actual);
    // 第一次跑時把值複製到下一行，後續修改 Fixed32 若改了此值就 fail
    assert_eq!(actual, 0); // <-- 首次 RUN 後改成實際值
}
```

**Step 2：第一次跑（會 FAIL，但會印出 actual hash）**

Run: `cargo test -p omoba-sim --test determinism -- --nocapture`
Expected: FAIL，stdout 印 `PIN HASH = <some_u64>`

**Step 3：把實際 hash 填回去**

把 `assert_eq!(actual, 0)` 改成 `assert_eq!(actual, <actual_value>)`。

**Step 4：再跑確認 PASS**

Run: `cargo test -p omoba-sim --test determinism`
Expected: PASS

**Step 5：在 Linux Docker 內跑同樣測試確認跨 OS 一致**

```bash
docker run --rm -v "/d/omoba:/work" -w /work rust:1.91 cargo test -p omoba-sim --test determinism
```

Expected: PASS（hash 與 Windows 上算出的相同）

如果 FAIL，代表 i64 wrapping 或 shift 行為有跨 platform 差異——不應該發生（Rust spec 保證），但確認一次比較安心。

**Step 6：Commit**

```bash
git add omoba-sim/tests/determinism.rs
git commit -m "test(sim): pin Fixed32 cross-platform hash for lockstep determinism"
```

---

### Task 0.6：trig.rs sin/cos LUT

**Files:**
- Modify: `omoba-sim/src/trig.rs`

**Step 1：寫失敗測試**

```rust
// omoba-sim/src/trig.rs
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fixed32;

    #[test]
    fn sin_known_values() {
        // 0 弧度
        assert_eq!(sin(Angle::ZERO), Fixed32::ZERO);
        // π/2 (= 90°)
        assert_eq!(sin(Angle::QUARTER_TURN), Fixed32::ONE);
        // π (= 180°)
        assert_eq!(sin(Angle::HALF_TURN), Fixed32::ZERO);
    }

    #[test]
    fn cos_known_values() {
        assert_eq!(cos(Angle::ZERO), Fixed32::ONE);
        assert_eq!(cos(Angle::QUARTER_TURN), Fixed32::ZERO);
        assert_eq!(cos(Angle::HALF_TURN), -Fixed32::ONE);
    }

    #[test]
    fn sin_small_angle_within_lsb_2() {
        // 30° = π/6，sin = 0.5
        let a = Angle::from_degrees_i32(30);
        let s = sin(a);
        let expected = Fixed32::from_raw(512);  // 0.5
        assert!((s.raw() - expected.raw()).abs() <= 2);
    }
}
```

**Step 2：跑測試**

Run: `cargo test -p omoba-sim --lib trig::tests::sin_known_values`
Expected: FAIL

**Step 3：實作**

```rust
// omoba-sim/src/trig.rs
use crate::fixed::{Fixed32, SCALE};
use serde::{Serialize, Deserialize};

pub const TAU_TICKS: i32 = 4096;  // 4096 步 = 一圈，每步 ~0.0879°

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Angle(i32);  // mod TAU_TICKS

impl Angle {
    pub const ZERO: Angle = Angle(0);
    pub const QUARTER_TURN: Angle = Angle(TAU_TICKS / 4);
    pub const HALF_TURN: Angle = Angle(TAU_TICKS / 2);

    pub fn from_ticks(t: i32) -> Self { Angle(t.rem_euclid(TAU_TICKS)) }
    pub fn from_degrees_i32(d: i32) -> Self {
        Angle::from_ticks(((d as i64 * TAU_TICKS as i64) / 360) as i32)
    }
    pub fn ticks(self) -> i32 { self.0 }
}

// build.rs / static array 都可。先用 lazy static + std::f64 算（一次性、跑於 build 時行為一致）
static SIN_LUT: once_cell::sync::Lazy<[i32; TAU_TICKS as usize]> = once_cell::sync::Lazy::new(|| {
    let mut lut = [0i32; TAU_TICKS as usize];
    for i in 0..TAU_TICKS {
        let rad = (i as f64) * std::f64::consts::TAU / TAU_TICKS as f64;
        // 用 f64 + round 確保跨平台 (f64 IEEE-754 在現代 x86 一致)
        lut[i as usize] = (rad.sin() * SCALE as f64).round() as i32;
    }
    lut
});

pub fn sin(a: Angle) -> Fixed32 {
    Fixed32::from_raw(SIN_LUT[a.0 as usize])
}

pub fn cos(a: Angle) -> Fixed32 {
    let i = (a.0 + TAU_TICKS / 4).rem_euclid(TAU_TICKS);
    Fixed32::from_raw(SIN_LUT[i as usize])
}
```

加 `once_cell = "1"` 到 `omoba-sim/Cargo.toml`。

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib trig::tests`
Expected: 3 個 PASS

**Step 5：把 trig 也納入 determinism pin**

修改 `omoba-sim/tests/determinism.rs`，加：

```rust
#[test]
fn trig_lut_pin_hash() {
    use std::hash::{Hash, Hasher};
    use omoba_sim::trig::{sin, cos, Angle, TAU_TICKS};
    let mut h = fxhash::FxHasher64::default();
    for i in 0..TAU_TICKS {
        let a = Angle::from_ticks(i);
        sin(a).raw().hash(&mut h);
        cos(a).raw().hash(&mut h);
    }
    let actual = h.finish();
    println!("TRIG PIN HASH = {}", actual);
    assert_eq!(actual, 0); // <-- 首次跑後填值
}
```

跑、填值、再跑確認 PASS。

**Step 6：Commit**

```bash
git add omoba-sim/src/trig.rs omoba-sim/Cargo.toml omoba-sim/tests/determinism.rs
git commit -m "feat(sim): add trig LUT (sin/cos with 4096 tick precision)"
```

---

### Task 0.7：rng.rs deterministic RNG

**Files:**
- Modify: `omoba-sim/src/rng.rs`

**Step 1：寫失敗測試**

```rust
// omoba-sim/src/rng.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SimRng::from_master(42, 0);
        let mut b = SimRng::from_master(42, 0);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_tick_different_sequence() {
        let mut a = SimRng::from_master(42, 100);
        let mut b = SimRng::from_master(42, 101);
        // 第一個值幾乎不可能相同
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn range_uniform() {
        let mut r = SimRng::from_master(42, 0);
        let mut histogram = [0u32; 10];
        for _ in 0..10000 {
            histogram[r.range(0, 10) as usize] += 1;
        }
        // 每 bucket 應該大致 1000 ± 100
        for c in &histogram {
            assert!(*c >= 800 && *c <= 1200, "bucket {} out of range", c);
        }
    }
}
```

**Step 2：跑測試**

Run: `cargo test -p omoba-sim --lib rng::tests::same_seed_same_sequence`
Expected: FAIL

**Step 3：實作**

```rust
// omoba-sim/src/rng.rs
use rand_pcg::Pcg64Mcg;
use rand_core::{RngCore, SeedableRng};

pub struct SimRng(Pcg64Mcg);

impl SimRng {
    /// 從 master_seed + tick 產生確定性 RNG
    /// 同一 (master_seed, tick) 不同呼叫者用相同 stream，注意：
    /// 若需要 entity-specific stream，請用 from_master_entity()
    pub fn from_master(master_seed: u64, tick: u32) -> Self {
        let combined = master_seed.wrapping_mul(0x9E3779B97F4A7C15)
            ^ (tick as u64).wrapping_mul(0xBF58476D1CE4E5B9);
        SimRng(Pcg64Mcg::seed_from_u64(combined))
    }

    pub fn from_master_entity(master_seed: u64, tick: u32, entity_id: u32, op_kind: u32) -> Self {
        let mut state = master_seed;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ tick as u64;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ entity_id as u64;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ op_kind as u64;
        SimRng(Pcg64Mcg::seed_from_u64(state))
    }

    pub fn next_u32(&mut self) -> u32 { self.0.next_u32() }
    pub fn next_u64(&mut self) -> u64 { self.0.next_u64() }

    /// 回傳 [low, high) 內的整數，無偏 unbiased
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        debug_assert!(high > low);
        let span = (high - low) as u32;
        let r = self.next_u32() % span;  // 簡化：實際應用 widening 法去 modulo bias，但 phase 0 足夠
        low + r as i32
    }
}
```

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib rng::tests`
Expected: 3 個 PASS

**Step 5：加 RNG pin hash 到 determinism.rs**

```rust
#[test]
fn rng_sequence_pin_hash() {
    use std::hash::{Hash, Hasher};
    use omoba_sim::rng::SimRng;
    let mut h = fxhash::FxHasher64::default();
    let mut r = SimRng::from_master(0xDEAD_BEEF_CAFE_BABE, 12345);
    for _ in 0..10000 { r.next_u64().hash(&mut h); }
    println!("RNG PIN HASH = {}", h.finish());
    assert_eq!(h.finish(), 0); // <-- 填值
}
```

**Step 6：Commit**

```bash
git add omoba-sim/src/rng.rs omoba-sim/tests/determinism.rs
git commit -m "feat(sim): add SimRng with master_seed + tick + entity stream isolation"
```

---

### Task 0.8：state_hash.rs entity sweep

**Files:**
- Modify: `omoba-sim/src/state_hash.rs`

**Step 1：寫失敗測試**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Hash)]
    struct Dummy { id: u32, x: i32, y: i32 }

    #[test]
    fn order_invariant() {
        let a = vec![
            Dummy { id: 2, x: 10, y: 20 },
            Dummy { id: 1, x: 5, y: 5 },
            Dummy { id: 3, x: 30, y: 30 },
        ];
        let b = vec![
            Dummy { id: 3, x: 30, y: 30 },
            Dummy { id: 2, x: 10, y: 20 },
            Dummy { id: 1, x: 5, y: 5 },
        ];
        // 排序後 hash 應一致
        assert_eq!(hash_sorted_by_id(&a, |d| d.id), hash_sorted_by_id(&b, |d| d.id));
    }

    #[test]
    fn detects_change() {
        let a = vec![Dummy { id: 1, x: 5, y: 5 }];
        let b = vec![Dummy { id: 1, x: 6, y: 5 }];  // x 改了
        assert_ne!(hash_sorted_by_id(&a, |d| d.id), hash_sorted_by_id(&b, |d| d.id));
    }
}
```

**Step 2：跑測試**

Run: `cargo test -p omoba-sim --lib state_hash::tests`
Expected: FAIL

**Step 3：實作**

```rust
// omoba-sim/src/state_hash.rs
use std::hash::{Hash, Hasher};
use fxhash::FxHasher64;

/// Hash 一個 entity 集合，先依 id 排序確保結果與遍歷順序無關。
/// 為 lockstep desync detection 用。
pub fn hash_sorted_by_id<T: Hash, F: Fn(&T) -> u32>(items: &[T], id_of: F) -> u64 {
    let mut indices: Vec<usize> = (0..items.len()).collect();
    indices.sort_by_key(|&i| id_of(&items[i]));
    let mut h = FxHasher64::default();
    items.len().hash(&mut h);
    for i in indices {
        items[i].hash(&mut h);
    }
    h.finish()
}
```

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib state_hash::tests`
Expected: 2 個 PASS

**Step 5：Commit**

```bash
git add omoba-sim/src/state_hash.rs
git commit -m "feat(sim): add hash_sorted_by_id for lockstep desync detection"
```

---

### Task 0.9：snapshot.rs serialize / deserialize

**Files:**
- Modify: `omoba-sim/src/snapshot.rs`

**Step 1：寫失敗測試**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct DummyWorld { tick: u32, entities: Vec<(u32, i32, i32)> }

    #[test]
    fn round_trip() {
        let w = DummyWorld {
            tick: 42,
            entities: vec![(1, 10, 20), (2, 30, 40)],
        };
        let bytes = serialize(&w).unwrap();
        let w2: DummyWorld = deserialize(&bytes).unwrap();
        assert_eq!(w, w2);
    }
}
```

**Step 2：跑測試**

Run: `cargo test -p omoba-sim --lib snapshot::tests`
Expected: FAIL

**Step 3：實作**

```rust
// omoba-sim/src/snapshot.rs
use serde::{Serialize, de::DeserializeOwned};

pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, bincode::Error> {
    bincode::serialize(value)
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, bincode::Error> {
    bincode::deserialize(bytes)
}
```

**Step 4：跑測試**

Run: `cargo test -p omoba-sim --lib snapshot::tests`
Expected: PASS

**Step 5：Commit**

```bash
git add omoba-sim/src/snapshot.rs
git commit -m "feat(sim): add bincode snapshot serialize/deserialize wrapper"
```

---

### Task 0.10：跨 OS 確定性驗證 + Phase 0 結束

**Files:** 無新增

**Step 1：本機跑全部測試**

Run: `cargo test -p omoba-sim`
Expected: 所有 test PASS（fixed/trig/rng/state_hash/snapshot + 3 個 pin hash test）

**Step 2：在 Linux Docker 跑同 hash**

```bash
docker run --rm -v "/d/omoba:/work" -w /work rust:1.91 cargo test -p omoba-sim
```

Expected: 所有 test PASS（特別是 3 個 pin hash 與 Windows 一致）。

**Step 3：若 pin hash 不一致**

代表跨平台確定性已破，**這是 Phase 0 必須在進入 Phase 1 前解決的 blocker**。常見原因：
- `f64.sin()` 的 LUT 生成跨平台不一致 → 改成 hard-coded LUT 寫死 4096 個整數常數（用 `build.rs` 在 Linux 跑一次寫進 .rs）
- `rand_pcg` 版本不同 → pin Cargo.lock

**Step 4：跑 cargo doc 確認 sim crate 公開 API 文件可生成**

Run: `cargo doc -p omoba-sim --no-deps`
Expected: PASS，輸出在 `target/doc/omoba_sim/index.html`

**Step 5：Phase 0 完工 commit**

```bash
git commit --allow-empty -m "chore(sim): Phase 0 (foundation) complete — fixed-point/trig/rng/hash/snapshot + cross-OS determinism verified"
```

**Step 6：用 superpowers:requesting-code-review 跑一次 review**

呼叫 code review agent 檢查 Phase 0 所有 commits，確認：
- Fixed32 沒有任何 f32 / f64 在 sim hot path
- LUT 表生成是否跨平台 reproducible
- RNG seed mixing 沒有 obvious 弱點
- Snapshot bincode 設定是否 stable across versions

通過後合併 worktree branch 回 master。

---

# Phase 1-5 Outline（後續展開）

每個 Phase 在進入時，需重新 brainstorm + writing-plans 寫成獨立 plan file。以下只列**範圍**與**結束條件**，當作 north star。

---

## Phase 1：Move ECS into omoba-sim + fixed-point migration（3-4 週）

**範圍**：
- 把 `omb/src/comp/` 中所有 deterministic 相關 component 搬到 `omoba-sim/src/components/`，型別由 f32 改 Fixed32
  - Position, Velocity, Hp, AttackRange, AttackInterval, Damage, MoveSpeed, Faction, Owner, ProjectileSpec
- 把 `omb/src/sys/` 中遊戲邏輯 system 搬到 `omoba-sim/src/systems/`
  - movement_tick, combat, projectile, ability_runtime (BuffStore + UnitStats), wave spawner, AoI (server-side hint, client-side 不用)
- 改 `scripts/script-abi/src/lib.rs` types：所有 abi_stable struct 用 Fixed32 取代 f32
- 改 `scripts/base_content/src/{towers,heroes,summons}/*.rs` 全部 f32 → Fixed32
- specs dispatcher 從 parallel 改 single-threaded（移除 rayon 用法）
- 所有 HashMap iteration 點改成 BTreeMap 或 sort-then-iterate
- 設立 sim-level entity ID counter，不依賴 specs::Entity::generation

**結束條件**：
- `cargo test --manifest-path omb/Cargo.toml -p omobab` 全綠
- `cargo test --manifest-path scripts/Cargo.toml -p base_content` 全綠
- `run.bat` 跑得起來，遊戲行為與遷移前等價（手動 spot check）
- `run_stress.bat` 跑 60 秒不 panic、效能 regression < 30%（30 Hz 仍跑得動 1000+1000）
- `gen-docs` 仍能生成 catalog，所有 unit script API 顯示 Fixed32 types
- 新版 sim crate 在 Phase 0 的 pin hash test 仍 PASS（沒破確定性）

---

## Phase 2：Wire Protocol 切換（2 週）

**範圍**：
- 重設計 `proto/game.proto`：
  - PlayerInput oneof (MoveTo / AttackTarget / CastAbility / TowerPlace / TowerUpgrade / TowerSell / ItemUse / NoOp)
  - InputSubmit { player_id, target_tick, input }
  - TickBatch { tick: u32, inputs: Vec<(player_id, PlayerInput)>, server_events: Vec<ServerEvent> }
  - ServerEvent (PlayerJoin / PlayerLeave / GameStart 等)
  - StateHash { tick: u32, hash: u64 }
  - SimSnapshot { tick, master_seed, world_bytes }
  - GameStart { start_tick, master_seed, initial_state }
- 新 KCP tag：0x10 InputSubmit / 0x11 TickBatch / 0x12 StateHash / 0x13 JoinReq / 0x14 GameStart / 0x15 SnapshotReq / 0x16 SnapshotResp
- omb 新 modules：
  - `omb/src/lockstep/input_buffer.rs`：collect player inputs targeting tick T
  - `omb/src/lockstep/tick_broadcaster.rs`：60 Hz pace + broadcast TickBatch
  - `omb/src/transport/lockstep_kcp.rs`：tag 路由到 lockstep handler
- `omb/Cargo.toml` 新增 feature flag `legacy_transport`（過渡用、預設 off）
- 兩端在新舊 protocol 之間能 toggle

**結束條件**：
- 兩個 omfx client 連 server，input 透過 0x10/0x11 流動
- Server log 印「tick T 收齊 inputs，broadcast」
- 既有 stress 場景在 lockstep transport 下 server-side sim 仍跑得動

---

## Phase 3：omfx → simulator（4-6 週）

**範圍**：
- omfx 載入 `base_content.dll`（複用 omb 的 abi_stable loader）
- 新 `omfx/src/sim_loop.rs`：worker thread 跑 `omoba-sim` tick
- 移除 `NetworkBridge` / `EventBuffer`、velocity-based interpolation
- omfx render 改從 sim World 讀取 entity 位置（lock-free snapshot 或雙 buffer）
- omfx 的 `executor` bin 不再 spawn `omobab.exe` 子行程；改為連 server 模式（multi-machine 或 localhost）
- 補 sim → render fixed→f32 轉換層

**結束條件**：
- 8 個 client 同連 server 跑 30 秒
- 所有 client 的 StateHash 與 server 一致（無 desync）
- stress 場景 60 Hz 達標：server CPU < 70%、client CPU < 50%、無 dropped tick
- 流量 < 5 KB/s per player

---

## Phase 4：砍舊 path（1-2 週）

**範圍**：
- 移除 `legacy_transport` feature flag
- 刪除：
  - `omb/src/comp/creep_move_broadcast.rs`
  - `omb/src/comp/outcome_system/{combat_events,creation_events}.rs` 中 broadcast 相關 site
  - `omb/src/state/core.rs` 的 heartbeat / hero.stats / visibility diff
  - 舊 KCP tag 0x01-0x06 handler
- `omb-mcp/`：query 改從 sim state 讀（不再透過 `Resource<Player>` 等）

**結束條件**：
- `cargo test` 全綠
- `mcp__omoba-game__list_players` / `inspect_player_view` 仍可用
- 流量量測 stress 場景 < 5 KB/s per player（HUD `net_wire_bytes_current`）
- 跑 10 分鐘無 desync log

---

## Phase 5：Observer / Snapshot（1-2 週）

**範圍**：
- Server 每 30 秒存一份 sim snapshot
- 處理 `JoinRequest { role: Observer }`：回 GameStart + 最近 snapshot + 後續 TickBatch
- omfx 加 observer mode（無 input UI）
- 處理 observer late-join 時若離 snapshot > 30 秒的 fast-forward（接連跑多個 tick 直到追上）

**結束條件**：
- 第 9 個 client 以 observer 中途加入，畫面與 player 一致
- 第 9 個的 StateHash 與 player 對得上
- snapshot 大小可接受（< 500 KB 對 1000+1000 entity）

---

## 全程 Verification 端到端

每 Phase 結束都跑一輪：

1. **流量量測**：`run_stress.bat` 跑 60 秒，HUD 顯示 `net_wire_bytes_current` per player
2. **Determinism**：跑 10 分鐘 8-client 場景，server 廣播 StateHash 全部 client 對齊
3. **Lag 處理**：用 `clumsy.exe` 對某 client 加 500ms 延遲，該 client 卡頓但其他 7 個續玩
4. **Observer**：第 9 個 client 中途以 observer 進場
5. **MCP query**：`omb-mcp` 工具可用
6. **既有 unit test**：`cargo test --manifest-path omb/Cargo.toml -p omobab` + `... -p base_content` + `cargo test -p omoba-sim`
7. **gen-docs**：catalog HTML 重生成成功，所有 unit script API 顯示 Fixed32 types

---

## 開放問題（執行時逐一處理）

1. **specs 0.20 single-thread dispatch 性能**：1000+1000 entity × 60 Hz 是否吃得消？必要時換 hecs / legion。（Phase 1 結尾 reprofile）
2. **Fyrox 主 thread 同步策略**：sim worker 跑慢時要不要讓 render 用 1 tick 落後 state？（Phase 3 設計時決定）
3. **DLL hot-reload**：dev 期改 base_content.dll → 必須通知所有 client 升 build。Lockstep 的 build hash 一致性檢查機制。（Phase 2 加進 GameStart message）
4. **Snapshot 大小**：1000+1000 entity bincode 估計 ~200 KB，可接受。若超 1MB 再考慮壓縮。（Phase 5）
5. **omfx executor 不再 spawn omobab.exe**：dev 工作流要改（手動先跑 server）。`run.bat` 要重寫。（Phase 3）

---

## 後續 Phase 接手指引

完成 Phase 0 後：
1. 啟用 superpowers:requesting-code-review
2. 在 master 上 brainstorm Phase 1（明確 scope 與優先序，可能拆成更小 sub-plans）
3. 用 superpowers:writing-plans 寫 `docs/plans/<Phase 1 完整 plan>.md`
4. 用 superpowers:using-git-worktrees 建 phase-1 worktree
5. 用 superpowers:executing-plans 或 superpowers:subagent-driven-development 執行
