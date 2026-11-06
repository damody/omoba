# B06_old 真田幸村（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）只有 `DOTA_Tooltip_modifier_B06W_old`「真田勇士」（2034-2035 行），四招都沒有技能說明，說明依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["B06"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:285`），choose_11 逐一 `AddAbility("B06?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），自動學習為空（`addon_game_mode.lua:388`）。沒有 D。
2. **真田勇士（B06W_old）**：說明依 `npc/B_Unified/B06.txt:93-164` 還原。`modifier_B06W` 有 `MODIFIER_ATTRIBUTE_MULTIPLE`（140 行），推定每次施放另掛一份、各自計時並疊加數值，寫 `stacking = "independent"`。只回血、不回魔。
3. **炎突（B06E_old）**：說明依 `npc/B_Unified/B06.txt:270-366` 還原。`OnProjectileHitUnit`（310-325 行）沒有呼叫 `b06e_Shot`，所以不彈跳；`b06e_start`（`vscripts/heroes/B_Unified/B06_Sanada_Yukimura.lua:69-87`）設的計數沒有人讀。名稱沿用本體「連牙炎突」去掉「連牙」，因為沒有連續彈跳（推定，原作舊版沒有名稱）。
4. **心眼（B06R_old）**：說明依 `npc/B_Unified/B06.txt:489-563` 與 `vscripts/heroes/C_Neutral/C19.lua:284-305` 還原。只有閃避。OnUpgrade 移除後 0.1 秒重掛（500-519 行）的空窗不另外表達。
5. **日本第一武士（B06T_old）**：說明依 `npc/B_Unified/B06.txt:726-847` 還原。沒有 AbilityCastPoint（立即施放）、沒有全軍光環；移速是固定值 +250。名稱沿用本體。
