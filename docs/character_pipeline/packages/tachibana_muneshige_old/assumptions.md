# C11_old 立花宗茂（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C11*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。2.0 版（`*_20`）不在本資料夾。

1. **槽位**：`heromap_skill["C11"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:327`），沒有 D；自動學習為空（430 行）。
2. **雷切（C11W_old）**：依 `npc/C_Neutral/C11.txt:528-645`、`vscripts/heroes/C_Neutral/C11.lua:339-350` 還原。自身增益：基礎攻擊力 + 每次出手 20% 爆擊。原作以留存的爆擊 modifier 實作，v3 改成 `modify_attack` 只影響當次攻擊（design_intent）。KV 沒寫 AbilityCastPoint，推定 0（且 IMMEDIATE）。
3. **雷之疾（C11E_old）**：依 `C11.txt:646-732` 還原，只對自己。
4. **雷之眼（C11R_old）**：與本體共用 `C11R_start`（`C11.lua:179-248`），只差 KV 數值（`C11.txt:734-812`）。
5. **西國無雙（C11T_old）**：依 `C11.txt:813-877`、`C11.lua:363-412` 還原。AbilitySpecial 兩個條目都用鍵名 `"02"`（832-842 行），推定 KV 保留重複鍵、兩個倍率都生效（同 asai_nagamasa_old 的判斷）。學習等級照 Dota 大招預設 6/12/18。
