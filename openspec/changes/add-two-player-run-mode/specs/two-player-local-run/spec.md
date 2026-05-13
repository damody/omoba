## ADDED Requirements

### Requirement: 本機雙前端啟動

Repository root SHALL 提供 `run_2player.bat`，從 `D:\omoba` 執行時會建置或確認必要 artifacts、啟動一個 omb 後端，並啟動兩個 omfx frontend process 連到同一個後端。

`run_2player.bat` SHALL 沿用 `run.bat` 的 script DLL staging、backend build、frontend build 與 stale process cleanup 行為。新增或修改的 `.bat` 檔 MUST 使用 CRLF 行尾。

#### Scenario: run_2player 啟動一個後端兩個前端

- **WHEN** 使用者從 repo root 執行 `run_2player.bat`
- **THEN** 腳本確認 `scripts/base_content.dll`、`omb/target/debug/omobab.exe` 與 `omfx/target/debug/executor.exe` 可用
- **AND** 腳本啟動一個 `omobab.exe`
- **AND** 腳本啟動兩個 `executor.exe`
- **AND** 兩個 frontend 都連到同一個 `127.0.0.1:50061`

#### Scenario: frontend identity 不互相覆蓋

- **WHEN** `run_2player.bat` 啟動兩個 frontend
- **THEN** player 1 frontend 使用不同於 player 2 的 `OMB_PLAYER_NAME`
- **AND** player 1 frontend 使用不同於 player 2 的 `OMB_LOCKSTEP_PLAYER_NAME`
- **AND** log 或 window title SHALL 足以分辨兩個 frontend process

### Requirement: 每個 player 控制自己的英雄

Authoritative runtime SHALL 將 lockstep `player_id` 綁定到對應 Hero entity。所有 hero-owned input，包括 `MoveTo`、`CastAbility`、`UpgradeAbility` 與 `ItemUse`，SHALL 套用到 submitting `player_id` 的 hero，而不是任意第一個 Player faction hero。

omfx local replica SHALL 使用相同的 shared runtime ownership mapping，讓本地模擬與後端權威結果一致。

#### Scenario: player 1 移動只影響 player 1 hero

- **WHEN** 兩個 frontend 都已加入 lockstep，且 player 1 送出 `MoveTo`
- **THEN** omb 將 `MoveTarget` 寫入 player 1 綁定的 hero
- **AND** player 2 綁定的 hero 不會被該 input 改變 `MoveTarget`

#### Scenario: player 2 技能只由 player 2 hero 施放

- **WHEN** player 2 送出 `CastAbility { ability_index: 0 }`
- **THEN** omb 以 player 2 綁定的 hero 作為 caster 建立 `ScriptEvent::SkillCast`
- **AND** player 1 綁定的 hero 不會成為該次 caster

### Requirement: player 建造的塔具有 owner

Tower place SHALL 使用 submitting `player_id` 作為新 tower owner。該 owner SHALL 存在於 authoritative runtime 與 omfx local replica 可讀的 deterministic state 中，並 SHALL 進入 snapshot/render 所需資料，使 frontend 能判斷 selected tower 是否由本地 player 擁有。

#### Scenario: player 1 建塔後 owner 是 player 1

- **WHEN** player 1 送出有效的 `TowerPlace`
- **THEN** omb spawn 的 Tower entity 記錄 owner `player_id = 1`
- **AND** 下一個 snapshot 中該 tower 的 owner metadata 可讓 player 1 frontend 判斷它是本地塔
- **AND** player 2 frontend 可判斷它不是本地塔

#### Scenario: player 2 建塔後 owner 是 player 2

- **WHEN** player 2 送出有效的 `TowerPlace`
- **THEN** omb spawn 的 Tower entity 記錄 owner `player_id = 2`
- **AND** 該 tower 的升級與出售權限屬於 player 2

### Requirement: 只能操作自己建造的塔

Tower sell 與 tower upgrade SHALL 驗證 submitting `player_id` 等於 tower owner。若 requester 不是 owner，後端 MUST 拒絕操作、記錄 warning，且不得修改該 tower、不得扣金或加金。

frontend SHOULD 對非本地 owner 的 selected tower 隱藏或停用升級與出售控制；即使 frontend 送出非法 input，authoritative runtime 仍 MUST 拒絕。

#### Scenario: player 1 不能升級 player 2 的塔

- **WHEN** player 2 已建造一座 tower
- **AND** player 1 送出該 tower 的 `TowerUpgrade`
- **THEN** omb 拒絕該 input 並記錄包含 requester pid 與 tower owner 的 warning
- **AND** 該 tower 的 `upgrade_levels` 不變
- **AND** player 1 的 gold 不變

#### Scenario: player 2 不能賣 player 1 的塔

- **WHEN** player 1 已建造一座 tower
- **AND** player 2 送出該 tower 的 `TowerSell`
- **THEN** omb 拒絕該 input 並記錄包含 requester pid 與 tower owner 的 warning
- **AND** 該 tower 不會進入 `Outcome::EntityRemoved`
- **AND** player 2 的 gold 不會增加

#### Scenario: owner 可以升級自己的塔

- **WHEN** player 1 已建造一座 tower 並有足夠 gold
- **AND** player 1 送出該 tower 的 `TowerUpgrade`
- **THEN** omb 套用 upgrade、扣除 player 1 綁定 hero 的 gold
- **AND** 下一個 snapshot 反映該 tower 更新後的 `upgrade_levels`

### Requirement: 雙人啟動 smoke 可驗證

專案 SHALL 提供可重複的 smoke 驗證方式，確認兩個 player 都能 join、各自送 input、各自控制 hero，且 tower ownership enforcement 生效。

#### Scenario: smoke log 顯示兩位 player 加入

- **WHEN** 執行 `run_2player.bat` 並等待兩個 frontend 完成 lockstep join
- **THEN** omb log 包含兩筆 `JoinRequest`，分別指派不同 `player_id`
- **AND** 兩個 frontend log 各自記錄自己的 assigned `player_id`

#### Scenario: smoke 中交叉操作塔會被拒絕

- **WHEN** smoke 或測試讓 player 1 建塔，並讓 player 2 嘗試升級或出售該塔
- **THEN** omb 拒絕 player 2 的操作
- **AND** 該塔仍存在且狀態不變
