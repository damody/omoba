## MODIFIED Requirements

### Requirement: `SimWorldSnapshot` structure 與 read-only-except-queues invariant

`omoba-core::runtime::SimWorldSnapshot` SHALL 包含 omfx render-facing 所需的所有 state，包括 tick、entities、paths、removed entity ids、round data、lives、blocked regions、explosions、ability definitions、tower templates 與 tower upgrade definitions。`omfx` SHALL consume this type directly or through a wrapper/re-export whose source of truth is `omoba-core::runtime`，而不是 `omobab` crate。在 secure V2 path，snapshot MUST 只含該 team 已 disclosed 的 entity 與 render-safe team/public metadata；fog demo 的 disclosed count SHALL 只由此 filtered entity collection計算。

snapshot entity data SHALL 包含 optional hero extension data、optional tower upgrade levels，以及 render-safe fixed-point conversions。`omoba-core::runtime::extract_snapshot` SHALL 將 sim ECS world 視為 read-only，唯一例外是用 `std::mem::take(&mut q.pending)` drain producer-consumer queues。它 SHALL NOT write components、create entities、delete entities 或 mutate unrelated resources。Boundary values SHALL 透過 project fixed-point helpers，從 fixed-point 轉成 render `f32`。Fog demo 的 `LastKnown` records SHALL 位於獨立 render-only cache，MUST NOT 混入 snapshot deterministic entity list、target lookup、collision 或 team hash。

#### Scenario: extract_snapshot 只 drain outcome queues

- **WHEN** 搜尋 `omoba-core::runtime::extract_snapshot` implementation 中的 `write_storage`、`write_resource`、`entities.create` 與 `entities.delete`
- **THEN** 唯一允許的 writes 是 `RemovedEntitiesQueue`、`ExplosionFxQueue`、`TowerFireFxQueue` 與 `AttackPhaseFxQueue` 的 `mem::take` drains
- **AND** 沒有 component writes、entity creates 或 entity deletes
- **AND** implementation path 不在 `omfx/game/src` 且不透過 `omobab::*` 取得 ECS types

#### Scenario: omoba-sim determinism tests 通過

- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** omoba-sim determinism test suite 通過，包含 pin-hash tests

#### Scenario: `Outcome::EntityRemoved` 在同 tick boundary delete

- **WHEN** system 將 `Outcome::EntityRemoved { entity: e }` push 到 world outcome resource
- **THEN** `process_outcomes` 將 `e.id()` push 到 `RemovedEntitiesQueue.pending` 並呼叫 `entities().delete(e)`
- **AND** `world.maintain()` 後該 entity 不再 alive
- **AND** state hashing 在該 tick boundary 後不再包含被刪除的 entity

#### Scenario: Fog demo snapshot 不含 hidden entity
- **WHEN** grid unit 對 Team 1 hidden、對 Team 2 visible
- **THEN** Team 1 `SimWorldSnapshot.entities` 不含該單位
- **AND** Team 2 snapshot 含該單位的 disclosed render state

#### Scenario: LastKnown 與 live replica 分離
- **WHEN** 曾揭露的單位離開 team visibility
- **THEN** live filtered entity list 移除該單位
- **AND** renderer 可從獨立 cache 顯示低透明度 ghost
- **AND** gameplay query 與 team hash無法讀取該 ghost

