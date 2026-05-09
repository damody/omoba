## ADDED Requirements

### Requirement: all unit attacks use windup, impact, and backswing phases
所有會普攻的 unit，包括 hero、summon、creep 與 tower，SHALL 使用三段攻擊生命週期：attack windup（攻擊前搖）、attack impact event（攻擊瞬間事件）、attack backswing（攻擊後搖）。後端 SHALL authoritative 地排程 windup 與 backswing 兩段 duration，並在兩段交界處觸發 impact event。damage application、projectile spawn、hit outcome 或等效攻擊效果 SHALL 發生在 impact event，而不是 windup 開始時。

#### Scenario: attack phases exist for a tower attack
- **WHEN** `tower_dart` 決定對 creep 發動一次普攻
- **THEN** 後端建立一個 attack windup phase
- **AND** projectile spawn 或 damage effect 發生在 attack impact event
- **AND** attack impact 後進入 attack backswing phase

#### Scenario: attack phases exist for non-tower units
- **WHEN** hero、summon 或 creep 發動一次普攻
- **THEN** 該攻擊也使用 windup、impact、backswing 三階段
- **AND** gameplay outcome 仍只在 authoritative impact timing 生效

### Requirement: attack phase integer weights sum to 1000 and scale with effective attack speed
attack windup 與 attack backswing SHALL 使用整數權重設定，且 `windup + backswing` SHALL equal `1000`。attack impact SHALL be an instant event point at the boundary between windup and backswing, not a duration. 後端 SHALL 使用單位的 base attack interval、buff/stat aggregation 與 attack timing metadata 計算每次攻擊的 phase durations。計算 SHALL 使用整數或 fixed-point representation，避免浮點相等比較。當攻速變快、effective attack interval 變短時，windup 與 backswing SHALL 依權重縮短，且 `windup_duration + backswing_duration` SHALL equal the effective attack interval。

#### Scenario: faster attack speed shortens phases
- **WHEN** 某 tower 的 effective attack interval 從 `1.0s` 降到 `0.5s`
- **THEN** 該 tower 下一次攻擊的 windup duration 會縮短
- **AND** 該 tower 下一次攻擊的 backswing duration 會縮短
- **AND** windup duration 加 backswing duration 等於新的 effective attack interval
- **AND** impact timing 是 windup 結束的瞬間事件點

#### Scenario: invalid weights are rejected
- **WHEN** content declares `attack_timing = { windup = 350, backswing = 450 }`
- **THEN** codegen or content validation fails because the weights do not sum to `1000`
- **AND** no generated runtime metadata is emitted for that invalid unit timing

#### Scenario: impact has no duration
- **WHEN** a unit attack uses `windup = 350` and `backswing = 650`
- **THEN** impact occurs at 35% of the effective attack interval
- **AND** impact does not reserve any additional duration between windup and backswing

#### Scenario: fixed-point calculation preserves total interval
- **WHEN** effective attack interval is represented as fixed-point or ticks
- **THEN** backend computes windup duration from `effective_interval * windup / 1000`
- **AND** backend computes backswing duration as `effective_interval - windup_duration`
- **AND** no floating-point equality check is required to prove the durations sum to the interval

### Requirement: backend emits attack phase render cues at windup start
後端 SHALL 在 attack windup 開始時產生 render-only attack phase cue。cue SHALL 包含 unit entity id、attack sequence id、windup duration、impact event offset、backswing duration、攻擊方向或 target reference，以及必要的 render metadata。cue SHALL 透過 snapshot 或 render-only queue 提供給前端，並 SHALL NOT 影響 gameplay state hash。

#### Scenario: windup cue reaches frontend before impact
- **WHEN** unit 在 tick N 開始 attack windup，且 impact 預定在 tick N 之後
- **THEN** `SimWorldSnapshot` 或等效 render queue 包含該 attack phase cue
- **AND** cue 讓 omfx 能在 impact 前開始播放攻擊動畫
- **AND** damage 或 projectile 不會因 cue 提前生效

#### Scenario: cue includes enough timing data
- **WHEN** omfx 收到 attack phase cue
- **THEN** cue 包含 windup duration
- **AND** cue 包含 impact event offset，且該 offset 等於 windup duration
- **AND** cue 包含 backswing duration
- **AND** cue 包含可讓動畫朝向或定位的 target/direction data

### Requirement: existing projectile and damage outcomes occur at impact event
The impact event SHALL reuse existing attack outcome semantics. Current projectile creation or damage outcomes, including `Outcome::ProjectileLine2`, `Outcome::ProjectileDirectional`, script-driven `spawn_projectile_ex`, direct damage outcomes, and related `Outcome::UpdateAttack` / `asd_count` cooldown accounting, SHALL be scheduled so their gameplay effect occurs at the authoritative impact event point.

#### Scenario: existing projectile outcome is delayed to impact
- **WHEN** a tower attack would currently emit `Outcome::ProjectileLine2` when cooldown is ready
- **THEN** the new attack scheduler starts windup when cooldown is ready
- **AND** emits the projectile outcome at the impact event point
- **AND** does not treat impact as a duration

#### Scenario: cooldown accounting still covers the whole interval
- **WHEN** an attack with windup and backswing completes
- **THEN** `asd_count` / cooldown accounting represents the full effective attack interval
- **AND** adding windup/backswing does not allow an extra attack before the interval is complete

### Requirement: frontend starts attack animation during windup
omfx SHALL 在收到 attack windup cue 時立即開始該 unit 的攻擊動畫。動畫的 anticipation、barrel/body frame sequence 或角色動作 SHALL 從 windup phase 開始，而不是等 projectile spawn、damage event 或 impact event 才開始。攻擊動畫的關鍵 frame、recoil 或命中特效 SHALL 對齊 impact event timing。

#### Scenario: tower barrel animation starts before projectile spawn
- **WHEN** `tower_bomb` 收到 attack windup cue，且 impact 尚未發生
- **THEN** omfx 立即開始播放 barrel attack animation
- **AND** projectile spawn 對齊 cue 中的 impact timing
- **AND** recoil 或 fire frame 對齊 impact timing

#### Scenario: animated area tower starts body animation in windup
- **WHEN** 無砲管範圍傷害塔收到 attack windup cue
- **THEN** omfx 從 windup 開始播放 body frame animation
- **AND** 範圍傷害的視覺爆發 frame 對齊 impact timing

### Requirement: attack phase cues are render-only and deterministic-safe
Attack phase cue queues SHALL follow the render-only queue pattern used by explosion/fire cues. Cue production SHALL be driven by deterministic attack scheduling, but draining cues into snapshots SHALL NOT mutate gameplay components, entity existence, damage, cooldown, projectile state, or authoritative hash data.

#### Scenario: draining attack phase cues does not mutate gameplay
- **WHEN** `extract_snapshot` drains pending attack phase cues
- **THEN** drained cues appear in the snapshot
- **AND** source queue is empty after extraction
- **AND** gameplay state hash is unaffected by the drain

#### Scenario: repeated snapshots do not replay the same attack cue
- **WHEN** an attack phase cue was drained into one snapshot
- **THEN** later snapshots do not include the same cue again
- **AND** a new cue appears only when a new attack windup starts
