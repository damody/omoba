## ADDED Requirements

### Requirement: omb 端 legacy emit 禁播清單

omb 端 ECS tick / handler 系統 SHALL 不得對下列 `TypedOutbound` variant 或對應 `make_*` builder 呼叫 `mqtx.send` / `try_send`。所有渲染狀態變化 MUST 由 omfx 端的 lockstep sim 在自己的 ECS World 推算後 extract 進 `SimWorldSnapshot`。

**Phase 4.2 已遷移完成（不應再有呼叫點）** — 本 spec 確認其禁播狀態：
- `make_entity_facing` / `EntityFacing`（facing 角度）
- `make_creep_stall` / `CreepStall`（creep 路徑被擋停）
- `make_creep_slow` / `CreepSlow`（creep 被 slow buff）
- `make_creep_create` / `CreepCreate`（creep spawn）
- `CreepMove` / `CreepHp`
- `make_hp_update` / `make_hp_update_at`（HP 變動 — DoT / regen / 一般傷害）
- `make_projectile_create` / `make_projectile_create_script` / `ProjectileCreate` / `ProjectileDestroy`
- `UnitCreate` / `tower.C` / `unit.C` payload
- entity `Miss` payload
- `make_game_explosion` / `make_game_explosion_script` / `GameExplosion`（已改走 `Outcome::Explosion` → `ExplosionFxQueue` → `snapshot.explosions`）

**本 change 仍要砍**（~7 個 active emit 點）：
- `TypedOutbound::EntityDeath` — 砍 `state/resource_management.rs:704-720`（sell tower）跟 `comp/outcome_system/combat_events.rs:300-319`（combat death）— 改走 `Outcome::EntityRemoved` → `RemovedEntitiesQueue` → `snapshot.removed_entity_ids`
- `TypedOutbound::TowerCreate` — 砍 `state/resource_management.rs:339-351`；snapshot.entities 自然包含
- `TypedOutbound::TowerUpgrade` — 砍 `state/resource_management.rs:594-604`；改走 `snapshot.entities[].upgrade_levels`
- `TypedOutbound::GameRound` — 砍 `state/resource_management.rs:394-404`；改走 `snapshot.round` / `total_rounds` / `round_is_running`
- `TypedOutbound::HeroStatic` / `HeroHot` 跟相關 hero payload — 砍 `state/resource_management.rs::broadcast_hero_update` 函式體（約 :853-:921，含 static_msg / hot_msg / non-kcp hero.stats fallback / hero.inventory），跟 builder fn `build_hero_static_msg` / `build_hero_hot_msg` / `build_hero_stats_payload`；改走 `snapshot.entities[].hero_ext: HeroStatsExt`。注意：原 plan 寫 `push_hero_stats:920-992` 已是 Phase 5.2 留下的 no-op stub；本 change 處理的是真正 active broadcast 的 `broadcast_hero_update`

#### Scenario: TD_STRESS 60 秒壓測 wire 流量低於 5 KB/s

- **WHEN** 跑 `run_smoke_long.bat` 在 `STORY = "TD_STRESS"`（1000 塔 × 1000 creep）持續 60 秒
- **THEN** `omb_app.log` 內 `kcp-p7 .* bytes_per_sec` 抽樣最後 10 筆持續 < 5000 bytes_per_sec
- **AND** `grep -c "Removed disconnected KCP session" omb_app.log` 為 0
- **AND** `grep -c "no TickBatch in 1.0s" omfx_app.log` 為 0

#### Scenario: Phase 1 commit 後 omb 全域 grep 不到任何禁播 TypedOutbound

- **WHEN** 在 `omb/src/` 內 grep `TypedOutbound::EntityDeath\|TypedOutbound::TowerCreate\|TypedOutbound::TowerUpgrade\|TypedOutbound::GameRound\|TypedOutbound::HeroStatic\|TypedOutbound::HeroHot`
- **THEN** 沒有任何 `OutboundMsg::new_typed*(...)` 構造點包含這些 variant 作為 payload
- **AND** `omb/src/transport/kcp_transport.rs:551-571` 的 routing 表內這些 variant 已隨 enum 一起砍
- **AND** 對應 builder fn（`proto_build::entity_death*` / `tower_create` / `tower_upgrade` / `game_round` / `hero_static` / `hero_hot` 跟 `build_hero_static_msg` / `build_hero_hot_msg` / `build_hero_stats_payload`）已 0 callers 並刪除

#### Scenario: omb lib tests 全綠

