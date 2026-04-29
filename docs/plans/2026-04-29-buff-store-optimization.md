# BuffStore Optimization Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Eliminate the BuffStore-driven `regen_sys` (46% of run) + `buff_sys` (27% of run) bottleneck so stress map (1000 towers + 1000 creeps) holds max_tps=60 cap; root-cause fix is slow-buff dedup + reverse index; secondary is regen throttle + parallelization.

**Architecture:** Three layers. (1) Slow buffs become single instance per creep with refresh-duration / replace-if-stronger semantics, driven by payload's `slow_factor` field (BuffStore stays semantics-agnostic). (2) BuffStore gains an `entities_by_key: HashMap<String, HashMap<Entity, u32>>` reverse index with refcount, maintained in add/remove/tick/remove_all_for. (3) Consumers (regen_tick, buff_tick) replace full-table joins with index queries; regen_tick gets a 0.25s `dt_acc` throttle and rayon `par_iter` over candidates.

**Tech Stack:** Rust 1.91.0 (locked in `rust-toolchain.toml`); specs 0.20 fork at `D:\omoba\specs\`; abi_stable for cdylib boundary (this plan does NOT touch ABI); rayon for parallelism; crossbeam-channel; serde_json.

**Build commands (Windows cmd):**
- Compile: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
- Test: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
- Stress benchmark: `run_stress.bat` (release + stress map + log to console)

**Phase A (already done) — do not redo:**
- `omb/src/scripting/world_adapter.rs` — `AdapterCache` cached storages
- `omb/src/scripting/dispatch.rs` — adapter.cache.* migration
- `omb/src/comp/tick_profile.rs` — per-system / per-frame logs
- `omb/src/comp/ecs.rs` — `Job<T>::run` records system timing

This plan starts AFTER Phase A.

---

## Phase 1 — Reverse Index + Slow Dedup

Goal of Phase 1: BuffStore gains a maintained reverse index; ICE slow buffs collapse from N-per-creep to 1-per-creep. This phase alone should drop BuffStore size from ~50K to ~1.5K and meaningfully reduce regen / buff system cost (without yet consuming the index).

### Task 1.1: Add `entities_by_key` field to BuffStore (struct only, no maintenance)

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (struct definition, ~line 23-26)

**Step 1: Edit struct + Default**

Change struct in `omb/src/ability_runtime/buff_store.rs`:

```rust
/// 以 `(Entity, buff_id)` 為 key 的 O(1) buff 索引。
/// `entities_by_key` 是 stat key → entity → 引用計數的反向索引，
/// 加速「哪些 entity 受某類 stat 影響」的查詢（regen / DoT 系統用）。
#[derive(Default, Debug)]
pub struct BuffStore {
    buffs: HashMap<(Entity, String), BuffEntry>,
    entities_by_key: HashMap<String, HashMap<Entity, u32>>,
}
```

(The existing `#[derive(Default)]` covers the new HashMap field automatically.)

**Step 2: Compile**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: Success (no usage of new field yet, so no errors).

**Step 3: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): add entities_by_key reverse-index field"
```

---

### Task 1.2: Failing test for `entities_with_key` API

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (add #[cfg(test)] mod at end)

**Step 1: Add test module + first test**

Append to bottom of `omb/src/ability_runtime/buff_store.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use specs::world::Generation;

    fn ent(id: u32, gen: i32) -> Entity {
        Entity::new(id, Generation::new(gen))
    }

    #[test]
    fn entities_with_key_returns_entity_after_add() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "buff_a", 5.0, json!({ "move_speed_bonus": -0.5 }));
        let found: Vec<Entity> = s.entities_with_key("move_speed_bonus").collect();
        assert_eq!(found, vec![e]);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::entities_with_key_returns_entity_after_add`
Expected: FAIL — `no method named entities_with_key found for struct BuffStore` (API doesn't exist yet).

**Step 3: Add the API method (returns empty placeholder)**

Insert into `impl BuffStore` block, after `iter_for`:

```rust
    /// 反向查詢：哪些 entity 身上有 buff payload 含 `key`。
    /// 配合 `regen_tick` / `buff_tick` 的 DoT 掃描，把「對全表 sum_add」
    /// 變成「只對候選 entity sum_add」。返回 iterator，呼叫端可 collect 或 filter。
    pub fn entities_with_key<'a>(&'a self, key: &str) -> impl Iterator<Item = Entity> + 'a {
        self.entities_by_key
            .get(key)
            .into_iter()
            .flat_map(|m| m.keys().copied())
    }
