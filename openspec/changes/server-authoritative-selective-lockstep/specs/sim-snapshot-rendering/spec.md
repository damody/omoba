## MODIFIED Requirements

### Requirement: `SimWorldSnapshot` structure 與 read-only-except-queues invariant

Secure V2 path 的 `omoba-core::runtime::SimWorldSnapshot` SHALL 只包含該 team 已 disclosed 的 render-facing state，包括 tick、disclosed entities、public/team-private round data、paths、blocked regions、VFX cue 與 safe definitions。omfx SHALL 從 `SelectiveReplicaRuntime` 直接取得此 filtered snapshot，而不是 `omobab` crate 或 global authoritative snapshot。

Snapshot entity data SHALL 使用 team-scoped replica ID，並可包含 optional hero/tower extension、upgrade level 與 render-safe fixed-point conversion。Snapshot extraction SHALL 將 replica world 視為 read-only，唯一例外是以 `std::mem::take` drain render-only producer-consumer queues；它 MUST NOT create/delete entity 或修改 gameplay component。Remembered record SHALL 位於獨立 render cache，不得混入 deterministic entity list。

#### Scenario: Filtered extract_snapshot 只 drain render queues

- **WHEN** 搜尋 selective replica snapshot extraction 的 `write_storage`、`write_resource`、entity create/delete
- **THEN** 唯一允許 write 是明確列入 contract 的 render-only queue drain
- **AND** 沒有 gameplay component write、entity create 或 entity delete
- **AND** implementation 位於 `omoba-core::runtime`，不透過 `omobab::*` 取得 ECS types

#### Scenario: Hidden entity 不進 snapshot

- **WHEN** authoritative world 有 team A 未 disclosed 的 entity
- **THEN** team A `SimWorldSnapshot.entities` 不包含該 entity 或可對應 canonical ID
- **AND** render metadata/queue 不旁漏該 entity

#### Scenario: `Outcome::EntityRemoved` 在同 tick boundary delete

- **WHEN** disclosed replica system 套用 authoritative remove/forget transition
- **THEN** entity 在指定 barrier 從 deterministic replica 移除
- **AND** 下一個 filtered render snapshot 不再包含該 entity
- **AND** remembered presentation 只依 `RememberPolicy` 另行保存

## ADDED Requirements

### Requirement: Filtered snapshot 只由 team bootstrap/rebase 使用

Network delivery 的 filtered snapshot SHALL 只用於 V2 team join、rejoin、observer rebootstrap 或 replay ring 過期後的 authority rebase。Gameplay input MUST NOT 同步觸發 snapshot response。Player session MUST NOT 取得 global `SnapshotStore` bytes。

#### Scenario: Rejoin 取得 team snapshot

- **WHEN** player rejoin secure match
- **THEN** server 送出所屬 team 的 filtered snapshot
- **AND** snapshot 不包含其他 team hidden state 或 global seed

### Requirement: Remembered render cache 與 replica state 分離

omfx SHALL 以獨立 cache 保存 `LastKnown`/custom remembered presentation。該 cache MUST NOT 被 `SelectiveReplicaRuntime` gameplay query、input target lookup 或 team hash讀取。

#### Scenario: Remembered ghost 不在 team hash

- **WHEN** remembered presentation 被建立、更新顯示樣式或移除
- **THEN** deterministic team hash 不變
