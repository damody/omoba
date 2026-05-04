## Context

omb / omfx 之間在 Phase 0–3 之前是 **per-event broadcast 模型**：omb 每個 ECS tick 用 `mqtx.send(...)` / `tx.try_send(...)` 把 hp_update / facing / projectile_create / creep_create / entity_death / hero.stats / explosion / slow / stall 等事件序列化發給 omfx；omfx 端維護一份 mirror state 套 event 還原渲染。Phase 0–3 引入 **server-paced lockstep**（plan 文件：`docs/plans/2026-05-02-server-paced-lockstep-design.md`）：omb 改廣播 `TickBatch{tick, [PlayerInput]}`，omfx 端內嵌 `omoba_sim` 跑一份**完全 deterministic** 的 ECS World（fixed-point `Fixed64` + `Vec2`），所有渲染狀態從 sim World extract 成 `SimWorldSnapshot`。

spec 期間做完逐項 audit 後揭露 — 原 plan（`docs/plans/2026-05-04-lockstep-cleanup-and-hud.md`）寫於 Phase 4.2 那波遷移之前，**絕大多數工作已被該波遷移順手做完**，但 plan 文件跟 `graphify-out/GRAPH_REPORT.md` 都沒同步：

- **Phase 1 (24 emit)**：~17 個已砍（`make_entity_facing` / `make_creep_stall` / `make_creep_slow` / `make_creep_create` / `make_hp_update` / `make_projectile_create` / `make_game_explosion` 全系列）；剩餘 **~7 個 active**：`EntityDeath` ×2、`TowerCreate`、`TowerUpgrade`、`GameRound`、`HeroStatic` + `HeroHot` + `push_hero_stats` 內部 broadcast。雖然 wire 上 omfx 已不消費這些（snapshot 涵蓋了），但 send 動作仍在，純死信浪費。
- **Phase 2 (4 PlayerInput stub)**：**全部已實作**。`omfx/game/src/lib.rs` 4 個 click handler（TowerSell `:3160` / TowerUpgrade `:3185` / TowerPlace `:3234` / ItemUse `:3478`）；`omb/src/tick/player_input_tick.rs` 4 個 PendingQueue push 模式；`GameProcessor` 8 個 pub fn (`handle_*_from_input` ×4 + `drain_pending_*` ×4)；採用比 plan 預想更精緻的 `PendingXxxQueue` resource defer 模式避開 specs `System` borrow `&mut World` 限制。
- **Phase 3 / 4 (13 HUD 元素)**：**全部已實作**。`SimWorldSnapshot` 已含 `removed_entity_ids` / `round` / `total_rounds` / `lives` / `round_is_running` / `blocked_regions` / `explosions` / `abilities: Arc<...>` / 額外彩蛋 `tower_templates: Arc<...>`；`EntityRenderData` 已含 `upgrade_levels` + `hero_ext: Box<HeroStatsExt>`；`HeroStatsExt` 全 12 欄位（含 `inventory: [Option<String>; 6]` + `ability_levels: [i32; 4]` + `ability_ids: [Option<String>; 4]`）；`render_bridge.rs` `PATH_LINE_THICKNESS` + `PATH_COLOR` + `REGION_LINE_THICKNESS` const 已對齊 spec 數字。

**真實剩餘工作只有兩件**：(1) 砍 ~7 個 active emit；(2) `removed_entity_ids` 演算法**重構** — 目前 working code 是 `prev_alive: HashSet<u32>` 跨 tick state diff（`omfx/game/src/sim_runner.rs:383, 562, 630, 949-951`），用戶決定改成 `Outcome::EntityRemoved` 通道跟 Explosion 統一 pattern，並用 `delete_entity_tracked` helper 強制配對 outcome push。這是**重構**而非新增，本 change 不會引入新功能。

本 change 的範圍是把這三件事一次收掉，達成「omb 不再 per-event broadcast，所有渲染狀態走 lockstep sim 從 omfx side 跑出來」的單一資料流，並把 13 個 HUD 元素全部接回新的 snapshot 來源。

## Goals / Non-Goals

**Goals:**

