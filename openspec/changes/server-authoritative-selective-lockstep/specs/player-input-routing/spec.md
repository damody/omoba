## MODIFIED Requirements

### Requirement: PlayerInput 端到端流程

omb side player input routing SHALL 實作所有 supported `PlayerInputEnum` gameplay variants，不得保留 log-only stubs。Secure V2 match 的每個 variant SHALL 遵循以下流程：

1. omfx UI handler 收到 click 或 keyboard event。
2. omfx 將 UI state package 成對應的 `PlayerInputAction` variant；target entity 使用 team-scoped `ReplicaEntityId`、disclosure epoch 與 observed `view_epoch`。
3. omfx 透過 V2 lockstep client 送出 input。
4. omb 依 session/player/team binding、ownership、scheduled tick、replica mapping 與 input tick visibility history 驗證 input。
5. 通過驗證的 input 進入 scheduled authoritative tick，並記錄不含 hidden canonical detail 的 metadata。
6. Wave A gameplay system 呼叫對應 ECS entry point，同步產生 `Outcome` 與必要 `ObservableFact`。
7. Failure SHALL 使用 generalized rejection/warn，MUST NOT panic、送 bespoke existence ack 或透露 hidden target；player 透過後續 team frame 觀察權威結果。

#### Scenario: PlayerInput arms 已實作

- **WHEN** 搜尋 player input routing 中的 `TowerPlace TODO`、`TowerSell TODO`、`TowerUpgrade TODO` 或 `ItemUse TODO`
- **THEN** 不存在 TODO stub
- **AND** 每個 arm 都呼叫對應 shared runtime entry point

#### Scenario: Disclosed target input 端到端

- **WHEN** player 對目前 view epoch 可見且有權操作的 replica target 送出 input
- **THEN** server 將 replica ID 映射到 canonical entity 並在 scheduled tick 執行
- **AND** 後續 team frame 反映 authoritative result

## ADDED Requirements

### Requirement: Hidden-target anti-probing rejection

Unknown、stale、hidden 或 unauthorized replica reference SHALL 使用 generalized rejection class 與 uniform processing timing。Server response/log MUST NOT 向 player 暴露 canonical ID、entity existence、hidden position 或 visibility rule detail。Repeated invalid reference SHALL rate limit。

#### Scenario: Hidden-existing 與 nonexistent target 不可區分

- **WHEN** client 分別提交 hidden-existing 與 nonexistent replica reference
- **THEN** player-visible rejection class 相同
- **AND** timing bucket 與 response shape 相同
