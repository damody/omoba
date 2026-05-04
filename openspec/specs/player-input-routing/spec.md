## Purpose

Define how omfx UI actions are routed through lockstep `PlayerInput` and applied by omb ECS systems, so gameplay actions are deterministic and render feedback comes from snapshots.

## Requirements

### Requirement: PlayerInput end-to-end flow

The omb side `omb/src/tick/player_input_tick.rs` SHALL implement all supported `PlayerInputEnum` gameplay variants without log-only stubs. Each variant SHALL follow this flow:

1. omfx UI handler receives a click or keyboard event.
2. omfx packages UI state into the corresponding `PlayerInputAction` variant.
3. omfx sends the input through `lockstep_client.send_lockstep_input(PlayerInput { action: Some(...) })`.
4. omb `player_input_tick` reads the input for the scheduled tick.
5. omb logs input metadata including player id, tick, and variant data.
6. omb calls the corresponding ECS entry point.
7. Failures SHALL be logged with `warn` and MUST NOT panic or send a bespoke ack; the player observes success or failure through snapshot state.

#### Scenario: PlayerInput arms are implemented

- **WHEN** `omb/src/tick/player_input_tick.rs` is searched for `TowerPlace TODO`, `TowerSell TODO`, `TowerUpgrade TODO`, or `ItemUse TODO`
- **THEN** no TODO stub remains
- **AND** each arm calls the corresponding `GameProcessor::handle_*` or registry entry point

### Requirement: TowerPlace end-to-end

omfx tower button and map click handling SHALL send `PlayerInputAction::TowerPlace { tower_kind_id, pos }` through lockstep input. omb SHALL call `crate::comp::GameProcessor::handle_tower_spawn(world, kind_id: u32, pos: omoba_sim::Vec2, owner_pid: u32) -> Result<Entity, _>`, and that function SHALL be public. Wire integer positions SHALL be converted to `omoba_sim::Fixed64` values using `Fixed64::from_raw(...)`.

#### Scenario: TD_1 player places a tower

- **WHEN** a TD_1 player selects a tower kind and clicks the map
- **THEN** `omb_app.log` includes a `player_input_tick` line for `TowerPlace` with player id, tick, kind id, and raw position
- **AND** the sim ECS spawns a Tower entity
- **AND** the next snapshot contains that tower so omfx renders it without a legacy create event

#### Scenario: TowerPlace failure only warns

- **WHEN** a player submits `TowerPlace` with an invalid kind or invalid placement
- **THEN** `handle_tower_spawn` returns `Err(...)`
- **AND** omb logs a warning containing the player id and kind id
- **AND** omb does not panic or disconnect the player

### Requirement: TowerSell end-to-end

omfx sell button handling SHALL send `PlayerInputAction::TowerSell { tower_entity_id }` through lockstep input. omb SHALL call `crate::comp::GameProcessor::handle_tower_sell(world, entity_id: u32, owner_pid: u32) -> Result<(), _>`, and that function SHALL be public.

The sell handler SHALL look up the entity by id, verify it is a Tower owned by the submitting player, calculate refund gold using the active sell rule, add gold to the owning hero/player, and enqueue `Outcome::EntityRemoved { entity }`. Direct calls to `entities().delete()` or `world.delete_entity()` outside `process_outcomes` are prohibited.

#### Scenario: selling a tower refunds gold and removes the tower

- **WHEN** a TD_1 player places two towers and sells the first tower
- **THEN** the player's gold increases by the refund amount
- **AND** `process_outcomes` deletes the first tower through `Outcome::EntityRemoved`
- **AND** the next snapshot includes the first tower id in `removed_entity_ids`
- **AND** the second tower remains alive and rendered

#### Scenario: selling another player's tower fails

- **WHEN** player A sends `TowerSell` for a tower owned by player B
- **THEN** `handle_tower_sell` returns `Err(...)` and logs a warning
- **AND** the tower is not removed

### Requirement: TowerUpgrade end-to-end

omfx upgrade button handling SHALL send `PlayerInputAction::TowerUpgrade { tower_entity_id, path: u32, level: u32 }` through lockstep input. omb SHALL call `crate::comp::tower_upgrade_registry::apply_upgrade(world, entity_id, path: u8, level: u8, owner_pid: u32) -> Result<(), _>`, and that function SHALL be public. Applying an upgrade SHALL change the sim ECS tower stats and the next snapshot SHALL reflect the updated `upgrade_levels`.

#### Scenario: upgrading a tower changes upgrade level

- **WHEN** a player sends `TowerUpgrade { path: 0, level: 1 }` for their own tower
- **THEN** `omb_app.log` includes a `TowerUpgrade` line with entity id, path, and level
- **AND** the tower's relevant attack stats are updated in sim ECS
- **AND** the next snapshot has `upgrade_levels[0] == 1` for that tower

### Requirement: ItemUse end-to-end

omfx hero hotbar slot handling SHALL send `PlayerInputAction::ItemUse { item_slot, target_pos, target_entity }` through lockstep input. omb SHALL call `crate::comp::inventory::use_item(world, pid, slot: u8, target_pos: Option<Vec2>, target_entity: Option<u32>) -> Result<(), _>`.

#### Scenario: using an item consumes the slot

- **WHEN** a player picks up an item and clicks hotbar slot 0
- **THEN** `omb_app.log` includes an `ItemUse slot=0` line
- **AND** the item is consumed from the inventory
- **AND** the next snapshot inventory has slot 0 set to `None`

### Requirement: StartRound remains functional

`PlayerInputEnum::StartRound` SHALL continue using the same lockstep input pattern as the other PlayerInput variants: log metadata, call an ECS entry point, and warn on failure.

#### Scenario: StartRound advances the wave

- **WHEN** a player clicks the start round button
- **THEN** omb receives a `StartRound` input
- **AND** the next wave starts
- **AND** the next snapshot has `round_is_running == true` and an updated `round` value
