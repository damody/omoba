## ADDED Requirements

### Requirement: CastAbility 端到端

omfx 一般技能施放 SHALL 使用 `W/E/R/T` 作為四個英雄技能欄位快捷鍵，並分別送出 `PlayerInput::CastAbility { ability_index: 0/1/2/3 }`。點擊技能圖示本體 SHALL 送出同一個 `CastAbility` input，並依圖示欄位對應索引 `0/1/2/3`。`Q` SHALL NOT 作為這四個技能欄位的施放快捷鍵。

omb SHALL 將 `CastAbility` lockstep input 路由到待處理施法佇列，並在 dispatcher 輸入路由之後、腳本 dispatch 之前，透過 shared `GameProcessor` entry point 清空處理。成功施法時，SHALL 解析送出玩家的英雄、驗證欄位具有綁定技能、驗證技能已學且不在 cooldown，並排入 `ScriptEvent::SkillCast`，使能力腳本在同 tick 的 script dispatch 階段執行。

失敗時 SHALL log rejection，MUST NOT panic，也 MUST NOT 送出專用 acknowledgement；玩家透過後續 snapshot 與技能效果觀察結果。

#### Scenario: W casts first ability slot

- **WHEN** 本地英雄欄位 0 有已學技能且不在 cooldown，並且玩家按下 `W`
- **THEN** omfx 送出 `PlayerInput::CastAbility { ability_index: 0 }`
- **AND** omb 透過 pending cast drain 排入 `ScriptEvent::SkillCast`
- **AND** script dispatch 執行該 ability script

#### Scenario: T casts fourth ability slot

- **WHEN** 本地英雄欄位 3 有已學技能且不在 cooldown，並且玩家按下 `T`
- **THEN** omfx 送出 `PlayerInput::CastAbility { ability_index: 3 }`
- **AND** omb 透過 pending cast drain 排入 `ScriptEvent::SkillCast`

#### Scenario: Clicking ability icon casts matching slot

- **WHEN** 本地英雄欄位 1 有已學技能且不在 cooldown，並且玩家左鍵點擊欄位 1 的技能圖示本體
- **THEN** omfx 送出 `PlayerInput::CastAbility { ability_index: 1 }`
- **AND** 該次點擊不會落到 TD/map click handling
- **AND** omb 透過 pending cast drain 排入 `ScriptEvent::SkillCast`

#### Scenario: Unlearned ability cast is rejected

- **WHEN** 玩家對尚未學習的技能欄位送出 `CastAbility`
- **THEN** omb log rejection 而不 panic
- **AND** 不會排入 `ScriptEvent::SkillCast`

### Requirement: 技能升級端到端

omfx 英雄技能升級快捷鍵處理與技能 HUD 三角按鈕點擊 SHALL 送出 lockstep `PlayerInput` action，並攜帶技能欄位索引。`Shift+W`、`Shift+E`、`Shift+R` 與 `Shift+T` SHALL 分別對應到技能索引 `0`、`1`、`2` 與 `3`。點擊技能欄位 0..3 上的三角升級按鈕 SHALL 送出相同索引。

omb SHALL 將該輸入透過 `player_input_tick` 路由到待處理技能升級佇列，接著在 dispatcher 輸入路由之後、腳本 dispatch 之前，透過 `GameProcessor` 入口點清空處理該佇列。成功升級時，SHALL 驗證送出玩家的英雄、驗證欄位具有綁定技能、要求至少有一點可用技能點、拒絕已達最高等級的技能、將技能等級加一、將技能點扣一，並為已學習技能與新等級排入 `ScriptEvent::SkillLearn`。

失敗時 SHALL 以 warning 或資訊性拒絕記錄，MUST NOT panic，也 MUST NOT 送出專用 acknowledgement；玩家會透過下一個權威快照觀察結果。

#### Scenario: Shift W 升級第一個技能欄位

- **WHEN** 本地英雄具有 `skill_points > 0`、欄位 0 有綁定技能且尚未達最高等級，並且玩家按下 `Shift+W`
- **THEN** omfx 送出 `PlayerInput::UpgradeAbility { ability_index: 0 }`
- **AND** omb 透過 pending ability-upgrade drain 套用排程後的輸入
- **AND** 英雄欄位 0 的技能等級增加一級
- **AND** 英雄的技能點減少一點
- **AND** `ScriptEvent::SkillLearn` 會以已升級技能 id 與新等級排入 queue
- **AND** 下一個快照會公開更新後的技能等級與技能點值

#### Scenario: 點擊三角按鈕升級對應技能欄位

- **WHEN** 本地英雄具有 `skill_points > 0`、欄位 2 有綁定技能且尚未達最高等級，並且玩家點擊欄位 2 的三角升級按鈕
- **THEN** omfx 送出 `PlayerInput::UpgradeAbility { ability_index: 2 }`
- **AND** omb 透過與鍵盤快捷鍵相同的 pending ability-upgrade drain 套用排程後的輸入
- **AND** 若權威後端檢查仍通過，英雄欄位 2 的技能等級增加一級

#### Scenario: 沒有技能點時升級會被拒絕

- **WHEN** 本地英雄沒有可用技能點，且套用了 `UpgradeAbility` 輸入
- **THEN** omb 記錄拒絕原因而不 panic
- **AND** 英雄的技能等級與技能點保持不變
- **AND** 不會排入 `ScriptEvent::SkillLearn`

#### Scenario: 已達最高等級時升級會被拒絕

- **WHEN** 送出的欄位中技能等級已經大於或等於該技能的最高等級
- **THEN** omb 記錄拒絕原因而不 panic
- **AND** 英雄的技能等級與技能點保持不變
- **AND** 不會排入 `ScriptEvent::SkillLearn`
