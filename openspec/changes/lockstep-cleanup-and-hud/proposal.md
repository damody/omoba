## 為什麼

原 plan（`docs/plans/2026-05-04-lockstep-cleanup-and-hud.md`）寫於 Phase 4.2 那波遷移之前，認為還有三大塊待做：24 個 emit 沒砍 / 13 個 HUD 元素失聯 / 4 個 PlayerInput stub。本 change spec 期間做完 audit 後發現絕大多數已被 Phase 4.2 那波遷移順手做完 — 真正剩下的是：**(1) ~7 個 active emit 沒砍**（EntityDeath ×2 / TowerCreate / TowerUpgrade / GameRound / HeroStatic / HeroHot — wire 上死信，omfx 已不消費）、**(2) `removed_entity_ids` 目前用 `prev_alive: HashSet<u32>` diff 跑著（`omfx/game/src/sim_runner.rs:383, 949-951`），需重構成 `Outcome::EntityRemoved` 通道跟 Explosion 統一**、**(3) graph + plan 文件需同步**。Phase 2-4（13 個 HUD 元素 + 4 個 PlayerInput）皆已實作，本 change 純 verify-only。

## 變更內容

- **砍 ~7 個 active legacy emit**（EntityDeath ×2 / TowerCreate / TowerUpgrade / GameRound / HeroStatic / HeroHot / `push_hero_stats` 內部 broadcast）— omfx side snapshot 已涵蓋這些資料，wire 上是死信
- **重構 `removed_entity_ids` 演算法**：從現有 `prev_alive: HashSet<u32>` diff（`sim_runner.rs:383, 949-951`）改成 `Outcome::EntityRemoved { entity_id }` → `RemovedEntitiesQueue` resource → `extract_snapshot` drain，跟 `Outcome::Explosion` 統一 pipeline pattern；所有 `world.delete_entity(e)` SHALL 走新 helper `delete_entity_tracked(world, e)` 強制配對 outcome push，加 grep guard test 防漏 delete site
- **Verify Phase 2-4 已實作項目**：4 個 PlayerInput click handler 全已 wire（含 ItemUse 經 hotbar Digit1..6）/ `EntityRenderData` + `HeroStatsExt` + `BlockedRegionSnapshot` + `AbilityDefSnapshot` + `TowerTemplateSnapshot` (彩蛋) 全在 snapshot / `omb` 端 8 個 `handle_*_from_input` + `drain_pending_*` pub fn 已在 game_processor.rs / 粗 path style + Explosion ring + upgrade pip + inventory + ability bar 全接到 snapshot — 此 change 跑 audit + smoke 確認，發現缺漏才開 task
- **`graphify update .` 同步 graph** + `docs/plans/2026-05-04-lockstep-cleanup-and-hud.md` 加 audit 註記
- **保留不動**：`GameLives` ×1 / `GameEnd` ×3 兩條 HUD broadcast（玩家操作 ack 路徑、終局事件）
- **明確不做**：Observer / rejoin mode — `WorldSnapshot` 已涵蓋 9 個基礎 component（Pos / Vel / Facing / CProperty / Hero / Tower / Projectile / Creep / Other）並有 wire 框架（KCP 0x13–0x16）跟週期性 producer（30s 一次），但 omfx-side `apply_snapshot` 尚未實作（`omfx/game/src/lockstep_client.rs:257` 仍是 log-only）。要做完還需：(1) omfx 反序列化 `WorldSnapshot` 進本地 sim World、(2) `WorldSnapshot` schema 擴 buff_store / ability_levels / inventory / upgrade_levels / PlayerLives / CurrentCreepWave 等、(3) Observer UI（隱藏 player input）、(4) 多 client determinism 驗證 — 預計獨立 change 處理

## Capabilities

### New Capabilities

