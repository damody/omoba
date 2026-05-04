> **Audit summary（執行前必讀）**：原 plan（`docs/plans/2026-05-04-lockstep-cleanup-and-hud.md`）寫於 Phase 4.2 那波遷移之前，過期甚多。實際 audit 結果：
>
> - **Phase 1**：原列 24 個 emit，~17 個已被 Phase 4.2 順手砍（facing / stall / slow / creep_create / hp_update / projectile_create / make_game_explosion 全系列），剩 ~7 個 active
> - **Phase 2**：4 個 PlayerInput **全部已實作** — `omfx/game/src/lib.rs` 4 個 click handler + `omb/src/tick/player_input_tick.rs` 4 個 PendingQueue push + `GameProcessor::handle_*_from_input` ×4 + `drain_pending_*` ×4，皆已 wire（Phase 2.1 ~ 2.4 註解都在）
> - **Phase 3**：3 task 全部已實作 — 粗 path style (`render_bridge.rs:46,49`)、round/lives/round_is_running (`sim_runner.rs:57-67`)、HeroStatsExt + `UnitStats::from_refs` aggregation (`:660, 850`)
> - **Phase 4**：5 task 全部已實作 — BlockedRegion / Explosion / upgrade_levels / inventory / abilities Arc + ability_levels 全在 snapshot 裡；額外彩蛋 `tower_templates: Arc<...>` plan 沒列也已做
>
> 所以本 change 真實工作只剩三件事：(1) **砍 7 個 active emit**、(2) **重構** `removed_entity_ids` 從現有 prev_alive diff（`sim_runner.rs:383,949-951`）改成 `Outcome::EntityRemoved` 通道（用戶決定），(3) `graphify update .` 同步 graph + plan 文件。Phase 2/3/4 全 verify-only。

## 0. Pre-flight — graphify 同步 + audit 結果註記

- [x] 0.1 跑 `graphify update .` 把 graph 拉回現況（`graphify-out/GRAPH_REPORT.md` 仍顯示 `make_creep_create` / `make_entity_facing` 等已不存在的節點）；commit `chore: graphify update sync after phase 4.2 migration`

## 1. Phase 1 — 砍剩餘 active emit + Outcome::EntityRemoved 重構

> 這是本 change 的**主要工作量**。

- [x] 1.1 砍 EntityDeath 兩個 emit：(a) `omb/src/state/resource_management.rs:704-720` sell tower 段 — 整段 `EntityDeath` 廣播刪除（兩個 `#[cfg]` arm 都砍）；保留上方 `world.entities().delete(target_entity).ok()`（會在 1.6b 改走 `delete_entity_tracked`）；(b) `omb/src/comp/outcome_system/combat_events.rs:300-319` — 砍 `OutboundMsg::new_typed(... TypedOutbound::EntityDeath ...)` 跟下面 `let _ = mqtx.send(msg)` 整段；commit `phase1: drop EntityDeath emit (sell tower + combat death)`

- [x] 1.2 砍 TowerCreate emit：`omb/src/state/resource_management.rs:339-351` 整段 `OutboundMsg::new_typed_at(... TowerCreate ...)` + `mqtx.send(msg)` 刪除（兩個 `#[cfg]` arm 都砍）；保留上方 `tpl` lookup 跟 entity creation；commit `phase1: drop TowerCreate emit (snapshot covers spawn)`

- [x] 1.3 砍 GameRound emit：`omb/src/state/resource_management.rs:394-404` 整段 — 三個 `#[cfg(...)]` arm 全砍；保留上方 `ccw.is_running = true` 邏輯；commit `phase1: drop GameRound emit (snapshot covers round state)`

- [x] 1.4 砍 TowerUpgrade emit：`omb/src/state/resource_management.rs:594-604` 整段 — `OutboundMsg::new_typed_at(... TowerUpgrade ...)` + `mqtx.send(msg)` 刪除（兩個 `#[cfg]` arm）；保留上方 `t.upgrade_levels[path as usize] = next_level`；commit `phase1: drop TowerUpgrade emit (snapshot covers upgrade_levels)`