- **WHEN** 跑 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`
- **THEN** 145 個 lib test 全綠

### Requirement: 保留的 HUD broadcast 白名單

omb 端 SHALL 保留下列 broadcast emit，因為它們屬於玩家操作的 ack 路徑或 one-shot 終局事件：

- `TypedOutbound::GameLives` / `make_game_lives`（漏怪扣命 — `omb/src/comp/game_processor.rs:621`）
- `TypedOutbound::GameEnd` / `make_game_end` ×3（一局結束 — `:54`、`:276`、`:624`）

**已遷移走 outcome（不在 broadcast 白名單）**：`Outcome::Explosion` → `ExplosionFxQueue` → `snapshot.explosions` 已是現況（Phase 4.2 完成），不需要 broadcast。

#### Scenario: 漏怪 lives 變動 omfx 立即收到

- **WHEN** TD_1 任一 creep 走完路 reach base
- **THEN** omb 廣播 `game.lives` event
- **AND** omfx HUD 左上 lives 數字立即減 1

#### Scenario: 一局結束 omfx 收到 GameEnd

- **WHEN** TD_1 整局走完 / lives 歸零
- **THEN** omb 廣播 `game.end` event
- **AND** omfx 顯示 game end overlay

### Requirement: Entity 死亡走 `Outcome::EntityRemoved` 唯一通道

`omb/src/comp/outcome.rs` SHALL 加入 `Outcome::EntityRemoved { entity: Entity }` variant 跟 `RemovedEntitiesQueue { pending: Vec<u32> }` resource。`process_outcomes` 是**唯一**呼叫 `entities().delete()` 的位置，arm body 同時做 `q.pending.push(entity.id())` 跟 `entities().delete(entity)`。

所有需要 delete entity 的 site SHALL push `Outcome::EntityRemoved { entity: target_entity }` 進 World 的 `Vec<Outcome>` resource，**不得**直接呼叫 `entities().delete()` / `world.delete_entity()`。`drain_pending_*` 跑在 `process_outcomes` 之前（`state/core.rs:341-385`），所以同 tick 內 push outcome 後立刻被處理。Script boundary（abi_stable）的 `WorldAdapter::despawn` SHALL 透過 `cache.outcomes.push(...)` 走同通道。

`extract_snapshot` SHALL 用 `std::mem::take(&mut q.pending)` 把 `RemovedEntitiesQueue` drain 進 `SimWorldSnapshot.removed_entity_ids: Vec<u32>`。omfx render 端 SHALL 對該 list 釋放 per-eid cache（`scene_nodes_by_eid` / `labels` / `collision_rings`）。omb 端兩個 `EntityDeath` 廣播（`state/resource_management.rs:704-720` sell tower 跟 `comp/outcome_system/combat_events.rs:300-319` combat death）SHALL 整段刪除（已於 1.1 完成）。

#### Scenario: creep 死亡時 omfx 自動移除 scene node

- **WHEN** TD_1 任一 creep 在 sim 內死亡（damage system push `Outcome::Death`，handle_death 內 push `Outcome::EntityRemoved`）
- **THEN** 該 tick 的 process_outcomes 處理該 outcome 時呼叫 `entities().delete()` + push `RemovedEntitiesQueue.pending`
- **AND** 下一 snapshot 的 `removed_entity_ids` 包含該 creep 的 entity_id
- **AND** queue 在 drain 後 `pending` 為空
- **AND** omfx render 端對該 entity_id 釋放 `scene_nodes_by_eid` / `labels` / `collision_rings` 對應 entry
- **AND** omb 端**不**廣播 `EntityDeath` event

#### Scenario: 塔被拆除時 omfx 自動移除

- **WHEN** TD_1 玩家 sell 一個塔（`handle_tower_sell_from_input` push `Outcome::EntityRemoved { entity: target_entity }`）
- **THEN** 同 tick 內 process_outcomes 處理 outcome 並 delete + push queue
- **AND** 下一 snapshot 的 `removed_entity_ids` 包含該塔 entity_id
- **AND** 塔的 scene node + label + collision ring 從畫面消失

#### Scenario: process_outcomes 是唯一 delete sink

- **WHEN** 跑 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --test delete_entity_outcome_only`
- **THEN** test 通過 — `omb/src/` 內所有 `.rs` 檔（除 `comp/game_processor.rs` 的 process_outcomes 之外）grep 不到 `entities().delete(` 或 `.delete_entity(` 呼叫

### Requirement: 爆炸 VFX 走 `Outcome::Explosion` + drainable resource（現況確認）

`omb/src/comp/outcome.rs` SHALL 維持 `Outcome::Explosion { pos: omoba_sim::Vec2, radius: omoba_sim::Fixed64, duration: omoba_sim::Fixed64 }` variant 跟 `ExplosionFxQueue { pending: Vec<ExplosionFx> }` resource（Phase 4.2 已存在，本 change 不重做）。`game_processor.rs::process_outcomes` 的 Explosion arm SHALL push `ExplosionFx { pos_x, pos_y, radius, duration_ms, spawn_tick }` 進 queue。`world_adapter.rs::emit_explosion` SHALL 經 `self.cache.explosion_fx.pending.push(...)` 走同 queue（不走 mqtx）。`extract_snapshot` SHALL 用 `std::mem::take(&mut q.pending)` 把整批拉到 `SimWorldSnapshot.explosions` 並清空 queue。omfx render 端 SHALL 對每筆 spawn 紅圈 scene node，per-frame `(now_tick - spawn_tick) / duration_ms` ratio scale + alpha 漸消，duration 結束釋放 node。

本 requirement 為 Outcome pattern 的 reference 實作，後續 `Outcome::EntityRemoved` 採同 pattern。

#### Scenario: bomb tower 打中 creep 觸發紅圈漸消 VFX

- **WHEN** 放一座 bomb tower 在攻擊範圍內，creep 進入觸發 explosion outcome
- **THEN** 下一 snapshot 的 `explosions` 包含該爆炸 entry（`pos`, `radius`, `duration_ms`, `spawn_tick`）
- **AND** omfx render 端 spawn 紅圈，size 從 0 漸增到 `radius`、alpha 從 1.0 漸消到 0
- **AND** `duration_ms` 結束後紅圈 scene node 被釋放
- **AND** omb 端**不**廣播 `GameExplosion` event

#### Scenario: 既有實作對齊現況

- **WHEN** 在 `omb/src/` 全域 grep `Outcome::Explosion` / `ExplosionFxQueue` / `make_game_explosion`
- **THEN** `Outcome::Explosion` variant 在 `comp/outcome.rs` 定義
- **AND** `ExplosionFxQueue` resource 在 `comp/outcome.rs:174` 附近定義
- **AND** `make_game_explosion` / `make_game_explosion_script` 不存在於 codebase（除註解外）
- **AND** `comp/game_processor.rs::process_outcomes` 的 Explosion arm 推 queue
- **AND** `scripting/world_adapter.rs::emit_explosion` 推 queue 不送 mqtx
