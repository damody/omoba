# Phase 1a — ABI Boundary Migration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development for in-session execution) to implement this plan task-by-task.

**Goal:** 把整個 host↔DLL ABI 邊界從 `f32` 切成 `Fixed32` / `Vec2`，base_content 13 個 script 同步切型；omb internal ECS 仍維持 f32（暫時在 `WorldAdapter` 邊界做 lossy conversion，Phase 1b-1d 才陸續切除）。

**Architecture:** `omoba-sim` 為 abi-stable 來源（feature-gated `StableAbi` derive），`omoba-template-ids/build.rs` 生成 `Fixed32` 常數，`scripts/script-abi` 全部 trait / struct 改 `Fixed32`，`scripts/base_content` 13 個 script 檔內部運算切 `Fixed32`，`omb/src/scripting/world_adapter.rs` 在 host 內部 f32 ↔ ABI Fixed32 邊界做轉換（轉換點明確標 `// TODO Phase 1b-1d`）。

**Tech Stack:** Rust 1.91、abi_stable 0.11、specs 0.20、`omoba-sim` (Fixed32 / Vec2 / SimRng)、`omoba-template-ids` (build-time codegen)、Phase 0 鎖死的 5 個 cross-OS pin hashes。

---

## Context（新工程師必讀）

### 為何要做
Server-Paced Lockstep 要求 host 與 client 跑相同 deterministic sim。base_content.dll 兩邊都要載入（Phase 3）。所有跨邊界數值必須是跨平台 bit-identical。`omoba-sim::Fixed32` 已在 Phase 0 鎖死跨 OS hash；現在要把 ABI 邊界從 f32 切過來，base_content 內部運算同步切。

### 5 個受影響的 crate

| Crate | 路徑 | 角色 |
|---|---|---|
| `omoba-sim` | `D:\omoba\omoba-sim\` | Phase 0 ship 的確定性基礎；Phase 1a 加 abi-stable feature |
| `omoba-template-ids` | `D:\omoba\omoba-template-ids\` | build.rs 從 `omb/Story/templates.json` 生成 `TowerStats` const |
| `omb-script-abi` | `D:\omoba\scripts\script-abi\` | abi_stable 唯一共用 crate，定義 `Vec2f`、`DamageInfo`、`TowerMetadata`、`ProjectileSpec`、`UnitScript`、`AbilityScript`、`GameWorld` |
| `base_content` | `D:\omoba\scripts\base_content\` | cdylib，4 tower + 9 hero + 1 summon |
| `omobab` (omb) | `D:\omoba\omb\` | host，`omb/src/scripting/world_adapter.rs` 實作 GameWorld |

### 既有 f32 surface（Phase 0 explore 已盤點）
- `script-abi/src/types.rs`：`Vec2f` (2 f32)、`DamageInfo.amount` (1)、`TowerMetadata` (10 f32)、`ProjectileSpec` (8 f32) = **21 f32 in types**
- `script-abi/src/script.rs`：UnitScript trait 7 個 f32 hook param
- `script-abi/src/world.rs`：GameWorld trait 43 個 f32 method（returns + params）
- `omoba-template-ids/src/lib.rs:19-30`：`TowerStats { atk, asd_interval, range, ... }` 全 f32
- `omoba-template-ids/build.rs`：生成 const `TowerStats` 字面值
- `base_content/src/towers/{dart, bomb, tack, ice}.rs`：26 f32 literal + 大量算術
- `base_content/src/heroes/B0{1,2}_*/*.rs`：~9 個 hero ability handler，多數 JSON-based（少 f32 literal）
- `base_content/src/summons/saika_gunner.rs`：4 f32 literal + 移動邏輯
- `omb/src/scripting/world_adapter.rs`：實作 46 個 `GameWorld` method

### 邊界轉換策略
omb internal ECS（Pos / Vel / CProperty 的 f32 fields）**不在 Phase 1a 動**。`WorldAdapter` 在每個 GameWorld method 邊界做 f32 ↔ Fixed32 lossy conversion：
```rust
// Phase 1a: omb internal still f32
fn get_hp(&self, e: EntityHandle) -> Fixed32 {
    let hp_f32 = /* read from CProperty.hp */;
    // TODO Phase 1c: remove this conversion when CProperty switches to Fixed32
    Fixed32::from_raw((hp_f32 * SCALE as f32) as i32)
}
```
這個 lossy conversion 在 Phase 1a 是過渡，Phase 1b-1d 切完後 conversion 全部刪除（grep `Phase 1[bcd]` TODO 找）。

### Phase 0 pin hashes 不可破
`omoba-sim/tests/determinism.rs` 5 個 pin test 必須在 Phase 1a 後仍 PASS。Phase 1a 不該修改 omoba-sim 的 fixed/trig/rng/state_hash/snapshot/vec2 模組的行為，只加 derive。如果加 abi_stable feature 影響 hash（不應該，但確認），更新 pin。

---

## Tasks

每個 task 都包含：寫 / 改檔 → `cargo build` 整鏈通過 → spot test → commit。任何中斷狀態都不能 commit。

---

### Task 1a.1：omoba-sim 加 abi-stable feature

**Files:**
- Modify: `D:\omoba\omoba-sim\Cargo.toml`
- Modify: `D:\omoba\omoba-sim\src\fixed.rs`
- Modify: `D:\omoba\omoba-sim\src\vec2.rs`
- Modify: `D:\omoba\omoba-sim\src\trig.rs`（Angle）

**Step 1: Cargo.toml 加 optional dep + feature**

```toml
[dependencies]
rand_pcg = "0.3"
rand_core = "0.6"
fxhash = "0.2"
bincode = "1.3"
serde = { version = "1", features = ["derive"] }
once_cell = "1"
abi_stable = { version = "0.11", optional = true }

