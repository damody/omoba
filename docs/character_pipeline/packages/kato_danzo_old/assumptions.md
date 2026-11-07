# C08_old 加藤段藏（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C08*_old` 說明，五招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C08"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:324`），choose_11 逐一 `AddAbility("C08?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C08"]["11"] = "D"`（`addon_game_mode.lua:427`），C08D_old 出生一級。
2. **忍法．空禪之術（C08D_old）**：依 `npc/C_Neutral/C08.txt:543-585`、`vscripts/heroes/C_Neutral/C08.lua:564-590`、`vscripts/heroes/modifier_C08D_old.lua` 還原。觸發判定照本體 A05D 的 FC-024 改成結算前判定致死傷害（原作 `damage > GetHealth()` 在扣血後比較）。全域 `c08d_lock` 改為每位段藏自己的 10 秒鎖。化樹的 MOVESPEED_BASE_OVERRIDE 50 因同時 ROOTED 不寫。
3. **忍法．雲隱之術（C08W_old）**：依 `C08.txt:587-708`、`C08.lua:594-652` 還原。推定 KV 的 `ApplyModifier modifier_invisible`（628 行）因技能 Modifiers 沒定義這個名稱而失敗，原作隱形只有腳本寫死的 20 秒（622 行）；依 D22 修正為與增益同長的 `C08W_old_duration`。OnAttack 視為出手瞬間（同本體 C08D）。
4. **忍法．千鳥（C08E_old）**：依 `C08.txt:705-770`、`C08.lua:655-713` 還原。三次落雷在 1.0／1.4／1.8 秒；全域 `counter` 改為每次施放各自計數；目標實體失效時改在施放時的目標位置繼續落雷（原作報錯中斷）。
5. **忍法．紅蓮之術（C08R_old）**：依 `C08.txt:773-883`、`C08.lua:717-776` 還原。九根火柱的位置照 `nobu_atan2`（`vscripts/library/math.lua:5-8`）與 ±90° 的算式；光環同名不疊加、離開後殘留 0.5 秒、以 0.1 秒更新是 Dota 引擎行為的推定。`move_speed` −20～−50 原作沒有引用，依 D22 推定為火柱的緩速並套用（請工作線 D 確認是否採用）。
6. **飛鳶加藤（C08T_old）**：依 `C08.txt:886-915`，只有被動敏捷；學習等級照 Dota 大招預設 6/12/18。
