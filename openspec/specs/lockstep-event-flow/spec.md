## Purpose

定義哪些 legacy network render events 被禁止或保留，並建立 outcome queues 作為 omb simulation events 到 omfx snapshot rendering 的 authoritative bridge。

## Requirements

### Requirement: legacy render-event emits 禁止

omb ECS tick 與 handler systems SHALL NOT 為已由 omfx lockstep simulation 與 `SimWorldSnapshot` extraction 推導出的 state 送出 legacy render-state `TypedOutbound` events。

forbidden emit list 包含：

- `EntityFacing`、`CreepStall`、`CreepSlow`、`CreepCreate`、`CreepMove`、`CreepHp`、`ProjectileCreate`、`ProjectileDestroy`、`UnitCreate`、entity `Miss` 與 `GameExplosion` legacy render payloads。
- `TypedOutbound::EntityDeath`、`TypedOutbound::TowerCreate`、`TypedOutbound::TowerUpgrade`、`TypedOutbound::GameRound`、`TypedOutbound::HeroStatic` 與 `TypedOutbound::HeroHot`。
- 這些 payload 的 builder functions，包括 entity death、tower create、tower upgrade、game round、hero static、hero hot 與 game explosion builders。

所有等效 render state SHALL 來自 local omfx sim ECS world 與 extracted snapshots。

#### Scenario: TD_STRESS wire traffic stays low

- **WHEN** `run_smoke_long.bat` 以 `STORY = "TD_STRESS"` 跑 60 秒
- **THEN** `omb_app.log` 中 sampled `kcp-p7 .* bytes_per_sec` values 維持低於 5000 bytes per second
- **AND** `omb_app.log` 包含零行 `Removed disconnected KCP session`
- **AND** `omfx_app.log` 包含零行 `no TickBatch in 1.0s`

#### Scenario: forbidden TypedOutbound variants 不被 constructed

- **WHEN** 搜尋 `omb/src/` 中的 `TypedOutbound::EntityDeath`、`TypedOutbound::TowerCreate`、`TypedOutbound::TowerUpgrade`、`TypedOutbound::GameRound`、`TypedOutbound::HeroStatic` 與 `TypedOutbound::HeroHot`
- **THEN** 沒有任何 `OutboundMsg::new_typed*` construction 使用這些 variants 作為 payload
- **AND** 對應 KCP routing entries 與 dead builder functions 不存在

#### Scenario: omb lib tests 通過

- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`
- **THEN** omb library test suite 通過

### Requirement: retained HUD broadcasts 進入 allowlist

omb SHALL 只保留仍需要作為 player-visible acknowledgements 或 one-shot terminal events 的 gameplay broadcasts。

allowlist 是：

- `TypedOutbound::GameLives` / `make_game_lives` 作為 life-loss acknowledgement。
- `TypedOutbound::GameEnd` / `make_game_end` 作為 game end overlays。

Explosion VFX SHALL NOT broadcast，且 SHALL 使用 `Outcome::Explosion` 進入 `ExplosionFxQueue` 與 `snapshot.explosions`。

#### Scenario: life loss 仍可見

- **WHEN** TD_1 creep 到達 base 並降低 lives
- **THEN** omb broadcast retained `game.lives` event
- **AND** omfx HUD 反映新的 lives value

#### Scenario: game end 仍可見

- **WHEN** TD_1 因 game completed 或 lives 歸零而結束
- **THEN** omb broadcast retained `game.end` event
- **AND** omfx 顯示 game end overlay

### Requirement: entity removal 使用 `Outcome::EntityRemoved` 作為唯一 delete channel

`omb/src/comp/outcome.rs` SHALL 定義 `Outcome::EntityRemoved { entity: Entity }` 與 `RemovedEntitiesQueue { pending: Vec<u32> }`。`process_outcomes` SHALL 是唯一呼叫 `entities().delete()` 的 code path。`EntityRemoved` arm SHALL 將 `entity.id()` push 到 `RemovedEntitiesQueue.pending`，並在同一次 outcome-processing pass 中 delete entity。

所有需要 remove entity 的 systems SHALL 將 `Outcome::EntityRemoved { entity }` push 到 world `Vec<Outcome>` resource。Script boundary despawn calls SHALL 透過相同 outcome resource routing。禁止在 `process_outcomes` 之外直接 delete entity。

`extract_snapshot` SHALL drain `RemovedEntitiesQueue` 到 `SimWorldSnapshot.removed_entity_ids`。omfx render code SHALL release 該 ids 的 per-entity render caches，包含 scene nodes、labels 與 collision rings。

#### Scenario: creep death 移除 omfx scene node

- **WHEN** TD_1 creep 在 sim 中死亡
- **THEN** death handling enqueue `Outcome::EntityRemoved { entity }`
- **AND** `process_outcomes` delete entity 並將其 id 記錄在 `RemovedEntitiesQueue.pending`
- **AND** 下一個 snapshot 的 `removed_entity_ids` 包含該 creep id
- **AND** omfx release 該 id 的 render caches
- **AND** omb 不 broadcast `EntityDeath`

#### Scenario: sold tower 透過 outcome channel 移除

- **WHEN** TD_1 player sell tower
- **THEN** sell handling enqueue `Outcome::EntityRemoved { entity }`
- **AND** `process_outcomes` delete tower 並記錄其 id
- **AND** 下一個 snapshot 的 `removed_entity_ids` 包含該 tower id
- **AND** tower 的 scene node、label 與 collision ring 消失

#### Scenario: process_outcomes 是唯一 delete sink

- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --test delete_entity_outcome_only`
- **THEN** grep guard 通過，且沒有在 allowed outcome-processing sink 之外找到 `entities().delete(` 或 `.delete_entity(` calls

### Requirement: explosion VFX 使用 `Outcome::Explosion` 與 drainable queue

`omb/src/comp/outcome.rs` SHALL 定義 `Outcome::Explosion { pos: omoba_sim::Vec2, radius: omoba_sim::Fixed64, duration: omoba_sim::Fixed64 }` 與 `ExplosionFxQueue { pending: Vec<ExplosionFx> }`。`process_outcomes` SHALL 將 explosion data push 到 `ExplosionFxQueue`。Script boundary explosion calls SHALL push 到相同 queue，且 SHALL NOT 送 legacy network event。

`extract_snapshot` SHALL drain queue 到 `SimWorldSnapshot.explosions`。omfx render code SHALL 為每個 explosion 建立 red-circle VFX，依 `duration_ms` animate scale 與 alpha，並在 duration 結束時 release scene node。

#### Scenario: bomb tower explosion 在 local render

- **WHEN** bomb tower 擊中 creep 並 emit explosion outcome
- **THEN** 下一個 snapshot 包含 position、radius、duration 與 spawn tick 的 explosion entry
- **AND** omfx render fading red-circle VFX
- **AND** duration 結束時 release VFX node
- **AND** omb 不 broadcast `GameExplosion`

#### Scenario: legacy explosion builders 不存在

- **WHEN** 搜尋 `omb/src/` 中的 `Outcome::Explosion`、`ExplosionFxQueue` 與 legacy game explosion builders
- **THEN** `Outcome::Explosion` 與 `ExplosionFxQueue` 存在
- **AND** legacy `make_game_explosion` builders 除 comments 外不存在
- **AND** extraction drain queue 到 `snapshot.explosions`
