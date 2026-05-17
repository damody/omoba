## ADDED Requirements

### Requirement: RTS command tick decisions are read-only and outcome-driven
本變更新增的 hero command、path planning、path following decision、attack-move acquisition 與 specified attack decision systems，在 ECS tick 決策階段 SHALL 以 read-only gameplay state 為主要輸入，並輸出 `Outcome`、pending command result 或等效集中套用資料。這些決策階段 MUST NOT 直接套用 damage、spawn projectile、delete entity、任意改寫 attack phase，或在候選搜尋/排序 hot path 中直接寫入多個 gameplay storages。

實際 gameplay mutation SHALL 由既有 outcome processing、pending queue drain 或明確的 serial apply phase 執行。任何為平行決策產生的中間結果 SHALL 具有 deterministic ordering，且不得依賴 thread scheduling 順序。

ECS tick pipeline SHALL follow this order for this change: input drain, read-only decision pass, deterministic result sort, serial outcome/apply phase, then snapshot extraction.

#### Scenario: attack-move acquire 只輸出決策結果
- **WHEN** attack-move system 在 tick 中掃描英雄攻擊範圍內的合法敵人
- **THEN** 該掃描與排序階段只讀取 hero、position、attack、faction、path/ranking 與 target state
- **AND** 它輸出 deterministic command result 或 `Outcome`
- **AND** 實際 attack state transition 或 attack windup start 由集中 apply phase 執行

#### Scenario: parallel decision order is deterministic
- **WHEN** 多個英雄或塔在同一 tick 平行執行 command/target decision
- **THEN** 每個 decision result 帶有 deterministic sort key
- **AND** apply phase 依固定順序套用結果
- **AND** 相同 input 與 world state 不會因 thread scheduling 產生不同 gameplay state

#### Scenario: ECS tick pipeline keeps mutation centralized
- **WHEN** a tick applies player inputs and runs RTS command decisions
- **THEN** inputs are drained before read-only decisions
- **AND** decision results are sorted deterministically before mutation
- **AND** gameplay mutation happens only in the serial outcome/apply phase before snapshot extraction

### Requirement: 英雄支援右鍵移動與自動尋路
omfx SHALL 在玩家右鍵點擊地面且沒有命中可互動 HUD 或 enemy entity 時，送出 lockstep `PlayerInput::MoveTo` 或等效地面移動 input。input SHALL 攜帶 queue modifier，表示該命令要取代目前英雄命令 queue 或 append 到 queue tail。omb SHALL 將該 input 套用到送出玩家擁有的英雄，建立後端權威移動命令，並使用 deterministic 尋路產生可跟隨路徑，避開 `BlockedRegions`、建築物、塔與其他不可穿越碰撞體。

尋路結果 SHALL 在相同 world state 與相同 input 下產生相同 waypoint 序列。若目的地不可到達，omb SHALL 將命令導向最接近的可到達位置或拒絕命令並記錄原因；不得 panic 或 disconnect player。

#### Scenario: 右鍵地面產生權威路徑
- **WHEN** 玩家右鍵點擊 blocked region 另一側的可到達地點
- **THEN** omfx 送出地面移動 lockstep input
- **AND** omb 為該玩家英雄建立移動命令與 waypoint path
- **AND** 英雄沿 waypoint 移動而不是直線穿越 blocked region

#### Scenario: 不可到達目的地不會造成 panic
- **WHEN** 玩家右鍵點擊完全被 blocked regions 包住的位置
- **THEN** omb 拒絕或修正該目的地到 deterministic 的最近可到達點
- **AND** log 包含玩家 id、目的地與拒絕或修正原因
- **AND** sim 不 panic 且 lockstep 持續前進

### Requirement: 英雄支援 Shift queued commands
omfx SHALL 在玩家按住 `Shift` 下達 `MoveTo`、`AttackMove` 或 `AttackTarget` 時，將對應 lockstep input 的 queue modifier 設為 append。Hero command queue SHALL allow at most 16 commands total, including the active command and queued tail。當 append 會讓 queue 超過 16 commands 時，omb SHALL reject that append command, log warning, and leave the existing queue unchanged。

