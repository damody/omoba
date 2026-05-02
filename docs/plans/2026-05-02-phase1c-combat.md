# Phase 1c — omb battle layer migration

> **For Claude:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development。

**Goal:** omb 戰鬥 layer (CProperty / Unit / Hero / TAttack / TProperty / DamageInstance / DamageResult / BuffStore.remaining / UnitStats) + 8 個戰鬥 tick systems + ScriptEvent enum + WorldAdapter 39 個 TODO[cd] 全切 Fixed32。**最關鍵：`damage_tick.rs` 的 `rand::thread_rng()` 換成 deterministic `omoba_sim::SimRng`** — Lockstep 必要的最後 RNG 確定性 fix。

**Architecture:** Component newtypes 直接 wrap Fixed32；`Vf32` wrapper 改為內部 2× Fixed32；BuffStore.remaining 切 Fixed32；UnitStats final_* methods 返 Fixed32；ScriptEvent f32 變 Fixed32 / Vec2；damage_tick 用 `SimRng::from_master_entity(master_seed, tick, attacker_id, op_kind)` 為 dodge / crit roll 提供 per-entity stream。Outcome enum Vec2<f32> 切 Vec2 (omoba_sim::Vec2)。

**Tech Stack:** 同 Phase 1a/1b。8 cross-OS pin hashes locked from 1a/1b；Phase 1c 不新增 pin（純 type 切型 + 既有 SimRng 行為已 pin 在 rng_sequence_pin_hash）。

---

## Context

Phase 1a 邊界 + base_content + ABI ship。Phase 1b movement layer ship 到 master，omb 0 build errors。126 個 `// TODO Phase 1[c]/[cd]/[d]/[e]` markers 留待後續處理：
- Phase 1c: 16 + 92 = 108 markers (battle + battle/ability mixed)
- Phase 1d: 15 (vision / wire format)
- Phase 1e: 3 (Searcher)

Phase 1c 預期清掉 ~108 markers，剩 ~18 (1d + 1e)。

### Reviewer 提的 polish (Phase 1b → 1c carry-over)
- **I-1**: `6434` magic in creep_tick / hero_move_tick → 提到 `omoba_sim::trig::TAU_FIXED_RAW + fixed_rad_to_ticks(rad)` helper
- **M-1**: scripts/base_content 7 個 `[bcd]` markers rename to `[cd]` 或 `[d]`
- **M-2**: comp/phys.rs `instant_distance::Point` 標 `Phase 1b` 改 `Phase 1e`

這 3 個 polish 整合進 Task 1c.1 順手做。

### damage_tick RNG 設計
```rust
// 之前（不 deterministic）：
let mut rng = rand::thread_rng();
if rng.gen::<f32>() < dodge_chance { ... }

// Phase 1c 後：
let master_seed: u64 = world.read_resource::<MasterSeed>().0;
let tick: u32 = world.read_resource::<Tick>().0 as u32;
let mut dodge_rng = SimRng::from_master_entity(master_seed, tick, victim_id.id(), OP_DODGE);
let dodge_roll: Fixed32 = dodge_rng.gen_fixed32_unit();  // [0, 1) Fixed32
if dodge_roll < dodge_chance_fixed { /* dodged */ }
```

需要：
1. 加 `MasterSeed(u64)` resource（如果還沒有）— 全 game seed，game start 時設置（暫時 hard-code 0xDEAD_BEEF；real lockstep host 時由 Phase 2 GameStart 訊息提供）
2. `omoba_sim::SimRng::gen_fixed32_unit() -> Fixed32` 新 method — 內部 `(self.next_u32() % 1024) as i32` 給 [0, 1) 範圍 deterministic
3. `op_kind` 編號：`OP_DODGE = 0`, `OP_CRIT = 1`，定義為 omb internal const（dispatch.rs 或 damage_tick.rs）

---

## Tasks

### Task 1c.1: omoba-sim helpers + Phase 1b polish

