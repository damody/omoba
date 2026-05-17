## ADDED Requirements

### Requirement: AttackMove 端到端
omfx `A` 點地 handling SHALL 透過 lockstep input 送出 `PlayerInputAction::AttackMove { target_pos, queued }` 或等效 proto action。omb `player_input_tick` SHALL log input metadata，包含 player id、tick、raw target position 與 queued flag，並將 input 路由到 pending hero command queue 或 shared `GameProcessor` entry point。

失敗時 SHALL 用 warning log，且 MUST NOT panic、disconnect player 或送 bespoke acknowledgement；玩家透過後續 tick batch、snapshot 與 render cues 觀察結果。

#### Scenario: A 點地送出 AttackMove
- **WHEN** 玩家按下 `A` 後左鍵點擊地面
- **THEN** omfx 送出 `PlayerInput::AttackMove`，包含 clicked world position
- **AND** omb `player_input_tick` 記錄 `AttackMove` line
- **AND** sim 將該 input 套用為玩家英雄的 attack-move command

#### Scenario: AttackMove failure 只 warn
- **WHEN** 玩家沒有可控制英雄但送出 `AttackMove`
- **THEN** omb log warning，內容包含 player id
- **AND** omb 不 panic 且不送專用 acknowledgement

### Requirement: AttackTarget 端到端套用到英雄命令
omfx 右鍵 enemy entity handling SHALL 透過 lockstep input 送出 `PlayerInputAction::AttackTarget { target_id, queued }`。omb `player_input_tick` SHALL 將該 input 路由到英雄命令處理，而不是只記錄 log。處理成功時，玩家英雄 SHALL 取得指定攻擊命令或將指定攻擊命令 append 到 command queue。

#### Scenario: 右鍵敵人送出 AttackTarget
- **WHEN** 玩家右鍵點擊可攻擊敵人
- **THEN** omfx 送出 `PlayerInput::AttackTarget { target_id }`
- **AND** omb `player_input_tick` 記錄 target id
- **AND** sim 將該 input 套用為玩家英雄的指定攻擊命令

### Requirement: MoveTo 端到端支援 queued flag
omfx 右鍵地面 handling SHALL 透過 lockstep input 送出 `PlayerInputAction::MoveTo { target, queued }` 或等效 proto action。`queued` SHALL 反映輸入當下 `Shift` 是否被按住。omb `player_input_tick` SHALL log queued flag，並將 queued flag 傳入 hero command queue drain。

#### Scenario: Shift MoveTo appends
- **WHEN** 玩家按住 `Shift` 右鍵點擊地面
- **THEN** omfx 送出 `MoveTo` input 且 `queued == true`
- **AND** omb 將該 move command append 到英雄 command queue

#### Scenario: Non Shift MoveTo replaces
- **WHEN** 玩家未按 `Shift` 右鍵點擊地面
- **THEN** omfx 送出 `MoveTo` input 且 `queued == false`
- **AND** omb 以該 move command 覆蓋目前 command 並清除英雄現有 command queue

#### Scenario: Non Shift attack input clears queued moves
- **WHEN** 玩家已有 queued movement commands，且未按 `Shift` 右鍵點擊敵人或使用 `A` 點地
- **THEN** omfx 送出的 `AttackTarget` 或 `AttackMove` input 具有 `queued == false`
- **AND** omb 以該 attack command 覆蓋目前 command 並清除英雄現有 command queue

### Requirement: Non-append gameplay inputs clear hero command queue
Any accepted `PlayerInput` gameplay action that is not an explicit queued append SHALL clear the submitting player's hero command queue before applying that action. This includes `CastAbility`, `UpgradeAbility`, `ItemUse`, `TowerPlace`, `TowerUpgrade`, `TowerSell`, `SetTowerTargetPriority`, and `StartRound` if accepted. `NoOp` and client-only UI selection state SHALL NOT clear the queue.

#### Scenario: CastAbility clears queued movement
- **WHEN** 玩家已有 queued hero commands 並送出 accepted `CastAbility`
- **THEN** omb clears the hero command queue before applying the ability input
- **AND** previously queued movement or attack commands no longer execute

#### Scenario: Tower action clears queued movement
- **WHEN** 玩家已有 queued hero commands 並送出 accepted `TowerUpgrade` 或 `SetTowerTargetPriority`
- **THEN** omb clears the hero command queue before applying the tower input
- **AND** previously queued movement or attack commands no longer execute

#### Scenario: AttackTarget 不再是 log-only
- **WHEN** 搜尋 `player_input_tick` 中的 `AttackTarget` arm
- **THEN** 該 arm 呼叫 pending queue 或 `GameProcessor` entry point
- **AND** 不存在只 log 而不套用 gameplay state 的 stub

### Requirement: SetTowerTargetPriority 端到端
omfx tower panel target-priority control SHALL 透過 lockstep input 送出 `PlayerInputAction::SetTowerTargetPriority { tower_entity_id, priority }` 或等效 proto action。omb SHALL route 該 input 到 shared tower command entry point，驗證權限後更新 Tower priority。

#### Scenario: 塔策略 input routed
- **WHEN** 玩家在 tower panel 選擇 `first`
- **THEN** omfx 送出 `SetTowerTargetPriority` lockstep input
- **AND** omb `player_input_tick` log tower id 與 priority
- **AND** sim 更新該 Tower priority

#### Scenario: invalid priority 被拒絕
- **WHEN** client 送出 schema 允許範圍外或無法解析的 priority
- **THEN** omb 拒絕 input 並 log warning
- **AND** Tower priority 不變

### Requirement: 地圖點擊與 HUD hit-test 優先序保持明確
omfx SHALL 在處理新增 RTS 操作時維持固定 hit-test 優先序：HUD/面板控制優先於技能與 item 控制，技能與 item 控制優先於 entity hit-test，entity hit-test 優先於地面命令。塔策略選單、技能升級按鈕、技能圖示、item slot 與 tower panel button 被點擊時，該次點擊 MUST NOT 同時送出 `MoveTo`、`AttackMove` 或 `AttackTarget`。

#### Scenario: 點塔策略不觸發地圖命令
- **WHEN** 玩家點擊 tower panel 的 target priority 控制
- **THEN** omfx 只送出 `SetTowerTargetPriority`
- **AND** 不送出 `MoveTo`、`AttackMove` 或 `AttackTarget`

#### Scenario: A 模式點擊 HUD 不送 AttackMove
- **WHEN** 玩家已進入 attack-move targeting mode 並點擊技能圖示或 tower panel
- **THEN** omfx 處理該 HUD 點擊
- **AND** 不送出 `AttackMove`
