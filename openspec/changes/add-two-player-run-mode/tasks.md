## 1. Ownership Data Model

- [ ] 1.1 在 shared runtime component 中新增 deterministic `PlayerOwner { player_id: u32 }` 或等效 owner metadata，並註冊到 authoritative world 與 omfx local replica world
- [ ] 1.2 調整 snapshot extraction 與 omfx render entity snapshot，讓 Hero/Tower owner metadata 可被 frontend 讀取
- [ ] 1.3 補測試確認 owner metadata 不進入非 deterministic 診斷路徑，且 authoritative/local replica 初始化一致

## 2. Hero Binding

- [ ] 2.1 調整 campaign/story hero initialization，確保 TD_1 至少產生兩個 Player heroes 並以穩定順序綁定 `player_id = 1`、`player_id = 2`
- [ ] 2.2 將 `player_hero_entity()` 改為依 `owner_pid` 查找 `PlayerOwner` hero，不再 fallback 到第一個 `FactionType::Player` hero
- [ ] 2.3 調整 `drain_pending_moves`、`handle_ability_cast_from_input`、`handle_ability_upgrade_from_input` 與 `handle_item_use_from_input`，確保全都套用到 submitting player 的 hero
- [ ] 2.4 新增 unit tests：player 1/2 的 `MoveTo`、`CastAbility`、`UpgradeAbility`、`ItemUse` 不會互相改到對方 hero

## 3. Tower Ownership Enforcement

- [ ] 3.1 調整 `spawn_td_tower` 或新增 owner-aware spawn entry point，讓 `handle_tower_spawn_from_input` 建立 tower 時寫入 submitting `player_id`
- [ ] 3.2 調整 `handle_tower_sell_from_input`，要求 target tower owner 等於 requester，拒絕時不 refund、不 enqueue `Outcome::EntityRemoved`
- [ ] 3.3 調整 `handle_tower_upgrade_from_input`，要求 target tower owner 等於 requester，拒絕時不扣金、不改 stats/buffs/flags/levels
- [ ] 3.4 新增 unit tests：owner 可升級/出售自己的塔，非 owner 升級/出售會被拒絕且狀態不變

## 4. Frontend Local Player UX

- [ ] 4.1 在 omfx lockstep connected flow 保存本地 assigned `player_id`，並讓 UI selection/side panel 可讀取
- [ ] 4.2 調整 tower selection/sidebar，非本地 owner 的 tower 不顯示或停用 sell/upgrade controls，且不送出 `TowerSell` / `TowerUpgrade`
- [ ] 4.3 調整 hero camera/selection 或 command path，讓本地操作明確對應本地 player hero
- [ ] 4.4 新增前端或 shared tests，驗證 non-owner tower UI 不會送出升級/出售 input

## 5. Two-player Launcher

- [ ] 5.1 新增 `run_2player.bat`，沿用 `run.bat` freshness/build/stage/start_backend/stop_backend 流程
- [ ] 5.2 在 `run_2player.bat` 以不同 `OMB_PLAYER_NAME`、`OMB_LOCKSTEP_PLAYER_NAME`、log/window identity 啟動兩個 `executor.exe`
- [ ] 5.3 確保 `run_2player.bat` 等待兩個 frontend process 結束後停止 backend，並處理 frontend 啟動失敗時的 cleanup
- [ ] 5.4 確認 `run_2player.bat` 使用 CRLF 行尾

## 6. Logging And Diagnostics

- [ ] 6.1 調整 executor logging，使雙前端可輸出到不同 log 檔或在 log/window title 中帶 player identity
- [ ] 6.2 調整 omb warning/info log，owner rejection 訊息包含 requester pid、tower entity id 與 actual owner pid
- [ ] 6.3 確認新增 diagnostics 不影響 state hash 或 gameplay deterministic state

## 7. Verification

- [ ] 7.1 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`
- [ ] 7.2 執行 `cargo test --manifest-path omfx/Cargo.toml -p omfx`
- [ ] 7.3 執行 `cargo test --manifest-path omoba-core/Cargo.toml`
- [ ] 7.4 手動執行 `run_2player.bat` smoke，確認 omb log 有兩個不同 `player_id` join，兩個 frontend 各自 connected
- [ ] 7.5 在 smoke 中驗證兩個 frontend 各自控制自己的 hero，且交叉升級/出售對方 tower 會被後端拒絕