```

**Step 4: Run test — still expected to fail (returns empty)**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::entities_with_key_returns_entity_after_add`
Expected: FAIL — `assertion failed: left: [], right: [e]`. (API exists but index isn't maintained yet.)

**Step 5: Commit (failing test as scaffold for next task)**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "test(buff_store): add failing test for entities_with_key (impl pending)"
```

---

### Task 1.3: Maintain index in `add()` — make Task 1.2 test pass

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` `add()` (~line 36-55)

**Step 1: Add helper for payload-key extraction**

Insert into `impl BuffStore`, near `iter_for`:

```rust
    /// 從 payload 抽出所有頂層 key（這些就是 stat key 字串）。
    /// payload 不是 Object 時返回空 iterator。
    fn payload_keys(payload: &Value) -> impl Iterator<Item = &str> {
        payload
            .as_object()
            .into_iter()
            .flat_map(|m| m.keys().map(|s| s.as_str()))
    }

    fn index_inc(&mut self, entity: Entity, key: &str) {
        let inner = self.entities_by_key.entry(key.to_string()).or_default();
        *inner.entry(entity).or_insert(0) += 1;
    }

    fn index_dec(&mut self, entity: Entity, key: &str) {
        if let Some(inner) = self.entities_by_key.get_mut(key) {
            if let Some(cnt) = inner.get_mut(&entity) {
                *cnt = cnt.saturating_sub(1);
                if *cnt == 0 {
                    inner.remove(&entity);
                }
            }
            if inner.is_empty() {
                self.entities_by_key.remove(key);
            }
        }
    }
```

**Step 2: Update `add()` to maintain the index**

Replace existing `add()` body:

```rust
    pub fn add(&mut self, entity: Entity, buff_id: &str, duration: f32, payload: Value) {
        let key = (entity, buff_id.to_string());
        match self.buffs.get_mut(&key) {
            Some(e) => {
                if duration > e.remaining {
                    e.remaining = duration;
                }
                // 索引：扣舊 payload 的 key、加新 payload 的 key（差集）
                let old_keys: Vec<String> = Self::payload_keys(&e.payload).map(String::from).collect();
                let new_keys: Vec<String> = Self::payload_keys(&payload).map(String::from).collect();
                e.payload = payload;
                for k in &old_keys {
                    if !new_keys.contains(k) {
                        self.index_dec(entity, k);
                    }
                }
                for k in &new_keys {
                    if !old_keys.contains(k) {
                        self.index_inc(entity, k);
                    }
                }
            }
            None => {
                let new_keys: Vec<String> = Self::payload_keys(&payload).map(String::from).collect();
                self.buffs.insert(
                    key,
                    BuffEntry {
                        remaining: duration,
                        payload,
                    },
                );
                for k in &new_keys {
                    self.index_inc(entity, k);
                }
            }
        }
    }
```

**Step 3: Run Task 1.2 test — should pass**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::entities_with_key_returns_entity_after_add`
Expected: PASS.

**Step 4: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): maintain entities_by_key on add()"
```

---

### Task 1.4: Failing test + impl for `remove()` index update

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod + `remove()`)

**Step 1: Add failing test**

Append to `mod tests`:

```rust
    #[test]
    fn remove_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "b", 5.0, json!({ "x": 1.0 }));
        s.remove(e, "b");
        let found: Vec<Entity> = s.entities_with_key("x").collect();
        assert!(found.is_empty(), "expected empty, got {:?}", found);
    }
```

**Step 2: Run — should FAIL** (remove doesn't update index)

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::remove_clears_index`
Expected: FAIL — assertion failure (index still contains entity).

**Step 3: Update `remove()`**

Replace existing `remove()`:

```rust
    pub fn remove(&mut self, entity: Entity, buff_id: &str) {
        if let Some(entry) = self.buffs.remove(&(entity, buff_id.to_string())) {
            let keys: Vec<String> = Self::payload_keys(&entry.payload).map(String::from).collect();
            for k in &keys {
                self.index_dec(entity, k);
            }
        }
    }
```

**Step 4: Run test — should PASS**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::remove_clears_index`
Expected: PASS.

**Step 5: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): maintain entities_by_key on remove()"
```

---

### Task 1.5: Failing test + impl for `tick()` expired index update

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod + `tick()`)

**Step 1: Add failing test**

Append to `mod tests`:

```rust
    #[test]
    fn tick_expired_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "b", 1.0, json!({ "x": 1.0 }));
        let expired = s.tick(2.0); // duration < dt → expire
        assert_eq!(expired.len(), 1);
        let found: Vec<Entity> = s.entities_with_key("x").collect();
        assert!(found.is_empty(), "expected empty after expire, got {:?}", found);
    }