[features]
default = []
abi-stable = ["dep:abi_stable"]
```

**Step 2: Fixed32 加 conditional StableAbi derive**

`fixed.rs:9-10`：
```rust
#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(transparent))]
#[cfg_attr(not(feature = "abi-stable"), repr(Rust))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed32(i32);
```

對 Vec2 (`vec2.rs`)：
```rust
#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(C))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Fixed32,
    pub y: Fixed32,
}
```

對 Angle (`trig.rs`)：
```rust
#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(transparent))]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Angle(i32);
```

**Step 3: Verify Phase 0 tests 仍綠（feature off）**

```
cd /d/omoba && cargo test -p omoba-sim --manifest-path /d/omoba/omoba-sim/Cargo.toml
```
Expected: 39 PASS（含 5 pin hashes）。

**Step 4: Verify abi-stable feature 開啟也綠**

```
cargo test -p omoba-sim --manifest-path /d/omoba/omoba-sim/Cargo.toml --features abi-stable
```
Expected: 39 PASS（5 pin hashes 不變）。

**Step 5: Commit**

```
feat(sim): optional abi-stable feature for cross-DLL Fixed32/Vec2/Angle

Adds abi_stable 0.11 as optional dep; Fixed32 (transparent over i32),
Vec2 (repr(C), 2 Fixed32 fields), Angle (transparent over i32) get
StableAbi derive when feature enabled. Default off keeps the crate
small for non-FFI consumers (tests, future replay tools).

5 cross-OS pin hashes still pass with feature on/off — derive injects
no behavior change.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

### Task 1a.2：omoba-template-ids 改用 Fixed32

**Files:**
- Modify: `D:\omoba\omoba-template-ids\Cargo.toml`
- Modify: `D:\omoba\omoba-template-ids\src\lib.rs`
- Modify: `D:\omoba\omoba-template-ids\build.rs`

**Step 1: Cargo.toml 加 omoba-sim dep（with abi-stable feature）**

```toml
[dependencies]
omoba-sim = { path = "../omoba-sim", features = ["abi-stable"] }
# 既有的其他 deps 不動
```

