# Lockstep Phase 4-5 Cleanup & HUD Restoration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or superpowers:subagent-driven-development if running this session) to implement this plan task-by-task.

**Goal:** 收尾 Phase 0-3 + 部分 4-5 的漏網之魚 — 砍掉 omb 端 24 個殘留 legacy GameEvent emit、補完 4 個 PlayerInput wire（TowerPlace/Sell/Upgrade/ItemUse）、把 Phase 5.1 砍掉的 13 個 HUD 元素全部從 SimWorldSnapshot 重接，達成 TD_1 + TD_STRESS 完整可玩 + wire 流量 < 5 KB/s。

**Architecture:** 不再做 omb→omfx 的 per-event broadcast；所有渲染狀態走 lockstep sim 在 omfx side 跑出來的 ECS World，extract 成 `SimWorldSnapshot` 後 render thread 讀取。omb 仍保留 `GameLives` / `GameEnd` 兩個 HUD broadcast 不動（玩家操作 ack 路徑），其他全靠 sim 端推算。Phase 5（Observer mode）暫不做 — 若未來要做，採 Path Y（specs SerializeComponents 完整 World serialize）。

**Tech Stack:** Rust 1.91.0 / specs 0.20 / abi_stable / KCP（feature default）/ Fyrox 1.0.1 / `omoba_sim::Vec2 / Fixed64::to_f32_for_render` boundary conversion / 既有 `omb/src/ability_runtime/{buff_store,unit_stats}` 的 hero stats aggregation logic

---

## Context（為什麼有這個 plan）

Phase 0-3 + 部分 4-5 已 merge，原 plan 標記「Phase 5 final complete」但實際漏了三大塊：

1. **omb 端 24 個 legacy GameEvent emit 還在**（Phase 4.4 只砍 feature flag、Phase 5.2 只刪 gated code，但 tick system 內 `tx.try_send(make_*_event(...))` site 沒清）
   - 後果：wire 流量 ~24 KB/s，KcpClient backpressure 暴露 reader-task deadlock（已用 `try_send` 暫補但治標）
2. **omfx 端 13 個 UI 元素被 Phase 5.1 砍 `apply_event` 後沒重接**（粗 zigzag path / 塔選單 / hero 衍生屬性 / 物品欄 / Q/W/E/R 列表 / 爆炸 VFX 等）— 玩起來缺一大半 HUD
3. **4 個 PlayerInput 只 stub log 沒實作**：TowerPlace / TowerSell / TowerUpgrade / ItemUse → TD_1 玩家**不能放塔玩戰鬥**

本 session 已修：
- StartRound input 全鏈
- sim_runner Time/DeltaTime
- script_dispatch
- process_outcomes
- KcpClient `try_send` freeze fix（治標）

驗證過原始計畫文檔的關鍵 line numbers：
- `make_entity_death` 實際只有 **2 個** cut sites（490 + 938），不是 3 個
- `proto/game.proto` 4 個 PlayerInput variant 全已存在（TowerPlace=5 / TowerUpgradeInput=6 / TowerSell=7 / ItemUse=8）
- `player_input_tick.rs` 4 個 stub arm 在 132/142/152/160
- `Outcome` enum 在 `comp/outcome.rs:20` — Phase 4b 加 `Explosion` variant

---

## 階段順序與依賴

```
Phase 1: 砍 omb legacy emit（含 entity_death snapshot diff）
  ↓
Phase 2: 補完 4 個 PlayerInput wire
  ↓
Phase 3: B-visible（粗 path / round / lives / hero stats panel）
  ↓
Phase 4: B-rest（regions / inventory / abilities / explosions / tower upgrade levels）
```

順序理由：
- Phase 1 先砍才能讓 wire 不 deadlock，後續 phase 才能跑長期 smoke
- Phase 2 在 Phase 3 之前 → 玩家放完塔才能驗證 Phase 3 的 hero stats panel 隨塔升級正確更新
- Phase 1 順帶砍 `make_entity_death`（採 Snapshot diff 方案）為 Phase 4 打底

每個 task 完成後跑該 phase 的 verify command。每 task 一個 commit。

---

## Phase 1 — 砍 omb legacy GameEvent broadcast（24 sites）

**估時：1.5-2 天 / 8 tasks**

**KEEP（HUD 用，4 sites — 不動）：**
- `omb/src/comp/game_processor.rs:941` `make_game_lives` (try_send)
- `omb/src/comp/game_processor.rs:496, 944` `make_game_end` ×2
- `omb/src/comp/game_processor.rs:413` + `world_adapter.rs:589` `make_game_explosion` ×2 — Phase 4b 改 sim Outcome 後才砍

### Task 1.1: 砍 creep_tick.rs 內 facing/stall emit

**Files:**
- Modify: `omb/src/tick/creep_tick.rs:249` — delete `tx.try_send(make_entity_facing(...))` 整行 + 上方 `if needs_emit { facing_bc.0 = Some(...); ... }` block 簡化（保留 `facing_bc.0 = Some(new_facing_rad)` 若仍被讀）
- Modify: `omb/src/tick/creep_tick.rs:320` — delete `tx.try_send(make_creep_stall(...))` 整行（外層 `if blocked` block 保留 `pos.0` 不變那個邏輯）

**Step 1: Read** `omb/src/tick/creep_tick.rs` 240-330 確認 emit 區段範圍

**Step 2: Delete** 兩個 `tx.try_send` 呼叫並 audit 上下文 — `facing_bc` resource 是否有其他讀取者？若無，連 SystemData 都拿掉

**Step 3: Verify build**
```
cargo check --manifest-path D:/omoba/omb/Cargo.toml -p omobab
```
Expected: clean，可能有 `unused import` warning → 一併清掉

**Step 4: Run lib tests**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: 145 全綠

**Step 5: Commit**
```
git -C D:/omoba/omb commit -am "phase1: drop creep_tick legacy facing/stall emit"
```

