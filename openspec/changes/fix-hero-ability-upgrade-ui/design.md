## 背景

omfx 會根據 `SimWorldSnapshot` 英雄資料與 `AbilityDefSnapshot` metadata 渲染英雄技能列。HUD 已經知道本地英雄的 `skill_points`、各欄位技能 id、各欄位等級，以及每個技能的 `max_level`，但目前只渲染等級點與 tooltip 文字。

鍵盤處理目前會把 `Shift+W/E/R/T` 當成舊版 `UpgradeSkill` 指令路徑。該舊版送出路徑已在 lockstep 遷移期間移除，現在只會記錄 stub；而 `CastAbility`、TD 塔行為、物品與移動都使用 `PlayerInput`，並透過 omb 主機與 omfx replica 都會 drain 的 pending queue 以決定性方式套用。

## 目標 / 非目標

**目標：**

- 讓 `Shift+W/E/R/T` 送出用於升級技能欄位 0..3 的決定性 lockstep 輸入。
- 讓一般 `W/E/R/T` 施法送出技能欄位 0..3 的決定性 lockstep 輸入，且 backend 實際排入 `ScriptEvent::SkillCast`。
- 讓點擊技能圖示本體也送出對應技能欄位的 `CastAbility` lockstep 輸入。
- 透過 omb 的 `&mut World` 入口點套用技能升級，並驗證玩家英雄擁有權、可用技能點、綁定技能 id 與最高等級。
- 保留 `ScriptEvent::SkillLearn` 行為，讓被動技能/on-learn hook 在成功升級後仍會執行。
- 使用以快照為依據的 `skill_points`、技能等級與技能最高等級，在每個可升級技能圖示上方渲染類 LoL 的可點擊三角升級按鈕。
- 讓玩家可以點擊技能圖示上的三角升級按鈕，送出與 `Shift+W/E/R/T` 相同的技能升級 lockstep 輸入。
- 讓永久/被動 buff 的剩餘時間使用 `∞` 顯示，而不是將 permanent sentinel 換算為超大秒數。
- 讓快照維持為可見技能點與等級變更的真實來源。

**非目標：**

- 重新設計英雄技能樹或新增完整技能學習面板。
- 為技能升級新增專用的成功/失敗 ack channel。
- 除了使用既有 `max_level` 資料之外，不變更技能 script ABI 或技能 metadata schema。
- 重做物品商店的舊版 stub。

## 決策

1. 將 `UpgradeAbility` 新增為 `PlayerInput.oneof` variant，並帶有 `ability_index: uint32`。

   理由：技能升級是玩家遊戲行為，應該與 `CastAbility`、`TowerUpgrade`、`ItemUse`、`MoveTo` 走相同的決定性 lockstep 路徑。使用欄位索引可以讓 wire payload 穩定，即使不同英雄的技能 id 不同也一樣。曾考慮的替代方案：恢復舊版 `PlayerCommand { action: "upgrade_skill" }`；拒絕原因是它會繞過 lockstep scheduler，且無法在本地模擬 replica 上以相同方式 replay。

2. 在新的 pending ability-upgrade resource 中排隊升級，並在主機與 replica 上於 `player_input_tick` 之後 drain。

   理由：變更 `Hero.ability_levels`、扣除 `skill_points`、讀取 `AbilityRegistry`、推送 `ScriptEvent::SkillLearn` 都需要 `&mut World`，符合現有 pending queue 模式。drain 應該發生在 script dispatch 前，讓 learn hook 可以在同一個 tick 執行。曾考慮的替代方案：直接在 `player_input_tick::Sys` 內 mutate Hero；拒絕原因是 specs `SystemData` 會需要大量 mutable storages/resources，且會偏離目前輸入路由慣例。

3. 實作可重用的 `GameProcessor::handle_ability_upgrade_from_input(world, ability_index, owner_pid)` 入口點。

   理由：將 mutation 邏輯放在 `GameProcessor` 符合塔/物品/移動的 lockstep handler，也讓測試覆蓋更直接。handler 應該使用目前的 player-faction 模式解析玩家英雄，拒絕無效索引、缺少技能綁定、沒有技能點，並使用 `AbilityRegistry::get(id).max_level`；只有在 metadata 缺失時才使用安全 fallback。

4. 將升級就緒狀態渲染為每個技能圖示上的 overlay 文字按鈕。

   理由：目前 Fyrox HUD 已經使用 `TextBuilder` overlay 呈現按鍵標籤、等級點與 cooldown 文字。為每個技能新增一個 `Text` handle 與對應 hit-test rect 是最小改動，也能避免新增貼圖資產。只有當 `hero_state.skill_points > 0`、欄位有技能 id，且 `current_level < max_level` 時才顯示按鈕並啟用點擊；否則文字為空且 rect 移到螢幕外或停用。曾考慮的替代方案：把標記編進等級點字串；拒絕原因是使用者明確期待技能圖示上有三角形，且該三角形應可直接點擊升級。

