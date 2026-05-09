## 1. Content Metadata 與 Assets

- [ ] 1.1 在 `scripts/base_content/assets/towers/` 建立 tower combat asset 目錄與 `README.md`，說明 base/barrel/body frames 命名規則、PNG alpha、render mode、rotation mode、barrel layout、attack phase timing、pivot/offset/recoil 座標系與替換流程。
- [ ] 1.2 為 shipped TD towers 新增甜點戰爭 placeholder PNG：`tower_dart_base.png`、`tower_dart_barrel.png`、`tower_bomb_base.png`、`tower_bomb_barrel.png`、`tower_ice_base.png`、`tower_ice_barrel.png`、`tower_tack_base.png`。
- [ ] 1.3 為 `tower_tack` 新增動態砲管數 variant PNG：`tower_tack_barrel_8.png`、`tower_tack_barrel_12.png`、`tower_tack_barrel_16.png`，圖面需清楚呈現 8/12/16 根 radial 針孔或砲管。
- [ ] 1.4 為 shipped tower barrel 新增可選 animation frame PNG naming pattern，例如 `tower_dart_barrel_frame_01.png`、`tower_bomb_barrel_frame_01.png`、`tower_ice_barrel_frame_01.png`，並為 tack 8/12/16 variants 提供對應 frame pattern。
- [ ] 1.5 新增無砲管範圍傷害塔 body animation placeholder，例如 `tower_cake_splash_frame_01.png` 到 `tower_cake_splash_frame_06.png`。
- [ ] 1.6 在 `openspec/changes/split-tower-base-barrel-rendering/asset-prompts.md` 為每張 combat tower PNG 與 animation frame pattern 提供完整甜點戰爭 ChatGPT 生圖提示詞。
- [ ] 1.7 維護 `openspec/changes/split-tower-base-barrel-rendering/combat-tower-layout.svg` 示意圖，讓它顯示一般 target-facing tower、barrel frame animation、`tower_tack` 8/12/16 variants、`scale_pulse` recoil 與無砲管 animated-area tower 的意圖。
- [ ] 1.8 在 `scripts/lua_data/templates/towers.lua` 為每個 shipped tower 加入 `render` table，包含 render mode、base path、barrel path/frames、rotation mode、barrel layout、barrel offset、barrel pivot、muzzle offset 與 recoil distance/scale/duration/return 參數。
- [ ] 1.9 將 `tower_tack` 設為不跟目標旋轉的針塔特例，例如 `rotation_mode = "fixed"`、`barrel_layout = "radial_count_variants"`、8/12/16 variants、variant frames 與 `recoil.mode = "scale_pulse"`。
- [ ] 1.10 新增無砲管範圍傷害塔 metadata，例如 `tower_cake_splash` 使用 `render_mode = "animated_area"`、body animation frames 與 `scale_pulse` recoil。
- [ ] 1.11 補上缺值 default 規則，確保未宣告 render metadata 的塔仍能產生可用 default，且 default render mode 為 base/barrel、default rotation mode 為 target-facing。

## 2. Codegen 與 Script ABI Metadata

- [ ] 2.1 在 `omoba-template-ids` 的 tower codegen model 中加入 render metadata 欄位，讀取 `towers.lua` 的 `render` table 並輸出 generated const。
- [ ] 2.2 在 `omoba-template-ids/src/lib.rs` 增加 ABI-neutral/const-facing tower render metadata 型別或欄位，避免使用 runtime-only dependency。
- [ ] 2.3 在 `scripts/script-abi` 新增 ABI-safe tower render metadata 型別，欄位使用 `RString`、`Fixed64`、primitive integer 或其他 abi_stable 相容型別。
- [ ] 2.4 更新 `scripts/base_content` tower metadata builder，將 generated render metadata 填入 `TowerMetadata`。
- [ ] 2.5 更新 `omb/src/comp/tower_registry.rs` 與 registry populate path，讓 `TowerTemplateRegistry` 保存 tower render metadata。
- [ ] 2.6 更新 gen-docs/catalog 顯示或至少不遺漏新增 tower render metadata，並確認缺值 default 不讓 docs pipeline panic。
- [ ] 2.7 新增所有單位共用 attack timing metadata，支援 `windup_ratio`、`backswing_ratio`、minimum durations 與 default fallback。

