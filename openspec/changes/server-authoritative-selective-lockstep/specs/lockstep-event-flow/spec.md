## MODIFIED Requirements

### Requirement: legacy render-event emits 禁止

omb ECS tick 與 handler systems SHALL NOT 為已由 omfx selective lockstep simulation、team projection 與 filtered render snapshot 推導出的 state 送出 legacy render-state `TypedOutbound` events。

forbidden emit list 包含：

- `EntityFacing`、`CreepStall`、`CreepSlow`、`CreepCreate`、`CreepMove`、`CreepHp`、`ProjectileCreate`、`ProjectileDestroy`、`UnitCreate`、entity `Miss` 與 `GameExplosion` legacy render payloads。
- `TypedOutbound::EntityDeath`、`TypedOutbound::TowerCreate`、`TypedOutbound::TowerUpgrade`、`TypedOutbound::GameRound`、`TypedOutbound::HeroStatic` 與 `TypedOutbound::HeroHot`。
- 這些 payload 的 builder functions，包括 entity death、tower create、tower upgrade、game round、hero static、hero hot 與 game explosion builders。

所有等效 render state SHALL 來自該 session 所屬 team 的 `SelectiveReplicaRuntime` 與 filtered render snapshot。Secure fog match MUST NOT 以 legacy event 補送 hidden 或 global state。

#### Scenario: TD_STRESS wire traffic stays low

- **WHEN** final verification 以 `STORY = "TD_STRESS"` 跑 steady-state bandwidth scenario
- **THEN** secure team stream sampled bytes per second 維持低於每位玩家 5000 bytes per second
- **AND** `omb_app.log` 包含零行 `Removed disconnected KCP session`
- **AND** `omfx_app.log` 不包含持續的 team frame starvation

#### Scenario: forbidden TypedOutbound variants 不被 constructed

- **WHEN** 搜尋 `omb/src/` 中的 `TypedOutbound::EntityDeath`、`TypedOutbound::TowerCreate`、`TypedOutbound::TowerUpgrade`、`TypedOutbound::GameRound`、`TypedOutbound::HeroStatic` 與 `TypedOutbound::HeroHot`
- **THEN** 沒有任何 secure player path 使用這些 variants 作為 payload
- **AND** 對應 KCP routing entries 與 dead builder functions 不存在

#### Scenario: omb lib tests 通過

- **WHEN** Phase 6 執行 `cargo test --manifest-path D:/code/omoba/omb/Cargo.toml -p omobab --lib`
- **THEN** omb library test suite 通過

## ADDED Requirements

### Requirement: Outcome 與 ObservableFact 同步產生

Specs Wave A gameplay system SHALL 在計算 authoritative `Outcome` 時同步產生 projection-ready `ObservableFact`。Parallel buffers SHALL 使用 stable ordering key，並在 deterministic commit barrier 合併排序。Server MUST NOT 以事後 full-world scan 推測 public effect。

#### Scenario: Parallel completion order 不改變 projected events

- **WHEN** 相同 tick 的 gameplay systems 以不同 thread completion order 執行
- **THEN** 合併後的 `Outcome` 與 `ObservableFact` canonical order 相同
- **AND** per-team encoded event bytes 相同

### Requirement: Retained event 也必須經 team projection

仍需保留的 player acknowledgement 或 one-shot terminal event SHALL 經 `TeamViewProjector` 判斷 audience 與 redaction，不得直接 global fan-out。

#### Scenario: Team-private terminal event

- **WHEN** terminal event 只允許 team A 得知
- **THEN** event 只出現在 team A stream
- **AND** team B empty frame 的 cadence/padding 不洩漏該 event detail
