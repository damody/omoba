# C10_old 香宗我部親泰（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C10*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位與沒有 D**：`heromap_skill["C10"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:326`），但 KV 沒有 `C10D_old`，`AddAbility("C10D_old")` 失敗，舊版實際只有四招；自動學習為空（429 行）。
2. **黑龍波（C10W_old）**：依 `npc/C_Neutral/C10.txt:541-637`、`vscripts/heroes/C_Neutral/C10.lua:200-316` 還原。行為是連鎖閃電：jump_count 20 包含第一個目標（第一擊後減一、大於零才跳），所以共命中 20 個單位；寫成 `chain jumps = jump_count - 1`。原作以目前目標反查串號、兩串重疊時會混在一起，依 D22 改成每串獨立。
3. **能量奪取（C10E_old）**：依 `C10.txt:638-719` 還原，全是 datadriven 動作（純粹傷害、固定量治療、暈眩）。
4. **無想轉生（C10R_old）**：依 `C10.txt:720-789`、`vscripts/heroes/C_Neutral/C19.lua:284-305` 還原。modifier_ninja_cloth 被移除後每秒補回，等於永久閃避，寫成被動屬性。
5. **昇龍亂舞（C10T_old）**：依 `C10.txt:790-873`、`C10.lua:319-372` 還原。OnAttack 視為出手瞬間；劍氣朝親泰當下面向飛出；RandomInt(1,100) > 50 才不觸發，機率正好 50%。學習等級照 Dota 大招預設 6/12/18。
