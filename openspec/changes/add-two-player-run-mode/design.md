## Context

目前 omb 的 KCP lockstep join path 會在 `JoinRequest` 時分配遞增 `player_id`，`TickBatch` 也會保留 `(player_id, PlayerInput)`。但需求是 client 在連到 server 前就要知道自己的 `player_id`，所以此 change 會把身份來源改成 launcher/client 設定，再由 server 驗證與登記。omfx 目前已支援用環境變數設定 `OMB_PLAYER_NAME` 與 `OMB_LOCKSTEP_PLAYER_NAME`，並把 input 送到 lockstep client；需要新增 `OMB_PLAYER_ID` 或等效設定。

缺口在 runtime ownership：`player_hero_entity()` 仍用第一個 `FactionType::Player` hero，塔建立時也只掛 `Faction::new(FactionType::Player, 0)`。因此第二個 player 的 input 會落到同一隻英雄，且塔 sell/upgrade 只能分辨「玩家陣營」，不能分辨是哪個 player 建的塔。

## Goals / Non-Goals

**Goals:**

- 提供 `run_2player.bat`，一鍵啟動一個 omb 後端與兩個 omfx 前端。
- 兩個前端在連線前就具有不同 `player_id`，並使用不同 player name / lockstep player name，共用同一個 server address、story、script DLL。
- authoritative runtime 與 omfx local replica 都能依 lockstep `player_id` 找到對應英雄。
- 英雄顯示名稱以前綴標示 `player_id`，讓兩個使用相同 hero template 的玩家可直接辨識。
- Tower place 會把 tower owner 設為 submitting `player_id`；tower sell/upgrade 只能由 owner 執行。
- 前端 UI 能辨識本地 `player_id`，避免對非本地 owner 的 tower 顯示或送出升級/出售操作。
- 兩個 player 的 `Faction.team_id` 保持相同 combat team，不使用 `player_id` 作為 `team_id`。

**Non-Goals:**

- 不建立網際網路 matchmaking、lobby、帳號系統或遠端 NAT traversal。
- 不要求超過兩個 player 的完整 UX；資料模型可支援 N 個 player，但本變更驗收聚焦兩個本機前端。
- 不重做 combat faction/team 規則；player ownership 與 combat faction 分開處理，兩位 player 仍可屬於同一 Player 陣營對抗 Enemy。
- 不把 observer/rejoin snapshot 流程改成完整多人 lobby 狀態同步。

## Decisions

### Decision: 新增獨立 owner 資料，兩個玩家共用同一個 `Faction.team_id`

新增或復用一個清楚表示 lockstep owner 的 component/resource，例如 `PlayerOwner { player_id: u32 }`，套用在 Hero 與玩家建造的 Tower。`Faction` 繼續表示 combat side；兩個玩家使用相同 `FactionType::Player` 與相同 `team_id`，避免 tower/hero attack filtering、vision、creep targeting 因為 `team_id` 被改成 player id 而產生副作用。

替代方案是把 `Faction.team_id` 改成 `player_id`。這看似省 component，但會把「玩家身份」與「戰鬥隊伍」綁死；目前 Player vs Enemy 規則已有 team 0/1 約定，改動風險較高。

### Decision: client-declared `player_id`，server 驗證而不臨時指派

`run_2player.bat` 會在啟動 frontend 前設定 `OMB_PLAYER_ID=1` 與 `OMB_PLAYER_ID=2`。omfx lockstep client 在連線前讀取這個值，並把它帶入 join handshake；server 收到 join 時驗證該 `player_id` 在允許範圍內且尚未被其他 active session 使用，成功後回覆同一個 `player_id`。若 proto `JoinRequest` 目前沒有 player id 欄位，實作需新增 optional/zero-default 欄位，並保留舊 client 的相容處理或明確拒絕未帶 id 的 player join。

這讓 client 在啟動 local replica、UI 與 input path 前就知道自己的身份，也避免「連上 server 後才知道 id」造成本地 hero/tower ownership 初始化延遲。

### Decision: hero 綁定保持 deterministic

runtime initialization 需在 authoritative world 與 local replica world 建立相同的 player hero owner mapping。若 TD_1 story 只有一個 hero template，第二個 player 可複製同一 hero template 並放在不同 spawn offset；若 story 已提供多個 heroes，則依穩定順序綁定 player 1、player 2。

替代方案是在 `JoinRequest` 當下動態 spawn hero。這會讓 omb authoritative world 與已啟動的 omfx local replica 更難保持一致，還要處理中途加入補快照與 entity id 對齊；本變更先採初始化時固定建立兩個 player heroes。

### Decision: hero display name 加上 `player_id` 前綴

