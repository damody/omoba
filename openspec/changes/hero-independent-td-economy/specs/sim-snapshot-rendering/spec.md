## ADDED Requirements

### Requirement: SimWorldSnapshot 提供 entity-independent 玩家金錢

`SimWorldSnapshot` SHALL 以 deterministic、依 `player_id` 可查詢的欄位公開 `PlayerEconomy` 餘額。snapshot extraction SHALL 為 read-only，且 SHALL NOT 需要任何 Hero、Pos、Gold 或 render entity 才能輸出玩家餘額。

#### Scenario: 零英雄 snapshot 仍包含玩家金錢
- **WHEN** TD World 有 player 1 餘額 650 且沒有 Hero entity
- **THEN** extracted snapshot 的 player 1 金錢為 650
- **AND** snapshot 不包含為錢包建立的假 render entity

#### Scenario: snapshot extraction 不修改帳戶
- **WHEN** 對含有多個玩家帳戶的 World 連續執行 snapshot extraction
- **THEN** 每次 snapshot 的玩家金錢相同
- **AND** ECS `PlayerEconomy` 餘額與帳戶集合不變