---

### Task 1.2: 砍 hero_tick / tower_tick / hero_move_tick facing+M emit

**Files:**
- Modify: `omb/src/tick/hero_tick.rs:256` — delete `make_entity_facing` emit
- Modify: `omb/src/tick/tower_tick.rs:215` — delete `make_entity_facing` emit
- Modify: `omb/src/tick/hero_move_tick.rs:260` — delete `hero.M` send

**Verify:**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
```
145 全綠。

**Commit:** `phase1: drop hero/tower facing + hero move legacy emit`

---

### Task 1.3: 砍 buff_tick / regen_tick HP/slow emit

**Files:**
- Modify: `omb/src/tick/buff_tick.rs:80` — delete `make_hp_update` (DoT) emit
- Modify: `omb/src/tick/buff_tick.rs:112` — delete `make_creep_slow` emit
- Modify: `omb/src/tick/regen_tick.rs:129` — delete `make_hp_update` emit

**Verify:**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
```

**Commit:** `phase1: drop buff/regen legacy hp/slow emit`

---

### Task 1.4: 砍 game_processor.rs hero static/hot/stats + projectile create + creep create + tower C 等 9 sites

**Files (lines per current file state — 砍前 re-grep 確認):**
- Modify: `omb/src/comp/game_processor.rs:605` — delete `mqtx.send(static_msg)`
- Modify: `omb/src/comp/game_processor.rs:612` — delete `mqtx.send(hot_msg)`
- Modify: `omb/src/comp/game_processor.rs:624` — delete legacy hero.stats `OutboundMsg::new_s_at` send（含 `#[cfg(not(feature="kcp"))]` arm，整 arm 砍掉，只留 kcp branch）
- Modify: `omb/src/comp/game_processor.rs:813,1001` — delete `make_projectile_create` emits
- Modify: `omb/src/comp/game_processor.rs:875` — delete `make_creep_create` emit
- Modify: `omb/src/comp/game_processor.rs:916` — delete `make_creep_slow` emit
- Modify: `omb/src/comp/game_processor.rs:1019` — delete `tower.C` emit
- Modify: `omb/src/comp/game_processor.rs:1145` — delete `entity.Miss` emit
- Modify: `omb/src/comp/game_processor.rs:1155` — delete `make_hp_update_at` emit

**Caveats:**
- `:605/612/624` 在 `if leveled_up { ... }` 區段 — 整段 hero static/hot 廣播在 Phase 3c 後由 sim 端 snapshot 完全取代，可整段砍。但 `let leveled_up = ...` 上方算分配 gold/exp 的邏輯**保留**（純 ECS 寫入，Phase 3c 仍要從 sim 讀到正確值）
- `:813,1001` 兩個 projectile_create site 是不同 spawn 路徑（一個 hero attack、一個 tower attack）— audit 兩處都砍
- `make_creep_create:875` 砍掉前確認 sim 端 creep spawn 系統有同步走（不能只砍廣播不砍 ECS spawn）

**Verify:**
- Build clean
- `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib` → 145 全綠
- 砍完後 `cargo build` 應有些 unused fn warning（`make_projectile_create` / `make_hp_update_at` 等若 0 callers）— Task 1.8 統一清

**Commit:** `phase1: drop game_processor projectile/creep/tower/hero legacy emit (9 sites)`

---

### Task 1.5: 砍 world_adapter.rs script-side legacy emit

**Files:**
- Modify: `omb/src/scripting/world_adapter.rs:486` — delete `unit.C` emit
- Modify: `omb/src/scripting/world_adapter.rs:577` — delete `make_projectile_create_script` emit

**保留：** `make_game_explosion_script:589` — Phase 4b 補 `Outcome::Explosion` 後再砍

**Verify:**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
```

**Commit:** `phase1: drop world_adapter unit.C + projectile script emit`

---

### Task 1.6: Phase 1b — entity_death → SimWorldSnapshot.removed_entity_ids

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — `SimWorldSnapshot` 加欄位
- Modify: `omfx/game/src/sim_runner.rs` `extract_snapshot` — 計 alive diff
- Modify: `omfx/game/src/lib.rs` `update_sim_batches` 跟 `sim_entity_labels`（搜 `EntityKind::Tower => ...` / `EntityKind::Creep => ...` 周邊，dispose 邏輯接 removed_entity_ids）

**Step 1: 加 snapshot 欄位**

`omfx/game/src/sim_runner.rs`，`SimWorldSnapshot`：
```rust
#[derive(Default, Clone, Debug)]
pub struct SimWorldSnapshot {
    pub tick: u32,
    pub entities: Vec<EntityRenderData>,
    pub paths: Vec<Vec<(f32, f32)>>,
    /// Entity ids whose ECS entity was removed since the previous snapshot.
    /// Render side frees scene nodes / labels keyed on these ids.
    pub removed_entity_ids: Vec<u32>,
}
```

**Step 2: extract_snapshot 維護 prev_alive HashSet**

在 sim worker thread loop 找 `extract_snapshot` 呼叫處（worker-local state，不是 snapshot 本身）：
```rust
// worker-local state, persists across iterations:
let mut prev_alive: std::collections::HashSet<u32> = HashSet::new();