```

**Step 2: Run — FAIL**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::tick_expired_clears_index`
Expected: FAIL.

**Step 3: Update `tick()`**

Replace existing `tick()` with:

```rust
    pub fn tick(&mut self, dt: f32) -> Vec<(Entity, String, Value)> {
        let mut expired = Vec::new();
        // 先收集 expired，避免 retain 內動態借 self（index_dec 也要 &mut self）
        let mut to_drop: Vec<(Entity, String)> = Vec::new();
        for ((e, id), v) in self.buffs.iter_mut() {
            v.remaining -= dt;
            if v.remaining <= 0.0 {
                to_drop.push((*e, id.clone()));
            }
        }
        for (e, id) in to_drop {
            if let Some(entry) = self.buffs.remove(&(e, id.clone())) {
                let keys: Vec<String> = Self::payload_keys(&entry.payload).map(String::from).collect();
                for k in &keys {
                    self.index_dec(e, k);
                }
                expired.push((e, id, entry.payload));
            }
        }
        expired
    }
```

**Step 4: Run all buff_store tests**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests`
Expected: All PASS (3 tests so far).

**Step 5: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): maintain entities_by_key on tick() expire"
```

---

### Task 1.6: Failing test + impl for `remove_all_for()` index update

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod + `remove_all_for()`)

**Step 1: Failing test**

```rust
    #[test]
    fn remove_all_for_clears_index() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "a", 5.0, json!({ "x": 1.0, "y": 2.0 }));
        s.add(e, "b", 5.0, json!({ "z": 3.0 }));
        s.remove_all_for(e);
        for k in &["x", "y", "z"] {
            let found: Vec<Entity> = s.entities_with_key(k).collect();
            assert!(found.is_empty(), "key {} not cleared: {:?}", k, found);
        }
    }
```

**Step 2: Run — FAIL**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::remove_all_for_clears_index`
Expected: FAIL.

**Step 3: Update `remove_all_for()`**

```rust
    pub fn remove_all_for(&mut self, entity: Entity) {
        // 收集要清掉的 buff payload key 集合
        let drained: Vec<((Entity, String), BuffEntry)> = self
            .buffs
            .iter()
            .filter(|((e, _), _)| *e == entity)
            .map(|((e, id), v)| ((*e, id.clone()), v.clone()))
            .collect();
        for ((e, id), entry) in &drained {
            self.buffs.remove(&(*e, id.clone()));
            for k in Self::payload_keys(&entry.payload) {
                self.index_dec(*e, k);
            }
        }
    }
```

**Step 4: Run test — PASS**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests`
Expected: All PASS (4 tests).

**Step 5: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): maintain entities_by_key on remove_all_for()"
```

---

### Task 1.7: Refcount test — multiple buffs sharing the same key

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod only)

**Step 1: Add test**

```rust
    #[test]
    fn refcount_multiple_buffs_same_key() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "buff1", 5.0, json!({ "k": 1.0 }));
        s.add(e, "buff2", 5.0, json!({ "k": 2.0 }));

        // both present — entity still in index
        assert_eq!(s.entities_with_key("k").count(), 1);

        s.remove(e, "buff1");
        // one still left → still indexed
        let found: Vec<Entity> = s.entities_with_key("k").collect();
        assert_eq!(found, vec![e], "after removing 1 of 2, entity should still be indexed");

        s.remove(e, "buff2");
        // both gone → not indexed
        assert!(s.entities_with_key("k").next().is_none());
    }
```

**Step 2: Run — should PASS** (refcount logic from Task 1.3 should handle this)

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::refcount_multiple_buffs_same_key`
Expected: PASS. If FAIL, debug index_inc/dec.

