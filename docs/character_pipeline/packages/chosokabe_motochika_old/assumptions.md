# C20_old 長宗我部元親（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C20*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名沿用本體同位置的招式（KV 註解 `npc/C_Neutral/C20.txt:335、432、491、549` 與本體同名）。

1. **槽位**：`heromap_skill["C20"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:336`），choose_11 逐一 `AddAbility("C20?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；自動學習為空（`addon_game_mode.lua:439`），沒有 D。
2. **炎之加護（C20W_old）**：說明依 `npc/C_Neutral/C20.txt:335-431` 與 `vscripts/heroes/C_Neutral/C20.lua:131-167` 還原。和本體行為相同，只有數值不同；鎖與反射的檢查順序不同但結果一樣。
3. **鬼血之流（C20E_old）**：說明依 `npc/C_Neutral/C20.txt:432-490` 還原（datadriven Heal＋HEALTH_REGEN_PERCENTAGE）。
4. **炎之舞（C20R_old）**：說明依 `npc/C_Neutral/C20.txt:491-548` 與本體共用的 `C20R_OnSpellStart`（`C20.lua:41-65`）還原。
5. **鬼若子（C20T_old）**：說明依 `npc/C_Neutral/C20.txt:549-657`、`C20.lua:104-129`、`vscripts/AON_Cleave.lua:299-329`（AON_Cleave_C20）還原。攻擊間隔與攻擊力加成在 KV 裡被註解掉、`attack`／`Basic_Attack_Speed` 沒被使用，視為作者有意移除（no_effect），不當 bug 修。`%attack_damage` 推定為主目標護甲後的攻擊傷害，同本體。大招等級 6/12/18 是 Dota ULTIMATE 預設。
