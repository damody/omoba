## Purpose

定義 omfx 用來 render lockstep simulation state 的 snapshot data contract，包含 HUD state、entity removal、hero 與 tower UI data、VFX、blocked regions 與 path styling。

## Requirements

### Requirement: `SimWorldSnapshot` structure 與 read-only-except-queues invariant

`omfx/game/src/sim_runner.rs::SimWorldSnapshot` SHALL 包含 omfx render-facing 所需的所有 state，包括 tick、entities、paths、removed entity ids、round data、lives、blocked regions、explosions、ability definitions、tower templates 與 tower upgrade definitions。

snapshot entity data SHALL 包含 optional hero extension data、optional tower upgrade levels，以及 render-safe fixed-point conversions。`extract_snapshot` SHALL 將 sim ECS world 視為 read-only，唯一例外是用 `std::mem::take(&mut q.pending)` drain producer-consumer queues。它 SHALL NOT write components、create entities、delete entities 或 mutate unrelated resources。Boundary values SHALL 透過 project fixed-point helpers，從 fixed-point 轉成 render `f32`。

#### Scenario: extract_snapshot 只 drain outcome queues

- **WHEN** 搜尋 `omfx/game/src/sim_runner.rs::extract_snapshot` 中的 `write_storage`、`write_resource`、`entities.create` 與 `entities.delete`
- **THEN** 唯一允許的 writes 是 `RemovedEntitiesQueue`、`ExplosionFxQueue`、`TowerFireFxQueue` 與 `AttackPhaseFxQueue` 的 `mem::take` drains
- **AND** 沒有 component writes、entity creates 或 entity deletes

#### Scenario: omoba-sim determinism tests 通過