- 砍掉剩餘 ~7 個 omb legacy `TypedOutbound` emit（EntityDeath ×2 / TowerCreate / TowerUpgrade / GameRound / HeroStatic + HeroHot 一系列）；保留 4 個 HUD broadcast（`GameLives` ×1 + `GameEnd` ×3）
- `removed_entity_ids` **重構**：從現有 `prev_alive: HashSet<u32>` diff 改成 `Outcome::EntityRemoved` 通道，跟 Explosion 統一 pipeline pattern
- 加 `delete_entity_tracked` helper + grep guard test，所有 `world.delete_entity(e)` site 強制走 helper 配對 outcome push
- Verify Phase 2-4 已實作項目（13 HUD 元素 + 4 PlayerInput）皆 working — TD_1 / TD_STRESS smoke 全綠
- TD_STRESS 60s smoke wire 流量 < 5 KB/s 持續穩定，無 KCP session 斷線、無 reader-task deadlock
- omb 145 lib tests / omoba-sim 69 tests / omfx lib tests 全綠
- `graphify update .` 同步 graph + plan 文件補 audit 註記

**Non-Goals:**

- **Observer / rejoin mode** — 基礎建設已部分完成（KCP 0x13–0x16 wire 框架 / 30s 週期 producer / `WorldSnapshot` 含 9 個基礎 component），但 omfx-side `apply_snapshot` 還在 log-only 階段（`omfx/game/src/lockstep_client.rs:257`），且 `WorldSnapshot` schema 未涵蓋本 change 加進 sim 的擴展狀態（buff_store / ability_levels / inventory / upgrade_levels）— 完整 observer 留待獨立 change 處理
- 不改 wire 協定本體（KCP tag 編碼、`PlayerInput` proto schema 不動）
- 不改 sim crate 內部 ECS 邏輯（`omoba_sim` determinism 保證不動）
- 不引入 client-side prediction / rollback（保持單一 lockstep 流，渲染只 lag `render_delay_ms`）
- 不做向下相容 — 客戶端與 server 同步發行；舊 omfx 與新 omb 不相容是預期行為
- 不重寫 `proto/game.proto`（4 個 PlayerInput variant 已存在）
- 不調整 KCP 傳輸層 framing 或 backpressure 機制（Phase 1 砍 emit 後流量自然降到不會壓爆）

## Decisions

### Decision 1: Entity 死亡走 `Outcome::EntityRemoved` 唯一通道

**選擇：** `omb/src/comp/outcome.rs` 加 `Outcome::EntityRemoved { entity: Entity }` variant 跟 `RemovedEntitiesQueue { pending: Vec<u32> }` resource。entity 帶 generation 是必要的（specs reuses indices；只記 u32 id 不夠）。

`process_outcomes` 是**唯一**呼叫 `entities().delete()` 的位置，arm body：

```rust
Outcome::EntityRemoved { entity } => {
    let mut q = ecs.write_resource::<RemovedEntitiesQueue>();
    q.pending.push(entity.id());
    let _ = ecs.entities().delete(entity);
}
```

所有現有 raw delete site SHALL 改 push outcome：
- `comp/game_processor.rs::handle_tower_sell_from_input` 用 `world.write_resource::<Vec<Outcome>>().push(Outcome::EntityRemoved { entity: target_entity })`
- `state/resource_management.rs::sell_tower` legacy 路徑同上
- `scripting/world_adapter.rs::despawn`（abi_stable 邊界）走 `cache.outcomes.push(...)` — `WorldAdapterCache` 加 `outcomes: Write<Vec<Outcome>>` 欄位

`extract_snapshot` 用 `std::mem::take(&mut q.pending)` drain 進 `snapshot.removed_entity_ids`，render 端對該 list 釋放 `scene_nodes_by_eid` / `labels` / `collision_rings`。omb 端兩個 `EntityDeath` 廣播（`state/resource_management.rs:704-720` 跟 `comp/outcome_system/combat_events.rs:300-319`）整段刪除（已於 1.1 完成）。

