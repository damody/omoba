# Selective Lockstep State Inventory

本文件是 server-authoritative selective lockstep 的 state inventory。來源範圍以 `omoba-core/src/runtime/native/comp/**` 為主，並另列 authoritative world 初始化時註冊的 shared component。

## Classification 欄位契約

每一筆 inventory row 必須且只能有一個 `classification`。允許值如下：

| 值 | 意義 |
|---|---|
| `Public` | 所有 player team 都可取得，且不會洩漏 hidden state。 |
| `TeamPrivate` | 只有指定 team 可取得，與 entity visibility 無關。 |
| `VisibilityBound` | 只有 entity／effect 對該 team 已 disclosure 時才可取得。 |
| `ServerOnly` | 不得進入 player wire、client deterministic replica 或 player-visible diagnostics。 |

表格中的 `—` 表示 classification 尚未由後續任務指派；它不是允許值，也不能通過 `G-CONTRACT-STATE` completeness gate。

## Blocking Migration View

| Blocking ID | Selector | Affected rows | Resolution tasks | Gate |
|---|---|---:|---|---|
| `BLK-STATE-UNCLASSIFIED` | 本文件中 `classification == —` 的 row | 0 | `1.1.28`–`1.1.33` | `G-CONTRACT-STATE` |
| `BLK-STATE-DUPLICATE` | 任一 row 的 classification 欄同時含兩個以上允許值 | 0 | 發現時必須先收斂為唯一值 | `G-CONTRACT-STATE` |
| `BLK-PROJECTION-MISSING` | 核准 action family 沒有唯一 `*.v1` 四象限 policy section | 0 | 新增 policy 前必須保持 blocking | `G-CONTRACT-STATE` |

此 view 是 blocking list，不是預設分類。只要 selector 仍匹配任何 row，secure-match contract 就不得視為完成。

## Deterministic Components

