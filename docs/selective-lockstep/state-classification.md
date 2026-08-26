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
| `BLK-STATE-UNCLASSIFIED` | 本文件中 `classification == —` 的 row | 116 | `1.1.28`–`1.1.33` | `G-CONTRACT-STATE` |

此 view 是 blocking list，不是預設分類。只要 selector 仍匹配任何 row，secure-match contract 就不得視為完成。

## Deterministic Components

下列 component 由 `Component` implementation／derive 或 authoritative world registration 確認。

| Type | Source | classification |
|---|---|---|
| `RegionBlocker` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | — |
| `Bounty` | `omoba-core/src/runtime/native/comp/bounty.rs` | — |
| `IsBuilding` | `omoba-core/src/runtime/native/comp/building.rs` | — |
| `CircularVision` | `omoba-core/src/runtime/native/comp/circular_vision.rs` | — |
| `Creep` | `omoba-core/src/runtime/native/comp/creep.rs` | — |
| `CProperty` | `omoba-core/src/runtime/native/comp/creep.rs` | — |
| `CreepMoveBroadcast` | `omoba-core/src/runtime/native/comp/creep_move_broadcast.rs` | — |
| `DamageInstance` | `omoba-core/src/runtime/native/comp/damage.rs` | — |
| `DamageResult` | `omoba-core/src/runtime/native/comp/damage.rs` | — |
| `Facing` | `omoba-core/src/runtime/native/comp/facing.rs` | — |
| `FacingBroadcast` | `omoba-core/src/runtime/native/comp/facing.rs` | — |
| `TurnSpeed` | `omoba-core/src/runtime/native/comp/facing.rs` | — |
| `Gold` | `omoba-core/src/runtime/native/comp/gold.rs` | — |
| `Hero` | `omoba-core/src/runtime/native/comp/hero.rs` | — |
| `Inventory` | `omoba-core/src/runtime/native/comp/inventory.rs` | — |
| `IsBase` | `omoba-core/src/runtime/native/comp/is_base.rs` | — |
| `ItemEffects` | `omoba-core/src/runtime/native/comp/item_effects.rs` | — |
| `Pos` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Rot` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Vel` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `MoveTarget` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `HeroCommandQueue` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `PosVelOriDefer` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `PreviousPhysCache` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Scale` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Mass` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Sticky` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Immovable` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `ForceUpdate` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `CollisionRadius` | `omoba-core/src/runtime/native/comp/phys.rs` | — |
| `Projectile` | `omoba-core/src/runtime/native/comp/projectile.rs` | — |
| `Tower` | `omoba-core/src/runtime/native/comp/tower.rs` | — |
| `TowerSpawnOrder` | `omoba-core/src/runtime/native/comp/tower.rs` | — |
| `TAttack` | `omoba-core/src/runtime/native/comp/tower.rs` | — |
| `TProperty` | `omoba-core/src/runtime/native/comp/tower.rs` | — |
| `Unit` | `omoba-core/src/runtime/native/comp/unit.rs` | — |
| `Faction` | `omoba-core/src/runtime/native/comp/unit.rs` | — |
| `PlayerOwner` | `omoba-core/src/runtime/native/comp/unit.rs` | — |
| `SummonedUnit` | `omoba-core/src/runtime/native/comp/unit.rs` | — |
| `Last<Pos>` | `omoba-core/src/runtime/native/comp/last.rs` | — |
| `Last<Vel>` | `omoba-core/src/runtime/native/comp/last.rs` | — |

## Registered Shared Components

這些 type 不定義於 `native/comp/**`，但由 `runtime/native/initialization.rs` 註冊進相同 authoritative world，因此後續分類不得遺漏。

| Type | Source | classification |
|---|---|---|
| `Enemy` | `omoba-core/src/comp/enemy.rs` | — |
| `Campaign` | `omoba-core/src/comp/campaign.rs` | — |
| `Stage` | `omoba-core/src/comp/campaign.rs` | — |
| `Player` | `omoba-core/src/comp/player.rs` | — |
| `ScriptUnitTag` | `omoba-core/src/runtime/native/scripting/tag.rs` | — |

## Deterministic Resources

下列 resource 由 `runtime/native/initialization.rs` 的 world insertion、SystemData access 或 snapshot extraction 確認。Queue payload type 不另列為 resource；它由 owning queue row 覆蓋。

| Type | Source | classification |
|---|---|---|
| `Time` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `DeltaTime` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `GamePause` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `SandboxMode` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `GameSpeed` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `MatchKillCounter` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `TickStart` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `Tick` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `MasterSeed` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `TimeOfDay` | `omoba-core/src/runtime/native/comp/resources.rs` | — |
| `TowerSpawnOrderCounter` | `omoba-core/src/runtime/native/comp/tower.rs` | — |
| `PlayerEconomy` | `omoba-core/src/runtime/native/comp/player_economy.rs` | — |
| `BlockedRegions` | `omoba-core/src/runtime/native/comp/blocked_region.rs` | — |
| `CurrentCreepWave` | `omoba-core/src/runtime/native/comp/creep.rs` | — |
| `Vec<CreepWave>` | `omoba-core/src/runtime/native/comp/creep.rs` | — |
| `Vec<TakenDamage>` | `omoba-core/src/runtime/native/comp/creep.rs` | — |
| `Vec<Outcome>` | `omoba-core/src/runtime/native/comp/outcome.rs` | — |
| `KnowledgeBonusResource` | `omoba-core/src/runtime/native/comp/knowledge.rs` | — |
| `PendingPlayerInputs` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerSellQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingDebugCreepSpawnQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingAbilityUpgradeQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerAbilityPulseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerAbilityCastQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerAbilityActivationQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `TowerAbilityCastResult` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `TowerAbilityCastResults` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingItemUseQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingMoveQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingHeroCommandClearQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `PendingTowerTargetPriorityQueue` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `SnapshotStore` | `omoba-core/src/runtime/native/comp/lockstep_resources.rs` | — |
| `ExplosionFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | — |
| `TowerFireFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | — |
| `AttackPhaseFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | — |
| `AttackCancelFxQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | — |
| `RemovedEntitiesQueue` | `omoba-core/src/runtime/native/comp/fx_queues.rs` | — |
| `GameMode` | `omoba-core/src/runtime/native/comp/game_mode.rs` | — |
| `PlayerLives` | `omoba-core/src/runtime/native/comp/game_mode.rs` | — |
| `CollisionIndex` | `omoba-core/src/runtime/native/comp/collision_index.rs` | — |
| `Searcher` | `omoba-core/src/runtime/native/comp/collision_index.rs` | — |
| `TerrainHeightMap` | `omoba-core/src/runtime/native/comp/heightmap.rs` | — |
| `TerrainConfig` | `omoba-core/src/runtime/native/comp/heightmap.rs` | — |
| `TowerTemplateRegistry` | `omoba-core/src/runtime/native/comp/tower_registry.rs` | — |
| `TowerUpgradeRegistry` | `omoba-core/src/runtime/native/comp/tower_upgrade_registry.rs` | — |
| `BTreeMap<String, CheckPoint>` | `omoba-core/src/runtime/native/comp/check_point.rs` | — |
| `BTreeMap<String, Path>` | `omoba-core/src/runtime/native/comp/check_point.rs` | — |

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
