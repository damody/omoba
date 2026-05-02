# Phase 1d/1e — Vision/Wire Triage + Final Determinism Fix

> **For Claude:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development。

**Goal:** 完成 Phase 1 — 把剩餘真正 lockstep-critical 的 f32 sim hot path 全切 Fixed32（Projectile struct internals + WorldAdapter `rand_unit` 換 SimRng + fastrand 換 deterministic + buff_store wire payload encoding raw i32），對其他 marker 做 triage 區分「intentional f32」/「Phase 2 wire protocol」/「真要切」。

**Architecture:** scope 比初估小 — 大部分 Phase 1[d] markers 經 audit 屬於 intentional / wire-format / Phase 2-scope，不該在 Phase 1 切。Phase 1e Searcher 維持 boundary lossy（render hint cache，不影響 sim 確定性，per-tick rebuild 從權威 Pos 拿）。

**Tech Stack:** 同 Phase 1a-c。8 cross-OS pin hashes from Phase 0/1a/1b 不變動。

---

## Marker Triage（在 Task 1de.3 集中處理）

從 audit 135 個 [d] markers + 3 個 [e] markers 出來：

### 真正 Phase 1d 該做的（8-10 markers，影響 sim 確定性）

| Site | Issue | Action |
|---|---|---|
| `omb/src/comp/projectile.rs` (struct 8 fields) | Projectile.tpos vek::Vec2<f32> + msd / time_left / damage_phys / magi / real / slow_factor / slow_duration / stun_duration / hit_radius f32 | 全切 Vec2 / Fixed32 |
| `omb/src/comp/game_processor.rs` Projectile spawn / tick connectors (~5 sites) | Convert at boundary for legacy spawn | clean once Projectile struct migrates |
| `omb/src/tick/projectile_tick.rs` (Phase 1b deferred) | Pos boundary lossy at read/write | clean once Projectile struct migrates |
| `omb/src/scripting/world_adapter.rs` `rand_unit` | `Pcg64Mcg::gen_range::<f32>(0..1)` then quantize | swap to `SimRng::from_master(seed, tick).gen_fixed32_unit()` — final f32 path removed |
| `omb/src/tick/creep_tick.rs` fastrand jitter | `fastrand::f32()` for spawn pos jitter — non-deterministic | swap to `SimRng::from_master_entity(seed, tick, creep_id, OP_SPAWN_JITTER).gen_fixed32_unit()` |
| `omb/src/ability_runtime/buff_store.rs` (3 sites) | f32 payload via `(v * 1024.0) as i32` quantize from JSON | switch to raw i32 wire encoding (still JSON Number, but encode `Fixed32::raw()` directly) |

### Intentional f32（不切；改 doc comment 表示「by design」）

| Marker | Reason |
|---|---|
| `bin/gen_docs_lib/dll.rs:39` | render-only HTML reporting struct |
| `comp/campaign_manager.rs:99` Unit i32 hp | Unit.{current_hp,max_hp,base_damage} 仍 i32 (intentional) |
| `comp/facing.rs:12, 21` `from_xy_f32`/`from_rad_f32` legacy helpers | transition utility 給 wire format / config spawn 邊界用，不該移除 |
| `comp/enemy.rs:126` Enemy migrate Fixed32 | Enemy 是 spawn template — 跟 Unit 同 i32 design |
| `comp/circular_vision_refactored.rs:64, 117` | Vision = client-side fog of war / render hint (per Phase 1b explore: "not sim state") |
| `comp/game_processor.rs` proto helpers / JSON outbound / log f32 / wire-format builders (~15 markers) | All wire format — Phase 2 重設計 |
| `state/core.rs / state/resource_management.rs / mqtt_handler.rs` heartbeat / hero stats payload sites | Wire format — Phase 2 |
| `ability_runtime/unit_stats.rs:267` apply_incoming_damage f32 sig | Phase 1c.3 deferred — caller chain f32; rewriting requires Outcome::Damage i32 fields too which is also wire-related |