下列 component 由 `Component` implementation／derive 或 authoritative world registration 確認。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `RegionBlocker` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `Bounty` | `omoba-core/src/runtime/native/comp/bounty.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `IsBuilding` | `omoba-core/src/runtime/native/comp/building.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `CircularVision` | `omoba-core/src/runtime/native/comp/circular_vision.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | TeamPrivate |
| `Creep` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `CProperty` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `CreepMoveBroadcast` | `omoba-core/src/runtime/native/comp/creep_move_broadcast.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |
| `DamageInstance` | `omoba-core/src/runtime/native/comp/damage.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `DamageResult` | `omoba-core/src/runtime/native/comp/damage.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Facing` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `FacingBroadcast` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |
| `TurnSpeed` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Gold` | `omoba-core/src/runtime/native/comp/gold.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | TeamPrivate |
| `Hero` | `omoba-core/src/runtime/native/comp/hero.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `Inventory` | `omoba-core/src/runtime/native/comp/inventory.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | TeamPrivate |
| `IsBase` | `omoba-core/src/runtime/native/comp/is_base.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `ItemEffects` | `omoba-core/src/runtime/native/comp/item_effects.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | TeamPrivate |
| `Pos` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `Rot` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Vel` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep） | VisibilityBound |
| `MoveTarget` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `HeroCommandQueue` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `PosVelOriDefer` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PreviousPhysCache` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Scale` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Mass` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Sticky` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Immovable` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `ForceUpdate` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `CollisionRadius` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Projectile` | `omoba-core/src/runtime/native/comp/projectile.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `Tower` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | VisibilityBound |
| `TowerSpawnOrder` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `TAttack` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `TProperty` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Unit` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Faction` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `PlayerOwner` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |
| `SummonedUnit` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Last<Pos>` | `omoba-core/src/runtime/native/comp/last.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Last<Vel>` | `omoba-core/src/runtime/native/comp/last.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | ServerOnly |

## Registered Shared Components

這些 type 不定義於 `native/comp/**`，但由 `runtime/native/initialization.rs` 註冊進相同 authoritative world，因此後續分類不得遺漏。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `Enemy` | `omoba-core/src/comp/enemy.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | VisibilityBound |
| `Campaign` | `omoba-core/src/comp/campaign.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `Stage` | `omoba-core/src/comp/campaign.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `Player` | `omoba-core/src/comp/player.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | TeamPrivate |
| `ScriptUnitTag` | `omoba-core/src/runtime/native/scripting/tag.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | VisibilityBound |

## Deterministic Resources

下列 resource 由 `runtime/native/initialization.rs` 的 world insertion、SystemData access 或 snapshot extraction 確認。Queue payload type 不另列為 resource；它由 owning queue row 覆蓋。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `Time` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `DeltaTime` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `GamePause` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `SandboxMode` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `GameSpeed` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `MatchKillCounter` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | ServerOnly |
| `TickStart` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Tick` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep） | Public |
| `MasterSeed` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep） | ServerOnly |
| `TimeOfDay` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `TowerSpawnOrderCounter` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PlayerEconomy` | `omoba-core/src/runtime/native/comp/player_economy.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 是（legacy global hash） | 是（global render） | TeamPrivate |
| `BlockedRegions` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `CurrentCreepWave` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `Vec<CreepWave>` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `Vec<TakenDamage>` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Vec<Outcome>` | `omoba-core/src/runtime/native/comp/outcome.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `KnowledgeBonusResource` | `omoba-core/src/runtime/native/comp/knowledge.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingPlayerInputs` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerSellQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingDebugCreepSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingAbilityUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerAbilityPulseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerAbilityActivationQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `TowerAbilityCastResult` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | TeamPrivate |
| `TowerAbilityCastResults` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | TeamPrivate |
| `PendingItemUseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingMoveQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingHeroCommandClearQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `PendingTowerTargetPriorityQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `SnapshotStore` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `ExplosionFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `TowerFireFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `AttackPhaseFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `AttackCancelFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `RemovedEntitiesQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `GameMode` | `omoba-core/src/runtime/native/comp/game_mode.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `PlayerLives` | `omoba-core/src/runtime/native/comp/game_mode.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | TeamPrivate |
| `CollisionIndex` | `omoba-core/src/runtime/native/comp/collision_index.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `Searcher` | `omoba-core/src/runtime/native/comp/collision_index.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | ServerOnly |
| `TerrainHeightMap` | `omoba-core/src/runtime/native/comp/heightmap.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `TerrainConfig` | `omoba-core/src/runtime/native/comp/heightmap.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `TowerTemplateRegistry` | `omoba-core/src/runtime/native/comp/tower_registry.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `TowerUpgradeRegistry` | `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | Public |
| `BTreeMap<String, CheckPoint>` | `omoba-core/src/runtime/native/comp/check_point.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |
| `BTreeMap<String, Path>` | `omoba-core/src/runtime/native/comp/check_point.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | Public |

## Input Inventory

此處先固定 input envelope 的權責與生命週期；各 `PlayerInput::Action` variant 的 producer／consumer 由 1.1.11 逐項展開。

| Type | Source | Owner | Authoritative phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `InputSubmit` | `proto/game.proto` | Client request；server validation authority | ingest／validate；accepted input 進入 Wave A | 否；只透過造成的 committed state 間接反映 | 否 | TeamPrivate |
| `InputForPlayer` | `proto/game.proto` | Server accepted-input envelope | Wave A input drain；V2 預定投影至 `Step` | 否；transport timing metadata 不納入 gameplay hash | 否 | TeamPrivate |
| `PlayerInput` | `proto/game.proto` | Server accepted-input action | Wave A `player_input_tick` evaluation／deterministic commit | 否；只透過造成的 committed state 間接反映 | 否 | TeamPrivate |

### PlayerInput Variant Inventory

所有 variant 都由 client `InputSubmit` producer 送入、由 server ingest 驗證後才成為 accepted input；表內 consumer 是 authoritative runtime 的下一個實際消費邊界。

| Variant | Producer | Consumer | Authoritative phase | classification |
|---|---|---|---|---|
| `NoOp` | Accepted `InputSubmit` | `player_input_tick` | Wave A input evaluation；無 mutation | TeamPrivate |
| `MoveTo` | Accepted `InputSubmit` | `PendingMoveQueue` → `drain_pending_moves` | Wave A input evaluation／commit | TeamPrivate |
| `AttackTarget` | Accepted `InputSubmit` | `PendingMoveQueue` → `drain_pending_moves` | Wave A input evaluation／commit | TeamPrivate |
| `CastAbility` | Accepted `InputSubmit` | `PendingAbilityCastQueue` → script dispatch | Wave A input evaluation／commit | TeamPrivate |
| `TowerPlace` | Accepted `InputSubmit` | `PendingTowerSpawnQueue` → `drain_pending_tower_spawns` | Wave A input evaluation／commit | TeamPrivate |
| `TowerUpgrade` | Accepted `InputSubmit` | `PendingTowerUpgradeQueue` → `drain_pending_tower_upgrades` | Wave A input evaluation／commit | TeamPrivate |
| `TowerSell` | Accepted `InputSubmit` | `PendingTowerSellQueue` → `drain_pending_tower_sells` | Wave A input evaluation／commit | TeamPrivate |
| `ItemUse` | Accepted `InputSubmit` | `PendingItemUseQueue` → `drain_pending_item_uses` | Wave A input evaluation／commit | TeamPrivate |
| `StartRound` | Accepted `InputSubmit` | `player_input_tick`／`CurrentCreepWave` | Wave A input evaluation／direct deterministic mutation | TeamPrivate |
| `UpgradeAbility` | Accepted `InputSubmit` | `PendingAbilityUpgradeQueue` → `drain_pending_ability_upgrades` | Wave A input evaluation／commit | TeamPrivate |
| `AttackMove` | Accepted `InputSubmit` | `PendingMoveQueue` → `drain_pending_moves` | Wave A input evaluation／commit | TeamPrivate |
| `SetTowerTargetPriority` | Accepted `InputSubmit` | `PendingTowerTargetPriorityQueue` → priority drain | Wave A input evaluation／commit | TeamPrivate |
| `TogglePause` | Accepted `InputSubmit` | `player_input_tick`／`GamePause` | Wave A input evaluation／direct deterministic mutation | TeamPrivate |
| `ToggleGameSpeed` | Accepted `InputSubmit` | `player_input_tick`／`GameSpeed` | Wave A input evaluation／direct deterministic mutation | TeamPrivate |
| `DebugSpawnCreep` | Accepted sandbox `InputSubmit` | `PendingDebugCreepSpawnQueue` → `creep_wave::Sys` | Wave A validation／commit；production 必須拒絕 | TeamPrivate |
| `TowerAbilityCast` | Accepted `InputSubmit` | `PendingTowerAbilityCastQueue` → tower ability dispatch | Wave A input evaluation／commit | TeamPrivate |

## Outcome／Script Event Inventory

| Type | Source | Owner | Authoritative phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `Outcome` | `omoba-core/src/runtime/native/comp/outcome.rs` | Gameplay／script producer；server commit authority | Wave A stable buffer；deterministic reduce／commit | 否；只 hash 套用後的 state subset | 否；render cue 由 commit 後 queue 衍生 | ServerOnly |
| `ScriptEvent` | `omoba-core/src/runtime/native/scripting/event.rs` | Authoritative script dispatcher | Wave A evaluation；script dispatch 產生 `Outcome` | 否；只 hash committed outcome state | 否 | ServerOnly |
| `ScriptVisualEvent` | `omoba-core/src/runtime/native/scripting/event.rs` | Commit-side presentation adapter | Post-commit render extraction；drain-on-snapshot | 否 | 是（global render snapshot） | VisibilityBound |
| `RuntimeEvent` | `omoba-core/src/runtime/native/events.rs` | Deterministic runtime producer；backend projection adapter | Wave A/post-commit transport projection | 否；transport metadata 不進 gameplay hash | 否；retained network 規則另於 1.1.27 | ServerOnly |

### ScriptEvent Variant Inventory

| Variant | Producer | Consumer | Authoritative phase | classification |
|---|---|---|---|---|
| `Spawn` | entity spawn commit | script `on_spawn` dispatch | Wave A commit-adjacent script dispatch | ServerOnly |
| `Death` | death commit | script `on_death` dispatch | Wave A commit-adjacent script dispatch | ServerOnly |
| `Respawn` | lifecycle commit | script `on_respawn` dispatch | Wave A commit-adjacent script dispatch | ServerOnly |
| `Damage` | damage pipeline pre-HP mutation | script damage hook → adjusted `Outcome` | Wave A evaluation before damage commit | ServerOnly |
| `AttackHit` | attack/projectile resolution | script hit hook | Wave A evaluation | ServerOnly |
| `ProjectileHit` | projectile resolution | script projectile-hit hook | Wave A evaluation | ServerOnly |
| `AttackStart` | attack scheduler | script attack-start hook | Wave A evaluation | ServerOnly |
| `AttackLanded` | confirmed hit | script landed hook | Wave A evaluation | ServerOnly |
| `AttackFail` | miss/evasion resolution | script fail hook | Wave A evaluation | ServerOnly |
| `Attacked` | victim-side attack resolution | script attacked hook | Wave A evaluation | ServerOnly |
| `HealthGained` | heal/resource pipeline | script health-gained hook | Wave A evaluation | ServerOnly |
| `ManaGained` | resource pipeline | script mana-gained hook | Wave A evaluation | ServerOnly |
| `SpentMana` | accepted ability cast | script spent-mana hook | Wave A evaluation | ServerOnly |
| `HealReceived` | heal pipeline | script heal-received hook | Wave A evaluation | ServerOnly |
| `StateChanged` | state controller | script state-change hook | Wave A evaluation | ServerOnly |
| `ModifierAdded` | BuffStore commit request | script modifier-added hook | Wave A evaluation／commit | ServerOnly |
| `ModifierRemoved` | BuffStore removal request | script modifier-removed hook | Wave A evaluation／commit | ServerOnly |
| `SkillCast` | accepted ability queue | ability script `on_cast` → `Outcome` | Wave A evaluation | ServerOnly |
| `SkillLearn` | ability upgrade commit | ability script `on_learn` → `Outcome` | Wave A evaluation／commit | ServerOnly |
| `Order` | accepted hero command | script order hook | Wave A evaluation | ServerOnly |

### Outcome Variant Inventory

除表內特別標示的 presentation cue 外，consumer 均由 `GameProcessor::process_outcomes` 在 stable reduce 後依 canonical order commit；producer 欄記錄主要來源族群，不授予 producer 直接改 authoritative world 的權限。

| Variant | Producer | Consumer | Authoritative phase | classification |
|---|---|---|---|---|
| `Damage` | combat／script adapter | damage commit pipeline | Wave A reduce／commit | ServerOnly |
| `ProjectileHit` | projectile tick／script | hit dispatch + script event | Wave A reduce／commit | ServerOnly |
| `ProjectileLine2` | projectile tick | projectile spawn commit | Wave A reduce／commit | ServerOnly |
| `Death` | combat/creep tick | death commit + removal scheduling | Wave A reduce／commit | ServerOnly |
| `Creep` | creep wave／debug spawn | creep spawn commit | Wave A reduce／commit | ServerOnly |
| `CreepUpdate` | creep tick | movement/state commit | Wave A reduce／commit | ServerOnly |
| `CreepStop` | creep combat | creep stop commit | Wave A reduce／commit | ServerOnly |
| `CreepWalk` | creep lifecycle | creep walk commit | Wave A reduce／commit | ServerOnly |
| `Tower` | input/script | tower spawn commit | Wave A reduce／commit | ServerOnly |
| `Heal` | gameplay system | heal commit | Wave A reduce／commit | ServerOnly |
| `UpdateAttack` | tower/attack tick | attack state commit | Wave A reduce／commit | ServerOnly |
| `GainExperience` | death/reward pipeline | hero progression commit | Wave A reduce／commit | ServerOnly |
| `GainGold` | death/reward pipeline | economy commit | Wave A reduce／commit | ServerOnly |
| `SpawnUnit` | script adapter | summon spawn commit | Wave A reduce／commit | ServerOnly |
| `CreepLeaked` | creep path completion | lives/economy commit | Wave A reduce／commit | ServerOnly |
| `AddBuff` | ability/item/script | `BuffStore` commit | Wave A reduce／commit | ServerOnly |
| `Explosion` | projectile/script | `ExplosionFxQueue` | Post-commit presentation cue | ServerOnly |
| `ProjectileDirectional` | projectile/script | projectile spawn commit | Wave A reduce／commit | ServerOnly |
| `AttackPhaseCue` | attack tick | `AttackPhaseFxQueue` | Post-commit presentation cue | ServerOnly |
| `ScriptSetPos` | script world adapter | `Pos` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetFacing` | script world adapter | `Facing` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetAsdCount` | script world adapter | `TAttack` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetTowerAtk` | script world adapter | `TProperty` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetTowerRange` | script world adapter | `TProperty` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetAsdInterval` | script world adapter | `TAttack` commit | Wave A reduce／commit | ServerOnly |
| `ScriptSetTowerInternalCooldown` | script world adapter | tower script state commit | Wave A reduce／commit | ServerOnly |
| `ScriptDirectDamage` | script world adapter | damage commit pipeline | Wave A reduce／commit | ServerOnly |
| `ScriptHeal` | script world adapter | heal commit pipeline | Wave A reduce／commit | ServerOnly |
| `ScriptRemoveBuff` | script world adapter | `BuffStore` commit | Wave A reduce／commit | ServerOnly |
| `ScriptProjectile` | script world adapter | projectile spawn commit | Wave A reduce／commit | ServerOnly |
| `ScriptTowerFireFx` | script world adapter | `TowerFireFxQueue` | Post-commit presentation cue | ServerOnly |
| `ScriptAttackPhaseCue` | script world adapter | `AttackPhaseFxQueue` | Post-commit presentation cue | ServerOnly |
| `ScriptStartCooldown` | script world adapter | ability cooldown commit | Wave A reduce／commit | ServerOnly |
| `EntityRemoved` | lifecycle/script adapter | entity deletion + removed queue | Wave A reduce／commit boundary | ServerOnly |

## Render Cue Inventory

這些 cue 是 presentation data，不得回饋 simulation、targeting、collision 或 deterministic hash。現行 global render snapshot 會 drain queue；V2 必須先依 team projection policy 過濾，再進 team-local retention。

| Cue | Owner | Producer | Consumer | Retention rule | Hash | Snapshot | classification |
|---|---|---|---|---|---|---|---|
| `ExplosionFx` | Commit-side presentation adapter | committed `Outcome::Explosion` | `SimWorldSnapshot.explosions` → omfx | Queue 保留至下一次 snapshot extract 後 drain；V2 frame 只保留到 delivery/replay window | 否 | global render | VisibilityBound |
| `TowerFireFx` | Commit-side presentation adapter | committed `Outcome::ScriptTowerFireFx`／tower attack | `SimWorldSnapshot.tower_fire_fx` → omfx | Queue 保留至下一次 snapshot extract 後 drain；依 spawn tick 去重 | 否 | global render | VisibilityBound |
| `AttackPhaseFx` | Commit-side presentation adapter | committed attack phase outcome | `SimWorldSnapshot.attack_phase_fx` → omfx | Queue 保留至下一次 snapshot extract 後 drain；以 entity + attack sequence 去重 | 否 | global render | VisibilityBound |
| `AttackCancelFx` | Commit-side presentation adapter | attack cancellation commit | `SimWorldSnapshot.attack_cancel_fx` → omfx | Queue 保留至下一次 snapshot extract 後 drain；只取消同 sequence cue | 否 | global render | VisibilityBound |
| `ScriptVisualEvent` | Script presentation adapter | script dispatch hook aggregation | `SimWorldSnapshot.script_visual_events` → omfx | 同 tick `Tick` hook 可聚合；snapshot extract 後 drain；不得進 remembered gameplay state | 否 | global render | VisibilityBound |

## Retained Network Event Inventory

| Event | Owner | Producer | Consumer | Current retention rule | Hash | Snapshot | classification |
|---|---|---|---|---|---|---|---|
| `LockstepFrame::TickBatch` | Legacy lockstep transport | legacy `TickBroadcaster` | every legacy player session | Outbound queue only；沒有可要求重播的 bounded replay ring | 否 | 否 | ServerOnly |
| `LockstepFrame::StateHash` | Legacy lockstep diagnostics | `state_hash_producer`／broadcaster | every legacy player session | Producer channel保留最新值；約 10 秒 checkpoint，沒有 per-team history | 攜帶 legacy global hash；event envelope 不入 hash | 否 | ServerOnly |
| `LockstepFrame::GameStart` | Legacy join bootstrap | join handler | single joining session | One-shot bootstrap envelope；送出後不 retention | 否 | 內嵌 global initial state／master seed | ServerOnly |
| `LockstepFrame::SnapshotResp` | Legacy snapshot bootstrap | `SnapshotStore`／join handler | single joining session | `SnapshotStore` 只保留最新 global snapshot bytes；V2 禁止 player 使用 | 否 | global lockstep snapshot | ServerOnly |

## omb Server-owned Component／Resource Inventory

「server-owned」表示 type 或 state 定義在 `omb`，不預先等同 `ServerOnly` classification；後續分類任務仍必須逐項決定 disclosure policy。

| Type／State | Kind | Source | classification |
|---|---|---|---|
| `Campaign` | Component | `omb/src/comp/campaign.rs` | Public |
| `Stage` | Component | `omb/src/comp/campaign.rs` | Public |
| `Enemy` | Component | `omb/src/comp/enemy.rs` | VisibilityBound |
| `Player` | Component | `omb/src/comp/player.rs` | TeamPrivate |
| `CircularVision` | Legacy component | `omb/src/comp/vision/components.rs` | ServerOnly |
| `SysMetrics` | Diagnostic ECS resource | `omb/src/comp/ecs.rs` | ServerOnly |
| `TickProfile` | Diagnostic ECS resource | `omb/src/comp/tick_profile.rs` | ServerOnly |
| `Clock` | Server loop timing state | `omb/src/comp/clock.rs` | ServerOnly |
| `VisionSystemManager` | Legacy vision runtime state | `omb/src/comp/circular_vision_refactored.rs` | ServerOnly |
| `ResultManager` | Legacy vision result state | `omb/src/comp/vision/result_manager.rs` | ServerOnly |
| `Vec<Sender<OutboundMsg>>` | Backend transport ECS resource | `omb/src/state/core.rs` | ServerOnly |
| `client_viewports` | Per-session server state | `omb/src/state/core.rs` | ServerOnly |
| `client_visibility`／`VisSet` | Legacy per-session visibility state | `omb/src/state/core.rs` | ServerOnly |
| `hb_last_hp_sent` | Legacy heartbeat cache | `omb/src/state/core.rs` | ServerOnly |
| `hb_last_full_send` | Legacy heartbeat cache | `omb/src/state/core.rs` | ServerOnly |
| `last_visibility_tick` | Legacy visibility scheduler state | `omb/src/state/core.rs` | ServerOnly |
| `aoi_grid` | Legacy AOI acceleration state | `omb/src/state/core.rs` | ServerOnly |
| `state_hash_tx` | Global hash transport state | `omb/src/state/core.rs` | ServerOnly |
| `snapshot_store` | Global snapshot transport state | `omb/src/state/core.rs` | ServerOnly |
| `host_input_rx` | Authoritative input channel state | `omb/src/state/core.rs` | ServerOnly |

## Inventory 來源與更新規則

- Component 基準：`runtime/native/initialization.rs` 的 `register::<T>()`，加上 `native/comp/**` 的 `Component` implementation／derive。
- Resource 基準：authoritative initialization 的 `World::insert`、system `Read/Write` access 與 snapshot extraction。
- 新增 component/resource 時，必須在同一變更更新本文件；未分類 row 進入 blocking migration list，secure match 不得以 default disclosure 啟動。
