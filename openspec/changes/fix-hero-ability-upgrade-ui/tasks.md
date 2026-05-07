## 1. Lockstep 傳輸格式

- [x] 1.1 在 `proto/game.proto` 新增帶有 `ability_index` 的 `UpgradeAbility` 輸入訊息。
- [x] 1.2 在不變更既有 variant 編號的前提下，將新的 `UpgradeAbility` variant 加入 `PlayerInput.oneof`。
- [x] 1.3 更新會模式比對 `PlayerInput` 動作種類的 Rust 呼叫點，讓新 variant 可被識別。

## 2. 後端技能升級處理

- [x] 2.1 新增 `PendingAbilityUpgrade` 與 `PendingAbilityUpgradeQueue` resources，並與其他 lockstep 待處理佇列一起匯出/初始化。
- [x] 2.2 在 `omb/src/tick/player_input_tick.rs` 中，將 `PlayerInputEnum::UpgradeAbility` 路由到 pending ability-upgrade queue，並記錄 player id 與 slot index。
- [x] 2.3 實作 `GameProcessor::handle_ability_upgrade_from_input`，用來解析玩家英雄、驗證欄位/綁定技能/技能點/最高等級、更新 `Hero.ability_levels`、扣除 `Hero.skill_points`，並 enqueue `ScriptEvent::SkillLearn`。
- [x] 2.4 實作 `GameProcessor::drain_pending_ability_upgrades`，並在 `omb/src/state/core.rs` 與 `omfx/game/src/sim_runner.rs` 中，於其他待處理輸入清空處理相同的 post-dispatch 邊界呼叫它。
- [x] 2.5 新增目標後端測試，或擴充既有測試以覆蓋升級成功、無技能點拒絕與已達最高等級拒絕。

## 3. 前端輸入與 HUD

- [x] 3.1 將 `omfx/game/src/lib.rs` 中舊版 `Shift+W/E/R/T` 的 `UpgradeSkill` stub，替換為送出 lockstep `PlayerInput::UpgradeAbility`，並將 W/E/R/T 對應到索引 0/1/2/3。
- [x] 3.2 為每個技能欄位新增一個三角升級按鈕 UI handle 與 hit-test rect，並在 HUD layout 更新時讓每個按鈕跟著其技能圖示定位。
- [x] 3.3 根據 snapshot-backed `hero_state.skill_points`、綁定技能 id、目前技能等級，以及技能 `max_level` 計算按鈕可見性與可點擊性，不進行 optimistic 的本地技能點 mutation。
- [x] 3.4 在左鍵 UI handling 中優先處理三角升級按鈕點擊，命中時送出對應 slot 的 `PlayerInput::UpgradeAbility`，並阻止該點擊落到技能施放、放塔、選塔或地圖點擊邏輯。
- [x] 3.5 讓 tooltip 升級提示與技能等級點，和按鈕可見性使用相同的最高等級來源並保持一致。

## 4. 驗證

- [x] 4.1 實作後針對 proto consumers、omb 與 omfx 執行相關 Cargo check/test 指令。
- [ ] 4.2 手動 smoke test 一個有可用技能點的英雄：確認 `Shift+W/E/R/T` 會升級對應欄位，且下一個快照會更新 HUD。
- [ ] 4.3 手動確認三角升級按鈕只會出現在可升級技能上，點擊會升級對應技能，並在技能點耗盡或技能達最高等級時消失。

## 5. 施法與永久 Buff 修正

- [x] 5.1 修正 omfx 一般施法快捷鍵映射為 `W/E/R/T -> ability_index 0/1/2/3`，並移除 `Q` 對英雄技能欄位的施放映射。
- [x] 5.2 新增 `PendingAbilityCast` 與 `PendingAbilityCastQueue`，讓 `PlayerInputEnum::CastAbility` 走 queue/drain 而不是 log-only stub。
- [x] 5.3 實作 `GameProcessor::handle_ability_cast_from_input`：解析 Player-faction hero、驗證 slot/綁定技能/已學/cooldown，並 enqueue `ScriptEvent::SkillCast`。
- [x] 5.4 在 omb host 與 omfx sim_runner 的相同 post-dispatch、pre-script-dispatch 邊界 drain ability cast queue。
- [x] 5.5 修正 hero buff snapshot extraction，將 `Fixed64::from_raw(i64::MAX)` 或等效永久 sentinel 正規化為 `remaining_secs == -1.0`。
- [x] 5.6 新增或擴充測試，覆蓋 `W/E/R/T` cast mapping、backend cast enqueue、未學技能拒絕，以及永久 buff 顯示為 `∞`。
- [ ] 5.7 執行相關 `cargo test/check`，再由使用者手動驗證 W/E/R/T 施法與 R 被動 buff 顯示。
- [x] 5.8 新增技能圖示本體左鍵施法：點擊欄位 0..3 的 icon 送出對應 `CastAbility { ability_index }`，並阻止該點擊落到 TD/map click handling。
- [x] 5.9 將三角升級按鈕移到技能圖示上方的 LoL-style 位置，更新 UI positioning 與 hit-test，並確保三角點擊優先於 icon 施法。
- [ ] 5.10 手動驗證：點 icon 施法、點 icon 上方三角升級，兩者不互相觸發。