**Step 3: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "test(buff_store): verify refcount across multiple buffs sharing a key"
```

---

### Task 1.8: Failing test for slow dedup — stronger replaces weaker

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod only)

**Step 1: Add test**

```rust
    #[test]
    fn slow_dedup_stronger_replaces_weaker() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        // 先加弱 slow（factor 越小越強，0.5 比 0.3 弱）
        s.add(e, "slow", 5.0, json!({ "move_speed_bonus": -0.5, "slow_factor": 0.5 }));
        // 加強 slow
        s.add(e, "slow", 5.0, json!({ "move_speed_bonus": -0.7, "slow_factor": 0.3 }));
        // 應該保留強 slow（factor=0.3）
        let entry = s.get(e, "slow").expect("slow buff missing");
        let factor = entry.payload.get("slow_factor").and_then(|v| v.as_f64()).unwrap();
        assert!((factor - 0.3).abs() < 1e-6, "expected 0.3 (stronger), got {}", factor);
    }

    #[test]
    fn slow_dedup_weaker_does_not_replace_stronger() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "slow", 3.0, json!({ "move_speed_bonus": -0.7, "slow_factor": 0.3 }));
        s.add(e, "slow", 10.0, json!({ "move_speed_bonus": -0.5, "slow_factor": 0.5 }));
        let entry = s.get(e, "slow").expect("slow buff missing");
        let factor = entry.payload.get("slow_factor").and_then(|v| v.as_f64()).unwrap();
        assert!((factor - 0.3).abs() < 1e-6, "expected 0.3 to be preserved, got {}", factor);
        // duration 應取 max（既有行為）
        assert!(entry.remaining >= 9.99, "expected duration ≥ 10, got {}", entry.remaining);
    }
```

**Step 2: Run — FAIL**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::slow_dedup`
Expected: BOTH tests FAIL — `add()` currently always overwrites payload.

**Step 3: Commit failing tests**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "test(buff_store): add failing tests for slow dedup semantics"
```

---

### Task 1.9: Implement slow dedup in `add()`

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (`add()` method)

**Step 1: Update `add()` to compare slow_factor**

Replace the `Some(e) =>` branch in `add()`:

```rust
            Some(e) => {
                if duration > e.remaining {
                    e.remaining = duration;
                }
                // payload 替換策略：
                //   - 若雙方 payload 都帶 `slow_factor`，較小者（更強）勝出，
                //     僅在新 payload 較強時才覆寫；否則保留原 payload。
                //   - 否則維持原本行為：覆寫。
                let should_replace = match (
                    e.payload.get("slow_factor").and_then(|v| v.as_f64()),
                    payload.get("slow_factor").and_then(|v| v.as_f64()),
                ) {
                    (Some(old), Some(new)) => new < old,
                    _ => true,
                };
                if should_replace {
                    let old_keys: Vec<String> =
                        Self::payload_keys(&e.payload).map(String::from).collect();
                    let new_keys: Vec<String> =
                        Self::payload_keys(&payload).map(String::from).collect();
                    e.payload = payload;
                    for k in &old_keys {
                        if !new_keys.contains(k) {
                            self.index_dec(entity, k);
                        }
                    }
                    for k in &new_keys {
                        if !old_keys.contains(k) {
                            self.index_inc(entity, k);
                        }
                    }
                }
                // should_replace == false 的情況：duration 已 refresh，payload 不動，索引不動
            }
```

**Step 2: Run slow dedup tests — should PASS**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab buff_store::tests::slow_dedup`
Expected: BOTH PASS.

**Step 3: Run all tests in buff_store + workspace**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: all PASS (no regressions).

**Step 4: Commit**

```bash
git add omb/src/ability_runtime/buff_store.rs
git commit -m "feat(buff_store): replace payload only if new slow_factor is stronger"
```

---

### Task 1.10: Verify no callers depend on `slow_{id}` prefix

**Files:**
- Read-only: grep across repo

**Step 1: Grep for hard-coded slow ID patterns**

Run from `D:/omoba/`:
```bash
grep -rn "slow_" --include="*.rs" --include="*.ts" --include="*.json" -l 2>&1 | head -30
```
Expected: should find `projectile_tick.rs` (target of next task) and any other.

Then targeted:
```bash
grep -rn 'format!("slow_\|slow_{' --include="*.rs" 2>&1
```
Expected: only the production site at `omb/src/tick/projectile_tick.rs:227` (current).