// each tick after dispatcher.dispatch:
let mut current_alive: HashSet<u32> = HashSet::new();
let mut entities = Vec::new();
for (e, ...) in (&entities_storage, ...).join() {
    current_alive.insert(e.id());
    entities.push(EntityRenderData { ... });
}
let removed_entity_ids: Vec<u32> = prev_alive.difference(&current_alive).copied().collect();
prev_alive = current_alive;
let snap = SimWorldSnapshot {
    tick, entities, paths, removed_entity_ids,
};
```

**Step 3: render 端消費**

`omfx/game/src/lib.rs` 找渲染 cleanup 段（grep `entities.retain` / `scene_node_by_eid.remove`）— 加：
```rust
for &eid in &snapshot.removed_entity_ids {
    if let Some(node) = self.scene_nodes_by_eid.remove(&eid) {
        self.scene.remove_node(node);
    }
    self.labels.remove(&eid);
    self.collision_rings.remove(&eid);
    // ...其他 per-eid cache
}
```

**Step 4: 砍 omb 端 entity_death emit**
- Delete: `omb/src/comp/game_processor.rs:490` `mqtx.send(make_entity_death(...))`
- Delete: `omb/src/comp/game_processor.rs:938` `mqtx.try_send(make_entity_death("creep", ...))`

**Step 5: Verify**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p omfx
```
- omb 145 全綠
- omfx clean
- 手動 smoke：`./run.bat` (TD_1)，creep 走完路 / 被打死 / 塔被拆 都正常從畫面消失

**Commit:** `phase1b: entity death via snapshot diff; drop omb entity_death emit`

---

### Task 1.7: Smoke — TD_STRESS 60s 驗證 freeze 解了

**Files (no edits):**
- Run: `D:/omoba/run_smoke_long.bat`（已存在，TD_STRESS 60 秒 sim）

**Step 1: 確保 game.toml STORY = TD_STRESS**

PowerShell（避免 sed 寫不過 CRLF）：
```powershell
$p = 'D:/omoba/omb/game.toml'
(Get-Content -Raw $p) -replace 'STORY\s*=\s*"TD_1"', 'STORY = "TD_STRESS"' | Set-Content -Encoding utf8 $p
```

**Step 2: Run smoke**
```
cd /d D:/omoba && run_smoke_long.bat
```
Expected: 60 秒內無 panic 結束，產 `omb_app.log` 跟 `omfx_app.log`

**Step 3: Inspect logs**
- omfx：`grep -c "no TickBatch in 1.0s" omfx_app.log` → 0
- omb：`grep -c "Removed disconnected KCP session" omb_app.log` → 0
- omb：`grep "kcp-p7 .* bytes_per_sec" omb_app.log | tail -10` → 確認 < 5000 bytes_per_sec 持續穩定

**Step 4 (recover):** 復原 STORY
```powershell
$p = 'D:/omoba/omb/game.toml'
(Get-Content -Raw $p) -replace 'STORY\s*=\s*"TD_STRESS"', 'STORY = "TD_1"' | Set-Content -Encoding utf8 $p
```

**No commit** — 只是驗證 gate。若失敗 → revert Phase 1 cuts 並排查（不該失敗，因為砍掉的全是 wire emit，sim ECS 沒動）

---

### Task 1.8: 清 dead code — 砍掉 0 callers 的 `make_*` builder

**Files:**
- Modify: `omb/src/comp/game_processor.rs` 上方 builder fn 區
- Modify: `omb/src/scripting/world_adapter.rs` builder fn 區

**Step 1:** `cargo build --manifest-path D:/omoba/omb/Cargo.toml` 看 unused fn warning

**Step 2:** 對每個 warning：grep callers，若真 0 callers → 砍 fn。預期砍掉：
- `make_entity_facing`
- `make_creep_stall`
- `make_creep_slow`
- `make_creep_create`
- `make_hp_update`
- `make_hp_update_at`
- `make_entity_death` (function 本身)
- `make_projectile_create`
- `make_projectile_create_script`
- 對應的 unused imports

**保留：** `make_game_lives` / `make_game_end` / `make_game_explosion` / `make_game_explosion_script`

**Verify:** `cargo build` clean，無 warning。`cargo test --lib` 145 全綠。

**Commit:** `phase1: clean dead-code legacy event builders`

---

## Phase 2 — PlayerInput wire 補完 4 個

**估時：1 天 / 4 tasks**

`StartRound` 已是 template — 同樣 pattern 套用。**Phase 1 必須先 100% 完成**，避免 tower spawn 經 input 後又走到已砍的 legacy emit 出 dead branch。

### Task 2.1: TowerPlace input wire

**Files:**
- Modify: `omfx/game/src/lib.rs` — 找原 TowerPlace TODO log（grep `TowerPlace TODO` 或 `tower place` mouse handler 區）
- Modify: `omb/src/tick/player_input_tick.rs:132` — `PlayerInputEnum::TowerPlace(t)` arm

**Step 1: omfx side — UI click handler 改 send_lockstep_input**

替換原 `log::info!("TowerPlace TODO ...")`：
```rust
let pos = world_render_to_vec2i(self.mouse_world_pos);
self.lockstep_client.send_lockstep_input(
    PlayerInput { action: Some(PlayerInputAction::TowerPlace(TowerPlace {
        tower_kind_id: selected_tower_kind_id,
        pos: Some(pos),
    })) }
);
log::info!("Tower place lockstep input submitted: kind_id={} pos=({}, {})",
    selected_tower_kind_id, pos.x, pos.y);
```

**Step 2: omb side — 接 GameProcessor::handle_tower_spawn**

`omb/src/tick/player_input_tick.rs`，`Some(PlayerInputEnum::TowerPlace(t)) => {` arm 改：
```rust
Some(PlayerInputEnum::TowerPlace(t)) => {
    let pos_raw = t.pos.as_ref();
    let (px, py) = pos_raw.map(|v| (v.x, v.y)).unwrap_or((0, 0));
    log::info!(
        "player_input_tick: pid={} tick={} TowerPlace kind_id={} pos_raw=({}, {})",
        player_id, tick, t.tower_kind_id, px, py,
    );
    let kind_id = t.tower_kind_id;
    let pos = omoba_sim::Vec2::new(
        omoba_sim::Fixed64::from_raw(px as i64),
        omoba_sim::Fixed64::from_raw(py as i64),
    );
    if let Err(e) = crate::comp::GameProcessor::handle_tower_spawn(world, kind_id, pos, player_id) {
        log::warn!("TowerPlace failed pid={} kind_id={}: {:?}", player_id, kind_id, e);
    }
}
```