**Tick-order 保證：** `drain_pending_tower_sells` / `drain_pending_*` 跑在 `process_outcomes` 之前（看 `state/core.rs:341-385`）— 同 tick 內 push outcome 後立刻被 process_outcomes 處理。Server 跟 client 都跑同一個 dispatch sequence，兩端 process_outcomes 在邏輯相同的 tick 點刪掉 entity，`world.maintain()` 在 dispatcher tick 結尾跑時兩邊看到的 alive set 一致 → `StateHash`（每 600 tick 廣播）自然對齊。

**為什麼不用 helper：** 早期 design 提過 `delete_entity_tracked(world, e)` helper（先 push queue 再 `entities().delete()`）— 後來改成單一 outcome 通道。理由：(1) 一致性 — 跟 `Outcome::Death` / `Outcome::Explosion` 同 pattern；(2) script boundary（abi_stable）沒 `&mut World`，本來就只能 push outcome，helper 路徑無法統一；(3) 砍 helper 程式碼更精簡，呼叫端只需 push outcome。

**替代方案：**
- (A) 留 `entity_death` event 不改：vs 「sim 跑」的單一資料流原則衝突；wire 上多 1–N 個 event/tick；TD_STRESS 死亡密集時又是流量瓶頸。
- (B) `prev_alive: HashSet<u32>` diff（被動推算，曾是初版設計）：自動推算不會漏 delete site，但 sim worker thread 要跨 tick 維護 stateful HashSet，而且每 tick 都得收完整 alive set 做 set difference，TD_STRESS 1000 entity 仍 < 100µs 但開銷比增量 push 大；演算法分散在 worker loop 跟 render cleanup 兩處，policy 不顯式。
- (C) `delete_entity_tracked` helper（中間版本）：跟 Outcome 重複能力；script boundary 仍要走 outcome，雙路 producer 不必要。

**理由：** Outcome 唯一通道是最小 surface area + 最一致 — 沒有 helper、沒有跨 tick 演算法。grep guard test 強制只有 `process_outcomes` 能呼叫 `entities().delete()`。

### Decision 2: 爆炸 VFX 已走 `Outcome::Explosion` + drainable resource（現況確認，非新增）

**現況：** 本 change 開工前 audit 發現 Phase 4.2 已完成此 pipeline：`omb/src/comp/outcome.rs:161-176` 定義 `Outcome::Explosion` + `ExplosionFxQueue`、`game_processor.rs:187-207` 處理 outcome arm、`world_adapter.rs:489` 走 queue 不走 mqtx、`omfx/game/src/sim_runner.rs:996` drain queue、`omfx/game/src/lib.rs:1590-1593` render consumer 完整。本 change 視為 reference 實作而非新增工作。

**Decision 1 (`Outcome::EntityRemoved`) 採同 pipeline pattern**：variant → queue resource → `mem::take` drain → snapshot field → render cleanup。差異只在 render 動作（Explosion 是 spawn 紅圈漸消、EntityRemoved 是釋放 per-eid cache）。

**替代方案：**
- (A) 直接在 ECS 加 `ExplosionFx` component + entity：違反 sim/render 分離，sim crate 不應該為了 render fx 增加 ECS entity（影響 determinism hash）。
- (B) 留 `make_game_explosion` event：跟 Decision 1 同理，不走單一資料流。

**理由：** Outcome queue 已是 omb 處理「sim 推算出的事件」的標準通道（Phase 4 加 Death / GoldDrop），Explosion 走同 pipeline；resource drain 比 component-on-entity 開銷低，且 lifecycle 由 render 端 ratio 控制不需 sim 端管 frame-accurate 結束時間。

### Decision 3: HeroStatsExt aggregation 在 omfx side 跑，read-only 跨 process API

**選擇：** `omfx/game/src/sim_runner.rs::extract_snapshot` 對 Hero kind entity 呼叫 `omobab::ability_runtime::UnitStats::from_refs(&buff_store, e, false)`，再對每個 `final_*` method 算出實際 stat（armor / atk / range / msd / asd / mana），寫進 `EntityRenderData.hero_ext: Option<Box<HeroStatsExt>>`。要求：(1) `UnitStats::from_refs` **嚴禁寫 ECS** — 否則破壞 lockstep determinism；(2) `BuffStore` / `UnitStats::from_refs` / `CProperty` / `TAttack` 改 `pub` 並 re-export 到 `omobab::ability_runtime::*` / `omobab::comp::*`；(3) buff `remaining` field：`-1.0` 代表 toggle / 無限期，否則為剩餘秒數，render side per-frame 自行扣 `frame_dt`，下次 snapshot 重設權威值避免漂移（同 Phase 0 的 hero_stats broadcast 設計）。

