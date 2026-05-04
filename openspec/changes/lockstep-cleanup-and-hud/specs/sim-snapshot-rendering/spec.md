## ADDED Requirements

### Requirement: `SimWorldSnapshot` 結構與 read-only-except-queues 不變式

`omfx/game/src/sim_runner.rs::SimWorldSnapshot` SHALL 包含下列欄位：

```
pub struct SimWorldSnapshot {
    pub tick: u32,
    pub entities: Vec<EntityRenderData>,
    pub paths: Vec<Vec<(f32, f32)>>,
    pub removed_entity_ids: Vec<u32>,
    pub round: u32,
    pub total_rounds: u32,
    pub lives: i32,
    pub round_is_running: bool,
    pub blocked_regions: Vec<Vec<(f32, f32)>>,
    pub explosions: Vec<ExplosionFx>,
    pub abilities: std::sync::Arc<Vec<AbilityDefSnapshot>>,
}
```

`extract_snapshot` 函式 SHALL 對 sim ECS World 嚴守只讀 — 唯一例外是 outcome queue 的 drain（`std::mem::take(&mut q.pending)`），因為 queue 是 producer-consumer 設計，drain 即是 consumer 端的 take ownership。除此之外不得寫任何 component / 不得寫其他 resource / 不得呼叫 `entities.create()` / 不得呼叫 `entities.delete()`，否則破壞 lockstep determinism。所有 boundary 數值 SHALL 用 `omoba_sim::Fixed64::to_f32_for_render` 與 `omoba_sim::Vec2` 對應 helper 從 fixed-point 轉 f32。

#### Scenario: extract_snapshot 只寫 outcome queue

- **WHEN** 在 `omfx/game/src/sim_runner.rs::extract_snapshot` 內 grep `write_storage` / `write_resource` / `entities.create` / `entities.delete`
- **THEN** 唯一寫入是對 `RemovedEntitiesQueue` 與 `ExplosionFxQueue` 的 `mem::take` drain
- **AND** 沒有任何 `write_storage` / `entities.create` / `entities.delete` 呼叫

#### Scenario: omoba-sim determinism tests 全綠

- **WHEN** 跑 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** 69 個 test 全綠（含 8 個 determinism pin hash）
- **AND** 加 `Outcome::EntityRemoved` variant **不**影響 8 個 pin（pin 對象是 Fixed64 / trig / RNG / bincode wire byte-level，與 omobab Outcome enum 無關 — omoba-sim 不依賴 omobab）

#### Scenario: `Outcome::EntityRemoved` 同 tick 完成 delete

- **WHEN** 系統 push `Outcome::EntityRemoved { entity: e }` 進 `Vec<Outcome>` resource，dispatcher tick 結尾跑 `process_outcomes`
- **THEN** process_outcomes arm 同步呼叫 `entities().delete(entity)` + push `entity.id()` 進 `RemovedEntitiesQueue.pending`
- **AND** `world.maintain()`（dispatcher tick 邊界）後 entity 在當下 tick 已 dead（`world.is_alive(e) == false`）
- **AND** server / client `StateHash` 對該 entity 在當下 tick 後不再 hash 它（兩端跑同一 process_outcomes 邏輯，必然對齊）

### Requirement: `removed_entity_ids` 從 `RemovedEntitiesQueue` drain

`extract_snapshot` SHALL 用 `std::mem::take(&mut q.pending)` 把 `RemovedEntitiesQueue.pending: Vec<u32>` 整批拉到 `SimWorldSnapshot.removed_entity_ids`，drain 後 queue 必須為空。**禁止**用 `prev_alive: HashSet<u32>` 跨 tick state diff 演算法（曾為初版設計，已改 outcome 通道，避免 worker 維護 stateful HashSet）。

#### Scenario: drain 後 queue 為空