**前提：** `GameProcessor::handle_tower_spawn` 簽章存在且 pub。若不存在或 private：
- 找現有 tower spawn 路徑（grep `ScriptEvent::Spawn` + `Tower`）
- 抽取成 `pub fn handle_tower_spawn(world: &mut World, kind_id: u32, pos: Vec2, owner_pid: u32) -> Result<Entity, ...>`
- 流程：lookup template by kind_id → 建 Entity（含 ScriptUnitTag + Tower + Pos + ...）→ push `ScriptEvent::Spawn` → return entity

**Step 3: Verify**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p omfx
```

手動 smoke (`run.bat`, TD_1):
- 點塔按鈕 → 點地圖 → omb log: `player_input_tick: ... TowerPlace ...` + spawn 成功
- omfx 端塔出現在地圖（透過 sim 端 spawn 後 snapshot diff）

**Commit:** `phase2: TowerPlace input wired to GameProcessor::handle_tower_spawn`

---

### Task 2.2: TowerSell input wire

**Files:**
- Modify: `omfx/game/src/lib.rs` — sell button click handler
- Modify: `omb/src/tick/player_input_tick.rs:152` — `TowerSell` arm

**Step 1: omfx side**
```rust
self.lockstep_client.send_lockstep_input(
    PlayerInput { action: Some(PlayerInputAction::TowerSell(TowerSell {
        tower_entity_id: target_eid,
    })) }
);
```

**Step 2: omb side — 加 sell handler**

```rust
Some(PlayerInputEnum::TowerSell(s)) => {
    log::info!("player_input_tick: pid={} tick={} TowerSell entity_id={}",
        player_id, tick, s.tower_entity_id);
    if let Err(e) = crate::comp::GameProcessor::handle_tower_sell(world, s.tower_entity_id, player_id) {
        log::warn!("TowerSell failed: {:?}", e);
    }
}
```

新加 `GameProcessor::handle_tower_sell(world, entity_id, owner_pid)`：
- entity_from_id(world, entity_id)
- 確認該 entity 有 `Tower` component + ownership 屬於 player_id
- 算 refund gold（template cost × 0.5 之類，看現有 sell rule；若無 → 80%）
- 加 gold 到 player.hero
- world.delete_entity(e)（snapshot diff 自動移除 render）

**Step 3: Verify** + smoke：放兩個塔 → 賣掉第一個 → gold + 塔消失。

**Commit:** `phase2: TowerSell input + GameProcessor::handle_tower_sell`

---

### Task 2.3: TowerUpgrade input wire

**Files:**
- Modify: `omfx/game/src/lib.rs` — upgrade button click handler
- Modify: `omb/src/tick/player_input_tick.rs:142` — `TowerUpgrade` arm

**Step 1: omfx side**
```rust
self.lockstep_client.send_lockstep_input(
    PlayerInput { action: Some(PlayerInputAction::TowerUpgrade(TowerUpgradeInput {
        tower_entity_id: target_eid,
        path: clicked_path,
        level: target_level,
    })) }
);
```

**Step 2: omb side — 沿用 comp::tower_upgrade_registry**

```rust
Some(PlayerInputEnum::TowerUpgrade(u)) => {
    log::info!("player_input_tick: pid={} tick={} TowerUpgrade eid={} path={} level={}",
        player_id, tick, u.tower_entity_id, u.path, u.level);
    if let Err(e) = crate::comp::tower_upgrade_registry::apply_upgrade(
        world, u.tower_entity_id, u.path as u8, u.level as u8, player_id
    ) {
        log::warn!("TowerUpgrade failed: {:?}", e);
    }
}
```

若 `tower_upgrade_registry::apply_upgrade` 不存在或簽章不同 → audit existing upgrade flow（grep `upgrade_levels` 寫入點）並抽出。

**Step 3: Verify** + smoke：升級塔一級 → tower stats 改變（攻速 / 攻擊力）。視覺驗證留 Phase 4c。

**Commit:** `phase2: TowerUpgrade input wired to tower_upgrade_registry`

---

### Task 2.4: ItemUse input wire

**Files:**
- Modify: `omfx/game/src/lib.rs` — hero hotbar slot click
- Modify: `omb/src/tick/player_input_tick.rs:160` — `ItemUse` arm

**Step 1: omfx side**

Hotbar slot 0..5 click：
```rust
self.lockstep_client.send_lockstep_input(
    PlayerInput { action: Some(PlayerInputAction::ItemUse(ItemUse {
        item_slot: slot_idx as u32,
        target_pos: target_pos.map(world_render_to_vec2i),
        target_entity: hovered_eid,
    })) }
);
```

**Step 2: omb side**

```rust
Some(PlayerInputEnum::ItemUse(i)) => {
    log::info!("player_input_tick: pid={} tick={} ItemUse slot={}",
        player_id, tick, i.item_slot);
    let target_pos = i.target_pos.map(|v|
        omoba_sim::Vec2::new(
            omoba_sim::Fixed64::from_raw(v.x as i64),
            omoba_sim::Fixed64::from_raw(v.y as i64),
        )
    );
    if let Err(e) = crate::comp::inventory::use_item(
        world, player_id, i.item_slot as u8, target_pos, i.target_entity
    ) {
        log::warn!("ItemUse failed: {:?}", e);
    }
}
```

`comp::inventory::use_item` 是現有 inventory 系統 entry point；若不存在 → 此 task 變兩階段：先 stub `use_item` 把 slot 從 inventory 移除（消耗）+ log，等 Phase 4d snapshot inventory ready 再回填邏輯。

**Step 3: Verify** + smoke：撿物品 → 點 hotbar → log + slot 消耗（視覺 Phase 4d 驗證）。

**Commit:** `phase2: ItemUse input wire (stub or full per inventory.rs availability)`

---

## Phase 3 — B-visible：粗 path / round / lives / hero stats

**估時：2-3 天 / 3 tasks**

### Task 3.1: 粗 zigzag path render style

**Files:**
- Modify: `omfx/game/src/render_bridge.rs::ensure_paths_drawn`

**Step 1: 找原渲染段**

Grep `ensure_paths_drawn` 內：
- 線寬 const（可能名 `PATH_LINE_THICKNESS` 或 inline literal `0.12`）
- 線色 const（grep `255, 200, 60`）
- Checkpoint marker dot 段（grep `build_circle` + `checkpoint`）

**Step 2: 改寬+變色**

```rust
const PATH_LINE_THICKNESS: f32 = 64.0 * crate::WORLD_SCALE * 2.0; // 1.28 render units
const PATH_COLOR: (u8, u8, u8, u8) = (170, 140, 90, 255);          // 奶油色