- [x] 1.5 砍 hero broadcast 一系列：**實際發現** plan 的 `push_hero_stats` 在 Phase 5.2 已被改成 no-op stub；real active broadcast 在 `omb/src/state/resource_management.rs::broadcast_hero_update` (約 :853-:921)。本 task 把 `broadcast_hero_update` 整段函式體清空（保留空殼讓 4 個 caller 不破，1.8 dead code 一起砍 fn + callers + 3 個 builder fn `build_hero_static_msg` / `build_hero_hot_msg` / `build_hero_stats_payload`）。emit 涵蓋：(a) `:954` `static_msg`、(b) `:959` `hot_msg`、(c) `:970-972` non-kcp `hero.stats` fallback、(d) `:984-990` hero `inventory` 廣播；commit `phase1: drop hero static/hot/stats/inventory broadcast (snapshot covers HeroStatsExt)`

- [x] 1.6 Phase 1b — entity_death **重構** 從 prev_alive diff → `Outcome::EntityRemoved`：

  > **背景**：目前 `omfx/game/src/sim_runner.rs:383, 949-951` 用 `prev_alive: HashSet<u32>` 跨 tick state diff 算 `removed_entity_ids`，working 但分散兩處（worker init + extract_snapshot），且 entity 死亡邏輯被動推算。重構成 Outcome pattern 跟 Explosion 統一。

  (a) `omb/src/comp/outcome.rs` 加 `Outcome::EntityRemoved { entity_id: u32 }` variant 跟 `RemovedEntitiesQueue { pending: Vec<u32> }` resource（World setup 處 `world.insert(RemovedEntitiesQueue::default())` — 看 `omb/src/state/initialization.rs:394` 旁邊已 insert 了 `ExplosionFxQueue`，同樣寫法）；
  (b) 加 helper `pub fn delete_entity_tracked(world: &mut World, e: Entity)`，**同一 fn body 內**先 `q.pending.push(e.id())` 再 `world.entities().delete(e)`（不可拆 system 跨 tick — StateHash desync 風險）；
  (c) 加 lib test `omb/tests/delete_entity_tracked.rs`：呼叫 helper 後 `world.maintain()` verify `world.is_alive(e) == false` 且 `q.pending` 含 `e.id()`；
  (d) `omfx/game/src/sim_runner.rs::extract_snapshot` — **砍** `:383` `prev_alive` 宣告、`:562` 傳遞、`:630` 參數、`:949-951` diff 演算 三處；改用 `std::mem::take(&mut q.pending)` drain `RemovedEntitiesQueue` 進 `snapshot.removed_entity_ids`（對應 `:996` 既有 `ExplosionFxQueue` drain 旁邊放）；
  (e) `omfx/game/src/lib.rs` render cleanup 段對 `snapshot.removed_entity_ids` 釋放 `scene_nodes_by_eid` / `labels` / `collision_rings`（**現況確認**：lib.rs 目前已從 snapshot 讀，邏輯不動）；
  smoke run.bat (TD_1) 確認 creep 死亡 / 塔被拆 都正常從畫面消失；
  commit `phase1b: refactor removed_entity_ids from prev_alive diff to Outcome::EntityRemoved`

- [x] 1.6b Delete site 全清查 + grep guard：**design 校正** — 棄用 `delete_entity_tracked` helper 改用 `Outcome::EntityRemoved` 通道（架構一致：跟 `Outcome::Death` / `Outcome::Explosion` 同 pattern；script boundary 不需 `&mut World`）。實際做：(a) 把 `Outcome::EntityRemoved` variant 從 `{entity_id: u32}` 改成 `{entity: Entity}` 帶 generation；(b) `process_outcomes` arm 同時做 push queue + `entities().delete()`；(c) 砍 `delete_entity_tracked` helper；(d) 3 個 raw delete site 改 push outcome（`game_processor.rs:892` / `resource_management.rs:649` / `world_adapter.rs:498`）；(e) `WorldAdapterCache` 加 `outcomes: Write<Vec<Outcome>>` 欄位；(f) 加 grep guard test `omb/tests/delete_entity_outcome_only.rs`，allowlist `process_outcomes` 是唯一 sink；verify omb 155 lib tests 全綠 + grep guard 通過；commit `phase1b2: route all entity deletes through Outcome::EntityRemoved`