- **WHEN** 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`
- **THEN** omoba-sim determinism test suite 通過，包含 pin-hash tests

#### Scenario: `Outcome::EntityRemoved` 在同 tick boundary delete

- **WHEN** system 將 `Outcome::EntityRemoved { entity: e }` push 到 world outcome resource
- **THEN** `process_outcomes` 將 `e.id()` push 到 `RemovedEntitiesQueue.pending` 並呼叫 `entities().delete(e)`
- **AND** `world.maintain()` 後該 entity 不再 alive
- **AND** state hashing 在該 tick boundary 後不再包含被刪除的 entity

### Requirement: `removed_entity_ids` 從 `RemovedEntitiesQueue` drain

`extract_snapshot` SHALL 以 `std::mem::take` drain `RemovedEntitiesQueue.pending` 來填入 `SimWorldSnapshot.removed_entity_ids`。drain SHALL 讓 queue 保持 empty。`extract_snapshot` SHALL NOT 使用跨 tick 的 `prev_alive: HashSet<u32>` diff algorithm。

#### Scenario: drain 後 queue 為 empty

- **WHEN** sim worker 完成一個 tick 且 `RemovedEntitiesQueue.pending` 含 N 個 ids
- **THEN** `extract_snapshot` 將所有 N 個 ids move 到 `snapshot.removed_entity_ids`
- **AND** extraction 後 `RemovedEntitiesQueue.pending` 為 empty

#### Scenario: 沒有 deletion 時產生 empty removed list

- **WHEN** snapshot 前沒有處理任何 `Outcome::EntityRemoved`
- **THEN** `snapshot.removed_entity_ids` 為 empty

#### Scenario: removed ids 不會跨 ticks 重複

- **WHEN** tick N 記錄 removed entity id 2
- **THEN** 下一個 snapshot 包含 id 2
- **AND** 再下一個 snapshot 不包含 id 2，除非又有新的 id 2 removal 被記錄

### Requirement: HUD 從 snapshots 讀取 round、lives 與 running state

`extract_snapshot` SHALL 從與 `omobab::comp` 對齊的 sim ECS resources 讀取 round、total rounds、round running state 與 lives：`CurrentCreepWave` 與 `PlayerLives`。omfx HUD SHALL 從 snapshot 讀取這些 values，且 SHALL NOT 對這些 fields 使用 legacy heartbeat 或 mirror state。

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

對每個 Hero entity，`extract_snapshot` SHALL 使用 `omobab::ability_runtime::UnitStats::from_refs(...)` 與 final stat accessors 填入 `HeroStatsExt`。omfx hero panel UI SHALL 從 local hero 的 snapshot entity data 讀取 hero stats。Buff countdown display MAY 在 snapshots 之間 locally decrement，且 SHALL 被 authoritative snapshot values reset。

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

### Requirement: 永久 buff 剩餘時間顯示為無限

omfx hero panel SHALL 將永久或 toggle 型 buff 顯示為 `∞`，而不是顯示 permanent duration sentinel 換算出的巨大秒數。`SimWorldSnapshot` 的 hero buff projection SHALL 將 permanent sentinel 或等效超大 remaining duration 正規化為 `remaining_secs == -1.0`，前端 SHALL 依既有規則把負值顯示為 `∞`。

#### Scenario: Passive permanent buff displays infinity

- **WHEN** hero 具有被動技能產生的永久 buff，且 backend BuffStore 使用 permanent sentinel duration
- **THEN** snapshot 中該 buff 的 `remaining_secs == -1.0`
- **AND** omfx hero panel 顯示該 buff 剩餘 `∞`
- **AND** omfx 不顯示 `2097147.9秒` 或其他巨大秒數

#### Scenario: Finite buff still counts down

- **WHEN** hero 具有一般有限時間 buff，且 snapshot 回報正的 remaining seconds
- **THEN** omfx 仍以秒數顯示並在 frame 間本地遞減

### Requirement: tower upgrade levels render from `EntityRenderData.upgrade_levels`

`EntityRenderData` SHALL 包含 `upgrade_levels: Option<[u8; 3]>`，且只對 Tower entities 填入。`extract_snapshot` SHALL 從 sim ECS Tower component 讀取 upgrade levels。omfx SHALL 從此 snapshot value render tower upgrade state。

#### Scenario: upgraded tower 在 snapshot expose levels

- **WHEN** player 將某 tower 的 path 0 upgrade 到 level 2
- **THEN** 下一個 snapshot 中該 tower 的 `upgrade_levels == Some([2, 0, 0])`
- **AND** omfx 在 tower UI 反映 upgraded state

### Requirement: tower selection 與 upgrade panel 使用 snapshot-backed mirror

omfx render code SHALL 在取得 snapshot 後，將 `EntityKind::Tower` snapshot entities mirror 到 `network_entities: HashMap<u32, NetworkEntity>`。mirror SHALL map tower entity type、render position、tower kind、upgrade levels、collision radius 與 attack range。mirror 後，omfx SHALL 移除 current snapshot 中不再存在的 stale tower entries。

Tower click hit-testing、sell/upgrade panel rendering 與 attack-range display SHALL 使用 `network_entities`，而不是在 UI handlers 中直接 lock snapshot。

#### Scenario: 點擊 TD tower 會開啟 tower panel

- **WHEN** TD_1 player left-clicks existing tower
- **THEN** `selected_tower_entity` 從 snapshot-backed mirror 設定
- **AND** 顯示 sell button 與三個 upgrade buttons
- **AND** available 時顯示 selected tower attack range

#### Scenario: selling tower 會移除 mirror entry

- **WHEN** sold tower 不再出現在下一個 snapshot
- **THEN** `network_entities` 不再包含該 tower id
- **AND** 點擊舊 tower 位置不會選到 sold tower

#### Scenario: upgrading tower 會更新 mirror

- **WHEN** tower upgrade applied，且下一個 snapshot 有 updated levels
- **THEN** 對應 `network_entities` entry 有相同 updated `upgrade_levels`
- **AND** upgrade panel text 反映新的 next-level state

### Requirement: tower upgrade definitions 透過 snapshot Arc data 共享

`SimWorldSnapshot.tower_upgrades` SHALL 是從 `TowerUpgradeRegistry` 建立的 `Arc<Vec<TowerUpgradeDefSnapshot>>`。`TowerUpgradeDefSnapshot` SHALL 包含 tower kind、path、level、name 與 cost。sim worker SHALL build 此 data 一次並為 snapshots clone `Arc`。omfx SHALL 以 `(unit_id, path, level)` cache 這些 definitions，供 sell refund 與 upgrade button text 使用。

#### Scenario: omfx sell refund 與 omb 相符

- **WHEN** player 在買 upgrades 後賣掉 tower
- **THEN** omfx sell panel refund calculation 使用 base tower cost 與 snapshot tower upgrade definitions 中的 upgrade costs
- **AND** displayed refund 與 omb sell logic 相符

#### Scenario: upgrade buttons 顯示 next-level names

- **WHEN** TD_1 player 選中未 upgrade 的 dart monkey tower
- **THEN** 每個 path button 顯示 next level name 與 cost
- **AND** button text 不使用 unsupported unicode pip glyphs

#### Scenario: maxed path 顯示 MAX

- **WHEN** tower path 達到 max level
- **THEN** 該 path 的 upgrade button 顯示 `MAX`

### Requirement: tower template snapshots expose combat render metadata

`SimWorldSnapshot.tower_templates` SHALL expose render-facing tower combat metadata needed by omfx composite rendering. For each tower template, the snapshot data SHALL include render mode, base image path, barrel image path, script-owned `render.visual_size`, script-owned `placement_radius`, barrel frame paths, barrel animation timing, body animation frame paths for animated-area towers, rotation mode, barrel layout, barrel count variants, barrel offset, barrel pivot, muzzle offset, recoil distance, recoil scale, recoil attack duration, recoil return duration, and recoil mode. The metadata SHALL originate from scripts content data and SHALL be shared through `Arc` with the same static template lifecycle as existing tower template snapshot data.

#### Scenario: tower template snapshot contains base and barrel paths

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_dart`
- **THEN** the snapshot entry contains a non-empty base image path for `tower_dart`
- **AND** the snapshot entry contains a non-empty barrel image path for `tower_dart`
- **AND** omfx can cache the metadata by `unit_id`