// 在原 build_line_segment 呼叫處用上面常數
```

**Step 3: 移除 checkpoint marker dot**

註解掉或刪除 corner 處 `build_circle(...)` block — 粗線本身會把 corner 蓋住。

**Step 4: Verify**
- `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p omfx`
- `run.bat` (TD_1) — path 應為粗奶油色 zigzag（diff with image 6 reference）

**Commit:** `phase3a: thick zigzag path render style`

---

### Task 3.2: Round / Lives / round_is_running 進 SimWorldSnapshot

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — `SimWorldSnapshot` 加欄位 + `extract_snapshot` 讀 resource
- Modify: `omfx/game/src/lib.rs` HUD 段 — 替換 `self.heartbeat.lives` / `self.current_round`

**Step 1: 加 snapshot 欄位**

```rust
#[derive(Default, Clone, Debug)]
pub struct SimWorldSnapshot {
    pub tick: u32,
    pub entities: Vec<EntityRenderData>,
    pub paths: Vec<Vec<(f32, f32)>>,
    pub removed_entity_ids: Vec<u32>,
    pub round: u32,
    pub total_rounds: u32,
    pub lives: i32,
    pub round_is_running: bool,
}
```

**Step 2: extract_snapshot 讀 resource**

```rust
let cw = world.read_resource::<omobab::comp::CurrentCreepWave>();
let pl = world.read_resource::<omobab::comp::PlayerLives>();
let round = cw.wave;
let total_rounds = cw.total;
let round_is_running = cw.is_running;
let lives = pl.0;
```

實際 resource type / field name 視 omb 端定義 — grep `CurrentCreepWave` 對齊。

**Step 3: HUD 換 source**

`omfx/game/src/lib.rs` HUD 區段 grep `heartbeat.lives` / `current_round`：
```rust
let snap = self.sim_runner.snapshot.lock().unwrap();
let lives = snap.lives;
let round = snap.round;
let total_rounds = snap.total_rounds;
let round_running = snap.round_is_running;
drop(snap);
// ...UI render with these
```

**Step 4: Verify**
- omoba-sim 全綠：`cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features` → 69 全綠（不該動 sim crate）
- omb 全綠
- `run.bat` (TD_1)：左上 HUD lives / round 隨遊戲進行更新

**Commit:** `phase3b: round/lives/running HUD reads from snapshot`

---

### Task 3.3: Hero stats panel — sim 端 aggregation + EntityRenderData.hero_ext

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — 加 `HeroStatsExt` struct + `EntityRenderData.hero_ext`
- Modify: `omfx/game/src/sim_runner.rs::extract_snapshot` — Hero kind 跑 aggregation
- Modify: `omfx/game/src/lib.rs` hero panel UI — read `snapshot.entities.find(...).hero_ext`

**Step 1: 定義 HeroStatsExt + BuffSnapshot**

`sim_runner.rs`：
```rust
#[derive(Clone, Debug, Default)]
pub struct BuffSnapshot {
    pub buff_id: String,
    pub remaining_secs: f32,    // -1.0 = toggle / infinite
    pub payload_json: String,
}

#[derive(Clone, Debug, Default)]
pub struct HeroStatsExt {
    pub armor: f32,
    pub magic_resist: f32,
    pub attack_damage: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub attack_speed_sec: f32,
    pub bullet_speed: f32,
    pub mana: f32,
    pub max_mana: f32,
    pub buffs: Vec<BuffSnapshot>,
}

#[derive(Clone, Debug, Default)]
pub struct EntityRenderData {
    // ... 既有欄位 ...
    pub hero_ext: Option<Box<HeroStatsExt>>,  // Box 避免 size bloat 普通 entity
}
```

**Step 2: extract_snapshot 對 Hero 跑 aggregation**

對每個 Hero kind entity：
```rust
let buff_store = world.read_resource::<omobab::ability_runtime::BuffStore>();
let stats = omobab::ability_runtime::UnitStats::from_refs(&buff_store, e, /*is_building*/ false);
let cprop = world.read_storage::<omobab::comp::CProperty>();
let tatk = world.read_storage::<omobab::comp::TAttack>();
let prop = cprop.get(e);
let atk = tatk.get(e);

let armor = prop.map(|p| stats.final_armor(p).to_f32_for_render()).unwrap_or(0.0);
let attack_damage = atk.map(|a| stats.final_atk_phys(a).to_f32_for_render()).unwrap_or(0.0);
let attack_range = atk.map(|a| stats.final_attack_range(a).to_f32_for_render()).unwrap_or(0.0);
let move_speed = prop.map(|p| stats.final_move_speed(p).to_f32_for_render()).unwrap_or(0.0);
let attack_speed_sec = atk.map(|a| stats.final_attack_interval(a).to_f32_for_render()).unwrap_or(0.0);
// ...

let buffs: Vec<BuffSnapshot> = buff_store.list(e).iter().map(|b| BuffSnapshot {
    buff_id: b.id.clone(),
    remaining_secs: if b.is_toggle { -1.0 } else { b.remaining.to_f32_for_render() },
    payload_json: serde_json::to_string(&b.payload).unwrap_or_default(),
}).collect();