**替代方案：**
- (A) omb 端繼續廣播 hero.stats payload（既有 `build_hero_stats_payload` 0.3s 一次）：sim/render 分離原則破功，hero stats 反而是最容易在 omfx 端算的（buff_store / property / attack 全在 sim ECS）。
- (B) 把 aggregation 抽進 sim crate：`UnitStats::from_refs` 跟 `BuffStore` 是 omb-side ability_runtime 的東西，不在 sim 邊界內。

**理由：** Phase 0 的 hero_stats broadcast 已驗證 aggregation logic 正確，本 change 把同一個函式從 omb side（每 0.3s 廣播）搬到 omfx side（每 snapshot tick 跑），避免 wire 上多送一份 payload；read-only 限制是 lockstep 的硬約束，aggregation 純讀 component 不會破壞此邊界。

### Decision 4: AbilityRegistry 用 `Arc<Vec<AbilityDefSnapshot>>`，per-snapshot O(1) clone

**選擇：** `extract_snapshot` worker init 時跑一次 `extract_ability_defs(&world)` 建出 `Arc<Vec<AbilityDefSnapshot>>`，之後每 snapshot 直接 `.clone()` Arc 不複製 inner data；`HeroStatsExt.ability_levels: [i32; 4]` 給每個 hero 自己的 Q W E R 等級。

**替代方案：**
- (A) 每 snapshot 重 build：snapshot 60Hz × 4 ability × 多 hero 反覆 String clone 浪費。
- (B) ability defs 寫 globals（`OnceCell<Vec<AbilityDefSnapshot>>`）：global state 跨 game session 不乾淨，重連 / 換 scene 後需 reset。

**理由：** ability defs 是 static 資料（map load 後不變），Arc 共享是 zero-cost 抽象的標準作法；ability_levels 是 per-hero 動態狀態，跟 `hero_ext` 同陣營。

### Decision 5: 4 個 PlayerInput 端到端 wire — 抽出 `GameProcessor::handle_*` public API

**選擇：** omb 端把現有 tower spawn / tower sell / inventory use_item / tower upgrade 流程抽出成 `pub fn`：
- `comp::GameProcessor::handle_tower_spawn(world, kind_id, pos: omoba_sim::Vec2, owner_pid: u32) -> Result<Entity, _>`
- `comp::GameProcessor::handle_tower_sell(world, entity_id: u32, owner_pid: u32) -> Result<(), _>`
- `comp::tower_upgrade_registry::apply_upgrade(world, entity_id, path: u8, level: u8, owner_pid: u32) -> Result<(), _>`
- `comp::inventory::use_item(world, pid, slot: u8, target_pos: Option<Vec2>, target_entity: Option<u32>) -> Result<(), _>`

`player_input_tick` 4 個 stub arm 直接呼叫，失敗 `log::warn!` 後丟（不回 ack，玩家透過 snapshot diff 看到結果）。omfx side click handler 把 `mouse_world_pos` / `selected_tower_kind_id` / `target_eid` 等 UI state 包進對應 `PlayerInputAction` variant 並 `send_lockstep_input(...)`。

**替代方案：**
- (A) 在 `player_input_tick` 內 inline 寫 spawn 邏輯：duplication，現有 spawn 路徑（`ScriptEvent::Spawn` / world_adapter）已有完整流程，重寫風險高。
- (B) 加 ack event 從 omb 回 omfx：跟 Decision 1 同理，違反單一資料流；玩家的視覺 feedback 從 snapshot 來（塔出現、gold 變動）。

**理由：** 抽 public API 是最小破壞 — 既有 spawn / sell / upgrade / inventory 邏輯不動，只是把 entry point 從 internal 改成 pub；ownership 檢查 / refund 規則 / template lookup 全在 handler 內封裝。