對這類 markers 用 `Edit` 把 `TODO Phase 1[d]:` 改成 `NOTE:` 或 `// Wire format (Phase 2 redesign):`，不再是 active TODO。

### Phase 2 scope（wire protocol 重設計才會碰）

state/core.rs heartbeat broadcast 邏輯、KCP wire encoding、proto helpers、mqtt_handler、entire wire layer。Phase 2 會重寫整個 KCP tag protocol（從 0x01-0x06 換 0x10-0x16 lockstep tags），那時順手 deterministic 編碼。標記重命名 `// PHASE 2: wire protocol redesign`。

### Phase 1e（3 markers — keep boundary lossy）

`omb/src/tick/nearby_tick.rs:113, 155, 195` — Searcher / instant_distance::Point f32 boundary。因 Searcher 是 cache（per-tick rebuild from authoritative Pos），final distance check 在 caller 用 sim-Fixed32 算。**結論**: 維持 boundary lossy，加 deterministic insertion order guarantee（按 Entity ID 排序 insert，不依賴 specs Join 順序），把 markers 改成 `// NOTE: Searcher uses f32 internally for instant_distance LIB compat;...` doc comment。

---

## Tasks

### Task 1de.1: Projectile struct + connectors migration

**Files**:
- `omb/src/comp/projectile.rs` — full Fixed32 / Vec2 migration
- `omb/src/tick/projectile_tick.rs` — drop Pos boundary lossy
- `omb/src/comp/game_processor.rs` — Projectile spawn / tick connector ~5 sites cleanup
- `omb/src/comp/outcome_system/creation_events.rs` (likely cascade)

Migration:
- `Projectile.tpos: Vec2<f32>` → `Vec2`
- `Projectile.msd / time_left / damage_phys / magi / real / slow_factor / slow_duration / stun_duration / hit_radius` 9 f32 → Fixed32
- `Projectile.kind_id`, lifetime / type tags 不變

Verify cargo check clean for projectile + game_processor; full omb cargo build clean.

Commit: omb side + parent bump.

### Task 1de.2: rand_unit SimRng + fastrand + buff_store payload

**Files**:
- `omb/src/scripting/world_adapter.rs` `rand_unit()` — final swap
- `omb/src/tick/creep_tick.rs` (or wherever) `fastrand::*` — replace
- `omb/src/ability_runtime/buff_store.rs` — wire payload encoding raw i32

`rand_unit`:
```rust
fn rand_unit(&mut self) -> Fixed32 {
    let master_seed = /* read MasterSeed resource via cache or stored field */;
    let tick = /* read Tick */;
    // Each rand_unit call from a single dispatch: deterministic stream by tick seed only
    // (multiple rolls in same tick OK — RNG state evolves)
    self.rng.gen_fixed32_unit()  // self.rng already seeded by master_seed in WorldAdapter::new
}
```

實際上 WorldAdapter 已有 `self.rng: Pcg64Mcg` seeded once per dispatch. 改：
```rust
self.rng: omoba_sim::SimRng,  // type swap
// in WorldAdapter::new(): self.rng = SimRng::from_master(master_seed, tick),
fn rand_unit(&mut self) -> Fixed32 { self.rng.gen_fixed32_unit() }
```

fastrand 替換：grep `fastrand::` in omb/src/, 替換 SimRng instance（以 entity_id + tick + op_kind 為 stream isolation）。

buff_store wire payload：
- 之前: `Fixed32::from_raw((v.as_f64()? * 1024.0) as i32)` — quantization from f64
- 改: `Fixed32::from_raw(v.as_i64()? as i32)` — read raw i32 directly from JSON
- write side: `serde_json::json!({ "atk_bonus": fixed.raw() })` (raw integer not float)
- 影響 base_content scripts 寫 buff payload 的方式 — heroes' add_stat_buff sites

scripts/base_content cascade: heroes B01/B02 各自寫 `serde_json::json!({ "key": fixed.to_f32_for_render() })` 改成 `.raw()`。對應 buff_store 讀邊界也改。

Verify cascade build clean.

Commit (parent + omb + parent bump if base_content also touched).