**Step 2: lib.rs `TowerStats` struct 切 Fixed32**

讀 `D:\omoba\omoba-template-ids\src\lib.rs` 找 `TowerStats` 定義。把所有 `f32` 欄位改 `Fixed32`：
```rust
use omoba_sim::Fixed32;

pub struct TowerStats {
    pub atk: Fixed32,
    pub asd_interval: Fixed32,
    pub range: Fixed32,
    pub bullet_speed: Fixed32,
    pub splash_radius: Fixed32,
    pub hit_radius: Fixed32,
    pub slow_factor: Fixed32,
    pub slow_duration: Fixed32,
    pub cost: i32,                // 不變（已是 i32）
    pub footprint: Fixed32,
    pub hp: Fixed32,
    pub turn_speed_deg: Fixed32,
}
```

對 `HeroStats`（lib.rs:128 附近）同樣切。注意：`base_damage: i32`、`base_hp: i32`、`base_mana: i32`、`strength/agility/intelligence: i32` 維持 i32（design doc 已定）。

**Step 3: build.rs 生成 Fixed32 const**

`build.rs:38-49` `TowerEntry` deserialize 仍從 JSON 讀 `f32`（JSON literal 就是 floating），但生成 const 時轉成 `Fixed32::from_raw((v * 1024.0) as i32)`。例如：

```rust
// 在 emit_tower_consts() 之類地方
fn fixed32_literal(v: f32) -> String {
    let raw = (v * 1024.0).round() as i32;
    format!("Fixed32::from_raw({})", raw)
}

// 生成的 const 從原本：
//   pub const TOWER_DART_STATS: TowerStats = TowerStats { atk: 30.0, ... };
// 改為：
//   pub const TOWER_DART_STATS: TowerStats = TowerStats {
//       atk: Fixed32::from_raw(30720),  // 30.0
//       asd_interval: Fixed32::from_raw(819),  // 0.8
//       ...
//   };
```

**注意**：`Fixed32::from_raw` 是 `const fn`（Phase 0 已實作），所以可以放 const context。

**Step 4: Build chain verify**

```
cargo build --manifest-path /d/omoba/omb/Cargo.toml
cargo build --manifest-path /d/omoba/scripts/Cargo.toml
```
**Expected: 預期會 FAIL** — script-abi/types.rs 還是 f32，base_content 用 `STATS.atk` 是 Fixed32 但 `set_tower_atk(e, v: f32)` 期待 f32。**這是預期 — Task 1a.3/1a.4 才會修好**。

實際做法：先讓 `omoba-template-ids` 自己 `cargo build -p omoba-template-ids` 通過（沒下游依賴的話）：
```
cd /d/omoba && cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: PASS（self-contained）。

**Step 5: Commit**

```
refactor(template-ids): TowerStats / HeroStats from f32 to Fixed32

omoba-template-ids now generates Fixed32 const values via
Fixed32::from_raw((v * 1024.0).round() as i32) at build time. JSON
literal values unchanged (floating-point in templates.json is fine
as a serialization format).

Downstream (script-abi, base_content, omb) will be updated in
Task 1a.3 / 1a.4. omb won't build until those are done — this is a
deliberate cascading change.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

### Task 1a.3：script-abi types.rs 切 Fixed32 / Vec2 + 連動 base_content 用 site

**Files:**
- Modify: `D:\omoba\scripts\script-abi\Cargo.toml`
- Modify: `D:\omoba\scripts\script-abi\src\types.rs`
- Modify: `D:\omoba\scripts\base_content\Cargo.toml` (加 omoba-sim dep)
- Modify: `D:\omoba\scripts\base_content\src\towers\*.rs`（4 個檔）— 只改 type usage（不改 trait sig）
- Modify: `D:\omoba\omb\Cargo.toml`（加 omoba-sim dep）
- Modify: `D:\omoba\omb\src\scripting\world_adapter.rs`（types.rs 相關 site：邊界轉 f32 ↔ Fixed32 / Vec2）