**Step 2: Verify no front-end / mcp / script writes match `slow_*` prefix**

```bash
grep -rn '"slow_"' D:/omoba/omfx/src D:/omoba/omb-mcp/src D:/omoba/scripts/base_content/src --include="*.rs" 2>&1
```
Expected: empty.

**Step 3: Note results in commit message — no commit if nothing changed**

(This task is verification only; if grep returns unexpected hits, stop and consult before continuing.)

---

### Task 1.11: Switch projectile_tick.rs slow buff_id to "slow" + add slow_factor to payload

**Files:**
- Modify: `omb/src/tick/projectile_tick.rs:218-232`

**Step 1: Read current block to confirm structure**

Run: `Read tool on D:/omoba/omb/src/tick/projectile_tick.rs` lines 215-235. Confirm code matches:

```rust
if proj.slow_factor > 0.0 && proj.slow_factor < 1.0 && proj.slow_duration > 0.0 {
    let bonus = -(1.0 - proj.slow_factor);
    let mut payload = serde_json::Map::new();
    payload.insert("move_speed_bonus".into(), json!(bonus));
    outcomes.push(Outcome::AddBuff {
        target: ...,
        buff_id: format!("slow_{}", proj.owner.id()),
        duration: proj.slow_duration,
        payload: serde_json::Value::Object(payload),
    });
}
```

**Step 2: Apply edit**

```rust
if proj.slow_factor > 0.0 && proj.slow_factor < 1.0 && proj.slow_duration > 0.0 {
    let bonus = -(1.0 - proj.slow_factor);
    let mut payload = serde_json::Map::new();
    payload.insert("move_speed_bonus".into(), json!(bonus));
    payload.insert("slow_factor".into(), json!(proj.slow_factor));
    outcomes.push(Outcome::AddBuff {
        target: ...,
        buff_id: "slow".to_string(),
        duration: proj.slow_duration,
        payload: serde_json::Value::Object(payload),
    });
}
```

(Keep the `target: ...` field as it currently is — the only changes are buff_id and the new payload entry.)

**Step 3: Compile**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: clean build.

**Step 4: Run tests**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: all PASS.

**Step 5: Commit**

```bash
git add omb/src/tick/projectile_tick.rs
git commit -m "feat(projectile): unify slow buff_id, embed slow_factor in payload for dedup"
```

---

### Task 1.12: Stress benchmark Phase 1

**Files:**
- Read-only: run stress, capture log

**Step 1: Build release**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab --release`

**Step 2: Add temporary log line for BuffStore size**

Edit `omb/src/state/core.rs` `tick()` after the dispatch block, add (will revert later):

```rust
if self.local_tick % 60 == 0 {
    let store = self.ecs.read_resource::<crate::ability_runtime::BuffStore>();
    log::info!("[buff_store] size={}", store.len());
}
```

**Step 3: Run stress**

Run: `run_stress.bat`
Wait 60 seconds.

**Step 4: Capture metrics**

From log, record:
- `tick_profile run / dispatch / total / max_tps`
- `system regen / buff` ms/frame
- `[buff_store] size=` value

Expected (vs baseline `run=24ms, regen=11.1ms, buff=6.5ms, buff_store size ~50K`):
- `buff_store size` ≈ 1500–2500 (1000 creep × 1 slow + small overhead)
- `regen ms/frame` partially down (sum_add internal iter shorter)
- `buff ms/frame` partially down (tick iterates fewer buffs)
- `max_tps` may not yet hit 60 — Phase 2 will close the gap

**Step 5: Revert temporary log**

Remove the [buff_store] size log block from core.rs.

**Step 6: Commit Phase 1 baseline metrics in plan / notes (optional)**

If keeping notes:
```bash
git commit --allow-empty -m "chore(perf): record Phase 1 stress metrics"
```

---

## Phase 2 — Consume the Index in regen + buff_tick

### Task 2.1: buff_tick swap full-table join to entities_with_key

**Files:**
- Modify: `omb/src/tick/buff_tick.rs:64-70`

**Step 1: Edit DoT scan**

Replace lines 64-70:

```rust
// 改前
let dot_targets: Vec<(specs::Entity, f32)> = (&data.entities)
    .join()
    .filter_map(|e| {
        let d = data.buffs.sum_add(e, StatKey::DotDamage);
        if d > 0.0 { Some((e, d)) } else { None }
    })
    .collect();