5. 三角升級按鈕點擊時送出與快捷鍵相同的 `UpgradeAbility` input。

   理由：鍵盤與滑鼠升級應共用同一條 lockstep 輸入路徑，避免兩套 gameplay mutation。omfx 左鍵 UI hit-test 應在技能施放、放塔或地圖點擊邏輯之前處理這些小按鈕，命中時只送出 input，不做 optimistic 等級或技能點變更。

6. 修正 `CastAbility` 走完整 lockstep gameplay path。

   理由：目前前端一般施法仍使用 `Q/W/E/R` 索引映射，導致 `W/E/R/T` 欄位錯位；backend `player_input_tick` 對 `CastAbility` 也只記錄 log，沒有排入 `SkillCast`。應改為前端 `W/E/R/T -> 0/1/2/3`，backend queue/drain pattern 與 ability upgrade 相同，drain 時解析 Player-faction hero、驗證技能已學且不在 cooldown，並在 script dispatch 前排入 `ScriptEvent::SkillCast`。點擊技能圖示本體也應共用此 `CastAbility` input helper，避免鍵盤與滑鼠施法分叉。

7. 將三角升級按鈕放在技能圖示上方，並讓 hit-test 優先於圖示施法。

   理由：使用者期待類 LoL 的升級按鈕位置，應在 icon 上方獨立呈現，而不是覆蓋在圖示內部。左鍵 hit-test 順序應為：升級三角按鈕優先，其次技能圖示本體施法，最後才落到 TD/map click。這樣點三角形會升級，點圖示會施法，兩者不會互相觸發。

8. 將永久 buff sentinel 正規化為 infinity display。

   理由：永久 buff 目前可能使用 `Fixed64::from_raw(i64::MAX)` 作為 duration sentinel；若 snapshot extraction 只判斷 `i32::MAX`，前端會顯示約 2,097,147 秒。snapshot extraction 應把 permanent sentinel 或明顯超大的 remaining 值轉成 `remaining_secs = -1.0`，讓既有 HUD 顯示 `∞`。

## 風險 / 取捨

- Proto regeneration 會觸及多個 workspace 中的 generated code -> 緩解方式是透過正常 Cargo build 使用既有 `build.rs`/prost generation，而不是手動編輯 generated files。
- 如果只有 omb 清空處理新佇列，主機與 replica 可能 desync -> 緩解方式是在 `omb/src/state/core.rs` 與 `omfx/game/src/sim_runner.rs` 的相同邊界，加入同一個 `GameProcessor::drain_pending_ability_upgrades` 呼叫，與其他待處理輸入清空處理保持一致。
- 既有 server-side `upgrade_skill` 有 hard-coded 最高等級檢查 -> 緩解方式是讓 lockstep handler 改用 `AbilityRegistry.max_level`，同時只在舊版行為仍存在時把它留作相容用途。
- UI 按鈕可能在送出輸入與下一個 tick 快照之間保持可點擊 -> 緩解方式是不 optimistic 地扣除技能點或隱藏按鈕；若重複點擊產生多個 input，後端仍以權威技能點與最高等級檢查拒絕無效升級。
- 技能圖示點擊與升級三角點擊可能 hit-test 重疊 -> 緩解方式是將三角按鈕移到 icon 上方並讓三角 rect 優先處理；三角命中時停止後續點擊流程。
- `CastAbility` 與 `UpgradeAbility` 若 drain 順序錯誤會影響同 tick 新學技能是否能立即施放 -> 緩解方式是維持明確 tick boundary：input routing 後先 drain upgrade/cast queues，再進 script dispatch；若同 tick 同時收到升級與施法，依 queue drain order 定義行為並以測試固定。
- 只在前端修正永久 buff 顯示會掩蓋資料問題 -> 緩解方式是在 snapshot extraction 端正規化 sentinel，前端只負責呈現 `remaining < 0` 為 `∞`。

## 遷移計畫

1. 擴充 `proto/game.proto`，並讓相依 Rust crates 透過既有 build 重新產生 prost code。
2. 新增 lockstep 路由、待處理佇列 resource，以及主機/replica 清空處理呼叫。
3. 更新 omfx 鍵盤處理、HUD 三角按鈕渲染與滑鼠 hit-test。
4. 在可行範圍內，針對 `omoba-core`、`omb` 與 `omfx` 程式碼路徑執行目標 build/tests 驗證。

Rollback 可透過一起 revert proto/input/UI 變更安全完成。沒有新輸入 variant 的舊 client 仍可執行既有行為；它們只是無法送出 lockstep 技能升級。

## 未決問題

- 無。