**Step 1: Cargo.toml deps**

`scripts/script-abi/Cargo.toml`：
```toml
[dependencies]
abi_stable = "0.11"
omoba-template-ids = { path = "../../omoba-template-ids" }
omoba-sim = { path = "../../omoba-sim", features = ["abi-stable"] }
```

`scripts/base_content/Cargo.toml` 同樣加。

`omb/Cargo.toml` 加 `omoba-sim = { path = "../omoba-sim", features = ["abi-stable"] }`。

**Step 2: types.rs 切型**

`scripts/script-abi/src/types.rs:18-27`：刪掉 `Vec2f` struct 整段，全改用 `omoba_sim::Vec2`（也是 `#[repr(C)]` + `StableAbi`）。

```rust
//! Stable-ABI value types that cross the host/DLL boundary.

use abi_stable::{StableAbi, std_types::{ROption, RString}};
pub use omoba_sim::{Fixed32, Vec2};

#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct EntityHandle { /* unchanged */ }

#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq)]
pub enum DamageKind { Physical, Magical, Pure }

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct DamageInfo {
    pub attacker: ROption<EntityHandle>,
    pub amount: Fixed32,    // was f32
    pub kind: DamageKind,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub enum Target {
    Entity(EntityHandle),
    Point(Vec2),    // was Vec2f
    None,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub enum PathSpec {
    Homing { target: EntityHandle },
    Straight { end_pos: Vec2 },    // was Vec2f
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug, Default)]
pub struct TowerMetadata {
    pub atk: Fixed32,
    pub asd_interval: Fixed32,
    pub range: Fixed32,
    pub bullet_speed: Fixed32,
    pub splash_radius: Fixed32,
    pub hit_radius: Fixed32,
    pub slow_factor: Fixed32,
    pub slow_duration: Fixed32,
    pub cost: i32,
    pub footprint: Fixed32,
    pub hp: Fixed32,
    pub turn_speed_deg: Fixed32,
    pub label: RString,
}

#[repr(C)]
#[derive(StableAbi, Clone, Debug)]
pub struct ProjectileSpec {
    pub from: Vec2,
    pub owner: EntityHandle,
    pub path: PathSpec,
    pub speed: Fixed32,
    pub damage: Fixed32,
    pub hit_radius: Fixed32,
    pub splash_radius: Fixed32,
    pub slow_factor: Fixed32,
    pub slow_duration: Fixed32,
    pub stun_duration: Fixed32,
    pub kind_id: u16,
}
```

注意 `Vec2` 沒有 `Default` derive — 加 `#[derive(Default)]` 到 `omoba-sim/src/vec2.rs`（`Vec2::ZERO` 已存在，但 trait Default 是分開的）。

**Step 3: prelude 更新**

`scripts/script-abi/src/lib.rs`：找 `mod prelude` 或 `pub mod prelude` 區塊，加 `pub use omoba_sim::{Fixed32, Vec2};`。

**Step 4: base_content 13 scripts 連動切型**

每個 tower script 找 `Vec2f` 用 `Vec2` 取代；但這個 task 只改 type alias、不改 logic。Logic 切型（`STATS.atk` 從 f32 變 Fixed32 的算術）放 Task 1a.4。

實際上 `Vec2f` 在 base_content 用法多半是接收 `from: w.get_pos(e)` 之類；Task 1a.4 才會把 `get_pos` 切回傳 `Vec2`。

→ **本 task 範圍縮小**：types.rs 切 + script-abi 自己 cargo build 通過 + 不動 base_content scripts 的 logic（只改 type alias `Vec2f` → `Vec2` 在 use 區塊就好）。

**Step 5: WorldAdapter site for types**

`omb/src/scripting/world_adapter.rs` 中所有用 `DamageInfo`、`TowerMetadata`、`ProjectileSpec`、`Vec2f` 的地方：暫時 inline lossy conversion（待 Task 1a.4 補完整）：

