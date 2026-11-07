# B07_old 足利義昭（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）3551-3561 行有四招的名稱與簡短說明：加勢、回復術、流星雨、君臨天下，單位名是足利帳篷與足利騎兵精英。名稱照原作。說明只寫部分效果，其餘依 KV 與腳本補齊（KV 與腳本路徑相對 `game/dota_addons/nobu/scripts/`）。英文名稱是自譯，原作 addon_english 沒有這幾招。

1. **槽位**：`heromap_skill["B07"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:286`），choose_11 逐一 `AddAbility("B07?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），自動學習為空（389 行）。沒有 D。`B07W_old_soldiercamp` 是兵營單位的 Ability1（`npc/npc_units_custom_load.txt:572`），寫成隱藏的單位模板技能。
2. **加勢（B07W_old）**：說明依 `npc/B_Unified/B07.txt:293-382`、`vscripts/heroes/B_Unified/B07.lua:193-266` 與單位 KV `npc/npc_units_custom_load.txt:558-676` 還原。
   - 兵營存在 41 秒（205 行 modifier_kill），每 interval 秒（35/30/25/20）召一名永久騎兵（231-247 行），騎兵掛 `modifier_B07W_old`。
   - 擊退以 TEAM_BOTH 選取（208-213 行），敵我與自己都會被推開並暈眩 0.3 秒。
   - `modifier_B07W_old_self` 每 3 秒設騎兵層數為義昭等級（250-266 行）。推定 datadriven 屬性不乘層數，只是顯示。
   - 騎兵的 `modifier_B07W_old` 沒寫 IsPurgable，照慣例暫定 basic。被驅散後就永久失去加成，沒有重掛機制。
   - 兵營是 `npc_dota_building`，碰撞取 FILLER 96、擋路（`blocks_pathing`），推定。
3. **回復術（B07E_old）**：說明依 `npc/B_Unified/B07.txt:432-510` 與腳本 268-284 行還原。AbilitySpecial 兩個條目都用 `"01"` 當鍵（497-509 行），推定兩個都生效（同 B08_old 的判斷）。`Passive_insight_gem` 沒有人套用，沒有效果。
4. **流星雨（B07R_old）**：說明依 `npc/B_Unified/B07.txt:513-563` 與腳本 287-314 行還原。延遲 0.5 秒的一次性傷害，flags 0，所以不打魔免。
5. **君臨天下（B07T_old）**：說明依 `npc/B_Unified/B07.txt:566-626` 與腳本 317-372 行還原。每 0.1 秒隨機打一個敵人，同一單位 0.35 秒內不重複。原作說明寫同一敵人最短 0.4 秒內不重複，腳本是 0.35 秒，照腳本。原作最多抽六次、全抽到剛打過的就空過一輪；資料改成直接從沒打過的單位中挑，差異見 quirks（design_intent）。