- **WHEN** sim worker 跑完一 tick 且 `RemovedEntitiesQueue.pending` 含 N 筆
- **THEN** `extract_snapshot` 之後 `q.pending.is_empty()` 為 true
- **AND** `snapshot.removed_entity_ids.len() == N`

#### Scenario: 第一 tick 無刪除時 removed 為空

- **WHEN** sim worker 啟動後第一個 snapshot 期間沒有任何 `Outcome::EntityRemoved` 被 process_outcomes 處理
- **THEN** `removed_entity_ids` 為空 Vec

#### Scenario: 連續 tick 不會重複報

- **WHEN** 第 N tick process_outcomes 處理 `Outcome::EntityRemoved { entity: e_id_2 }`
- **THEN** 第 N+1 snapshot 的 `removed_entity_ids` 包含 `2`
- **AND** 第 N+2 snapshot 的 `removed_entity_ids` **不**包含 `2`（因 N+1 drain 後 queue 已空）

### Requirement: HUD 共用欄位 round / lives / round_is_running

`extract_snapshot` SHALL 從 sim ECS resource 讀取下列欄位塞進 snapshot（resource 名稱以 omb 端 `omobab::comp` 對齊）：
- `round` 與 `total_rounds` ← `CurrentCreepWave` resource
- `round_is_running` ← `CurrentCreepWave` resource
- `lives` ← `PlayerLives` resource

omfx HUD render SHALL 從 snapshot 讀這些欄位顯示，**不得**從舊的 `heartbeat.lives` / `current_round` mirror state 讀取。

#### Scenario: HUD 左上 lives / round 同步遊戲狀態

- **WHEN** TD_1 進行中漏怪扣命
- **THEN** 下一 snapshot 的 `lives` 減 1
- **AND** omfx HUD 左上 lives 數字立即更新

#### Scenario: round_is_running 切換顯示

- **WHEN** 玩家按 StartRound 後一 tick
- **THEN** snapshot `round_is_running == true`、`round` 增加
- **AND** omfx 對應 UI（start button hide / wave counter）反映新狀態

### Requirement: HeroStatsExt aggregation in omfx

`EntityRenderData` SHALL 加 `pub hero_ext: Option<Box<HeroStatsExt>>` 欄位（`Box` 避免普通 entity size bloat）。`HeroStatsExt` SHALL 包含：

```
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
    pub inventory: [Option<u32>; 6],
    pub ability_levels: [i32; 4],
}

pub struct BuffSnapshot {
    pub buff_id: String,
    pub remaining_secs: f32,    // -1.0 = toggle / 無限期
    pub payload_json: String,
}
```

`extract_snapshot` 對每個 Hero kind entity SHALL 呼叫 `omobab::ability_runtime::UnitStats::from_refs(&buff_store, e, /*is_building*/ false)`，再對每個 `final_*` method 算出實際數值並寫進 `hero_ext`。`UnitStats::from_refs` SHALL 為 `pub`，`BuffStore` / `CProperty` / `TAttack` SHALL 為 `pub` 並 re-export 到 `omobab::ability_runtime::*` / `omobab::comp::*`。

omfx hero panel UI SHALL 從 `snapshot.entities.find(|e| e.kind == Hero && e.entity_id == self.local_hero_eid).hero_ext.as_deref()` 讀數值；buff `remaining_secs` 在 render 端 per-frame 自行扣 `frame_dt`，下次 snapshot 重設為權威值避免漂移。

#### Scenario: hero panel 顯示正確 stat

- **WHEN** TD_1 載入 image 6 reference 場景
- **THEN** hero panel 顯示 armor 3.6 / atk 53 / asd 0.60s / range 900 / msd 350（與 omb side 既有 `build_hero_stats_payload` 結果一致）

#### Scenario: buff 倒數平滑遞減

- **WHEN** hero 上一個剩 5 秒的 buff
- **THEN** snapshot `remaining_secs == 5.0`
- **AND** render 端每 frame 扣 `frame_dt` 顯示 4.98 / 4.96 / ...
- **AND** 下次 snapshot（~16ms 後）重設為 omb side 權威值（如 4.984）