初始化或 snapshot/render 顯示時，player-owned Hero 的可見名稱 SHALL 加上穩定前綴，例如 `[P1] Saika Magoichi`、`[P2] Saika Magoichi`。前綴來源為 owner metadata 的 `player_id`，不是 `team_id` 或連線順序。若內部 hero id/script id 需要維持原值，則只改 display name / label / HUD text，不改 script lookup 用的 unit id。

### Decision: 所有 gameplay input 以 owner_pid 查找 owner entity

`MoveTo`、`CastAbility`、`UpgradeAbility`、`ItemUse`、`TowerPlace`、`TowerSell`、`TowerUpgrade` 的 shared runtime entry point 都 SHALL 使用 submitting `player_id` 查找 `PlayerOwner` 對應 hero 或 tower。`player_input_tick` 只負責把 `player_id` 放進 pending queue，真正驗證集中在 `GameProcessor` drain/handler。

這維持現有 queue pattern：input system 不直接借 `&mut World`，authoritative omb 與 omfx local replica 透過同一份 `omoba-core` code apply input。

### Decision: `run_2player.bat` 沿用 `run.bat` freshness 流程，但自行管理兩個前端 process

`run_2player.bat` 應複用 `scripts\dev_run_freshness.ps1`、`scripts\start_backend.ps1` 與同樣的 build/stage 步驟。啟動後端後，分別以不同環境變數啟動兩個 `executor.exe` process，例如：

- player 1: `OMB_PLAYER_NAME=player1`、`OMB_LOCKSTEP_PLAYER_NAME=player1_lockstep`
- player 2: `OMB_PLAYER_NAME=player2`、`OMB_LOCKSTEP_PLAYER_NAME=player2_lockstep`
- player 1: `OMB_PLAYER_ID=1`
- player 2: `OMB_PLAYER_ID=2`

腳本需等待兩個前端結束，然後停止後端。`.bat` 必須保持 CRLF。

### Decision: 前端 log/window identity 可用環境變數隔離

雙前端同時在同一 working directory 啟動時，固定 `omfx_app.log` / `omfx.log` 會互相覆寫或交錯。實作時應讓 executor 支援可選 `OMFX_LOG_SUFFIX` 或等效環境變數，`run_2player.bat` 為兩個前端指定不同 log 檔與 window title suffix，方便人工驗證。

## Risks / Trade-offs

- [Risk] 兩個 local replica 的 entity id 若因初始化順序不一致而分歧，input 會在不同世界套到不同 entity。→ Mitigation：hero duplication 與 owner assignment 放在 shared `omoba-core` initialization，並以 deterministic order 建立。
- [Risk] 只在前端 UI 阻擋操作別人的塔不夠安全。→ Mitigation：後端 `handle_tower_sell_from_input` 與 `handle_tower_upgrade_from_input` 必須做 owner check，UI 只做 usability guard。
- [Risk] 使用 `Faction.team_id` 作 owner 可能破壞戰鬥規則。→ Mitigation：ownership 使用獨立 component/resource，兩位玩家維持同一個 combat `team_id`。
- [Risk] client-declared `player_id` 可能重複或被偽造。→ Mitigation：server join state 驗證 id 範圍與 active session uniqueness；本 change 聚焦本機 launcher，不引入帳號安全模型。
- [Risk] `run_2player.bat` 啟動兩個 GUI process 後 cleanup 不完整。→ Mitigation：記錄 backend PID，前端 wait 後停止後端；啟動前沿用 stale `taskkill` 清理。
- [Risk] 第二個 hero 使用同 template 可能影響辨識。→ Mitigation：hero display name 加上 `player_id` 前綴，並可搭配 log/player_id/position offset；角色選擇與不同英雄配置留給後續 change。

## Migration Plan

1. 新增 ownership component/resource 並註冊到 authoritative 與 local replica world。
2. 調整 lockstep join protocol/client/server，讓 client 在連線前設定並宣告 `player_id`，server 驗證後回覆相同 id。
3. 調整 hero initialization，確保 TD_1 至少產生兩個 `PlayerOwner` heroes，且兩個 heroes 使用相同 combat `team_id`。
4. 調整 shared `GameProcessor` owner lookup，讓所有 player-owned input 使用 `player_id`。
5. 調整 tower spawn/sell/upgrade，tower spawn 寫入 owner，sell/upgrade 驗證 owner。
6. 調整 snapshot/render metadata，讓 omfx 能知道 tower owner 與 local player id，並顯示帶 `player_id` 前綴的 hero name。
7. 新增 `run_2player.bat` 與雙前端 log/window identity。
8. 加入 unit/integration tests，最後用 `run_2player.bat` 做人工 smoke。