- [x] 1.7 Smoke gate：把 `omb/game.toml` 的 `STORY` 用 PowerShell 換成 `TD_STRESS`，跑 `run_smoke_long.bat` 60 秒；verify `omfx_app.log` `grep -c "no TickBatch in 1.0s"` 為 0、`omb_app.log` `grep -c "Removed disconnected KCP session"` 為 0、`grep "kcp-p7 .* bytes_per_sec" omb_app.log | tail -10` 持續 < 5000；最後 PowerShell 復原 `STORY = TD_1`；no commit

- [x] 1.8 清 dead code：`cargo build` 看 unused fn / unused TypedOutbound variant warning；對每個 warning grep callers，0 callers 就砍 — 預期砍掉：(a) `proto_build::entity_death*` / `proto_build::tower_create` / `proto_build::tower_upgrade` / `proto_build::game_round` / `proto_build::hero_static` / `proto_build::hero_hot` 跟對應 `TypedOutbound::*` variant（含 `omb/src/transport/kcp_transport.rs:551-571` routing 表內 0-caller entry）；(b) `broadcast_hero_update` 空殼函式（1.5 已清空 body）+ 4 個呼叫點（`grep "broadcast_hero_update("` 重新 audit 確認位置）；(c) `push_hero_stats` / `push_hero_static` 兩個 Phase 5.2 留下的 no-op stub fn；(d) `build_hero_static_msg` / `build_hero_hot_msg` / `build_hero_stats_payload` 三個 builder fn；保留 `GameLives` / `GameEnd` / `GameExplosion`；verify `cargo build` clean 無 warning + 155 lib tests 全綠；commit `phase1: clean dead-code legacy event builders + TypedOutbound variants`

## 2. Phase 2-4 — Verify-only：原 plan 已實作項目逐一確認

> Audit 結果：原 plan 列的 12 個任務（4 PlayerInput + 3 HUD baseline + 5 HUD rest）已全部實作。本 phase 純 verify。**任何 verify 失敗開新 task 補救，不在現有 task 範圍**。

- [x] 2.1 Verify Phase 2 PlayerInput wire 4 個皆 active：(a) grep `omfx/game/src/lib.rs` 確認 4 個 send：`Action::TowerSell` (~3169) / `Action::TowerUpgrade` (~3212) / `Action::TowerPlace` (~3245) / `Action::ItemUse` (~3485)；(b) grep `omb/src/tick/player_input_tick.rs` 確認 4 個 PendingQueue push（TowerPlace ~:184 / TowerUpgrade ~:206 / TowerSell ~:224 / ItemUse ~:244）；(c) grep `omb/src/comp/game_processor.rs` 確認 8 個 pub fn 存在（`handle_tower_spawn_from_input` ~:698、`handle_tower_sell_from_input` ~:762、`handle_tower_upgrade_from_input` ~:932、`handle_item_use_from_input` ~:1144、4 個對應 `drain_pending_*`）；(d) 手動 smoke：`run.bat` (TD_1) 點塔按鈕 → 出塔 / 點 sell → 退錢消失 / 點 upgrade → stats 變 / 按 Digit1..6 → ItemUse log 出現；no commit

- [x] 2.2 Verify Phase 3 HUD baseline 3 項：(a) `render_bridge.rs:46,49` `PATH_LINE_THICKNESS = 64.0 * WORLD_SCALE * 2.0` + `PATH_COLOR = (170, 140, 90, 255)`；(b) `sim_runner.rs:57-67` `round/total_rounds/lives/round_is_running` 4 個欄位 + extract 從 `CurrentCreepWave` / `Vec<CreepWave>::len()` / `PlayerLives` 讀；(c) `HeroStatsExt` (`:207`) 12 欄位 + `UnitStats::from_refs(&*buff_store, false)` 在 `:660` 跑 + `:850` `Some(Box::new(HeroStatsExt {...}))`；(d) 手動 smoke：粗奶油色 path / 左上 lives 跟著漏怪扣 / hero panel image 6 reference armor 3.6 atk 53 asd 0.60s range 900 msd 350；no commit

