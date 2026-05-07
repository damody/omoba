## ADDED Requirements

### Requirement: PlayerInput 端到端流程

omb 端 `omb/src/tick/player_input_tick.rs` SHALL 對 `PlayerInputEnum` 的 5 個 variant 全部實作 — 不得保留 stub log-only arm。每 variant 的端到端流程 MUST 是：

1. omfx UI handler 收到 click / 鍵盤事件
2. omfx 把 UI state（mouse_world_pos / selected_tower_kind_id / target_eid 等）包進對應 `PlayerInputAction` variant
3. 呼叫 `lockstep_client.send_lockstep_input(PlayerInput { action: Some(...) })`
4. omb 端 `player_input_tick` 在對應 tick 取出 `PlayerInputEnum` match arm
5. log 輸入 metadata（`pid`, `tick`, variant 內容）
6. 呼叫對應 ECS entry point（pub fn）
7. 失敗時 `log::warn!`，不 panic、不回 ack（玩家透過 snapshot diff 看到結果）

#### Scenario: 4 個 PlayerInput arm 都不再是 stub

- **WHEN** 在 `omb/src/tick/player_input_tick.rs` 內 grep `TowerPlace TODO` / `TowerSell TODO` / `TowerUpgrade TODO` / `ItemUse TODO`
- **THEN** 沒有 TODO stub 殘留
- **AND** 4 個 arm 都呼叫對應的 `GameProcessor::handle_*` 或 registry entry point

### Requirement: TowerPlace 端到端

omfx UI 點塔按鈕 + 點地圖 SHALL 經 `PlayerInputAction::TowerPlace { tower_kind_id, pos }` 送 lockstep input。omb 端 SHALL 呼叫 `crate::comp::GameProcessor::handle_tower_spawn(world, kind_id: u32, pos: omoba_sim::Vec2, owner_pid: u32) -> Result<Entity, _>`，該函式 SHALL 為 `pub`。`pos` SHALL 用 `omoba_sim::Fixed64::from_raw(...)` 從 wire i32 轉換。

#### Scenario: TD_1 玩家成功放塔

- **WHEN** TD_1 玩家點塔按鈕、選好 kind，再點地圖
- **THEN** `omb_app.log` 出現 `player_input_tick: pid=... tick=... TowerPlace kind_id=... pos_raw=(..., ...)`
- **AND** sim ECS 內 spawn 一個新 Tower entity
- **AND** 下一 snapshot 該塔出現在 omfx 畫面（透過 snapshot 不透過 event）

#### Scenario: TowerPlace 失敗只 log warn 不 panic

- **WHEN** 玩家送 `TowerPlace` 但 `kind_id` 不存在或 spawn 條件不符
- **THEN** `handle_tower_spawn` 回 `Err(...)`
- **AND** omb 端 `log::warn!("TowerPlace failed pid=... kind_id=...: {:?}", e)`
- **AND** omb 不 panic、不斷線

### Requirement: TowerSell 端到端

omfx UI 點 sell 按鈕 SHALL 經 `PlayerInputAction::TowerSell { tower_entity_id }` 送 lockstep input。omb 端 SHALL 呼叫 `crate::comp::GameProcessor::handle_tower_sell(world, entity_id: u32, owner_pid: u32) -> Result<(), _>`，該函式 SHALL 為 `pub` 並執行：
- 從 entity_id lookup entity
- 確認該 entity 有 `Tower` component 且 ownership 屬於 `owner_pid`
- 算 refund gold（依現有 sell rule，無則 80%）
- 加 gold 到 player.hero
- `world.delete_entity(e)` — render 移除由 snapshot diff 自動處理

#### Scenario: 賣塔回 gold + 塔消失

- **WHEN** TD_1 放兩個塔後賣掉第一個
- **THEN** 玩家 gold 增加 refund 數量
- **AND** 第一個塔從畫面消失（snapshot `removed_entity_ids` 含該 entity_id）
- **AND** 第二個塔不受影響

#### Scenario: 嘗試賣別人的塔失敗

- **WHEN** 玩家 A 送 `TowerSell` 對玩家 B 擁有的塔
- **THEN** `handle_tower_sell` 回 `Err(...)` 並 `log::warn!`
- **AND** 塔不被刪除

### Requirement: TowerUpgrade 端到端

omfx UI 點 upgrade 按鈕（path 0/1/2）SHALL 經 `PlayerInputAction::TowerUpgrade { tower_entity_id, path: u32, level: u32 }` 送 lockstep input。omb 端 SHALL 呼叫 `crate::comp::tower_upgrade_registry::apply_upgrade(world, entity_id, path: u8, level: u8, owner_pid: u32) -> Result<(), _>`，該函式 SHALL 為 `pub`。Upgrade 套用後 sim ECS 的 tower stats（攻速 / 攻擊力 / range）SHALL 變動且下次 snapshot 反映新數值。

#### Scenario: 升塔一級攻速變化

- **WHEN** 玩家對自己的塔送 `TowerUpgrade { path: 0, level: 1 }`
- **THEN** `omb_app.log` 出現 `TowerUpgrade eid=... path=0 level=1`
- **AND** sim ECS 內 tower 的攻速 / 攻擊力對應 path/level 改變
- **AND** snapshot 內該 tower 的 `upgrade_levels[0] == 1`

### Requirement: ItemUse 端到端

omfx hero hotbar slot click（slot 0..5）SHALL 經 `PlayerInputAction::ItemUse { item_slot, target_pos, target_entity }` 送 lockstep input。omb 端 SHALL 呼叫 `crate::comp::inventory::use_item(world, pid, slot: u8, target_pos: Option<Vec2>, target_entity: Option<u32>) -> Result<(), _>`。若 `inventory::use_item` 在當前 codebase 不存在，本 task SHALL 變兩階段：先 stub `use_item` 從 inventory 移除該 slot 並 log，等 snapshot inventory ready 再回填邏輯。

#### Scenario: 使用物品消耗 slot

- **WHEN** 玩家撿起物品後點 hotbar slot 0
- **THEN** `omb_app.log` 出現 `ItemUse slot=0`
- **AND** 該 slot 的 item 被消耗（snapshot inventory `slots[0] == None`）

### Requirement: StartRound 端到端（已有 baseline）

`PlayerInputEnum::StartRound` 已實作為 baseline template，本 spec 的其他 4 個 variant SHALL 沿用相同 pattern（log → 呼叫 ECS entry → 失敗 warn）。本 requirement 確認該 baseline 不被本次改動破壞。

#### Scenario: StartRound 仍正常

- **WHEN** 玩家點 start round 按鈕
- **THEN** omb 收到 `StartRound` input 並進入下一 wave
- **AND** snapshot `round_is_running` 變 `true`、`round` 數字增加
