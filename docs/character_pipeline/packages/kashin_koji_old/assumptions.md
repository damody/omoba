# C16_old 果心居士（舊版）假設

原作 `panorama/localization/addon_tchinese.txt`（UTF-16）沒有任何 `C16*_old` 說明，五招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位與 D**：`heromap_skill["C16"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:332`），choose_11 逐一 `AddAbility("C16?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C16"]["11"] = "D"`（`addon_game_mode.lua:435`），出生自動學 `C16D_old`，寫成 `hero.auto_learn` 且 `learnable = false`（MaxLevel 1）。
2. **幻術．殘視（C16W_old）**：名稱沿用本體 C16D（同一個 CreateMirror、圖示 `custom/C16/C16D`）。說明依 `npc/C_Neutral/C16.txt:598-649`、`vscripts/heroes/C_Neutral/C16.lua:15-94,562-569` 還原。KV 兩個 `"02"` 鍵推定都保留；目標只能是英雄，所以只用到 `incoming_damage2`。友方英雄目標推定可以指定自己（同本體寫 `self_cast_allowed`）。
3. **咒術．咒影（C16E_old）**：名稱取 KV 註解（`C16.txt:651`）。說明依 `C16.txt:651-782` 與 `C16.lua:571-585` 還原。鬼影的 `modifier_show_invi` 光環推定讓 900 內敵人對所有隊伍失去隱形，資料以 `revealed`（只對果心居士隊伍）近似，雙隊對戰下結果相同。
4. **咒術．行空魂（C16R_old）**：名稱取 KV 註解（`C16.txt:784`）。和本體共用 Exorcism 程式，只換數值；魂體 AI 沿用本體的 `chase_target` 寫法。
5. **亡魂不散（C16D_old）**：原作沒有名稱（只有圖示 `custom/C16/C16D_old`），名稱是依效果自取。說明依 `C16.txt:567-596`、`C16.lua:552-560`、`npc/npc_units_custom_load.txt:1900-1943` 還原。產生器不支援被動的 `on_death`，改由被動每 0.5 秒補上隱藏狀態 `c16d_old_wraith_bond`，由該狀態的 `on_death` 產生亡靈（同 A10R_old 的寫法）：出生後 0.5 秒內死亡不會觸發，與原作不同。亡靈沒有存活時間，推定永久存在。
6. **幻咒之術．奇門遁甲（C16T_old）**：名稱取 KV 註解（`C16.txt:983`）。說明依 `C16.txt:983-1087`、`C16.lua:540-550,587-614` 還原。每顆殺生石出生時的 275 暈眩在原作因 `modifier_stunned` 不在 Modifiers 而失效；依主 session 2026-11-07 決定照設計意圖修（quirk `fixed`，FC-159），資料寫成每顆石頭暈眩周圍敵人。`nobu_modifier_rooted`（移速固定 1）以 `root` 表示。`RequiredLevel` 沒寫，依 ULTIMATE 預設 6/12/18。
7. **召喚物**：四個模板 `c16_old_death_wraith`、`c16_old_shadow_ghost`、`c16_old_spirit`、`c16_old_stone`，等 D 重產生 `v3_summons.lua` 後才是 content summon（v3_rust 目前只產生 1/5 招）。