Any accepted non-append gameplay action SHALL immediately clear the hero command queue before applying that action。這包含未按住 `Shift` 的 `MoveTo`、`AttackMove`、`AttackTarget`，以及 accepted ability、item、tower action、start-round 或其他 `PlayerInput` gameplay action。Pure UI selection、hover、panel open/close 或 client-only focus state SHALL NOT by itself clear the queue。

omb SHALL 對 replace/non-append 命令立即覆蓋目前正在執行的 command、清除該英雄目前所有 queued commands，並將新命令設為新的 queue head；對 append 命令 SHALL 將新命令加入 queue tail，且不得中斷目前正在執行的 command。

Hero command queue SHALL 以 deterministic order 執行。Current command 完成後才 advance 到下一個 queued command。`MoveTo` 完成條件為抵達或被 deterministic 規則判定完成；`AttackMove` 完成條件為抵達 attack-move destination；`AttackTarget` 完成條件為指定 target 死亡、失效、不可攻擊或不可繼續追擊。

Queued command validation SHALL happen both when appending and when becoming active. If a queued `AttackTarget` is invalid when appended, it SHALL be rejected and SHALL NOT enter the queue. If it becomes invalid before execution, it SHALL be skipped and the queue SHALL advance. If a queued `MoveTo` or `AttackMove` destination becomes blocked before execution, omb SHALL deterministically repath to the nearest reachable point; if no reachable point exists, it SHALL skip that command and advance the queue.

#### Scenario: Shift 右鍵怪物再點地面
- **WHEN** 玩家按住 `Shift` 右鍵點擊敵方怪物 A，接著仍按住 `Shift` 右鍵點擊地面 B
- **THEN** omfx 依序送出 queued `AttackTarget(A)` 與 queued `MoveTo(B)`
- **AND** omb 將兩個命令 append 到英雄 command queue
- **AND** 英雄先攻擊怪物 A
- **AND** 怪物 A 死亡或失效後，英雄才 advance 到 `MoveTo(B)` 並移動到地面 B

#### Scenario: queued append over max is rejected
- **WHEN** 英雄 command queue already contains 16 commands
- **AND** 玩家按住 `Shift` 追加 `MoveTo(B)`
- **THEN** omb rejects the append command and logs warning
- **AND** the existing 16-command queue remains unchanged

#### Scenario: invalid queued target is rejected at append
- **WHEN** 玩家按住 `Shift` 對不存在或非法 target 送出 `AttackTarget`
- **THEN** omb rejects the append command
- **AND** the invalid command does not enter the hero command queue

#### Scenario: queued target invalid before execution is skipped
- **WHEN** `AttackTarget(A)` is queued behind the active command
- **AND** A dies before `AttackTarget(A)` becomes active
- **THEN** omb skips `AttackTarget(A)` when it reaches queue head
- **AND** the queue advances to the next command

#### Scenario: queued MoveTo blocked before execution repaths
- **WHEN** `MoveTo(B)` is queued and B becomes blocked before the command becomes active
- **THEN** omb deterministically repaths to the nearest reachable point to B
- **AND** if no reachable point exists, omb skips that command and advances the queue

#### Scenario: 非 Shift MoveTo 覆蓋並清除 queue
- **WHEN** 英雄 command queue 內已有多個 queued commands，且玩家未按 `Shift` 右鍵點擊地面 C
- **THEN** omfx 送出 replace `MoveTo(C)`
- **AND** omb 覆蓋目前 command 並清除所有既有 queued commands
- **AND** 英雄新的 queue head 改為移動到 C

#### Scenario: 非 Shift AttackTarget 覆蓋並清除 queue
- **WHEN** 英雄 command queue 內已有 `AttackTarget(A)` 與 `MoveTo(B)`，且玩家未按 `Shift` 右鍵點擊敵方怪物 C
- **THEN** omfx 送出 replace `AttackTarget(C)`
- **AND** omb 覆蓋目前 command 並清除所有既有 queued commands
- **AND** 英雄新的 queue head 改為攻擊怪物 C