// 改後
let dot_targets: Vec<(specs::Entity, f32)> = data
    .buffs
    .entities_with_key(StatKey::DotDamage.as_str())
    .filter_map(|e| {
        let d = data.buffs.sum_add(e, StatKey::DotDamage);
        if d > 0.0 { Some((e, d)) } else { None }
    })
    .collect();
```

**Step 2: Compile + test**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab && cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: clean, all tests pass.

**Step 3: Commit**

```bash
git add omb/src/tick/buff_tick.rs
git commit -m "perf(buff_tick): use entities_by_key reverse-index for DoT scan"
```

---

### Task 2.2: Add `dt_acc` field to regen Sys + throttle test

**Files:**
- Modify: `omb/src/tick/regen_tick.rs` (Sys struct + run signature)

**Step 1: Update Sys struct**

```rust
#[derive(Default)]
pub struct Sys {
    /// 累積 dt 達 0.25s 才觸發一次 regen 計算（4 Hz），降低每 frame 跑 1000 creep 的壓力。
    /// 觸發時用累積值當 effective dt，總回血量不變。
    dt_acc: f32,
}

const REGEN_INTERVAL: f32 = 0.25;
```

**Step 2: Wrap existing `run` body with throttle gate**

Modify the start of `run`:

```rust
fn run(job: &mut Job<Self>, mut data: Self::SystemData) {
    job.own.dt_acc += data.dt.0;
    if job.own.dt_acc < REGEN_INTERVAL {
        return;
    }
    let dt = std::mem::replace(&mut job.own.dt_acc, 0.0);
    let tx = data.mqtx.get(0).cloned();
    let entities_ref = &data._entities;

    // ... rest of existing function uses `dt` (from above, not data.dt.0)
}
```

(Search for `let dt = data.dt.0;` and remove it — replaced by the new accumulator pattern.)

**Step 3: Compile**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: clean.

**Step 4: Run tests + smoke regen via stress**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: pass. (No unit test for throttle — gameplay verification in Task 2.5.)

**Step 5: Commit**

```bash
git add omb/src/tick/regen_tick.rs
git commit -m "perf(regen_tick): throttle to 4 Hz via dt accumulator"
```

---

### Task 2.3: Use entities_with_key to find regen candidates

**Files:**
- Modify: `omb/src/tick/regen_tick.rs` (replace double join)

**Step 1: Replace the two creep + hero collect loops**

Wholesale replace the body after the throttle gate (the two `for (e, cp, _) in ...join()` loops + the writeback section):

```rust
fn run(job: &mut Job<Self>, mut data: Self::SystemData) {
    job.own.dt_acc += data.dt.0;
    if job.own.dt_acc < REGEN_INTERVAL {
        return;
    }
    let dt = std::mem::replace(&mut job.own.dt_acc, 0.0);
    let tx = data.mqtx.get(0).cloned();

    // 候選 entity：身上至少有一條 buff 含 HP regen 相關 key（任一）。
    // stress map 預期空集合 → 整個 system 跳過。
    use std::collections::HashSet;
    let candidates: HashSet<specs::Entity> = [
        StatKey::HealthRegenConstant.as_str(),
        StatKey::HealthRegenPercentage.as_str(),
        StatKey::HpRegenAmplifyPercentage.as_str(),
    ]
    .iter()
    .flat_map(|k| data.buffs.entities_with_key(k))
    .collect();

    if candidates.is_empty() {
        return;
    }

    // 序列計算 + 寫回（par_iter 在 Task 2.4 加上）。
    let mut hp_updates: Vec<(specs::Entity, f32, f32)> = Vec::with_capacity(candidates.len());
    for &e in &candidates {
        // 確認 entity 有 CProperty（creep / hero / 召喚物都有）
        let Some(cp) = data.cpropertys.get(e) else { continue };
        if cp.hp <= 0.0 {
            continue;
        }
        // creep / hero 之外的 entity（例：純塔）不算 regen
        let is_creep = data.creeps.get(e).is_some();
        let is_hero = data.heroes.get(e).is_some();
        if !is_creep && !is_hero {
            continue;
        }
        let stats = UnitStats::from_refs(&*data.buffs, data.is_buildings.get(e).is_some());
        let regen = stats.hp_regen(0.0, e);
        if regen.abs() < 0.0001 {
            continue;
        }
        let eff_max = cp.mhp + stats.max_hp_bonus(e);
        let new_hp = (cp.hp + regen * dt).clamp(0.0, eff_max);
        if (new_hp - cp.hp).abs() > 0.01 {
            hp_updates.push((e, new_hp, eff_max));
        }
    }

    for (e, new_hp, _mhp) in &hp_updates {
        if let Some(cp) = data.cpropertys.get_mut(*e) {
            cp.hp = *new_hp;
        }
    }

    if let Some(ref t) = tx {
        for (e, new_hp, mhp) in hp_updates {
            let _ = t.try_send(make_hp_update(e.id(), new_hp, mhp));
        }
    }
}
```

**Step 2: Compile**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: clean (may need `use crate::ability_runtime::UnitStats;` already present).

**Step 3: Test + smoke**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: pass.

**Step 4: Commit**

```bash
git add omb/src/tick/regen_tick.rs
git commit -m "perf(regen_tick): use entities_by_key for candidate selection"
```

---

### Task 2.4: Parallelize regen candidate calculation with rayon

**Files:**
- Modify: `omb/src/tick/regen_tick.rs` (add rayon par_iter)

**Step 1: Add rayon import**

At the top of `omb/src/tick/regen_tick.rs`, ensure:

```rust
use rayon::prelude::*;
```

**Step 2: Change the calculation loop to par_iter**

Replace the `let mut hp_updates: Vec<_> = Vec::with_capacity(...); for &e in &candidates { ... }` block with:

```rust
const PAR_MIN: usize = 32;
let candidates_vec: Vec<specs::Entity> = candidates.into_iter().collect();

