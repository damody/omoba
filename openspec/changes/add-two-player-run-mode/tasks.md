## 1. Ownership Data Model

- [x] 1.1 在 shared runtime component 中新增 deterministic `PlayerOwner { player_id: u32 }` 或等效 owner metadata，並註冊到 authoritative world 與 omfx local replica world
- [x] 1.2 調整 snapshot extraction 與 omfx render entity snapshot，讓 Hero/Tower owner metadata 可被 frontend 讀取
- [x] 1.3 確保兩個玩家的 Hero/Tower 使用相同 `Faction.team_id`，且所有 ownership 判斷只讀 owner metadata、不讀 `team_id`
- [x] 1.4 補測試確認 owner metadata 不進入非 deterministic 診斷路徑，且 authoritative/local replica 初始化一致

## 2. Client-declared Player ID

- [x] 2.1 新增 omfx client 設定讀取 `OMB_PLAYER_ID` 或等效欄位，並在建立 lockstep 連線前保存 local `player_id`
- [x] 2.2 擴充 lockstep join handshake，讓 client 宣告 `player_id`，server 成功 join 時回覆相同 `player_id`
- [x] 2.3 調整 omb lockstep state：不再為一般 player join 臨時分配不同 id，改為驗證 client-declared id 合法且未被 active session 使用
- [x] 2.4 新增 join tests：接受 `player_id = 1/2`，拒絕缺少 id、不合法 id、重複 id，且不靜默改派 id

## 3. Hero Binding

- [x] 3.1 調整 campaign/story hero initialization，確保 TD_1 至少產生兩個 Player heroes 並以穩定順序綁定 `player_id = 1`、`player_id = 2`
- [x] 3.2 確認兩個 Player heroes 使用同一個 combat `team_id`
- [x] 3.3 讓 player-owned Hero 的玩家可見名稱加上 `player_id` 前綴，例如 `[P1]` / `[P2]`，但不改 script lookup 使用的 hero id/unit id
- [x] 3.4 將 `player_hero_entity()` 改為依 `owner_pid` 查找 `PlayerOwner` hero，不再 fallback 到第一個 `FactionType::Player` hero
- [x] 3.5 調整 `drain_pending_moves`、`handle_ability_cast_from_input`、`handle_ability_upgrade_from_input` 與 `handle_item_use_from_input`，確保全都套用到 submitting player 的 hero
- [x] 3.6 新增 unit tests：player 1/2 的 `MoveTo`、`CastAbility`、`UpgradeAbility`、`ItemUse` 不會互相改到對方 hero，且 hero display name 帶正確 `player_id` 前綴

## 4. Tower Ownership Enforcement

- [x] 4.1 調整 `spawn_td_tower` 或新增 owner-aware spawn entry point，讓 `handle_tower_spawn_from_input` 建立 tower 時寫入 submitting `player_id`
- [x] 4.2 確認 player 1/2 建造的 tower 使用同一個 combat `team_id`
- [x] 4.3 調整 `handle_tower_sell_from_input`，要求 target tower owner 等於 requester，拒絕時不 refund、不 enqueue `Outcome::EntityRemoved`
- [x] 4.4 調整 `handle_tower_upgrade_from_input`，要求 target tower owner 等於 requester，拒絕時不扣金、不改 stats/buffs/flags/levels
- [x] 4.5 新增 unit tests：owner 可升級/出售自己的塔，非 owner 升級/出售會被拒絕且狀態不變

## 5. Frontend Local Player UX

- [x] 5.1 在 omfx 啟動/lockstep connected flow 使用 configured `player_id`，並讓 UI selection/side panel 可讀取
- [x] 5.2 調整 tower selection/sidebar，非本地 owner 的 tower 不顯示或停用 sell/upgrade controls，且不送出 `TowerSell` / `TowerUpgrade`
- [x] 5.3 調整 hero camera/selection 或 command path，讓本地操作明確對應本地 player hero
- [x] 5.4 新增前端或 shared tests，驗證 non-owner tower UI 不會送出升級/出售 input

## 6. Two-player Launcher

- [x] 6.1 新增 `run_2player.bat`，沿用 `run.bat` freshness/build/stage/start_backend/stop_backend 流程
- [x] 6.2 在 `run_2player.bat` 以不同 `OMB_PLAYER_ID`、`OMB_PLAYER_NAME`、`OMB_LOCKSTEP_PLAYER_NAME`、log/window identity 啟動兩個 `executor.exe`
- [x] 6.3 確保 `run_2player.bat` 等待兩個 frontend process 結束後停止 backend，並處理 frontend 啟動失敗時的 cleanup
- [x] 6.4 確認 `run_2player.bat` 使用 CRLF 行尾

## 7. Logging And Diagnostics

- [x] 7.1 調整 executor logging，使雙前端可輸出到不同 log 檔或在 log/window title 中帶 player identity
- [x] 7.2 調整 omb warning/info log，owner rejection 訊息包含 requester pid、tower entity id 與 actual owner pid
- [x] 7.3 確認新增 diagnostics 不影響 state hash 或 gameplay deterministic state

## 8. Verification

- [x] 8.1 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`
- [x] 8.2 執行 `cargo test --manifest-path omfx/Cargo.toml -p omfx`
- [x] 8.3 執行 `cargo test --manifest-path omoba-core/Cargo.toml`
- [x] 8.4 手動執行 `run_2player.bat` smoke，確認 omb log 接受兩個 client-declared `player_id`，兩個 frontend 各自 connected
- [ ] 8.5 在 smoke 中驗證兩個 frontend 各自控制自己的 hero，英雄名顯示 `player_id` 前綴，兩者使用相同 combat `team_id`，且交叉升級/出售對方 tower 會被後端拒絕
