## MODIFIED Requirements

### Requirement: `SimWorldSnapshot` structure 與 read-only-except-queues invariant

`omoba-core::runtime::SimWorldSnapshot` SHALL 包含 omfx render-facing 所需的所有 state，包括 tick、entities、paths、removed entity ids、round data、lives、blocked regions、explosions、ability definitions、tower templates 與 tower upgrade definitions。`omfx` SHALL consume this type directly or through a wrapper/re-export whose source of truth is `omoba-core::runtime`，而不是 `omobab` crate。

snapshot entity data SHALL 包含 optional hero extension data、optional tower upgrade levels，以及 render-safe fixed-point conversions。`omoba-core::runtime::extract_snapshot` SHALL 將 sim ECS world 視為 read-only，唯一例外是用 `std::mem::take(&mut q.pending)` drain producer-consumer queues。它 SHALL NOT write components、create entities、delete entities 或 mutate unrelated resources。Boundary values SHALL 透過 project fixed-point helpers，從 fixed-point 轉成 render `f32`。

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

### Requirement: HUD 從 snapshots 讀取 round、lives 與 running state

`extract_snapshot` SHALL 從 `omoba-core::runtime` sim ECS resources 讀取 round、total rounds、round running state 與 lives：`CurrentCreepWave` 與 `PlayerLives`。omfx HUD SHALL 從 snapshot 讀取這些 values，且 SHALL NOT 對這些 fields 使用 legacy heartbeat 或 mirror state。

#### Scenario: HUD lives 與 round 反映 sim state
- **WHEN** TD_1 進行中且 creep 漏怪
- **THEN** 下一個 snapshot 有 decrement 後的 `lives` value
- **AND** omfx HUD 更新顯示的 lives value

#### Scenario: round_is_running controls wave UI
- **WHEN** 玩家 start round
- **THEN** 下一個 snapshot 有 `round_is_running == true` 與 updated `round`
- **AND** omfx 相應更新 start button 與 wave counter UI

### Requirement: hero stats aggregated into `HeroStatsExt`

`EntityRenderData` SHALL 對 hero entities 包含 `hero_ext: Option<Box<HeroStatsExt>>`。`HeroStatsExt` SHALL 包含 omfx UI 需要的 armor、magic resist、attack damage、attack range、move speed、attack speed seconds、bullet speed、mana、max mana、buffs、inventory、ability levels 與 ability ids。

對每個 Hero entity，`extract_snapshot` SHALL 使用 `omoba-core::runtime::ability_runtime::UnitStats` equivalent 與 final stat accessors 填入 `HeroStatsExt`。omfx hero panel UI SHALL 從 local hero 的 snapshot entity data 讀取 hero stats。Authoritative snapshot values SHALL reset any local buff countdown display between snapshots。

#### Scenario: hero panel 顯示 expected reference stats
- **WHEN** TD_1 載入 reference hero scene
- **THEN** hero panel 顯示來自 `hero_ext` 的 authoritative armor、attack damage、attack speed、range 與 move speed values

#### Scenario: finite buff countdown 在 snapshots 之間保持 smooth
- **WHEN** hero 有一個剩餘 5 秒的 buff
- **THEN** snapshot 回報 `remaining_secs == 5.0`
- **AND** omfx 可在 snapshots 之間用 frame delta decrement displayed value
- **AND** 下一個 snapshot 將 display reset 到 authoritative remaining time

#### Scenario: toggle buff 不倒數
- **WHEN** hero 有 toggle 或 indefinite buff
- **THEN** snapshot 回報 `remaining_secs == -1.0`
- **AND** omfx 不對該 buff 顯示 countdown

### Requirement: tower upgrade definitions 透過 snapshot Arc data 共享

`SimWorldSnapshot.tower_upgrades` SHALL 是從 `omoba-core::runtime::TowerUpgradeRegistry` 建立的 `Arc<Vec<TowerUpgradeDefSnapshot>>`。`TowerUpgradeDefSnapshot` SHALL 包含 tower kind、path、level、name 與 cost。sim worker SHALL build 此 data 一次並為 snapshots clone `Arc`。omfx SHALL 以 `(unit_id, path, level)` cache 這些 definitions，供 sell refund 與 upgrade button text 使用。

#### Scenario: omfx sell refund 與 omb 相符
- **WHEN** player 在買 upgrades 後賣掉 tower
- **THEN** omfx sell panel refund calculation 使用 base tower cost 與 snapshot tower upgrade definitions 中的 upgrade costs
- **AND** displayed refund 與 backend sell logic 相符

#### Scenario: upgrade buttons 顯示 next-level names
- **WHEN** TD_1 player 選中未 upgrade 的 dart monkey tower
- **THEN** 每個 path button 顯示 next level name 與 cost
- **AND** button text 不使用 unsupported unicode pip glyphs

#### Scenario: maxed path 顯示 MAX
- **WHEN** tower path 達到 max level
- **THEN** 該 path 的 upgrade button 顯示 `MAX`

### Requirement: blocked regions 從 snapshots render

`SimWorldSnapshot.blocked_regions` SHALL 從 `omoba-core::runtime::BlockedRegions` resource populate。omfx SHALL 從此 snapshot data render polygon outlines 與 circle outlines。

#### Scenario: DEBUG_1 顯示 region outlines
- **WHEN** `STORY = "DEBUG_1"` 載入含 blocked regions 的 scene
- **THEN** snapshot 包含 blocked region data
- **AND** omfx visibly render region outlines

#### Scenario: TD_1 沒有 region outlines
- **WHEN** TD_1 載入且沒有 blocked regions
- **THEN** `blocked_regions` 為 empty
- **AND** omfx 不 render blocked-region outlines