#### Scenario: toggle buff 顯示不倒數

- **WHEN** hero 開啟 toggle 型 buff
- **THEN** snapshot 該 buff `remaining_secs == -1.0`
- **AND** render 端不對該 buff 顯示倒數秒數

### Requirement: Tower upgrade pip 走 EntityRenderData.upgrade_levels

`EntityRenderData` SHALL 加 `pub upgrade_levels: Option<[u8; 3]>` 欄位（僅 Tower kind 填值）。`extract_snapshot` 對 Tower kind entity SHALL 從 sim ECS `Tower` component 讀取 `upgrade_levels` 塞入。omfx render SHALL 對每個 Tower entity 在 body 旁畫 3 個 pip — 已升 path 顯綠色、未升 path 顯灰色。

#### Scenario: 升塔兩級顯示 2 個綠 pip

- **WHEN** 玩家對某 Tower path 0 升 2 級
- **THEN** 下一 snapshot 該 tower entity `upgrade_levels == Some([2, 0, 0])`
- **AND** omfx 該塔旁邊第一個 pip 顯綠色、剩下灰色

### Requirement: Tower 選擇與升級面板（snapshot mirror）

omfx render 端 SHALL 在每 frame 進 snapshot lock 後，把 `EntityKind::Tower` 的 entity 鏡射進 `network_entities: HashMap<u32, NetworkEntity>`，欄位 mapping：
- `entity_type = "tower"`
- `position = (pos_x * WORLD_SCALE, pos_y * WORLD_SCALE)`
- `tower_kind = Some(unit_id)`（`unit_id` 為空時 `None`）
- `upgrade_levels = e.upgrade_levels.unwrap_or([0; 3])`
- `collision_radius_render` / `attack_range_backend` 從 `td_templates` cache 查（後者由 `snapshot.tower_templates` 種好）

鏡射完 SHALL 用 `retain` 把 `entity_type == "tower"` 但不在當 frame snapshot 的 entry 砍掉，避免賣塔後殘留。

點選已蓋塔（`lib.rs ~3273`）/ Sell + 3 升級按鈕面板（`lib.rs ~2782`）/ 攻擊範圍紅圈（`~2209`）三處 UI consumer SHALL 從 `network_entities` 讀，**不**直接 lock snapshot（避免 click handler 等 sim worker）。

**Why this requirement exists**: Phase 5.1 砍掉 legacy GameEvent stream 後 `network_entities` 永遠空，導致 (a) 點塔沒反應 (b) Sell 按鈕 + 3 條升級路線面板不出現 (c) 選中塔時的攻擊範圍紅圈不顯示。本 requirement 補上 lockstep snapshot → `network_entities` 的鏡射通道。

#### Scenario: 點 TD 塔選中並出現面板

- **WHEN** TD_1 玩家在 TD 塔上左鍵點擊
- **THEN** `selected_tower_entity = Some(eid)` 設定成功（`network_entities[eid].entity_type == "tower"` 命中 click hit-test）
- **AND** 右側面板出現 `▸ <tower label>` + `[SELL] $<refund>` 按鈕
- **AND** 同面板下方出現 3 條升級按鈕（path 0/1/2），格式 `■■□□  [P1] L2->L3 $<cost>` 或滿級時 `■■■■  [P1] MAX`
- **AND** 該塔身上顯示攻擊範圍紅圈（`network_entities[eid].attack_range_backend > 0.0`）

#### Scenario: 賣塔後 mirror retain 砍 entry

- **WHEN** 玩家點 SELL 按鈕觸發 `Outcome::EntityRemoved`，omb 處理後該塔 entity 不再出現在下一個 snapshot
- **THEN** `network_entities` 內該 eid 被 retain 砍掉
- **AND** 點地面同位置不再選到該塔

#### Scenario: 升塔後 upgrade_levels mirror 立即更新