## 3. Snapshot 與 Render-Only Fire Cue

- [ ] 3.1 擴充 `omfx/game/src/sim_runner.rs::TowerTemplateSnapshot`，加入 render mode、base/barrel 圖片路徑、barrel frames、body animation frames、animation timing、rotation mode、barrel layout、barrel count variants、barrel offset、barrel pivot、muzzle offset、recoil mode、recoil scale 與 recoil 參數。
- [ ] 3.2 更新 `extract_snapshot` 建立 tower template Arc 的邏輯，從 `TowerTemplateRegistry` 投影 render metadata，並保持後續 snapshot 使用 O(1) `Arc::clone`。
- [ ] 3.3 確認 tower aiming 使用 authoritative snapshot data；若現有 `EntityRenderData.facing_rad` 未隨攻擊目標更新，補上 deterministic sim 內的 tower facing update 或 tower-specific aim field。
- [ ] 3.4 新增 `TowerFireFx` 與 `TowerFireFxQueue`，生命週期比照 `ExplosionFxQueue`，只作為 render-only snapshot event。
- [ ] 3.5 在 projectile/attack outcome processing 中，當 tower 實際開火時 push `TowerFireFx`，包含 tower entity id、spawn tick 與 firing direction。
- [ ] 3.6 在 `extract_snapshot` 以 `std::mem::take` drain `TowerFireFxQueue` 到 `SimWorldSnapshot.tower_fire_fx`，並確認 drain 後 queue 為 empty。
- [ ] 3.7 處理同一 tower 同 tick 多 projectile 的合併策略，避免 recoil pulse 疊加超過 metadata 設定。
- [ ] 3.8 在後端 attack scheduling 中建立 windup、impact、backswing 三階段，讓 projectile spawn/damage outcome 發生在 impact phase。
- [ ] 3.9 新增 render-only `AttackPhaseFx` 或等效 cue queue，在 windup 開始時推送 entity id、attack sequence、windup/impact/backswing timing、target/direction data。
- [ ] 3.10 讓 windup/backswing durations 隨 effective attack speed 縮短，並保留 minimum duration default。
- [ ] 3.11 在 `extract_snapshot` drain attack phase cues，確保 drain 不改 gameplay state/hash。

## 4. omfx Composite Tower Rendering

- [ ] 4.1 建立 tower combat texture/frame loader 與 cache，搜尋 `scripts/base_content/assets/towers/` 並提供 base/barrel/body frame fallback texture。
- [ ] 4.2 為 tower mirror/render cache 增加 composite handles，至少包含 base node、barrel node 或 animated-area node、last aim direction、active animation state 與 active recoil state。
- [ ] 4.3 在 tower entity 首次出現時建立 base/barrel render nodes；在 `removed_entity_ids` 移除時釋放或回收 nodes。
- [ ] 4.4 每 frame 依 snapshot-backed tower position 更新 base/barrel anchor，並依 tower render metadata 套用 barrel offset、pivot 與 z-order。
- [ ] 4.5 使用 authoritative `facing_rad` 或 tower aim field 旋轉 target-facing barrel，tower 暫時無目標時保持最近一次有效方向。
- [ ] 4.6 實作 fixed rotation mode：`tower_tack` 這類針塔的 barrel visual 不因目標或 fire cue direction 旋轉。
- [ ] 4.7 實作 `radial_count_variants`，讓 `tower_tack` 根據 snapshot `upgrade_levels` 在 8/12/16 barrel variant 間切換。
- [ ] 4.8 實作所有 barrel 的 frame animation，並在 attack windup cue 到達時立即開始或重播 barrel attack animation。
- [ ] 4.9 實作無砲管 `animated_area` tower，使用 body frame animation node，不建立 barrel node，也不朝單一目標旋轉。
- [ ] 4.10 消費 attack phase cue 與 `SimWorldSnapshot.tower_fire_fx` 啟動 animation/recoil state，target-facing tower 依 impact timing 對齊 fire frame 與 recoil。
- [ ] 4.11 實作 `scale_pulse` recoil mode，讓 `tower_tack` 與 animated-area tower 開火時整座塔先縮小再回彈放大，而不是選單一目標反方向後震。
- [ ] 4.12 實作 recoil default 與 metadata override，確認 `distance`、`scale`、`duration_ms`、`return_ms` 與 recoil mode 可依 tower 自定義。
- [ ] 4.13 確認缺圖、缺 barrel/body frame metadata 或 decode 失敗時 fallback，不 panic，且 tower 仍可見、可選取、可攻擊。
- [ ] 4.14 確認穩定 frame 不會每 frame 建立/刪除 tower composite nodes，也不會每 frame 重複從磁碟載入同一張 texture；`tower_tack` 只在升級 count state 改變時切換 variant。