### Task 1de.3: Marker triage + Searcher audit + verify + Phase 1d/1e close

**Marker triage** — 用 `Edit` (or `replace_all`) 對 ~120 個 markers 重命名：
- "render only" / Unit i32 / Enemy spawn template / facing legacy helpers / gen_docs / vision: → `// NOTE: <reason>` (drop active TODO)
- wire format / proto / mqtt / log / heartbeat / hero stats payload: → `// PHASE 2: wire protocol redesign — drop f32 boundary in lockstep tag rework`
- nearby_tick Searcher (3 sites): → `// NOTE: Searcher uses f32 internally for instant_distance lib compat; deterministic via per-tick rebuild from authoritative Pos`

**Searcher deterministic insertion**:
- 看 `omb/src/comp/outcome.rs::Searcher` rebuild logic
- 確認 entities insert 排序穩定（by Entity ID ascending），不依 specs Join order
- 如果不是穩定排序，加 `.sorted_by_key(|e| e.id())` step
- 若已 inherent stable，加 unit test 驗證

**Final verify**:
- `cargo build` whole chain clean
- `cargo test -p omoba-sim`: 65 PASS, 8 pin hashes locked
- `cargo test -p omobab`: PASS
- gen-docs renders 9 units / 8 abilities
- Active `// TODO Phase 1` markers count → near 0 (剩下的都改成 NOTE / PHASE 2 / 解釋性 doc)
- Active `// PHASE 2:` markers count for accountability

**Phase 1 全部 close commit**:
```
chore(phase1): Phase 1 (omb internal Fixed32 migration) complete

All omb internal ECS components, tick systems, ability runtime, and ABI
boundaries migrated to Fixed32 / Vec2 / Angle. Lockstep determinism
foundation complete:

- Phase 1a: ABI boundary + base_content scripts (commit a23a979)
- Phase 1b: movement layer (commit 166a75a)
- Phase 1c: combat layer + damage_tick SimRng (commit 8e1f79d)
- Phase 1d/1e: Projectile struct + WorldAdapter rand_unit + buff_store
              wire payload + Searcher boundary audit + marker triage

Cross-OS determinism: 8 pin hashes locked. SimRng seeded by
(master_seed, tick, entity_id, op_kind) for all sim RNG.

Remaining TODO markers categorized:
- // NOTE: intentional f32 (render-only / Unit i32 / vision client-side)
- // PHASE 2: wire protocol redesign sites (proto / mqtt / heartbeat /
  hero stats payload — handled by Phase 2 KCP tag rework)

Next: Phase 2 — wire protocol switching (new KCP tags 0x10-0x16,
InputBuffer + TickBroadcaster, eliminate f32 wire format).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>
```

After close commit, dispatch superpowers:code-reviewer for entire Phase 1d/1e (BASE = master tip before Phase 1d, HEAD = close). Approve → fast-forward merge to master.

---

## Verification end-to-end

完成 3 task 後：
- omoba-sim 65 tests pass, 8 pin hashes locked
- omb cargo build clean
- gen-docs 渲染 9 units / 8 abilities
- damage_tick + WorldAdapter::rand_unit + creep_tick spawn jitter 全 SimRng-driven
- Active TODO Phase 1 markers ~0 (其餘改成 NOTE / PHASE 2 doc)
- Phase 1 整體完工，準備 Phase 2 wire protocol switching

## 開放問題

- **`from_xy_f32` / `from_rad_f32` legacy helpers**: 留作 transition utility（init / config spawn / wire format read 邊界用）。Phase 2 wire protocol redesign 後可考慮 deprecate；目前 keep。
- **CircularVision / VisionSystemManager 內部 f32**: 不切（client-side fog of war，per-tick rebuild from sim-Fixed32 Pos）— 加 `// NOTE: vision is client-side render hint` doc。
- **Phase 2 vs Phase 1d 邊界**: 我把所有 wire format / proto / mqtt / log f32 sites 推到 Phase 2 — 這是 design call。若 user 想在 Phase 1 內全清，可額外加 task。
