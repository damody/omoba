# C15_old 玉子（舊版）假設

原作 `panorama/localization/addon_tchinese.txt`（UTF-16）沒有任何 `C15*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。KV 區段註記為「玉子 11.2B」（`npc/C_Neutral/C15.txt:469`）。

1. **槽位**：`heromap_skill["C15"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:331`），choose_11 逐一 `AddAbility("C15?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["C15"]["11"] = ""`（`addon_game_mode.lua:434`）。沒有 D。
2. **三連矢（C15W_old）**：說明依 `npc/C_Neutral/C15.txt:471-553` 與 `vscripts/heroes/C_Neutral/C15.lua:156-183` 還原。三支追蹤彈各自命中、各自暈眩與傷害。
3. **烈焰箭（C15E_old）**：說明依 `npc/C_Neutral/C15.txt:554-637` 與 `C15.lua:185-215` 還原。Orb 的 `CastAttack 0` 推定只換投射物外觀，`OnAttackStart`／`OnAttackLanded` 對每次普攻觸發。`modifier_C15E_old` 不是 Passive，死亡時被移除而開關推定仍亮；資料比照今川義元烈焰箭（FC-104）在死亡時關閉開關，quirk 標 `fixed`（FC-186 D22 舊版）。
4. **精準之光（C15R_old）**：說明依 `npc/C_Neutral/C15.txt:638-663` 還原。`DAMAGEOUTGOING_PERCENTAGE` 照本體寫成 `attack_damage_pct`。
5. **百箭齊發（C15T_old）**：說明依 `npc/C_Neutral/C15.txt:664-798`、`C15.lua:50-152`、`npc/npc_units_custom_load.txt:5187-5259` 還原。舊版沒有 `modifier_C15T_trigger`，所以沒有被動放箭。箭矢單位另立模板 `c15_old_arrow`（原作同一個 `C15T_dummy`），等 D 重產生 `v3_summons.lua` 後才會是 content summon。
