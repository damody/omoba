## ADDED Requirements

### Requirement: all unit attacks use windup, impact, and backswing phases
所有會普攻的 unit，包括 hero、summon、creep 與 tower，SHALL 使用三階段攻擊生命週期：attack windup（攻擊前搖）、attack impact（攻擊瞬間）、attack backswing（攻擊後搖）。後端 SHALL authoritative 地排程這三個 phase。damage application、projectile spawn、hit outcome 或等效攻擊效果 SHALL 發生在 impact phase，而不是 windup 開始時。

#### Scenario: attack phases exist for a tower attack
- **WHEN** `tower_dart` 決定對 creep 發動一次普攻
- **THEN** 後端建立一個 attack windup phase
- **AND** projectile spawn 或 damage effect 發生在 attack impact phase
- **AND** attack impact 後進入 attack backswing phase

#### Scenario: attack phases exist for non-tower units
- **WHEN** hero、summon 或 creep 發動一次普攻
- **THEN** 該攻擊也使用 windup、impact、backswing 三階段
- **AND** gameplay outcome 仍只在 authoritative impact timing 生效

### Requirement: attack phase durations scale with effective attack speed
attack windup、impact offset 與 backswing timing SHALL 隨 effective attack speed 縮短。後端 SHALL 使用單位的 base attack interval、buff/stat aggregation 與 attack timing metadata 計算每次攻擊的 phase durations。當攻速變快、effective attack interval 變短時，windup 與 backswing SHALL 依比例或 metadata 規則縮短。實作 SHALL 提供最小 phase duration default，以避免極高攻速讓前搖或後搖變成 0ms。

#### Scenario: faster attack speed shortens phases
- **WHEN** 某 tower 的 effective attack interval 從 `1.0s` 降到 `0.5s`
- **THEN** 該 tower 下一次攻擊的 windup duration 會縮短
- **AND** 該 tower 下一次攻擊的 backswing duration 會縮短
- **AND** impact timing 仍落在該次 attack interval 內

#### Scenario: minimum phase durations prevent zero-length animation
- **WHEN** buff 讓某 unit 的 effective attack interval 非常短
- **THEN** windup duration 不會低於 configured minimum windup duration
- **AND** backswing duration 不會低於 configured minimum backswing duration
- **AND** 後端仍維持合法的 attack cooldown/interval 規則

### Requirement: backend emits attack phase render cues at windup start
後端 SHALL 在 attack windup 開始時產生 render-only attack phase cue。cue SHALL 包含 unit entity id、attack sequence id、windup duration、time-to-impact、backswing duration、攻擊方向或 target reference，以及必要的 render metadata。cue SHALL 透過 snapshot 或 render-only queue 提供給前端，並 SHALL NOT 影響 gameplay state hash。

#### Scenario: windup cue reaches frontend before impact
- **WHEN** unit 在 tick N 開始 attack windup，且 impact 預定在 tick N 之後
- **THEN** `SimWorldSnapshot` 或等效 render queue 包含該 attack phase cue
- **AND** cue 讓 omfx 能在 impact 前開始播放攻擊動畫
- **AND** damage 或 projectile 不會因 cue 提前生效

#### Scenario: cue includes enough timing data
- **WHEN** omfx 收到 attack phase cue
- **THEN** cue 包含 windup duration
- **AND** cue 包含 time-to-impact 或 impact offset
- **AND** cue 包含 backswing duration
- **AND** cue 包含可讓動畫朝向或定位的 target/direction data

### Requirement: frontend starts attack animation during windup
omfx SHALL 在收到 attack windup cue 時立即開始該 unit 的攻擊動畫。動畫的 anticipation、barrel/body frame sequence 或角色動作 SHALL 從 windup phase 開始，而不是等 projectile spawn、damage event 或 impact phase 才開始。攻擊動畫的關鍵 frame、recoil 或命中特效 SHALL 對齊 impact timing。

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