let hero_ext = Some(Box::new(HeroStatsExt {
    armor, attack_damage, attack_range, move_speed, attack_speed_sec, ...,
    buffs,
}));
```

**確認** `UnitStats::from_refs` 簽章 + 對應 final_* method names（grep `omb/src/ability_runtime/unit_stats.rs`）。若簽章不同需對齊。

**關鍵 invariant：read-only！** `from_refs` 不能寫 ECS — 否則破壞 lockstep determinism（omfx side aggregation = 額外 read，sim ECS 狀態必須等 dispatcher tick 後才動）。Audit 一次。

**Step 3: 確保 omobab API public**

若 `BuffStore` / `UnitStats::from_refs` / `CProperty` / `TAttack` 是 pub(crate) → omb-side commit 改 pub。Re-export 到 `omobab::ability_runtime::*` 跟 `omobab::comp::*`（看 lib.rs 已 expose 哪些）。

**Step 4: hero panel UI 換 source**

`omfx/game/src/lib.rs` 找 hero panel render 段（grep `armor` 或 `attack_damage` UI label）：
```rust
let snap = self.sim_runner.snapshot.lock().unwrap();
let local_hero = snap.entities.iter().find(|e|
    e.kind == EntityKind::Hero && e.entity_id == self.local_hero_eid
);
if let Some(h) = local_hero {
    if let Some(ext) = h.hero_ext.as_deref() {
        // panel: armor=ext.armor, atk=ext.attack_damage, range=ext.attack_range, ...
        // buffs: 列 ext.buffs，倒數每 frame 自行扣（ext.remaining_secs - frame_dt）
        //   下次 snapshot 重置權威值
    }
}
```

**Step 5: Verify**
- omoba-sim 69 全綠
- omb 145 全綠
- `run.bat` (TD_1)：image 6 reference panel — armor 3.6 / atk 53 / asd 0.60s / range 900 / msd 350 應全對

**Commit:** `phase3c: hero stats panel from sim aggregation (HeroStatsExt)`

---

## Phase 4 — B-rest：regions / explosions / tower upgrade pips / inventory / abilities

**估時：2-3 天 / 5 tasks**

### Task 4.1: BlockedRegion polygons render

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — snapshot 加 `blocked_regions`
- Modify: `omfx/game/src/render_bridge.rs` — 用 `build_polygon_outline` + `build_circle_outline`

**Step 1: snapshot 欄位**
```rust
pub struct SimWorldSnapshot {
    // ...
    pub blocked_regions: Vec<Vec<(f32, f32)>>,
}
```

**Step 2: extract_snapshot 一次讀完**

`BlockedRegions` resource 是 static（map load 後不變）— 每個 snapshot clone 花費可忽略；TD_1 是空，不影響。
```rust
let regions = world.read_resource::<omobab::comp::BlockedRegions>();
let blocked_regions = regions.iter().map(|r| {
    r.points.iter().map(|p| (p.x.to_f32_for_render(), p.y.to_f32_for_render())).collect()
}).collect();
```

**Step 3: render_bridge 畫紅線輪廓 + 橘圓**

仿 `ensure_paths_drawn` pattern，用既有 `build_polygon_outline(...)` 紅線 + 圓區（若有 circle radius field）橘圓。

**Step 4: Verify**
- TD_DEBUG 場景跑（`STORY=DEBUG_1` 或類似有 BlockedRegion 的 scene）
- 紅線多邊形 + 橘圓 visible
- TD_1 不變（無 region）

**Commit:** `phase4a: BlockedRegion polygons via snapshot`

---

### Task 4.2: Active explosions — Outcome::Explosion + sim → render

**Files:**
- Modify: `omb/src/comp/outcome.rs` — 加 `Explosion` variant
- Modify: `omb/src/scripting/world_adapter.rs::make_game_explosion_script` callsite — push outcome 取代 mqtx send
- Modify: `omb/src/comp/game_processor.rs:413` 同上
- Modify: `omb/src/comp/game_processor.rs::process_outcomes` — 處理 Explosion outcome（sim ECS 端可能不需 side effect，純 render fx；那就只是放進 ExplosionFx queue resource）
- Modify: `omfx/game/src/sim_runner.rs` — `SimWorldSnapshot.explosions` + extract drains `ExplosionFx` resource
- Modify: `omfx/game/src/render_bridge.rs` — spawn ring node + 縮放 lifecycle

**Step 1: Outcome enum**

```rust
// outcome.rs
pub enum Outcome {
    // ... existing variants ...
    Explosion {
        pos: omoba_sim::Vec2,
        radius: omoba_sim::Fixed64,
        duration_ms: u32,
    },
}
```

**Step 2: outcome.rs 加 ExplosionFx resource**

```rust
pub struct ExplosionFxQueue {
    pub pending: Vec<ExplosionFx>,  // drained per snapshot extract
}