### Decision 6: 保留 `make_game_lives` / `make_game_end` 兩條 broadcast 不動

**選擇：** Phase 1 不動 `omb/src/comp/game_processor.rs:496, 941, 944` 三個呼叫點。

**替代方案：** 也把 lives / game_end 走 snapshot：可行，但 lives 變動是「玩家操作後立即 ack」場景，event push 比 snapshot poll 更即時；game_end 是 one-shot 終局事件，再經 snapshot diff 多一拍延遲不必要。

**理由：** 這兩條 broadcast 流量極小（lives 漏怪才送，game_end 一局一次），不在 TD_STRESS 流量問題範圍內；保留有玩家操作 ack 路徑語義價值。

### Decision 7: Tower upgrade pip 走 `EntityRenderData.upgrade_levels: Option<[u8; 3]>`

**選擇：** Tower kind entity 在 `extract_snapshot` 時讀 `omobab::comp::Tower.upgrade_levels` 塞進 `EntityRenderData`；render 端對 Tower entity 在 body 旁畫 3 個 pip — 已升 path 綠色、未升 path 灰色。

**替代方案：** 把 upgrade_levels 放到 HeroStatsExt 般的 TowerStatsExt：每個塔都加 `Option<Box<...>>` size bloat 不值得，3 個 u8 直接 inline。

**理由：** upgrade_levels 是 fixed-size 24-bit 資料，inline 比 boxed cheaper；tower 數量 TD_STRESS 達 1000 個，Box per entity 是 1000 個額外 alloc。

## Risks / Trade-offs

| Risk | Mitigation |
|---|---|
| Phase 1 砍 emit 後 omfx 端某 HUD 元素仍在 read 不存在的 event → silent UI 空白 | Phase 1 完成後 Task 1.7 跑 TD_STRESS 60s smoke + 手動 TD_1 走一輪，列出空白項目；Phase 3-4 回填 |
| `UnitStats::from_refs` 從 omfx side 呼叫時 borrow 衝突（omb 端是同一 `World` 在 dispatcher 之外的 reader）| sim worker thread 內 dispatcher.dispatch 後再 extract，無並發寫者；snapshot extract 全程只開 read storage |
| 有 caller 偷懶直接呼叫 `entities().delete(e)` 跳過 outcome 通道 → omfx render 端 scene node 殘留（snapshot.removed_entity_ids 沒收到那個 id）| (1) grep guard lib test (`tests/delete_entity_outcome_only.rs`) 走訪 omb/src/ 所有 .rs 檔，allowlist 只有 `process_outcomes`，其他位置出現 `entities().delete(` / `.delete_entity(` 即 fail；(2) `Outcome::EntityRemoved` 是 enum variant — 編譯期檢查保證 process_outcomes match arm 不會被誤刪；(3) script boundary（abi_stable）只能透過 cache.outcomes 借用，沒有別條路徑可走 |
| `delete_entity_tracked` helper 沒被全部 delete site 採用 → render 端 scene node 殘留 | (1) Phase 1c task 一次性把所有 `world.delete_entity` / `entities().delete` grep 出來改走 helper；(2) 加 lib test 或 CI lint 規則禁止直接呼叫 `world.entities().delete`（grep guard）；(3) `RemovedEntitiesQueue` 不應該被任何 system 直接 push，只能透過 helper |
| Tower spawn handler 的 template lookup 失敗（kind_id 不存在 / map 沒對應 spec）→ omb panic | `handle_tower_spawn` return `Result`，失敗在 `player_input_tick` `log::warn!` 後丟，不 panic |
| omfx side aggregation 跟 omb side broadcast 結果不一致（armor / atk 算法差一個 buff）| Phase 3c 用 image 6 reference panel（armor 3.6 / atk 53 / asd 0.60s / range 900 / msd 350）逐項對；不一致時 audit `final_*` method 跟 omb 端 `build_hero_stats_payload` 對齊 |
| Phase 1 中途 build 失敗（dead unused fn warning blocking）| Task 1.8 統一清 dead code；warning 不阻擋 build（`#![allow(warnings)]` 已在 main.rs） |
| `BuffStore` / `UnitStats` 在 omb crate 是 `pub(crate)` → omfx 跨 crate 不能呼叫 | 改 `pub` 並 re-export 到 `omobab::ability_runtime::*`；audit 是否有其他 internal-only 假設 |
| Phase 4b Outcome::Explosion 上線前還有 `make_game_explosion` broadcast 並存，render 端兩邊都收到造成 double-render | Phase 4b 完成後同 commit 砍 `make_game_explosion` 兩個 emit + builder fn；intermediate phase 不啟用 `ExplosionFxQueue` consumer |
| 如果 4 個 PlayerInput 之中有 entry point 不存在（如 `inventory::use_item`）| Task 2.4 註明：先 stub `use_item`（slot 從 inventory 移除 + log），等 Phase 4d snapshot inventory ready 再回填邏輯 |