- **WHEN** 玩家對 path 0 點升級按鈕，omb 處理後下一 snapshot 該 tower `upgrade_levels == Some([2, 0, 0])`
- **THEN** `network_entities[eid].upgrade_levels == [2, 0, 0]`
- **AND** Sell 面板的 path 0 升級按鈕顯示 `■■□□  [P1] L2->L3 $<cost>`

### Requirement: TowerUpgradeRegistry 透過 snapshot Arc 共享

`SimWorldSnapshot.tower_upgrades: Arc<Vec<TowerUpgradeDefSnapshot>>` SHALL 在 sim worker 跑第一個 tick 後從 `omobab::comp::tower_upgrade_registry::TowerUpgradeRegistry` 抽出共享的 `Arc`，後續每 snapshot `.clone()` Arc（O(1)）。`TowerUpgradeDefSnapshot` SHALL 含 `tower_kind`, `path`, `level`, `name`, `cost`。`TowerUpgradeRegistry` SHALL 提供 `iter_all()` method 讓 omfx sim worker 一次抽 48 個 def。

omfx render 端 SHALL 把 snapshot.tower_upgrades 種進 `td_upgrade_defs: HashMap<(unit_id, path, level), (name, cost)>` cache（lazy build；首個非空 snapshot 後永久 immutable，跟 `tower_templates` / `abilities` 同 pattern）。

#### Scenario: omfx Sell 退款公式對齊 omb

- **WHEN** 玩家對 path 0 升 2 級後（基礎 cost 100、L1 cost 25、L2 cost 50）點 SELL 按鈕
- **THEN** omfx Sell 面板顯示 refund = `100*0.85 + 25*0.75 + 50*0.75 = 141`
- **AND** omb 端 `sell_tower::refund` 計算同值

#### Scenario: 升級按鈕顯示 next-level upgrade 名稱

- **WHEN** TD_1 玩家選中 dart_monkey tower（未升級），3 條升級按鈕顯示
- **THEN** path 0 按鈕文字為 `[P1] L0->L1 Long Range Darts $50`
- **AND** path 1 按鈕為 `[P2] L0->L1 Quick Shots $50`
- **AND** path 2 按鈕為 `[P3] L0->L1 Keen Eyes $50`
- **AND** **不**使用 `■`/`□`/`●`/`○` 等 unicode pip glyph（Fyrox 預設字型缺字會 render 成 missing-glyph 方框）

#### Scenario: 滿級按鈕顯示 MAX

- **WHEN** path 0 已升到 L4
- **THEN** path 0 升級按鈕文字為 `[P1] MAX`

### Requirement: 塔身上 label 只顯示升級級別摘要

omfx render 端 sim_runner-backed name labels（`omfx/game/src/lib.rs:2516+`）對 `EntityKind::Tower` SHALL：
- 若 `upgrade_levels` 任一 path > 0：label 文字為 `"<L0>/<L1>/<L2>"`（例：`2/4/0`）
- 若全 0 / `None`：**不**建立 label widget（從 `sim_entity_labels` 移除既有 entry）
- **不**顯示塔的 hp / max_hp（塔不需要 HP 資訊）

Hero / Creep 走既有 `"name HP/MaxHP"` 格式不變。

#### Scenario: 未升級塔不顯示 label

- **WHEN** TD_1 玩家剛蓋一座 dart_monkey（`upgrade_levels == [0, 0, 0]`）
- **THEN** 塔身上**沒有**任何 name label widget
- **AND** scene 內該 entity 對應的 `sim_entity_labels` entry 不存在

#### Scenario: 升級後塔顯示 N/N/N

- **WHEN** 玩家對 path 0 升 2 級、path 1 升 4 級
- **THEN** 下一 snapshot 該塔 `upgrade_levels == Some([2, 4, 0])`
- **AND** 塔身上 label 文字為 `"2/4/0"`
- **AND** **不**含塔名稱、HP、`■`/`●` pip glyph