## 5. Tests 與驗證

- [ ] 5.1 新增或更新 codegen tests，確認 `tower_dart` generated metadata 包含 base/barrel path、target-facing rotation mode、pivot/offset 與 recoil 參數。
- [ ] 5.2 新增或更新 codegen tests，確認 `tower_tack` generated metadata 包含 `rotation_mode = "fixed"`、`barrel_layout = "radial_count_variants"`、8/12/16 count variants 與 `recoil.mode = "scale_pulse"`。
- [ ] 5.3 新增 snapshot/unit test，確認 `TowerTemplateSnapshot` 包含 tower combat render metadata 且 tower template Arc 不會每 tick 重建。
- [ ] 5.4 新增 `TowerFireFxQueue` 與 `AttackPhaseFxQueue` drain tests，確認 cue 進入 snapshot 後 source queue 為 empty，且同 cue 不會重複出現在後續 snapshot。
- [ ] 5.5 新增 attack speed scaling test，確認 effective attack interval 變短時 windup 與 backswing duration 會縮短。
- [ ] 5.6 新增 determinism/hash 相關檢查，確認 render-only fire/attack phase cue queue 不改變 gameplay state hash 或 ECS gameplay component。
- [ ] 5.7 執行 `cargo test --manifest-path scripts/Cargo.toml -p base_content` 或等效 script workspace 測試。
- [ ] 5.8 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab` 或受影響 crate 的 focused tests。
- [ ] 5.9 執行 `cargo check --manifest-path omfx/Cargo.toml` 或等效 omfx build 檢查。
- [ ] 5.10 手動驗證 TD_1：放置 `tower_dart`、`tower_bomb`、`tower_ice` 後可看到 base/barrel 組合，barrel 朝向目標，攻擊前搖開始時 barrel animation 開始播放，impact 時 fire frame/recoil 對齊。
- [ ] 5.11 手動驗證 `tower_tack`：barrel 不跟單一 creep 旋轉，攻擊前搖開始時對應 count variant animation 播放，impact 時整座塔縮小再回彈放大，針發射方向與命中仍依 sim 規則正常。
- [ ] 5.12 手動驗證 `tower_tack` 升級：未升級顯示 8 根，升到對應更多針狀態後顯示 12 根，再升級後顯示 16 根。
- [ ] 5.13 手動驗證無砲管範圍傷害塔：沒有 barrel node，攻擊前搖開始播放 body frame animation，impact frame 對齊範圍傷害瞬間。
- [ ] 5.14 以 `combat-tower-layout.svg` 對照手動驗證結果，確認 target-facing tower、fixed tack tower、8/12/16 variant、barrel/body animation 與 `scale_pulse` 表現符合示意圖。
- [ ] 5.15 手動驗證 fallback：暫時改名其中一張 barrel/body frame PNG，確認 omfx log 缺圖但不 panic，且 tower 仍可操作。
- [ ] 5.16 手動或壓測驗證大量 tower 場景，確認 composite render 不造成明顯 frame-time regression 或 node allocation churn。