#### Scenario: tower template snapshot contains script-owned sizing

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_dart`
- **THEN** the snapshot entry contains `render.visual_size` from scripts metadata
- **AND** the snapshot entry contains `placement_radius` from scripts metadata
- **AND** neither value is inferred from `footprint`, image dimensions, global frontend scale, or another snapshot field

#### Scenario: tower template snapshot contains barrel animation frames

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a tower whose barrel declares animation frames
- **THEN** the snapshot entry contains the ordered barrel frame paths
- **AND** the snapshot entry contains barrel animation timing metadata

#### Scenario: tower template snapshot contains animated-area frames

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for a no-barrel area damage tower
- **THEN** the snapshot entry contains `render_mode = "animated_area"`
- **AND** the snapshot entry contains ordered body animation frame paths
- **AND** the snapshot entry does not require a barrel image path to render safely

#### Scenario: tower template snapshot contains tack fixed rotation mode

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains `rotation_mode = "fixed"`
- **AND** the snapshot entry contains recoil mode data that allows omfx to play a `scale_pulse` instead of target-facing directional recoil

#### Scenario: tower template snapshot contains tack count variants

- **WHEN** `extract_snapshot` builds `TowerTemplateSnapshot` for `tower_tack`
- **THEN** the snapshot entry contains a radial barrel layout
- **AND** the snapshot entry contains count variants for 8, 12, and 16 barrels or needle holes
- **AND** each variant contains the image path needed by omfx to render that count state

#### Scenario: tower render metadata is built once and shared

- **WHEN** sim worker emits multiple snapshots after tower templates are available
- **THEN** tower render metadata is contained in the shared `tower_templates` Arc data
- **AND** subsequent snapshots use O(1) `Arc::clone` instead of rebuilding identical render metadata every tick

### Requirement: tower aim direction comes from snapshot-facing data

Tower barrel aiming SHALL use authoritative render-facing data from the simulation snapshot only for tower templates whose rotation mode is target-facing. `EntityRenderData.facing_rad` SHALL be treated as the authoritative tower aim direction when tower systems update it toward attack targets; otherwise, the snapshot SHALL expose an equivalent tower aim direction. For templates whose rotation mode is fixed, such as `tower_tack`, omfx SHALL keep barrel visual rotation fixed even when fire cues include a direction. omfx SHALL use snapshot data for target-facing barrel rotation and SHALL NOT compute target selection independently.

#### Scenario: snapshot exposes tower aim direction

- **WHEN** a tower has an active attack target and the sim has updated its aim direction
- **THEN** the corresponding `EntityRenderData` exposes that direction through `facing_rad` or a tower-specific aim field
- **AND** omfx uses that value to rotate the barrel sprite

#### Scenario: no target preserves last known direction

- **WHEN** a tower loses its target for a snapshot
- **THEN** snapshot/render data keeps a stable last known facing or a default facing
- **AND** omfx does not snap the barrel to a random creep or undefined angle

#### Scenario: fixed rotation tower ignores aim for barrel visual

- **WHEN** `tower_tack` has a fire cue direction and `rotation_mode = "fixed"`
- **THEN** omfx can keep the cue direction available for projectile visuals or diagnostics
- **AND** omfx SHALL NOT rotate the `tower_tack` barrel visual toward that cue direction

### Requirement: tower fire cues are drained as render-only snapshot events

`SimWorldSnapshot` SHALL include render-only tower fire cues for recoil animation. The source queue SHALL follow the `ExplosionFxQueue` pattern: deterministic gameplay processing pushes fire cue data when a tower actually fires, and `extract_snapshot` drains the pending queue with `std::mem::take`. The queue SHALL NOT be read by simulation systems for gameplay and SHALL NOT affect state hashing. The sim runner MAY retain recently drained cues in published snapshots for a short render handoff window so a render frame cannot miss a single-tick cue.

Each fire cue SHALL include at minimum the tower entity id, spawn tick, and firing direction in radians. If multiple projectile outcomes from the same tower occur in the same tick, the render cue producer or omfx SHALL allow them to collapse into one recoil pulse for that tower tick.

#### Scenario: firing tower appears in snapshot fire cues

- **WHEN** a tower attack creates a projectile or equivalent attack outcome at tick N
- **THEN** a tower fire cue for that tower entity is pushed during outcome processing
- **AND** the next `extract_snapshot` includes that cue in `SimWorldSnapshot`
- **AND** the cue contains the tower entity id, tick N, and firing direction

#### Scenario: fire cue queue is empty after drain

- **WHEN** `extract_snapshot` drains pending tower fire cues
- **THEN** the drained cues appear in the snapshot
- **AND** the source queue is empty after extraction
- **AND** any retained copies in later snapshots carry the same cue identity and are ignored by omfx after the first render-side consumption

#### Scenario: fire cues do not change determinism

- **WHEN** determinism tests hash the sim state before and after tower fire cue extraction
- **THEN** render-only fire cue queue contents are not part of the authoritative gameplay hash
- **AND** draining the cue queue does not mutate gameplay components, resources, entity existence, damage, cooldown, or projectile state

### Requirement: attack phase cues are exposed through render snapshots

`SimWorldSnapshot` SHALL expose render-only attack phase cues for unit attack animation. Each cue SHALL represent an attack windup start and include entity id, attack sequence id, windup duration, impact offset, backswing duration, and target or direction data. The cue source queue SHALL be drained with the same render-only pattern as explosion and tower fire cues.

#### Scenario: attack phase cue appears before impact

- **WHEN** a unit starts attack windup at tick N and impact is scheduled for a later tick or sub-tick offset
- **THEN** the next render snapshot includes an attack phase cue for that unit
- **AND** omfx can start attack animation before projectile spawn or damage impact

#### Scenario: attack phase cue queue drains once

- **WHEN** `extract_snapshot` drains pending attack phase cues
- **THEN** drained cues appear in the snapshot
- **AND** the source queue is empty after extraction
- **AND** any retained copies in later snapshots carry the same cue identity and are ignored by omfx after the first render-side consumption

#### Scenario: retained render cues survive snapshot overwrite

- **WHEN** sim publishes tick N with a tower fire or attack phase cue and then publishes later ticks before the render thread reads tick N
- **THEN** the latest snapshot may still include the recent cue within the render handoff window
- **AND** omfx consumes the cue once based on entity id, generation, spawn tick, and attack sequence id where available

### Requirement: tower body labels 只顯示 upgrade level summaries

omfx sim-runner-backed entity labels SHALL 只在至少一個 upgrade path 大於零時顯示 Tower labels。label text SHALL 是 `<L0>/<L1>/<L2>`，例如 `2/4/0`。Unupgraded towers SHALL 沒有 tower label。Tower labels SHALL NOT 包含 tower name 或 HP。Hero 與 Creep labels MAY 保留既有 `name HP/MaxHP` format。

#### Scenario: unupgraded tower 沒有 label

- **WHEN** TD_1 player 放置一座 `upgrade_levels == [0, 0, 0]` 的新 tower
- **THEN** omfx 不為該 tower 建立 name label widget
- **AND** 移除任何 stale tower label entry

#### Scenario: upgraded tower label 顯示 N/N/N

- **WHEN** tower 的 `upgrade_levels == Some([2, 4, 0])`
- **THEN** tower label text 是 `2/4/0`
- **AND** label 不包含 tower name、HP 或 unsupported pip glyphs

### Requirement: sell 與 upgrade panel width 避免 clipping

`omfx/game/src/lib.rs::Game::on_init` 中的 TD sell 與 upgrade panel text widgets SHALL 寬度至少為 360.0，相關 panel width calculation SHALL 也至少為 360.0。

#### Scenario: upgrade button text 不被 clipped

- **WHEN** TD_1 render dart monkey path 0 upgrade button
- **THEN** widget width 至少為 360.0
- **AND** 完整 upgrade name 與 cost 可見

### Requirement: blocked regions 從 snapshots render

`SimWorldSnapshot.blocked_regions` SHALL 從 `omobab::comp::BlockedRegions` populate。omfx SHALL 從此 snapshot data render polygon outlines 與 circle outlines。

#### Scenario: DEBUG_1 顯示 region outlines

- **WHEN** `STORY = "DEBUG_1"` 載入含 blocked regions 的 scene
- **THEN** snapshot 包含 blocked region data
- **AND** omfx visibly render region outlines

#### Scenario: TD_1 沒有 region outlines

- **WHEN** TD_1 載入且沒有 blocked regions
- **THEN** `blocked_regions` 為 empty
- **AND** omfx 不 render blocked-region outlines

### Requirement: ability definitions 透過 snapshot Arc data 共享

`SimWorldSnapshot.abilities` SHALL 是 sim worker build 一次並 clone 到後續 snapshots 的 `Arc<Vec<AbilityDefSnapshot>>`。`AbilityDefSnapshot` SHALL 包含 ability id、display name、max level、icon path 與其他 UI metadata。`HeroStatsExt.ability_levels` 與 `HeroStatsExt.ability_ids` SHALL 讓 omfx 能從 snapshot data render W/E/R/T ability bars。

#### Scenario: hero ability bar 反映 level changes

- **WHEN** TD_1 hero level up 且 player upgrade W slot
- **THEN** 下一個 snapshot 的 first ability level 增加
- **AND** omfx 將 W display 從 `0/4` 改成 `1/4`

#### Scenario: ability definitions 不會每個 snapshot rebuild

- **WHEN** sim worker emits N snapshots
- **THEN** inner ability definition vector 只 build 一次
- **AND** 每個 snapshot 使用 O(1) `Arc::clone`

### Requirement: 從快照狀態渲染可點擊技能升級按鈕

omfx 的技能 HUD SHALL 在目前可升級的每個技能圖示上方渲染三角升級按鈕，位置 SHALL 類似 LoL 的升級提示，而不是覆蓋在技能圖示內部。按鈕 SHALL 緊貼技能圖示上緣，且按鈕顯示寬度與 hit-test 寬度 SHALL 等於技能圖示寬度。三角箭頭本體 SHALL 比一般 HUD 文字更寬、更醒目。當本地英雄快照具有 `skill_points > 0`、該欄位有綁定的技能 id，且目前技能等級低於該技能 metadata 中的最高等級時，該欄位即為可升級。

按鈕 SHALL 由以快照為依據的英雄狀態與技能 metadata 推導。omfx SHALL NOT 在送出升級輸入時透過 optimistic 地扣除技能點來隱藏按鈕；按鈕可見性 SHALL 在權威快照值變更時更新。

按鈕可見時 SHALL 有對應的滑鼠 hit-test 區域。點擊按鈕 SHALL 送出對應欄位的 lockstep `UpgradeAbility` input，且 SHALL NOT 同時觸發技能施放、放塔、選塔、移動或其他地圖點擊行為。若三角按鈕 hit-test 與技能圖示 hit-test 皆可能命中，三角按鈕 SHALL 優先處理。

#### Scenario: 可升級技能顯示三角按鈕

- **WHEN** 英雄快照回報 `skill_points > 0`、技能欄位 1 有綁定技能，且其目前等級低於最高等級
- **THEN** 欄位 1 的技能圖示上方會顯示三角升級按鈕
- **AND** 按鈕會跟著圖示上方定位，底部緊貼圖示上緣，而不是覆蓋在圖示內部、tooltip 或英雄狀態文字中
- **AND** 按鈕顯示寬度與 hit-test 寬度等於技能圖示寬度
- **AND** 三角箭頭本體比一般 HUD 文字更寬、更醒目
- **AND** 按鈕有可命中的滑鼠點擊區域

#### Scenario: 不可升級技能隱藏按鈕

- **WHEN** 英雄沒有技能點、欄位沒有綁定技能，或技能已達最高等級
- **THEN** 對應的技能圖示不會顯示三角升級按鈕
- **AND** 對應的按鈕 hit-test 區域不會接受點擊

#### Scenario: 點擊三角按鈕送出升級輸入

- **WHEN** 技能欄位 2 顯示三角升級按鈕，且玩家左鍵點擊該按鈕
- **THEN** omfx 送出 `UpgradeAbility` input，且 `ability_index == 2`
- **AND** 該次點擊不會被後續技能圖示施法、地圖或 TD 點擊邏輯再次處理

#### Scenario: 三角按鈕位於圖示上方

- **WHEN** 技能欄位可升級且 HUD layout 更新
- **THEN** 三角升級按鈕的可見位置位於技能圖示上方，且底部緊貼技能圖示上緣
- **AND** 三角升級按鈕與技能圖示同寬
- **AND** 技能圖示本體仍保留可點擊施法區域

#### Scenario: 按鈕跟隨權威升級結果

- **WHEN** 玩家送出技能升級輸入，而目前快照仍回報舊的技能點與技能等級值
- **THEN** omfx 會根據該目前快照維持按鈕可見性與可點擊性
- **AND** 當後續快照回報已扣除的技能點或已提升的技能等級後，會根據新的權威值重新計算按鈕可見性

### Requirement: path rendering 使用 thick cream zigzag style

`omfx/game/src/render_bridge.rs::ensure_paths_drawn` SHALL 使用 line width `64.0 * crate::WORLD_SCALE * 2.0` 與 color `(170, 140, 90, 255)` render paths。Checkpoint marker dots SHALL NOT 被 render。

#### Scenario: TD_1 path is thick and cream colored

- **WHEN** TD_1 載入
- **THEN** path rendering 使用 thick cream zigzag line style
- **AND** corners 不 render 額外 checkpoint marker dots