### Requirement: Sell/Upgrade 面板寬度避免切字

`omfx/game/src/lib.rs::Game::on_init` 內三個 widget (`ui_td_sell_name_text` / `ui_td_sell_button_text` / `ui_td_upgrade_buttons[0..3]`) 的 `with_width` SHALL ≥ 360.0；面板算位 `panel_w` 同步 ≥ 360.0。原本 240.0 不夠寬，升級按鈕 `[P1] L0->L1 Long Range Darts $50` 末段 `$50` 會被視窗右緣截掉。

#### Scenario: 寬度足夠顯示完整按鈕文字

- **WHEN** TD_1 選中 dart_monkey tower，path 0 升級按鈕渲染
- **THEN** 按鈕 widget 的 `with_width` 為 360.0
- **AND** 文字 `[P1] L0->L1 Long Range Darts $50` 完整顯示，末段 `$50` 不被切

### Requirement: BlockedRegion polygons via snapshot

`SimWorldSnapshot.blocked_regions: Vec<Vec<(f32, f32)>>` SHALL 在 `extract_snapshot` 時從 `omobab::comp::BlockedRegions` resource 讀取（map load 後不變，每 snapshot clone 成本可忽略；TD_1 為空）。omfx render SHALL 用既有 `build_polygon_outline` 紅線 + `build_circle_outline` 橘圓畫出 region 輪廓。

#### Scenario: DEBUG_1 場景顯示 region 輪廓

- **WHEN** `STORY = "DEBUG_1"` 載入有 BlockedRegion 的場景
- **THEN** snapshot `blocked_regions` 非空
- **AND** omfx 畫出紅線多邊形 + 橘圓 visible

#### Scenario: TD_1 無 region 時為空

- **WHEN** TD_1 載入
- **THEN** `blocked_regions` 為空
- **AND** omfx 不畫任何 region

### Requirement: AbilityRegistry 用 Arc 共享

`SimWorldSnapshot.abilities: Arc<Vec<AbilityDefSnapshot>>` SHALL 在 sim worker init 時跑一次 `extract_ability_defs(&world)` 建出共享的 `Arc`，之後每 snapshot `.clone()` Arc（O(1) 不複製 inner data）。`AbilityDefSnapshot` SHALL 包含 `ability_id`, `display_name`, `max_level`, `icon_path` 等 UI 需要的 metadata。

`HeroStatsExt.ability_levels: [i32; 4]` SHALL 對應 hero 的 Q W E R 等級。omfx ability bar SHALL 從 `snapshot.abilities` lookup definition + `ext.ability_levels[i]` 顯示 `lvl/max`。

#### Scenario: hero 升 Q 後 ability bar 反映

- **WHEN** TD_1 hero level up 後玩家點 Q 升級
- **THEN** 下一 snapshot 該 hero `ext.ability_levels[0]` 增加 1
- **AND** ability bar Q 顯示從 "0/4" 變 "1/4"

#### Scenario: abilities Arc 不重新 build

- **WHEN** sim worker 跑 N 個 tick
- **THEN** `abilities` 內部 `Vec<AbilityDefSnapshot>` 在 worker init 後不再 rebuild
- **AND** 每 snapshot 只是 `Arc::clone` O(1) 操作

### Requirement: 粗 zigzag path render style

`omfx/game/src/render_bridge.rs::ensure_paths_drawn` SHALL 用線寬 `64.0 * crate::WORLD_SCALE * 2.0`（render unit 1.28）與顏色 `(170, 140, 90, 255)` 奶油色畫 path；checkpoint marker dot 不畫（粗線本身覆蓋 corner）。

#### Scenario: TD_1 path 為粗奶油色 zigzag

- **WHEN** TD_1 載入
- **THEN** path render 為粗奶油色 zigzag 線
- **AND** corner 處沒有額外的 checkpoint marker dot