**omoba-sim 加**:
- `Fixed32` impl `Mul<i32>`：`fn mul(self, rhs: i32) -> Fixed32 { Fixed32(self.0.wrapping_mul(rhs)) }`（注意：raw * i32 會 wrapping，但因 raw 已經是 *SCALE 後的，所以 raw * 5 = real * 5 — 直接 wrap OK）+ unit test
- `Fixed32` impl `Mul<Fixed32> for i32`（symmetric）
- `Fixed32` impl `MulAssign<Fixed32>` + `DivAssign`（base_content / battle ticks 會用 `cooldown -= dt` 等）
- `omoba_sim::trig` 加 `pub const TAU_FIXED_RAW: i64 = 6434;` + `pub fn fixed_rad_to_ticks(rad: Fixed32) -> i32`
- `omoba_sim::rng::SimRng` 加 `pub fn gen_fixed32_unit(&mut self) -> Fixed32`：返 [0, 1) Fixed32 (raw in [0, 1024))

對應 unit tests + 既有 pin hashes 不破。

**polish carry-over**:
- creep_tick.rs / hero_move_tick.rs 兩個 6434 magic 改用新 helper（小 commit 在 omb submodule）
- comp/phys.rs `Point` impl marker `Phase 1b` → `Phase 1e`
- scripts/base_content 7 個 `[bcd]` markers grep + sed → `[cd]`

**Verify**: `cargo test -p omoba-sim` 64 PASS（58 + 6 new unit tests）；既有 8 pin hashes 不變；omb cargo build clean。

**Commit**: 拆 omoba-sim commit (parent) + omb polish commit (submodule + parent bump)。

---

### Task 1c.2: omb battle component newtypes 切 Fixed32

**Files**:
- `omb/src/comp/creep.rs` — CProperty 5 f32 → Fixed32
- `omb/src/comp/unit.rs` — Unit ~8 f32 → Fixed32（base_armor, magic_resistance, attack_range, move_speed, attack_speed, last_attack_time, aggro_range；i32 fields 不動）
- `omb/src/comp/hero.rs` — Hero `level_growth` LevelGrowth 6 f32 → Fixed32；`ability_cooldowns: HashMap<String, f32>` → `HashMap<String, Fixed32>`；stat method returns f32 → Fixed32
- `omb/src/comp/tower.rs` — Tower.ultimate_cooldown f32 → Fixed32；TAttack（Vf32 wrapper）切；TProperty.hp（Vf32）切；NearbyEnt.dis f32 → Fixed32
- `omb/src/comp/damage.rs` — DamageTypes (3 f32) / DamageFlags (2 f32) / DamageResult (3 f32) → Fixed32
- `omb/src/comp/outcome.rs` — Outcome::Damage / ProjectileLine2 / Death / Heal / CreepData 等 Vec2<f32>+f32 fields → omoba_sim::Vec2 + Fixed32（Vec2 fields 跟 Pos 一致）
- `omb/src/scripting/event.rs` — ScriptEvent::{Damage.amount, AttackLanded.damage, HealthGained.amount, ManaGained.amount, SpentMana.cost, HealReceived.amount, SkillTarget::Point} → Fixed32 / Vec2

**`Vf32` wrapper 設計**：原本 `pub struct Vf32 { bv: f32, v: f32 }` (base + buffed values)。改為 `pub struct Vf32 { bv: Fixed32, v: Fixed32 }`。所有方法 (set_v / set_bv / get_v / get_bv) 同步切型。

**Component file 自身 build 通過。下游 tick systems / world_adapter 預期 cascade error**（200+ errors expected）。

**Commits**: 在 omb submodule 一個大 commit（component definitions only），parent bump。

---

### Task 1c.3: damage_tick rand→SimRng + ability_runtime + ScriptEvent dispatch sites

**Files**:
- `omb/src/comp/resources.rs` — 加 `pub struct MasterSeed(pub u64); impl Default for MasterSeed { ... = 0xDEAD_BEEF_CAFE_BABE; }`，在 omb world init 時插入 resource
- `omb/src/tick/damage_tick.rs` — 兩個 `rand::thread_rng()` 換 SimRng；`gen::<f32>` 換 `gen_fixed32_unit`；攻擊者/受害者 entity_id 從 specs::Entity::id() 拿（u32）；op_kind 0 = dodge, 1 = crit
- `omb/src/ability_runtime/buff_store.rs` — `BuffEntry.remaining: f32 → Fixed32`；`sum_add` / `product_mult` 返 Fixed32
- `omb/src/ability_runtime/unit_stats.rs` — 全 final_* methods 返 Fixed32（base parameter 也切 Fixed32）
- `omb/src/scripting/dispatch.rs` — ScriptEvent payload 切 site；移除 `Phase 1[bcd]/[cd]` lossy conversions