pub struct ExplosionFx {
    pub pos_x: f32,
    pub pos_y: f32,
    pub radius: f32,
    pub duration_ms: u32,
    pub spawn_tick: u32,
}
```

**Step 3: process_outcomes Explosion arm**

`game_processor.rs::process_outcomes`：
```rust
Outcome::Explosion { pos, radius, duration_ms } => {
    let mut q = ecs.write_resource::<ExplosionFxQueue>();
    q.pending.push(ExplosionFx {
        pos_x: pos.x.to_f32_for_render(),
        pos_y: pos.y.to_f32_for_render(),
        radius: radius.to_f32_for_render(),
        duration_ms,
        spawn_tick: current_tick,
    });
}
```

**Step 4: 改 emit 點 push outcome**

- `world_adapter.rs:589` `make_game_explosion_script` 區塊 → 換成 `next_outcomes.push(Outcome::Explosion { pos, radius, duration_ms })`
- `game_processor.rs:413` `make_game_explosion` → 同

**Step 5: snapshot extract**

```rust
pub struct SimWorldSnapshot {
    // ...
    pub explosions: Vec<ExplosionFx>,
}
// extract:
let mut q = world.write_resource::<ExplosionFxQueue>();
let explosions = std::mem::take(&mut q.pending);
```

**Step 6: render**

`render_bridge.rs` 對每個 `snapshot.explosions` entry → spawn 紅圈 scene node，每 frame 用 `(now_tick - spawn_tick) / duration` ratio scale + alpha 漸消，duration 結束 free node。

**Step 7: 砍 omb make_game_explosion (3 sites)**

確認 `make_game_explosion_script` + `make_game_explosion` 0 callers 後砍 fn 本身（同 Task 1.8）。

**Step 8: Verify**
- omb 全綠（145）
- omoba-sim 全綠（69 — Outcome 加 variant 是否影響 determinism hash？確認 sim crate 不依賴 outcome.rs；應該不依賴）
- `run.bat` 放 bomb tower 打 creep：紅圈爆炸 VFX 出現 + 漸消

**Commit:** `phase4b: Outcome::Explosion + ExplosionFxQueue + omfx ring render`

---

### Task 4.3: Tower upgrade level pips

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — `EntityRenderData.upgrade_levels: Option<[u8; 3]>`
- Modify: `omfx/game/src/sim_runner.rs::extract_snapshot` — Tower kind 讀 component
- Modify: `omfx/game/src/render_bridge.rs` — tower body 旁畫 3 條 pip

**Step 1: 加欄位**
```rust
pub struct EntityRenderData {
    // ...
    pub upgrade_levels: Option<[u8; 3]>,  // Tower only
}
```

**Step 2: extract**
```rust
if matches!(kind, EntityKind::Tower) {
    if let Some(t) = towers.get(e) {  // omobab::comp::Tower 或 ScriptUnitTag.upgrade_levels
        upgrade_levels = Some(t.upgrade_levels);
    }
}
```

**Step 3: render**

`render_bridge.rs` 對 Tower entity → 畫 3 個小 pip 列在 tower 旁 — 已升級 path 顯綠色，未升 path 顯灰。

**Step 4: Verify** + smoke：升級塔兩級 → 旁邊 2 個綠 pip。

**Commit:** `phase4c: tower upgrade level pips per entity`

---

### Task 4.4: Inventory slots in HeroStatsExt

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — `HeroStatsExt.inventory: [Option<u32>; 6]`
- Modify: `extract_snapshot` Hero arm — 讀 `Inventory` component
- Modify: `omfx/game/src/lib.rs` hero panel slots UI

**Step 1: HeroStatsExt 加 inventory + 擴 extract**

```rust
pub struct HeroStatsExt {
    // ...
    pub inventory: [Option<u32>; 6],  // item_id per slot
}
// extract (在 Hero arm)：
let invs = world.read_storage::<omobab::comp::Inventory>();
let inv = invs.get(e).map(|i| i.slots).unwrap_or([None; 6]);
hero_ext = Some(Box::new(HeroStatsExt { ..., inventory: inv }));
```

**Step 2: hero panel UI**

`lib.rs` hero panel inventory 6 slot grid → from `local_hero.hero_ext.as_deref().map(|e| e.inventory)`。Item icon 用既有 `item_id → texture` registry。

**Step 3: Verify** + smoke：撿物品 → slot 顯示 icon。

**Commit:** `phase4d: hero inventory in snapshot`

---

### Task 4.5: AbilityRegistry + per-hero ability levels

**Files:**
- Modify: `omfx/game/src/sim_runner.rs` — `SimWorldSnapshot.abilities: Arc<Vec<AbilityDefSnapshot>>` + `HeroStatsExt.ability_levels: [i32; 4]`
- Modify: `extract_snapshot` — abilities clone Arc（O(1)） + Hero ability levels 讀 component
- Modify: `omfx/game/src/lib.rs` ability bar UI — Q/W/E/R 行 from snapshot

**Step 1: AbilityDefSnapshot 結構**

```rust
#[derive(Clone, Debug)]
pub struct AbilityDefSnapshot {
    pub ability_id: String,
    pub display_name: String,
    pub max_level: u8,
    pub icon_path: String,
    // ...其他 UI 需要的 metadata
}
```

**Step 2: snapshot 欄位**

```rust
pub struct SimWorldSnapshot {
    // ...
    pub abilities: Arc<Vec<AbilityDefSnapshot>>,  // static-ish reference
}
```

extract：第一次 snapshot 時 build 一次（在 worker init），之後每 snapshot `.clone()` Arc 是 O(1)。

```rust
// worker init:
let abilities_arc = Arc::new(extract_ability_defs(&world));

// each tick:
let snap = SimWorldSnapshot {
    // ...
    abilities: abilities_arc.clone(),
};
```

**Step 3: HeroStatsExt 加 ability_levels**

```rust
pub struct HeroStatsExt {
    // ...
    pub ability_levels: [i32; 4],  // Q/W/E/R current levels
}
// extract：從 Hero component .ability_levels 讀
```

**Step 4: ability bar UI**

`lib.rs` 找 hero ability row — Q W E R 4 slot：
```rust
for i in 0..4 {
    let lvl = ext.ability_levels[i];
    let def = self.snapshot.abilities.get(local_hero.ability_def_indices[i]).cloned();
    // render: icon + "lvl/max" label
}
```

實際 ability_def index ↔ slot 對應：可能 `Hero` component 有 `ability_ids: [String; 4]` → render 端在 `snapshot.abilities` lookup。

**Step 5: Verify** + smoke：TD_1 hero level up → 點 Q 升級 → ability bar Q "0/4 → 1/4"。

**Commit:** `phase4e: AbilityRegistry + per-hero ability levels in snapshot`

---

## Final Task: 全 phase verify gates

**Files:** None (verification only)

**Step 1: omb lib tests**
```
cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: 145 全綠