let compute = |&e: &specs::Entity| -> Option<(specs::Entity, f32, f32)> {
    let cp = data.cpropertys.get(e)?;
    if cp.hp <= 0.0 {
        return None;
    }
    let is_creep = data.creeps.get(e).is_some();
    let is_hero = data.heroes.get(e).is_some();
    if !is_creep && !is_hero {
        return None;
    }
    let stats = UnitStats::from_refs(&*data.buffs, data.is_buildings.get(e).is_some());
    let regen = stats.hp_regen(0.0, e);
    if regen.abs() < 0.0001 {
        return None;
    }
    let eff_max = cp.mhp + stats.max_hp_bonus(e);
    let new_hp = (cp.hp + regen * dt).clamp(0.0, eff_max);
    if (new_hp - cp.hp).abs() > 0.01 {
        Some((e, new_hp, eff_max))
    } else {
        None
    }
};

let hp_updates: Vec<(specs::Entity, f32, f32)> = if candidates_vec.len() >= PAR_MIN {
    candidates_vec.par_iter().filter_map(compute).collect()
} else {
    candidates_vec.iter().filter_map(compute).collect()
};
```

(`compute` is a closure that only reads — no `WriteStorage` access in parallel section. The serial writeback below mutates CProperty.)

**Step 3: Compile**

Run: `cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: clean. If borrow errors with `data` capture inside `compute`, refactor to take needed refs explicitly:

```rust
let cp_storage = &data.cpropertys;
let creeps = &data.creeps;
let heroes = &data.heroes;
let is_buildings = &data.is_buildings;
let buffs = &*data.buffs;
let compute = move |&e: &specs::Entity| -> Option<(specs::Entity, f32, f32)> { ... use cp_storage etc. ... };
```

(Note: `&*data.buffs` requires data.buffs to outlive the closure. If lifetime issues, use `let buffs_ref: &BuffStore = &data.buffs;`.)

**Step 4: Run tests + cargo test**

Run: `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab`
Expected: pass.

**Step 5: Commit**

```bash
git add omb/src/tick/regen_tick.rs
git commit -m "perf(regen_tick): parallelize candidate calculation with rayon par_iter"
```

---

### Task 2.5: Stress benchmark Phase 2 + verify gameplay

**Files:**
- Read-only

**Step 1: Build release + run stress**

```bash
cargo build --manifest-path D:/omoba/omb/Cargo.toml -p omobab --release
run_stress.bat
```

Wait 60s, capture log.

**Step 2: Verify metrics**