## Migration Plan

不需 schema migration（無 DB / 序列化檔）。Wire 協定 BREAKING 但 client/server 同步發行 — 沒有「舊 client + 新 server」共存場景。

部署順序：
1. Phase 1 全部 commit 後跑 Task 1.7（TD_STRESS 60s）驗證 wire < 5 KB/s
2. Phase 2 全部 commit 後跑 TD_1 手動 smoke 驗證放塔流程
3. Phase 3-4 各 task 完成後跑該 phase verify command
4. Final Task 全 phase smoke gate 通過後 close plan

Rollback：每 task 一個 commit，任何階段失敗可 `git revert` 該 commit；Phase 1 砍 emit 是 additive-revert（恢復 emit 即可），無資料層動作。

## Open Questions

- ~~`comp::inventory::use_item` 是否已存在~~ — **已 audit**：不存在於 `comp::inventory`，但 `GameProcessor::handle_item_use_from_input` (game_processor.rs:1144) 已是 pub fn entry point，且 player_input_tick 已 push 進 `PendingItemUseQueue` defer 處理。Phase 2 全部已實作，2.1 verify-only 即可
- ~~`comp::CurrentCreepWave` field 名稱~~ — **已 audit**：`wave: usize` / `path: Vec<usize>` / `is_running: bool` / `wave_start_time: f32`；total 從 `Vec<CreepWave>` resource `.len()`。`extract_snapshot` 已使用此 schema (`omfx/game/src/sim_runner.rs:974-980`)
- ~~`tower_upgrade_registry::apply_upgrade` 簽章~~ — **已 audit**：不存在於 registry（registry 只有 `get(kind, path, level)` 純 metadata lookup）；但 `GameProcessor::handle_tower_upgrade_from_input` (game_processor.rs:932) 已是 pub fn，player_input_tick 已 push 進 `PendingTowerUpgradeQueue`
- ~~`EntityKind::Tower` 在 sim crate 是否已標出~~ — **已 audit**：在 omfx side (`omfx/game/src/sim_runner.rs:241-249` `pub enum EntityKind`)，**不在** omoba-sim；`extract_snapshot` 已用此 enum 區分 Hero / Tower / Creep / Projectile arm
- ~~Phase 4b Outcome::Explosion 是否真會影響 sim crate determinism hash~~ — **已 audit 結束**：omoba-sim 不依賴 omobab outcome.rs（lib.rs 只 17 行 export，依賴清單無 omobab / specs / ECS 相關），8 個 pin 是 hash Fixed64 / trig / RNG / bincode wire 的 byte-level 結果，跟 ECS World schema 無關。Phase 4.2 已加過 `Outcome::Explosion` variant，omoba-sim 69 tests 維持綠 — 加 `Outcome::EntityRemoved` 同理

- **omfx submodule 在 implementation 開始前已有 pre-existing uncommitted 改動**（`game/src/lib.rs` + `game/src/lockstep_client.rs` — 加 `latest_rtt_us: Option<u64>` field、`LockstepEvent::Latency { rtt_us }` handler、`LockstepInbound::Pong` handler、HUD `Ping: ...` 顯示）。這些是 RTT/Ping 量測基建，跟本 change 無關但是另一個 change `input-render-latency` 的依賴。本 change 跑時不動這些檔（commit 只挑自己改的 `sim_runner.rs`），讓 input-render-latency 之後 commit 接手