**Step 2: omoba-sim determinism tests**
```
cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features
```
Expected: 69 全綠（含 8 determinism pin hashes — 若 Phase 4b Outcome 影響 hash 重 baseline，commit 訊息註明）

**Step 3: omfx tests**
```
cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx --lib
```

**Step 4: TD_1 60s smoke**
```powershell
$p = 'D:/omoba/omb/game.toml'
(Get-Content -Raw $p) -replace 'STORY\s*=\s*"TD_STRESS"', 'STORY = "TD_1"' | Set-Content -Encoding utf8 $p
```
```
cd /d D:/omoba && run_smoke_long.bat
```
- omfx_app.log: `grep -c "no TickBatch in 1.0s"` → 0
- 視覺：放塔 / 升塔 / 賣塔 / 撿物品 / Q 升級 全 work
- HUD：粗 path / lives / round / hero stats panel / inventory / ability bar 全顯示
- TD_1 整局走完到 game end

**Step 5: TD_STRESS 60s smoke**
```powershell
(Get-Content -Raw 'D:/omoba/omb/game.toml') -replace 'STORY\s*=\s*"TD_1"', 'STORY = "TD_STRESS"' | Set-Content -Encoding utf8 'D:/omoba/omb/game.toml'
```
```
cd /d D:/omoba && run_smoke_long.bat
```
- omb_app.log: `grep -c "Removed disconnected KCP session"` → 0
- omb_app.log: `grep "kcp-p7 .* bytes_per_sec"` → 持續 < 5000
- 無 freeze / panic

**Step 6: 復原 TD_1**
```powershell
(Get-Content -Raw 'D:/omoba/omb/game.toml') -replace 'STORY\s*=\s*"TD_STRESS"', 'STORY = "TD_1"' | Set-Content -Encoding utf8 'D:/omoba/omb/game.toml'
```

**No commit** — 純 gate。全綠後 close plan。

---

## Critical Files Reference

| File | 主要動作 |
|---|---|
| `omb/src/tick/{creep,hero,tower,buff,regen,hero_move,summon}_tick.rs` | Phase 1: delete `tx.try_send(make_*)` lines |
| `omb/src/comp/game_processor.rs` | Phase 1: delete cut-list sites; Phase 2: handle_tower_spawn / handle_tower_sell |
| `omb/src/comp/outcome.rs` | Phase 4b: 加 `Outcome::Explosion` |
| `omb/src/scripting/world_adapter.rs` | Phase 1: delete script-side legacy emit; Phase 4b: explosion outcome |
| `omb/src/tick/player_input_tick.rs` | Phase 2: 4 個 PlayerInputEnum match arm 從 trace stub 改實作 |
| `omfx/game/src/sim_runner.rs` | Phase 1b: removed_entity_ids; Phase 3+4: snapshot 欄位擴展（HeroStatsExt / blocked_regions / explosions / abilities / inventory） |
| `omfx/game/src/render_bridge.rs` | Phase 3a: path style; Phase 4: pip / 爆炸 / region 線 |
| `omfx/game/src/lib.rs` | Phase 2: 4 個 click handler; Phase 3-4: HUD 從 snapshot 讀 |

## 既有可重用的 utilities

- `omb/src/state/resource_management.rs::build_hero_stats_payload` — Phase 3c 的 reference aggregation logic（要在 omfx side 重寫一份相同邏輯）
- `omfx/game/src/lib.rs::build_line_segment / build_circle_outline / build_polygon_outline` — Phase 3a / 4a 路徑與多邊形 render 用
- `omoba-sim::Vec2 / Fixed64::to_f32_for_render` — boundary conversion
- `omb/src/ability_runtime/{buff_store, unit_stats}` — Phase 3c hero stats aggregation logic（已 pub，可從 omfx 端 import）
- `omb/src/comp/tower_upgrade_registry` — Phase 2.3 升級邏輯入口

## Phase 5 (Observer) 未來方向（不在本 plan 範圍）

若未來需做：選 Path Y — `specs::SerializeComponents` 完整 World serialize，所有 component 加 `#[derive(Serialize, Deserialize)]`。Schema 工作量大但 rejoin O(1)。


---

## Audit 註記 (2026-05-04 by openspec change `lockstep-cleanup-and-hud`)

Apply 期間做了完整 audit，發現本 plan 的 Phase 2-4 大部分項目實際上在 plan 寫成前後就已經由 Phase 4.2 / 4.5 提早實作完成（snapshot.entities.upgrade_levels / HeroStatsExt 12 欄 / abilities Arc / tower_templates Arc / blocked_regions / explosions drain 全已落地）。所以 OpenSpec change 的實作工作集中在 Phase 1（cut legacy emit + Outcome::EntityRemoved + grep guard test + dead-code cleanup），Phase 2-4 純 verify。

Phase 1 落地調整：
- 原 plan 寫 `push_hero_stats:920-992` 已是 Phase 5.2 的 no-op stub，實際 active broadcast 是 `broadcast_hero_update:853-921`，本 change 處理後者並一併砍 push_hero_stats / push_hero_static stub。
- Entity 死亡走 `Outcome::EntityRemoved` 唯一通道（process_outcomes 是唯一 entities().delete() sink），由新增的 `omb/tests/delete_entity_outcome_only.rs` grep guard 守護。
- `RemovedEntitiesQueue` resource drain 取代原 plan 的 `prev_alive: HashSet` 跨 tick state diff。
- omfx render 端對 `snapshot.removed_entity_ids` 釋放 per-eid cache。
- Dead code cleanup 砍掉 16 個 TypedOutbound dead variant + 對應 kcp_transport routing entry + 7 個 proto_build builder + 3 個 top-level builder + broadcast_hero_update fn 殼 + 7 個 caller。
