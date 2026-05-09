## 原因

英雄升級後可以看到技能點，但目前 `Shift+W/E/R/T` 只會記錄舊版 stub，而不是送出權威的升級輸入，因此玩家無法透過預期的快捷鍵花費技能點。技能 HUD 也缺少在可升級技能上持續可見的升級標記，讓可用技能點容易被忽略。

## 變更內容

- 新增依欄位/索引升級英雄技能的 lockstep `PlayerInput` 路徑。
- 將 omfx 的 `Shift+W/E/R/T` 改接到該 lockstep 輸入，而不是已移除的舊版 `UpgradeSkill` stub。
- 在 omb 中針對玩家擁有的英雄套用升級，保留既有的可用技能點、綁定技能、最高等級與 `SkillLearn` 腳本事件檢查。
- 當本地英雄有技能點且技能尚未達最高等級時，在技能圖示上顯示可點擊的三角升級按鈕。
- 點擊技能圖示上的三角升級按鈕時，送出與 `Shift+W/E/R/T` 相同的 lockstep 技能升級輸入。
- 修正 `W/E/R/T` 一般按鍵施法，使四個技能欄位分別對應索引 `0/1/2/3`，並確保 backend lockstep 端真的 dispatch `SkillCast`。
- 點擊技能圖示本體時也能施放對應技能，走與 `W/E/R/T` 相同的 `CastAbility` lockstep 路徑。
- 將升級三角形做成類 LoL 的圖示上方按鈕，而不是覆蓋在技能圖示內部。
- 修正被動或永久 buff 的剩餘時間顯示，將 permanent sentinel 顯示為 `∞` 而不是超大秒數。
- 成功/失敗的升級回饋維持由快照提供權威結果；不新增專用的 client-side 成功狀態或 ack。

## 能力

### 新增能力

- 無。

### 修改能力

- `player-input-routing`：新增英雄技能升級輸入的端到端 lockstep 路由，並修正 `W/E/R/T` 施法輸入路由。
- `sim-snapshot-rendering`：新增以快照為依據的技能 HUD 可點擊升級按鈕，並修正永久 buff 顯示。

## 影響

- 受影響程式碼：`proto/game.proto`、`omoba-core` 中產生的 proto 使用者、`omfx/game/src/lib.rs` 中的 omfx 鍵盤/HUD 處理、`omb/src/tick/player_input_tick.rs` 中的 omb lockstep 路由，以及 omb 中處理英雄技能升級的 world mutation 入口點。
- 系統：KCP lockstep 輸入、英雄技能等級狀態、`ScriptEvent::SkillLearn`、技能圖示 HUD 渲染、滑鼠 hit-test，以及 tooltip 升級提示。
- 相容性：proto 新增一個 `PlayerInput.oneof` variant；既有輸入 variant 保持不變。