- `lockstep-event-flow`：定義 omb→omfx wire 上**只傳什麼 event**（PlayerCommandAck / GameLives / GameEnd / heartbeat / TickBatch），以及 stress 場景的吞吐量 budget（< 5 KB/s），明訂 Phase 1 廣播刪除清單與保留清單，並訂出「entity 死亡 / 爆炸 VFX 走 Outcome 不走 event」的不變式
- `player-input-routing`：定義 5 個 `PlayerInputEnum` variant（StartRound / TowerPlace / TowerSell / TowerUpgrade / ItemUse）的端到端流程 — omfx UI handler → `send_lockstep_input` → omb `player_input_tick` match arm → 對應 ECS 操作 entry point，含失敗 log 規則與 ownership 檢查
- `sim-snapshot-rendering`：定義 `SimWorldSnapshot` 的所有欄位（含 `RemovedEntitiesQueue` / `ExplosionFxQueue` drain 語義、`HeroStatsExt` aggregation 來源、`AbilityRegistry` Arc 共享），與 omfx render 端的消費規則（per-frame 倒數、權威值重設、scene node 釋放）；強制 `extract_snapshot` 對 sim ECS 寫入只能限於 outcome queue drain，禁止其他寫入以維持 lockstep determinism

### Modified Capabilities

（無 — `openspec/specs/` 目前無既有 capability）

## 影響範圍

- **Code**（實際變動範圍小於原 plan 預期）：
  - `omb/src/state/resource_management.rs` — 砍 5 個 active emit（EntityDeath sell / TowerCreate / GameRound / TowerUpgrade / `push_hero_stats` 整段函式 + 3 個 builder fn）
  - `omb/src/comp/outcome_system/combat_events.rs` — 砍 EntityDeath combat death emit
  - `omb/src/comp/outcome.rs` — 加 `Outcome::EntityRemoved` 新 variant + `RemovedEntitiesQueue` resource（既有 `Outcome::Explosion` + `ExplosionFxQueue` 已是 reference 實作）
  - 加 helper `pub fn delete_entity_tracked(world, e)`；所有現存 `world.delete_entity(e)` / `world.entities().delete(e)` 呼叫點改走 helper（grep guard test）
  - `omfx/game/src/sim_runner.rs` — **砍** `prev_alive` HashSet diff 三處（`:383, 562, 630, 949-951`），改 drain `RemovedEntitiesQueue` 進 `removed_entity_ids`
  - `omb/src/transport/kcp_transport.rs:551-571` 內 0-caller TypedOutbound variant 隨 enum 一起砍（dead code 清理）
  - `omb/src/comp/game_processor.rs` — 不動（`handle_tower_spawn_from_input` / `handle_tower_sell_from_input` / `handle_tower_upgrade_from_input` / `handle_item_use_from_input` 8 個 pub fn 已存在）
  - `omb/src/tick/player_input_tick.rs` — 不動（4 個 PendingQueue push 已實作）
  - `omfx/game/src/lib.rs` — 不動（4 個 click handler 已實作）
  - `omfx/game/src/render_bridge.rs` — path 樣式 / pip / explosion ring / region polygon 新 render
  - `omfx/game/src/lib.rs` — 4 個 click handler 改送 lockstep input、HUD 全部改讀 snapshot
- **Wire 協定**：BREAKING — Phase 1 後 omb 不再 emit 剩餘 ~7 個 event；舊版 omfx 客戶端與新 omb 不相容（無向下相容必要，client/server 同步發行）。Phase 4.2 已 BREAKING 過一次砍 ~17 個，本 change 是延續
- **Determinism**：`Outcome::Explosion` 加入 enum 不影響 sim crate（sim 不依賴 outcome.rs）；若 hash baseline 變動，commit 訊息標註並 re-pin 8 個 determinism test
- **Testing**：`omb` lib 145 / `omoba-sim` 69 / `omfx` lib 全綠 + TD_1 60s smoke + TD_STRESS 60s smoke (`< 5 KB/s` wire、無 freeze、無 KCP session 斷線)
- **Dependencies**：無新 crate；既有 `omb/src/ability_runtime/{buff_store,unit_stats}` 從 omfx 跨 process 複用 → 需確認 `BuffStore` / `UnitStats::from_refs` / `CProperty` / `TAttack` 為 `pub` 並 re-export 到 `omobab::ability_runtime::*` / `omobab::comp::*`
- **Docs**：`docs/plans/2026-05-04-lockstep-cleanup-and-hud.md` 為本 change 之來源 plan，已含逐 task line numbers