Expected:
- `system regen ms/frame` ≈ 0.0xx (candidates empty → early return)
- `system buff ms/frame` ≈ 0.5–1.5ms
- `tick_profile run` ≈ 6–8ms (down from 24ms)
- `tick_profile total` ≈ 10–14ms
- `max_tps` = 60 (cap)
- outcome `Damage` count similar to baseline (within 1%)

**Step 3: Manual gameplay smoke**

```bash
run.bat   # debug build, normal map (not stress)
```

Test:
- ICE tower hits creep: creep visibly slows
- Two ICE towers hit same creep: creep slows by stronger factor only (not stacking)
- Slow expires after duration: creep speed restores

**Step 4: Commit Phase 2 metrics note (optional)**

```bash
git commit --allow-empty -m "chore(perf): record Phase 2 stress metrics"
```

---

## Phase 3 (Optional) — Permanent Buff Skip in tick()

Run only if Phase 2 leaves `buff ms/frame > 1ms` and BuffStore::tick is still visible. Stress map likely doesn't hit this.

### Task 3.1: Failing test for permanent buff in tick

**Files:**
- Modify: `omb/src/ability_runtime/buff_store.rs` (test mod)

```rust
    #[test]
    fn permanent_buff_not_expired_by_tick() {
        let mut s = BuffStore::new();
        let e = ent(1, 1);
        s.add(e, "passive", f32::MAX, json!({ "x": 1.0 }));
        // tick 一年的 dt — permanent buff 不該過期
        let expired = s.tick(60.0 * 60.0 * 24.0 * 365.0);
        assert!(expired.is_empty());
        assert!(s.has(e, "passive"));
    }
```

Run: `cargo test ... permanent_buff_not_expired_by_tick`
Expected: PASS already (current behavior — `f32::MAX - dt` still > 0). Test confirms invariant; no impl change yet.

If it FAILS (subtraction overflow on f32::MAX), proceed to Task 3.2. Otherwise this Phase is unnecessary.

### Task 3.2: Skip permanent buffs in tick()

(Only if Task 3.1 fails or perf shows benefit.)

```rust
pub fn tick(&mut self, dt: f32) -> Vec<(Entity, String, Value)> {
    let mut expired = Vec::new();
    let mut to_drop: Vec<(Entity, String)> = Vec::new();
    for ((e, id), v) in self.buffs.iter_mut() {
        if v.remaining > 1e9 { continue; }  // permanent — skip
        v.remaining -= dt;
        if v.remaining <= 0.0 {
            to_drop.push((*e, id.clone()));
        }
    }
    // ... same as before
}
```

Commit:
```bash
git commit -m "perf(buff_store): skip permanent buffs in tick to amortize cost"
```

---

## Verification Summary

After full plan complete, expect (vs baseline `max_tps=33, run=24ms`):

| Metric | Before | After |
|---|---|---|
| `max_tps` | 33 | 60 (cap) |
| `tick_profile total` | 30ms | 10–14ms |
| `tick_profile run` | 24ms | 6–8ms |
| `system regen ms/frame` | 11.1 | 0.0x |
| `system buff ms/frame` | 6.5 | 0.5–1.5 |
| `tick_profile dispatch` | 4.9ms | 2.5–3ms (knock-on from smaller BuffStore) |
| BuffStore size after 60s stress | ~50K+ | ~1.5K |

Game correctness:
- ICE slow visually unchanged
- Multi-tower hit on same creep: strongest slow wins (not stacked)
- All `cargo test -p omobab` pass

---

## Critical Files

- `omb/src/ability_runtime/buff_store.rs` — main BuffStore changes (Phase 1 + 3)
- `omb/src/tick/projectile_tick.rs:218-232` — slow buff_id unification (Task 1.11)
- `omb/src/tick/regen_tick.rs` — full rewrite (Phase 2)
- `omb/src/tick/buff_tick.rs:64-70` — DoT scan swap (Task 2.1)

Reference (read-only):
- `scripts/script-abi/src/stat_keys.rs` — `StatKey::as_str()` source
- `omb/src/comp/game_processor.rs` — `handle_add_buff` consumes Outcome::AddBuff (no changes needed; payload pass-through)
- `omb/src/tick/projectile_tick.rs:66` — par_join template for reference
- `omb/src/state/core.rs:312-345` — tick orchestration (where TickProfile is read)