#### Scenario: 非 Shift AttackMove 覆蓋並清除 queue
- **WHEN** 英雄 command queue 內已有多個 queued commands，且玩家未按 `Shift` 使用 `A` 點擊地面 D
- **THEN** omfx 送出 replace `AttackMove(D)`
- **AND** omb 覆蓋目前 command 並清除所有既有 queued commands
- **AND** 英雄新的 queue head 改為 attack-move 到 D

#### Scenario: queued command 不中斷目前後搖
- **WHEN** 英雄正在執行目前 command 且攻擊已進入 backswing，玩家按住 `Shift` 追加 `MoveTo(B)`
- **THEN** `MoveTo(B)` 被 append 到 queue tail
- **AND** 目前 backswing 不會因 queued command 被取消
- **AND** 目前 command 完成後才執行 `MoveTo(B)`

#### Scenario: queued command 不中斷目前 windup
- **WHEN** 英雄正在普通攻擊 windup，玩家按住 `Shift` 追加 `AttackTarget(B)`
- **THEN** `AttackTarget(B)` 被 append 到 queue tail
- **AND** 目前 windup 不會因 queued command 被取消
- **AND** 目前 command 完成後才執行 `AttackTarget(B)`

#### Scenario: accepted ability clears command queue
- **WHEN** 英雄 command queue 內已有多個 commands，且玩家施放 accepted ability
- **THEN** omb clears the hero command queue before applying that ability input
- **AND** queued move or attack commands no longer execute

### Requirement: 英雄支援 A 點地 attack-move
omfx SHALL 在玩家按下 `A` 後進入 attack-move targeting mode，下一次合法地面左鍵點擊 SHALL 送出 lockstep `AttackMove` input，包含目標地點與 queue modifier。收到 attack-move 後，omb SHALL 讓英雄向目的地尋路移動，並在移動途中以自身有效攻擊範圍搜尋合法敵人。

當 attack-move 英雄發現合法敵人時，omb SHALL 讓英雄對該敵人發起普通攻擊。若該攻擊已進入 backswing，attack-move 的自動續走、自動重新索敵或目標失效處理 SHALL NOT 中斷 backswing lockout。若敵人死亡、離開可追擊條件或不再合法，英雄 SHALL 在 backswing 完成後繼續原本 attack-move 目的地，除非玩家送出新的明確命令。Attack-move 發起的攻擊 SHALL 遵守既有 windup/impact/backswing 與取消規則。

#### Scenario: A 點地途中自動攻擊敵人
- **WHEN** 玩家按下 `A` 並左鍵點擊目的地，且英雄前往途中有敵人進入攻擊範圍
- **THEN** omfx 送出 `AttackMove` lockstep input
- **AND** omb 將英雄命令設為 attack-move
- **AND** 英雄在途中對合法敵人發起普通攻擊
- **AND** 攻擊效果只在 authoritative impact timing 生效

#### Scenario: 目標死亡後繼續 attack-move
- **WHEN** attack-move 英雄攻擊途中取得的敵人在 impact 後死亡，且英雄仍在 backswing
- **THEN** 英雄清除該臨時攻擊目標
- **AND** 英雄不會因 attack-move 自動續走而取消 backswing
- **AND** backswing 完成後，若沒有新的合法敵人，英雄繼續往原 attack-move 目的地移動

#### Scenario: attack-move 自動重新索敵不取消後搖
- **WHEN** attack-move 英雄剛完成 impact 並進入 backswing，且另一個合法敵人進入攻擊範圍
- **THEN** omb 不會用 attack-move 自動重新索敵取消目前 backswing
- **AND** 英雄只能在 backswing 完成後發起下一次自動攻擊或繼續移動

### Requirement: 英雄支援右鍵指定攻擊敵人
omfx SHALL 在玩家右鍵點擊敵方 entity 時送出 lockstep `PlayerInput::AttackTarget`，包含 target entity id 與 queue modifier。omb SHALL 驗證送出玩家的英雄存在、target entity 存在且為合法敵人；驗證成功後，英雄 SHALL 將該 entity 作為指定攻擊目標或 append 到 command queue tail。

