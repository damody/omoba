## Purpose

定義 omfx UI actions 如何透過 lockstep `PlayerInput` routing 並由 omb ECS systems apply，讓 gameplay actions deterministic，且 render feedback 來自 snapshots。

## Requirements

### Requirement: PlayerInput 端到端流程

omb side `omb/src/tick/player_input_tick.rs` SHALL 實作所有 supported `PlayerInputEnum` gameplay variants，不得保留 log-only stubs。每個 variant SHALL 遵循以下流程：

1. omfx UI handler 收到 click 或 keyboard event。
2. omfx 將 UI state package 成對應的 `PlayerInputAction` variant。
3. omfx 透過 `lockstep_client.send_lockstep_input(PlayerInput { action: Some(...) })` 送出 input。
4. omb `player_input_tick` 讀取 scheduled tick 的 input。
5. omb log input metadata，包含 player id、tick 與 variant data。
6. omb 呼叫對應 ECS entry point。
7. Failures SHALL 用 `warn` log，且 MUST NOT panic 或送 bespoke ack；player 透過 snapshot state 觀察成功或失敗。

#### Scenario: PlayerInput arms 已實作

- **WHEN** 搜尋 `omb/src/tick/player_input_tick.rs` 中的 `TowerPlace TODO`、`TowerSell TODO`、`TowerUpgrade TODO` 或 `ItemUse TODO`
- **THEN** 不存在 TODO stub
- **AND** 每個 arm 都呼叫對應的 `GameProcessor::handle_*` 或 registry entry point

### Requirement: TowerPlace 端到端

omfx tower button 與 map click handling SHALL 透過 lockstep input 送出 `PlayerInputAction::TowerPlace { tower_kind_id, pos }`。omb SHALL 呼叫 `crate::comp::GameProcessor::handle_tower_spawn(world, kind_id: u32, pos: omoba_sim::Vec2, owner_pid: u32) -> Result<Entity, _>`，且該 function SHALL 是 public。Wire integer positions SHALL 使用 `Fixed64::from_raw(...)` 轉成 `omoba_sim::Fixed64` values。

#### Scenario: TD_1 player places a tower

- **WHEN** TD_1 player 選擇 tower kind 並點擊 map
- **THEN** `omb_app.log` 包含 `TowerPlace` 的 `player_input_tick` line，含 player id、tick、kind id 與 raw position
- **AND** sim ECS spawn 一個 Tower entity
- **AND** 下一個 snapshot 包含該 tower，讓 omfx 在沒有 legacy create event 的情況下 render

#### Scenario: TowerPlace failure 只 warn

- **WHEN** player submit invalid kind 或 invalid placement 的 `TowerPlace`
- **THEN** `handle_tower_spawn` 回傳 `Err(...)`
- **AND** omb log warning，內容包含 player id 與 kind id
- **AND** omb 不 panic 也不 disconnect player

### Requirement: TowerSell 端到端

omfx sell button handling SHALL 透過 lockstep input 送出 `PlayerInputAction::TowerSell { tower_entity_id }`。omb SHALL 呼叫 `crate::comp::GameProcessor::handle_tower_sell(world, entity_id: u32, owner_pid: u32) -> Result<(), _>`，且該 function SHALL 是 public。

sell handler SHALL 用 id lookup entity、確認它是由 submitting player 擁有的 Tower、依 active sell rule 計算 refund gold、把 gold 加到 owning hero/player，並 enqueue `Outcome::EntityRemoved { entity }`。`process_outcomes` 以外禁止直接呼叫 `entities().delete()` 或 `world.delete_entity()`。

#### Scenario: selling tower refund gold 並移除 tower

- **WHEN** TD_1 player 放置兩座 towers 並賣掉第一座 tower
- **THEN** player gold 依 refund amount 增加
- **AND** `process_outcomes` 透過 `Outcome::EntityRemoved` delete 第一座 tower
- **AND** 下一個 snapshot 的 `removed_entity_ids` 包含第一座 tower id
- **AND** 第二座 tower 仍 alive 並 rendered

#### Scenario: selling another player's tower fails

- **WHEN** player A 對 player B 擁有的 tower 送出 `TowerSell`
- **THEN** `handle_tower_sell` 回傳 `Err(...)` 並 log warning
- **AND** 該 tower 不被移除

### Requirement: TowerUpgrade 端到端

omfx upgrade button handling SHALL 透過 lockstep input 送出 `PlayerInputAction::TowerUpgrade { tower_entity_id, path: u32, level: u32 }`。omb SHALL 呼叫 `crate::comp::tower_upgrade_registry::apply_upgrade(world, entity_id, path: u8, level: u8, owner_pid: u32) -> Result<(), _>`，且該 function SHALL 是 public。Applying upgrade SHALL 改變 sim ECS tower stats，下一個 snapshot SHALL 反映 updated `upgrade_levels`。

#### Scenario: upgrading tower 會改變 upgrade level

- **WHEN** player 對自己的 tower 送出 `TowerUpgrade { path: 0, level: 1 }`
- **THEN** `omb_app.log` 包含 `TowerUpgrade` line，含 entity id、path 與 level
- **AND** tower 的相關 attack stats 在 sim ECS 中更新
- **AND** 下一個 snapshot 中該 tower 的 `upgrade_levels[0] == 1`

### Requirement: ItemUse 端到端

omfx hero hotbar slot handling SHALL 透過 lockstep input 送出 `PlayerInputAction::ItemUse { item_slot, target_pos, target_entity }`。omb SHALL 呼叫 `crate::comp::inventory::use_item(world, pid, slot: u8, target_pos: Option<Vec2>, target_entity: Option<u32>) -> Result<(), _>`。

#### Scenario: 使用 item 會 consume slot

- **WHEN** player 撿起 item 並點擊 hotbar slot 0
- **THEN** `omb_app.log` 包含 `ItemUse slot=0` line
- **AND** item 從 inventory consume
- **AND** 下一個 snapshot inventory 的 slot 0 為 `None`

### Requirement: StartRound 維持 functional

`PlayerInputEnum::StartRound` SHALL 持續使用與其他 PlayerInput variants 相同的 lockstep input pattern：log metadata、呼叫 ECS entry point，並在失敗時 warn。

#### Scenario: StartRound 推進 wave

- **WHEN** player 點擊 start round button
- **THEN** omb receives `StartRound` input
- **AND** 下一波開始
- **AND** 下一個 snapshot 有 `round_is_running == true` 與 updated `round` value