**特別注意**：
- damage_tick 內部運算 (damage_typed.physical * armor_mult 等) 全 Fixed32
- BuffStore.add_buff(target, id, duration: Fixed32, payload) — 簽名切
- HashMap iteration audit：BuffStore 用 HashMap lookup 不影響確定性（lookup 不 iter；sum_add iterate over Vec<BuffEntry> 是 deterministic）

**Verify cascade**: 部分 cargo build clean；hero_tick / tower_tick / 其他 ticks 仍 fail（Task 1c.4）。

**Commit**: omb commit (RNG fix + ability_runtime + dispatch) + parent bump。

---

### Task 1c.4: hero_tick + tower_tick + regen / item / summon / death / buff ticks

**Files**:
- `omb/src/tick/hero_tick.rs` — Hero ability_cooldowns / Unit stats reads；rotation rate 用 1c.1 的 fixed_rad_to_ticks helper
- `omb/src/tick/tower_tick.rs` — TAttack arithmetic, ultimate_cooldown decrement
- `omb/src/tick/regen_tick.rs` — CProperty.hp regen Fixed32
- `omb/src/tick/buff_tick.rs` — BuffEntry.remaining decrement Fixed32
- `omb/src/tick/item_tick.rs`
- `omb/src/tick/summon_tick.rs`
- `omb/src/tick/death_tick.rs`

每個 tick：讀新 Fixed32 components，內部運算 Fixed32。Pos / Facing 邊界已是 lossless（Phase 1b 完成）。Searcher 邊界仍 lossy（Phase 1e）。

**Verify**: omb cargo check 應大幅減錯。

**Commit**: omb commit + parent bump。

---

### Task 1c.5: WorldAdapter cleanup + Outcome boundary + final verify + close

**Files**:
- `omb/src/scripting/world_adapter.rs` — 39 個 `// TODO Phase 1[cd]` markers 全清。drop `f32_to_fixed` helper（無剩餘 user）。get_hp / get_max_hp / deal_damage / heal / add_buff / get_tower_atk / get_final_* 全改 lossless direct read/write
- `omb/src/comp/outcome.rs` — 任何剩餘 ScriptEvent 邊界轉換清掉
- `omb/src/state/{core, initialization, resource_management}.rs` — battle stat init / heartbeat broadcast site cleanup（broadcast wire format 仍 f32 因為 wire format 改寫是 Phase 1d 範圍 — 標 `// TODO Phase 1[d]`）
- 任何剩餘 cascade fix

**Verify**:
- `cargo build` 整 chain clean
- `cargo test -p omoba-sim` 64 PASS（or 62-65；本 phase 沒新增 pin hash）
- `cargo test -p omobab` PASS
- gen-docs renders 9 units / 8 abilities
- 剩餘 `// TODO Phase 1[bcd]/[c]/[cd]` markers grep — Phase 1c-related 全清；只剩 Phase 1[d] (vision/wire) + 1[e] (Searcher)
- damage_tick 跑時：種子化 SimRng 每 tick 給定 attacker entity 獨立 stream，dodge / crit roll 結果 deterministic（手動跑 stress 30 秒 + log 一些 dodge / crit 數字 — 應該每次 run 完全一致 if seed + tick 一致）

**Final code review** 派 superpowers:code-reviewer。

**Phase 1c close commit + fast-forward merge to master**。

---

## Verification end-to-end

完成 5 task 後：
- omoba-sim 64+ tests pass，8 cross-OS pin hashes locked
- omb cargo build clean
- damage_tick 用 SimRng（確定性的 dodge/crit 從此保證）
- gen-docs 渲染 catalog
- 剩餘 Phase 1 markers ≤ 18 (1d / 1e)
- omb master fast-forward to phase1c-combat tip

## 開放問題

- **MasterSeed source**: Phase 1c 暫用 hard-code `0xDEAD_BEEF_CAFE_BABE`；Phase 2 GameStart message 會帶 master_seed，host 收到後 set MasterSeed resource
- **gen_fixed32_unit modulo bias**: `next_u32() % 1024` 有 ~0.000023% modulo bias — 對 game math 可接受；Phase 1d 視需要可換 widening multiply
- **BuffStore HashMap iteration**: sum_add / product_mult iterate over BTreeMap or sorted Vec? 目前 Vec<BuffEntry> 確定性 OK；HashMap<entity → Vec> 的 entity-level lookup 不 iter，OK