若指定目標在攻擊範圍外但可追擊，英雄 SHALL 尋路靠近到攻擊範圍內再攻擊。`AttackTarget` 不限制追擊時間，但 SHALL use a leash distance equal to `effective_attack_range * 0.5`。若英雄為追擊該 target 造成的偏移超過 leash distance，該 `AttackTarget` command SHALL be abandoned, logged, and treated as complete/failed so the queue can advance。若指定目標失效、死亡、不可攻擊或無法解析，omb SHALL 清除或拒絕該命令並記錄 warning；不得 panic。

#### Scenario: 右鍵敵人指定攻擊
- **WHEN** 玩家右鍵點擊一個存活的敵方 creep
- **THEN** omfx 送出 `AttackTarget { target_id }`
- **AND** omb 驗證 target 是合法敵人
- **AND** 英雄追擊到攻擊範圍內並對該 target 發起普通攻擊

#### Scenario: 指定攻擊非法目標被拒絕
- **WHEN** 玩家右鍵點擊友方 entity 或不存在的 entity id
- **THEN** omb log rejection，內容包含 player id 與 target id
- **AND** 英雄目前命令不會被替換成非法攻擊命令

#### Scenario: 指定攻擊超過追擊 leash 後放棄
- **WHEN** 英雄執行 `AttackTarget(A)` 且 A 持續遠離
- **AND** 英雄為追擊 A 的位置偏移超過目前 effective attack range 的 0.5 倍
- **THEN** omb abandons that `AttackTarget` command
- **AND** 若 queue 內仍有下一個 command，英雄 advance 到下一個 command
- **AND** 追擊不因時間長短本身被放棄

### Requirement: 新明確命令取消既有英雄攻擊 windup
當英雄在 attack windup 期間收到 accepted non-append gameplay action 時，omb SHALL 清除 hero command queue，並依既有 attack cancellation contract 取消尚未 impact 的攻擊序列。若命令在 impact 之後才套用，已提交的傷害、projectile 或命中結果 SHALL 保留。

Queued append commands、attack-move command 內部的自動續走、自動重新索敵、目標死亡回退與路徑跟隨 SHALL NOT 視為新的明確玩家命令，且 SHALL NOT 自動取消已進入 windup 或 backswing 的攻擊。只有 accepted non-append gameplay action 可依既有 attack cancellation contract 影響目前攻擊狀態。

#### Scenario: MoveTo 取消 windup
- **WHEN** 英雄正在普通攻擊 windup，且玩家送出 accepted `MoveTo`
- **THEN** omb 取消該 attack sequence
- **AND** 不會為被取消的 sequence 套用傷害或生成 projectile
- **AND** omfx 透過 render-only cancel cue 停止 hit frame

#### Scenario: impact 後的新玩家命令不回滾攻擊
- **WHEN** 英雄攻擊已達 impact event，且玩家送出新的 `AttackMove`
- **THEN** 已提交的攻擊結果保持有效
- **AND** 新命令依既有明確命令規則處理後續移動或 backswing lockout
- **AND** 這不允許 attack-move 內部自動續走自行取消 backswing

### Requirement: 英雄命令狀態透過 snapshot expose
`SimWorldSnapshot` 或其 `EntityRenderData` hero extension SHALL expose 足以讓 omfx 顯示權威命令 feedback 的資料，至少包含目前命令 kind、destination、指定 target entity id、queued command count 或 queue summary，以及 active waypoints 或下一個 waypoint。該資料 SHALL 來自後端 sim state，不得由 omfx optimistic state 作為權威來源。

#### Scenario: attack-move 路徑可由 snapshot 顯示
- **WHEN** 英雄具有 active attack-move 命令與 waypoint path
- **THEN** 下一個 snapshot 包含該英雄的命令 kind 與 path/destination render data
- **AND** omfx 可依 snapshot 顯示移動路徑或命令提示

#### Scenario: 新 snapshot 覆蓋本地命令顯示
- **WHEN** omfx 送出 attack-move input 後尚未收到權威 snapshot
- **THEN** omfx 不把 optimistic 命令狀態視為權威 gameplay state
- **AND** 收到後續 snapshot 時，以 snapshot 中的英雄命令狀態更新顯示
