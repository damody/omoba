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
| `BLK-STATE-UNCLASSIFIED` | 本文件中 `classification == —` 的 row | 119 | `1.1.28`–`1.1.33` | `G-CONTRACT-STATE` |

此 view 是 blocking list，不是預設分類。只要 selector 仍匹配任何 row，secure-match contract 就不得視為完成。

## Deterministic Components

下列 component 由 `Component` implementation／derive 或 authoritative world registration 確認。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `RegionBlocker` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Bounty` | `omoba-core/src/runtime/native/comp/bounty.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `IsBuilding` | `omoba-core/src/runtime/native/comp/building.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `CircularVision` | `omoba-core/src/runtime/native/comp/circular_vision.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Creep` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `CProperty` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | — |
| `CreepMoveBroadcast` | `omoba-core/src/runtime/native/comp/creep_move_broadcast.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |
| `DamageInstance` | `omoba-core/src/runtime/native/comp/damage.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `DamageResult` | `omoba-core/src/runtime/native/comp/damage.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Facing` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | — |
| `FacingBroadcast` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |
| `TurnSpeed` | `omoba-core/src/runtime/native/comp/facing.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Gold` | `omoba-core/src/runtime/native/comp/gold.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `Hero` | `omoba-core/src/runtime/native/comp/hero.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | — |
| `Inventory` | `omoba-core/src/runtime/native/comp/inventory.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `IsBase` | `omoba-core/src/runtime/native/comp/is_base.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `ItemEffects` | `omoba-core/src/runtime/native/comp/item_effects.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Pos` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep、global render） | — |
| `Rot` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Vel` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 是（legacy global hash） | 是（legacy lockstep） | — |
| `MoveTarget` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `HeroCommandQueue` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `PosVelOriDefer` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |
| `PreviousPhysCache` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |
| `Scale` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Mass` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Sticky` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Immovable` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `ForceUpdate` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `CollisionRadius` | `omoba-core/src/runtime/native/comp/phys.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Projectile` | `omoba-core/src/runtime/native/comp/projectile.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | — |
| `Tower` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep、global render） | — |
| `TowerSpawnOrder` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `TAttack` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `TProperty` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Unit` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Faction` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `PlayerOwner` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `SummonedUnit` | `omoba-core/src/runtime/native/comp/unit.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Last<Pos>` | `omoba-core/src/runtime/native/comp/last.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |
| `Last<Vel>` | `omoba-core/src/runtime/native/comp/last.rs` | Authoritative ECS | Wave A 後段衍生／legacy projection cache | 否（目前 legacy global hash） | 否 | — |

## Registered Shared Components

這些 type 不定義於 `native/comp/**`，但由 `runtime/native/initialization.rs` 註冊進相同 authoritative world，因此後續分類不得遺漏。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `Enemy` | `omoba-core/src/comp/enemy.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Campaign` | `omoba-core/src/comp/campaign.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Stage` | `omoba-core/src/comp/campaign.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Player` | `omoba-core/src/comp/player.rs` | Authoritative ECS | 初始化；match progression deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `ScriptUnitTag` | `omoba-core/src/runtime/native/scripting/tag.rs` | Authoritative ECS | 初始化；Wave A evaluation／deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |

## Deterministic Resources

下列 resource 由 `runtime/native/initialization.rs` 的 world insertion、SystemData access 或 snapshot extraction 確認。Queue payload type 不另列為 resource；它由 owning queue row 覆蓋。

| Type | Source | Owner | Mutation phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `Time` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `DeltaTime` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `GamePause` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `SandboxMode` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `GameSpeed` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `MatchKillCounter` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `TickStart` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Tick` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep） | — |
| `MasterSeed` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（legacy lockstep） | — |
| `TimeOfDay` | `omoba-core/src/runtime/native/comp/resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `TowerSpawnOrderCounter` | `omoba-core/src/runtime/native/comp/tower.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `PlayerEconomy` | `omoba-core/src/runtime/native/comp/player_economy.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 是（legacy global hash） | 是（global render） | — |
| `BlockedRegions` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `CurrentCreepWave` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `Vec<CreepWave>` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `Vec<TakenDamage>` | `omoba-core/src/runtime/native/comp/creep.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Vec<Outcome>` | `omoba-core/src/runtime/native/comp/outcome.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `KnowledgeBonusResource` | `omoba-core/src/runtime/native/comp/knowledge.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `PendingPlayerInputs` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerSellQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingDebugCreepSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingAbilityUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerAbilityPulseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerAbilityActivationQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `TowerAbilityCastResult` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `TowerAbilityCastResults` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `PendingItemUseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingMoveQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingHeroCommandClearQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `PendingTowerTargetPriorityQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `SnapshotStore` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `ExplosionFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `TowerFireFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `AttackPhaseFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `AttackCancelFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `RemovedEntitiesQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | Authoritative resource | Wave A transient buffer；barrier drain／commit | 否（目前 legacy global hash） | 否 | — |
| `GameMode` | `omoba-core/src/runtime/native/comp/game_mode.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `PlayerLives` | `omoba-core/src/runtime/native/comp/game_mode.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `CollisionIndex` | `omoba-core/src/runtime/native/comp/collision_index.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `Searcher` | `omoba-core/src/runtime/native/comp/collision_index.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `TerrainHeightMap` | `omoba-core/src/runtime/native/comp/heightmap.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `TerrainConfig` | `omoba-core/src/runtime/native/comp/heightmap.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `TowerTemplateRegistry` | `omoba-core/src/runtime/native/comp/tower_registry.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `TowerUpgradeRegistry` | `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 是（global render） | — |
| `BTreeMap<String, CheckPoint>` | `omoba-core/src/runtime/native/comp/check_point.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |
| `BTreeMap<String, Path>` | `omoba-core/src/runtime/native/comp/check_point.rs` | Authoritative resource | 初始化；Wave A deterministic commit | 否（目前 legacy global hash） | 否 | — |

## Input Inventory

此處先固定 input envelope 的權責與生命週期；各 `PlayerInput::Action` variant 的 producer／consumer 由 1.1.11 逐項展開。

| Type | Source | Owner | Authoritative phase | Current global hash | Current snapshot | classification |
|---|---|---|---|---|---|---|
| `InputSubmit` | `proto/game.proto` | Client request；server validation authority | ingest／validate；accepted input 進入 Wave A | 否；只透過造成的 committed state 間接反映 | 否 | — |
| `InputForPlayer` | `proto/game.proto` | Server accepted-input envelope | Wave A input drain；V2 預定投影至 `Step` | 否；transport timing metadata 不納入 gameplay hash | 否 | — |
| `PlayerInput` | `proto/game.proto` | Server accepted-input action | Wave A `player_input_tick` evaluation／deterministic commit | 否；只透過造成的 committed state 間接反映 | 否 | — |

## omb Server-owned Component／Resource Inventory

「server-owned」表示 type 或 state 定義在 `omb`，不預先等同 `ServerOnly` classification；後續分類任務仍必須逐項決定 disclosure policy。

| Type／State | Kind | Source | classification |
|---|---|---|---|
| `Campaign` | Component | `omb/src/comp/campaign.rs` | — |
| `Stage` | Component | `omb/src/comp/campaign.rs` | — |
| `Enemy` | Component | `omb/src/comp/enemy.rs` | — |
| `Player` | Component | `omb/src/comp/player.rs` | — |
| `CircularVision` | Legacy component | `omb/src/comp/vision/components.rs` | — |
| `SysMetrics` | Diagnostic ECS resource | `omb/src/comp/ecs.rs` | — |
| `TickProfile` | Diagnostic ECS resource | `omb/src/comp/tick_profile.rs` | — |
| `Clock` | Server loop timing state | `omb/src/comp/clock.rs` | — |
| `VisionSystemManager` | Legacy vision runtime state | `omb/src/comp/circular_vision_refactored.rs` | — |
| `ResultManager` | Legacy vision result state | `omb/src/comp/vision/result_manager.rs` | — |
| `Vec<Sender<OutboundMsg>>` | Backend transport ECS resource | `omb/src/state/core.rs` | — |
| `client_viewports` | Per-session server state | `omb/src/state/core.rs` | — |
| `client_visibility`／`VisSet` | Legacy per-session visibility state | `omb/src/state/core.rs` | — |
| `hb_last_hp_sent` | Legacy heartbeat cache | `omb/src/state/core.rs` | — |
| `hb_last_full_send` | Legacy heartbeat cache | `omb/src/state/core.rs` | — |
| `last_visibility_tick` | Legacy visibility scheduler state | `omb/src/state/core.rs` | — |
| `aoi_grid` | Legacy AOI acceleration state | `omb/src/state/core.rs` | — |
| `state_hash_tx` | Global hash transport state | `omb/src/state/core.rs` | — |
| `snapshot_store` | Global snapshot transport state | `omb/src/state/core.rs` | — |
| `host_input_rx` | Authoritative input channel state | `omb/src/state/core.rs` | — |

## Inventory 來源與更新規則

- Component 基準：`runtime/native/initialization.rs` 的 `register::<T>()`，加上 `native/comp/**` 的 `Component` implementation／derive。
- Resource 基準：authoritative initialization 的 `World::insert`、system `Read/Write` access 與 snapshot extraction。
- 新增 component/resource 時，必須在同一變更更新本文件；未分類 row 進入 blocking migration list，secure match 不得以 default disclosure 啟動。