```rust
// 暫時：把 host 內部 f32 包成 Fixed32 給 ABI 邊界
fn make_damage_info(host_amount: f32, ...) -> DamageInfo {
    DamageInfo {
        attacker: ...,
        amount: Fixed32::from_raw((host_amount * 1024.0) as i32),  // TODO Phase 1c
        kind: ...,
    }
}
```

**Step 6: Build verify**

```
cd /d/omoba && cargo build --manifest-path /d/omoba/scripts/script-abi/Cargo.toml
```
Expected: PASS。

`cargo build --manifest-path /d/omoba/scripts/Cargo.toml` 預期會 FAIL（base_content 的 trait sigs 沒切）— 這是預期狀態。

**Step 7: Commit**

```
refactor(script-abi): Vec2f / DamageInfo / TowerMetadata / ProjectileSpec to Fixed32

types.rs replaces Vec2f with omoba_sim::Vec2 (re-exported in prelude)
and switches DamageInfo.amount + 18 stat fields across TowerMetadata /
ProjectileSpec to Fixed32. abi_stable derive preserved on all types
via omoba-sim's abi-stable feature.

base_content trait impls + omb WorldAdapter still in flux — both
will be completed in Task 1a.4. scripts/Cargo.toml does not yet
build cleanly (intentional — cascading change).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

### Task 1a.4：script-abi traits + base_content + WorldAdapter 全切（最大 task）

**Files:**
- Modify: `D:\omoba\scripts\script-abi\src\script.rs`（UnitScript trait）
- Modify: `D:\omoba\scripts\script-abi\src\world.rs`（GameWorld trait）
- Modify: `D:\omoba\scripts\script-abi\src\ability.rs`（AbilityScript trait）
- Modify: `D:\omoba\scripts\base_content\src\towers\{dart,bomb,tack,ice}.rs`（4 個）
- Modify: `D:\omoba\scripts\base_content\src\heroes\B01_saika_magoichi\*.rs`（4 個）
- Modify: `D:\omoba\scripts\base_content\src\heroes\B02_date_masamune\*.rs`（4 個）
- Modify: `D:\omoba\scripts\base_content\src\summons\saika_gunner.rs`
- Modify: `D:\omoba\scripts\base_content\src\ability_builder.rs`
- Modify: `D:\omoba\omb\src\scripting\world_adapter.rs`（46 個 GameWorld method）
- Modify: `D:\omoba\omb\src\scripting\dispatch.rs`（`dt: f32` 參數變 `dt: Fixed32`，呼叫端傳 `Fixed32::from_raw((dt_f32 * 1024.0) as i32)`）

這個 task 是 Phase 1a 主體。沒法再拆 — script-abi trait 一改，base_content 13 個 impl 必須同步改才能 build；同時 omb 也要改才能呼叫 trait。

**Step 1: script.rs UnitScript trait**

7 個 hook 的 f32 param 切 Fixed32：
```rust
fn on_tick(&self, _e: EntityHandle, _dt: Fixed32, _w: &mut GameWorldDyn<'_>) {}
fn on_damage_taken(&self, _e: EntityHandle, _info: &mut DamageInfo, _w: &mut GameWorldDyn<'_>) {}
fn on_damage_dealt(&self, _attacker: EntityHandle, _victim: EntityHandle, _final_amount: Fixed32, _w: &mut GameWorldDyn<'_>) {}
fn on_health_gained(&self, _e: EntityHandle, _amount: Fixed32, _w: &mut GameWorldDyn<'_>) {}
fn on_mana_gained(&self, _e: EntityHandle, _amount: Fixed32, _w: &mut GameWorldDyn<'_>) {}
fn on_spent_mana(&self, _caster: EntityHandle, _cost: Fixed32, _ability_id: RStr<'_>, _w: &mut GameWorldDyn<'_>) {}
fn on_heal_received(&self, _target: EntityHandle, _amount: Fixed32, _source: ROption<EntityHandle>, _w: &mut GameWorldDyn<'_>) {}
```

**Step 2: world.rs GameWorld trait — 46 method**

所有 `f32` returns / params 切 Fixed32。特別注意：
- `query_enemies_in_range(center: Vec2, radius: Fixed32, of: ...)` 
- `advance_with_collision(_e, target: Vec2, step: Fixed32) -> Vec2`
- `get_hp(...) -> Fixed32`、`get_max_hp(...) -> Fixed32`、`get_facing(...) -> Angle`（之前 f32 弧度，現在改 Angle 更語意明確）
- `set_facing(_e, angle: Angle)`（之前 angle_rad: f32）
- `deal_damage(_target, amount: Fixed32, ...)`
- `heal(_target, amount: Fixed32)`
- `add_buff(_target, _buff_id, duration: Fixed32)`
- `get_tower_range/atk/asd_interval/asd_count() -> Fixed32`
- `set_tower_atk/range/asd_interval(_e, v: Fixed32)`
- `emit_explosion(pos: Vec2, radius: Fixed32, duration: Fixed32)`
- `spawn_projectile_ex(spec: ProjectileSpec) -> EntityHandle`（已切，因 ProjectileSpec 內 fields 都已 Fixed32）
- `sum_stat / product_stat -> Fixed32`
- `get_final_*() -> Fixed32`（move_speed, atk, armor, magic_resist, crit_multiplier, cooldown_mult, max_hp_bonus, hp_regen）
- `rand_f32() -> Fixed32`：**改名 `rand_unit() -> Fixed32`**（[0, 1) 範圍 fixed32，避免名字誤導；deterministic via SimRng）
- `get_buff_remaining(...) -> Fixed32`（剩餘秒數；Phase 1d 改 tick-based 但本 task 先 Fixed32）

**Step 3: ability.rs AbilityScript**

`on_tick(_caster, _target: Target, _elapsed: Fixed32, _world)` — `elapsed` 從 f32 切 Fixed32。`execute()` 簽名不動（用 JSON）。

**Step 4: base_content 13 個 script impl**

對每個 script：
1. UnitScript trait method 簽名跟 trait 一致
2. 內部運算改 Fixed32（`asd_count += dt`、`asd_count < asd_interval` 等）
3. tower const literal 從 `0.25` 改 `Fixed32::from_raw(256)`（0.25 = 256/1024）
4. `BONUS_PROC_CHANCE: Fixed32 = Fixed32::from_raw(256);`（const）
5. `dy.atan2(dx)` / `len.sqrt()`：改用 `omoba_sim::trig` 的 `atan2_fixed`（Phase 1a 加新 helper！）+ `Fixed32::sqrt()`
6. `sin / cos`：用 `omoba_sim::trig::{sin, cos, Angle::from_degrees_i32}`

需要 `omoba-sim` 補一個 `atan2(y: Fixed32, x: Fixed32) -> Angle` 函式（CORDIC 或 LUT）。**這要新加 task or 在 Task 1a.4 內 inline 實作？** Plan 決定：**inline 實作 atan2 in `omoba-sim/src/trig.rs`** 作為 Task 1a.4 的 prerequisite。實作方法：用 4096-entry atan LUT 或 CORDIC。簡單版用 LUT（2D 入 1D 索引透過 octant 對稱）。

**Sub-step 4a: omoba-sim 加 `atan2`**

`omoba-sim/src/trig.rs` 加：
```rust
/// 2-arg arctangent. Returns Angle in [0, TAU_TICKS) with 0 = +x axis.
/// Implementation: octant decomposition + 1024-entry LUT for atan(y/x) in first octant.
pub fn atan2(y: Fixed32, x: Fixed32) -> Angle {
    // Implementation: use octant symmetry to reduce to atan(t) for t in [0, 1].
    // 1024-entry LUT, generated lazy via f64::atan().round().
    todo!("see plan Task 1a.4 sub-step 4a")
}
```

加 unit test + pin hash test in `tests/determinism.rs`：
```rust
#[test]
fn atan2_pin_hash() {
    let mut h = fxhash::FxHasher64::default();
    for &(y, x) in &[(0,1), (1,1), (1,0), (1,-1), (0,-1), (-1,-1), (-1,0), (-1,1)] {
        atan2(Fixed32::from_i32(y), Fixed32::from_i32(x)).ticks().hash(&mut h);
    }
    let actual = h.finish();
    println!("ATAN2 PIN HASH = {}", actual);
    assert_eq!(actual, 0u64);  // CAPTURE + LOCK
}
```

**Sub-step 4b: 13 script files 切型**

逐檔 grep f32：
- towers/dart.rs: 26 f32 literal（`STATS.atk`, `BONUS_PROC_CHANCE`, `BONUS_DAMAGE`, `dt`, `asd_count`...）
- towers/bomb.rs / tack.rs / ice.rs：類似但 fewer
- heroes/B01/* 4 files：JSON-based，改 `extra["damage"].as_f64() as f32` → `Fixed32::from_raw((extra["damage"].as_f64()? * 1024.0) as i32)`
- heroes/B02/* 4 files：類似
- summons/saika_gunner.rs：`BULLET_SPEED = 900.0` → `Fixed32::from_i32(900)`、移動向量 `dy.atan2(dx)` → `omoba_sim::trig::atan2(dy, dx)`、`len.sqrt()` → `len.sqrt()`（Fixed32 版）

**Step 5: omb world_adapter.rs**

每個 GameWorld method 邊界做 lossy conversion，omb internal 仍 f32：
```rust
fn get_hp(&self, e: EntityHandle) -> Fixed32 {
    let hp_f32 = self.cache.cproperty.get(specs_entity).map(|c| c.hp).unwrap_or(0.0);
    // TODO Phase 1c: remove conversion when CProperty.hp switches to Fixed32
    Fixed32::from_raw((hp_f32 * 1024.0) as i32)
}

fn set_tower_atk(&mut self, e: EntityHandle, v: Fixed32) {
    let v_f32 = v.to_f32_for_render();  // TODO Phase 1c: drop conversion
    /* set internal TAttack.atk_physic = v_f32 */
}
```

`omb/src/scripting/dispatch.rs:37` 的 `dt: f32` 也切 Fixed32：上游呼叫端（`omb/src/state/core.rs` 之類）傳 `Fixed32::from_raw((dt_f32 * 1024.0) as i32)`。

**Step 6: Build verify**

```
cd /d/omoba
cargo build --manifest-path /d/omoba/omoba-sim/Cargo.toml --features abi-stable
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
cargo build --manifest-path /d/omoba/scripts/Cargo.toml
cargo build --manifest-path /d/omoba/omb/Cargo.toml
```
**Expected: ALL PASS**。整個 chain 編得起來。

**Step 7: 跑 omoba-sim test 確認 atan2 pin 鎖死 + 既有 5 pin 仍綠**

```
cargo test -p omoba-sim --manifest-path /d/omoba/omoba-sim/Cargo.toml
```
Expected: 39 + 1 atan2 pin = 40 PASS（第一次跑 atan2_pin_hash 會 FAIL，capture + lock 同 Phase 0 流程）。

**Step 8: Commit（單一 atomic）**

```
refactor(abi): full ABI boundary switched to Fixed32 / Vec2 / Angle

script-abi UnitScript / GameWorld / AbilityScript trait signatures
全部 f32 → Fixed32 / Vec2 / Angle. base_content 13 scripts 內部運算
切 Fixed32 (towers + heroes + summons). omb WorldAdapter 在 ABI
邊界做 f32 ↔ Fixed32 lossy conversion，標 Phase 1[bcd] TODO 待
omb internal ECS 切型後刪除。

omoba-sim 加 atan2(y, x) -> Angle (4096-tick LUT)，鎖 ATAN2 pin hash。

整 chain build 通；run.bat 行為驗證放 Task 1a.5。

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

---

### Task 1a.5：Verify run.bat + spot-check

**Files:**（無新增 / 修改）

**Step 1: 跑 run.bat**

```
cd /d/omoba && cmd //c run.bat
```
Backend 啟動、frontend 啟動、無 crash。讓它跑 30 秒觀察。Ctrl-C 停。

**Step 2: Spot-check tower behavior**

跑 stress map（`run_stress.bat` 要 release build，所以可能要先 release rebuild — 或者用 dev build 跑簡單場景）。觀察：
- DartTower 攻擊 creep？
- 子彈 homing？
- 傷害計算合理？
- 沒有 panic? 沒有 NaN-equivalent values?

**Step 3: Spot-check hero ability**

開個 hero（Saika Magoichi 或 Date Masamune），施放 1 個 ability。觀察 damage 數字、cooldown 倒數正確。

**Step 4: 跑全部測試**

```
cargo test -p omoba-sim --manifest-path /d/omoba/omoba-sim/Cargo.toml
cargo test --manifest-path /d/omoba/omb/Cargo.toml -p omobab
cargo test --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```
Expected: 全綠（omb / base_content 既有測試應該還能跑）。

**Step 5: 跑 gen-docs**

```
cd /d/omoba/omb && cargo run -p omobab --bin gen-docs --features gen-docs --release
```
Expected: PASS，輸出 `target/docs/index.html`。打開應顯示 Fixed32 type 在 Stat Keys / Tower Stats sections。

**Step 6: Phase 1a close commit**

```
git -C /d/omoba commit --allow-empty -m "$(cat <<'EOF'
chore(abi): Phase 1a (ABI boundary migration) complete

ABI 邊界 + base_content 內部運算切 Fixed32 / Vec2 / Angle。omb
internal ECS 仍 f32（WorldAdapter 邊界做 lossy conversion，標
Phase 1[bcd] TODO 待後續 sub-phase 切除）。

Verified:
- run.bat boots, no crash, dev play 30s 行為等價
- 6 cross-OS pin hashes pass (5 from Phase 0 + atan2)
- gen-docs renders Fixed32 types in catalog
- existing omb / base_content tests pass

Next: Phase 1b — omb internal movement components / systems
(Pos, Vel, MoveTarget, CollisionRadius, Facing, TurnSpeed) +
relevant ticks (creep_tick, hero_move_tick, projectile_tick).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
EOF
)"
```

**Step 7: Final code review**

派 superpowers:code-reviewer review entire Phase 1a (BASE_SHA = master before phase, HEAD_SHA = the close commit). Focus areas:
- ABI 切型完整 / 沒漏 method
- WorldAdapter 邊界 conversion 都標 TODO Phase 1[bcd]
- atan2 LUT 跨平台 reproducible
- base_content 13 script 沒留 f32 算術
- 行為等價（從 git diff 推測）

通過 → fast-forward merge `lockstep/phase1a-abi-boundary` to master。

---

## Open Items / 後續 Phase 1b 接手指引

完成 Phase 1a 後 Phase 1b：
1. 移除 `omb/src/scripting/world_adapter.rs` 中所有 `// TODO Phase 1b` conversion（先處理 movement 相關：`get_pos`, `set_pos`, `advance_with_collision`, `get_facing`, `set_facing`）
2. 把 `omb/src/comp/{phys,position,velocity}.rs` 內部 f32 → Fixed32
3. `creep_tick.rs / hero_move_tick.rs / projectile_tick.rs` 切 Fixed32 + Vec2 計算
4. `BlockedRegions` 維持 f32 暫不動（Phase 1d 視野系統再處理）

CI / cross-OS Linux verification 也是 Phase 1 結尾要設置（GitHub Actions ubuntu-latest 跑 omoba-sim test）。