- [x] 2.3 Verify Phase 4 HUD rest 5 項 + 彩蛋：(a) `BlockedRegion` `:75` `blocked_regions: Vec<BlockedRegionSnapshot>` + `BlockedRegionSnapshot { points, circle }` (`:100`) + `REGION_LINE_THICKNESS` (`render_bridge.rs:31`)；(b) Explosion (前已 audit) — `Outcome::Explosion` + `ExplosionFxQueue` + `:996` drain + `lib.rs:1590-1593` render；(c) `EntityRenderData.upgrade_levels: Option<[u8; 3]>` (`:180`) + `lib.rs:3207` `network_entities[tid].upgrade_levels` 已讀；(d) `HeroStatsExt.inventory: [Option<String>; 6]` (`:229`)；(e) `abilities: Arc<Vec<AbilityDefSnapshot>>` (`:81`) + lazy build (`:512-527`) + `ability_levels: [i32; 4]` (`:233`) + `ability_ids: [Option<String>; 4]` (`:238`)；(f) **彩蛋** `tower_templates: Arc<Vec<TowerTemplateSnapshot>>` (`:88`)；(g) 手動 smoke：DEBUG_1 region 紅線 + bomb tower 爆炸紅圈 + 升塔旁綠 pip + hero hotbar item icon + ability bar Q "0/4 → 1/4"；no commit

- [x] 2.4 若任 verify 失敗：把該項缺的部分開新 task 補進此 phase（命名 `2.X-fix-<item>`）；驗證後 commit

- [x] 2.5-fix-tower-selection 鏡射 snapshot Tower → `network_entities`：手動 smoke 後發現 Phase 5.1 砍 legacy GameEvent 後 `network_entities` 永遠空，導致 (a) 點塔沒反應 (b) Sell + 3 條升級路線面板不出現 (c) 攻擊範圍紅圈不顯示。在 `omfx/game/src/lib.rs` 進 snapshot lock 後（`render_bridge.update` 之後）對 `EntityKind::Tower` 鏡射進 `network_entities`，欄位 mapping `entity_type/position/tower_kind/upgrade_levels/collision_radius_render/attack_range_backend`；`retain` 砍掉不在當 frame snapshot 的 tower entry；commit `omfx 08112af` + bump pointer `omoba 9ffcf30`

## 3. Final — 全 phase verify gates

- [x] 3.1 omb lib tests：`cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib` → 155 全綠（codebase 自原 plan 寫成後增加 10 個 test，本 change 中於 1.6 commit 後驗證為 155）
- [x] 3.2 omoba-sim determinism tests：`cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features` → 69 全綠（含 8 個 pin hash — omoba-sim 不依賴 omobab outcome.rs，加 variant 不影響 pin）
- [x] 3.3 omfx lib tests：`cargo test --manifest-path D:/omoba/omfx/Cargo.toml -p omfx --lib` 全綠
- [ ] 3.4 TD_1 60s smoke：PowerShell 確保 `STORY = "TD_1"` → `cd /d D:/omoba && run_smoke_long.bat`；verify `omfx_app.log` `grep -c "no TickBatch in 1.0s"` 為 0；視覺：放塔 / 升塔 / 賣塔 / 撿物 / Q 升 全 work；HUD：粗 path / lives / round / hero stats / inventory / ability bar 全顯示；TD_1 整局走完到 game end
- [ ] 3.5 TD_STRESS 60s smoke：PowerShell 換 `STORY = "TD_STRESS"` → `run_smoke_long.bat`；verify `omb_app.log` `grep -c "Removed disconnected KCP session"` 為 0；`grep "kcp-p7 .* bytes_per_sec" omb_app.log` 持續 < 5000；無 freeze / panic
- [ ] 3.6 復原 TD_1：PowerShell 換回 `STORY = "TD_1"`；no commit
- [x] 3.7 graphify update：`graphify update .` 把 Phase 1 的全部變動同步進 graph；commit `chore: graphify update post lockstep cleanup`
- [x] 3.8 plan 文件補註：`docs/plans/2026-05-04-lockstep-cleanup-and-hud.md` 加 trailing 註記說明本 change audit 結果（Phase 2-4 早於 plan 寫的就已實作）；commit `docs: annotate lockstep plan with audit findings`
