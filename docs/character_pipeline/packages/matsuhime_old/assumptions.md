# C19_old 松姬（舊版）假設

原作 `panorama/localization/addon_tchinese.txt`（UTF-16）沒有任何 `C19*_old` 說明，四招的名稱取 KV 註解，說明依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C19"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:335`），choose_11 逐一 `AddAbility("C19?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C19"]["11"] = ""`（`addon_game_mode.lua:438`）。沒有 D。
2. **癱瘓毒（C19W_old）**：說明依 `npc/C_Neutral/C19.txt:101-210` 與 `vscripts/heroes/C_Neutral/C19.lua:211-227` 還原。中毒的 Target 區塊沒有 Radius 與 Teams，照本體 C19R 的讀法只套被命中的英雄或一般單位。死亡時開關不會自動關閉，依主 session 2026-11-07 決定照設計意圖修（quirk `fixed`，FC-167），資料在死亡時關閉。
3. **治療噴霧（C19E_old）**：說明依 `C19.txt:239-281` 與 `C19.lua:230-258` 還原。Timer「先治療再檢查引導」的時序以 on_cast＋on_channel_tick＋on_channel_end 表示，次數相同，最後一次比原作早最多 0.5 秒。
4. **奇襲（C19R_old）**：說明依 `C19.txt:436-572`、`C19.lua:151-209,321-340` 還原。
   - 原作 `Teleport2` 第一行引用未定義的 `keys` 報錯，實際不傳送；依設計意圖修（`fixed`，FC-168）。
   - `check_FOW` 只看 HeroList 中第一個敵方英雄的隊伍視野；資料照寫（取第一個敵方英雄）。
   - 原作只擋目標點 800 內名為 `com_towera`（織田軍箭塔，`npc/npc_units_custom_load.txt:13432`）的敵方建築。推定原意是「敵方塔附近不能傳」；產生器不支援 `types = tower`，資料寫成任何敵方建築，比原作嚴格。這是不確定的判斷：也可能原作只想擋特定箭塔。
5. **不容侵犯（C19T_old）**：說明依 `C19.txt:680-744` 與 `C19.lua:284-319` 還原。原作主動只把 `modifier_ninja_cloth` 設為兩層 10 秒，但 datadriven 屬性不乘層數，實際沒有效果。依設計意圖修（`fixed`，FC-169），推定兩層＝兩份閃避，寫成 10 秒內再加一份 `bonus_evasion`；這是不確定的判斷，多個閃避來源的合成方式由規則層決定。
