## ADDED Requirements

### Requirement: PlayerInput 依 player ownership 路由

PlayerInput routing SHALL treat lockstep `player_id` as the authoritative requester identity. Any input that affects a player-owned entity SHALL resolve the target owner through deterministic runtime state rather than selecting the first `FactionType::Player` entity.

Hero-owned inputs SHALL resolve the submitting player's Hero. Tower-owned inputs SHALL compare the submitting `player_id` with the target tower owner. Failure to resolve a matching owner SHALL reject the input with a warning and MUST NOT panic.

`Faction.team_id` SHALL remain combat-team state only. PlayerInput routing MUST NOT use `team_id` to identify the requesting player, because multiple players can share the same `team_id`.

#### Scenario: MoveTo uses submitting player hero

- **WHEN** `player_input_tick` drains `MoveTo` for `player_id = 2`
- **THEN** `drain_pending_moves` writes `MoveTarget` only to the Hero owned by player 2
- **AND** it does not use a first Player-faction hero fallback

#### Scenario: Ability input uses submitting player hero

- **WHEN** `player_input_tick` drains `CastAbility` or `UpgradeAbility` for `player_id = 1`
- **THEN** the shared `GameProcessor` handler resolves player 1's Hero before validating ability state
- **AND** a missing player 1 Hero causes a warning rejection without script event enqueue

#### Scenario: ItemUse uses submitting player inventory

- **WHEN** `player_input_tick` drains `ItemUse` for `player_id = 2`
- **THEN** item use reads and writes player 2's Hero inventory/state
- **AND** player 1's Hero inventory/state is unchanged by that input

### Requirement: TowerPlace records submitting owner

`TowerPlace` routing SHALL preserve the submitting `player_id` through `PendingTowerSpawnQueue` and `handle_tower_spawn_from_input`. A successfully spawned tower SHALL store owner metadata equal to that `player_id`.

The spawned tower SHALL keep the shared Player combat `team_id`; the owner metadata SHALL be the only source for sell/upgrade ownership checks.

#### Scenario: TowerPlace owner survives queue drain

- **WHEN** `player_input_tick` receives `TowerPlace` from `player_id = 2`
- **THEN** `PendingTowerSpawn.owner_pid` is `2`
- **AND** `handle_tower_spawn_from_input` creates a tower owned by player 2
- **AND** the tower's `Faction.team_id` remains the shared player team id, not `2`

### Requirement: TowerSell validates exact tower owner

`TowerSell` SHALL reject a target tower unless its owner metadata equals the submitting `player_id`. Checking only `FactionType::Player` is insufficient.

#### Scenario: selling another player's tower fails by owner check

- **WHEN** player A 對 player B 擁有的 tower 送出 `TowerSell`
- **THEN** `handle_tower_sell_from_input` 回傳 `Err(...)` 並 log warning
- **AND** 該 tower 不被移除
- **AND** player A 不取得 refund gold

### Requirement: TowerUpgrade validates exact tower owner

`TowerUpgrade` SHALL reject a target tower unless its owner metadata equals the submitting `player_id`. Checking only `FactionType::Player` is insufficient.

#### Scenario: upgrading another player's tower fails by owner check

- **WHEN** player A 對 player B 擁有的 tower 送出 `TowerUpgrade`
- **THEN** `handle_tower_upgrade_from_input` 回傳 `Err(...)` 並 log warning
- **AND** 該 tower 的 stats、flags、buffs 與 `upgrade_levels` 不變
- **AND** player A 不被扣除 gold
