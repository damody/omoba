# A28 羽柴秀吉（舊版）說明還原註記

原作 `resource/addon_tchinese.txt` 沒有 A28 舊版六招的技能說明，只有 modifier 名稱與一句描述（:2422-2435：火炎彈「暈眩」、火神‧迦具土「提升基礎數值」、靈魂燃燒「無法施法，降低攻擊與移動速度，持續造成傷害」、熾烈焚身「持續對周圍敵人造成傷害」）。招式名取自 KV 註解，說明依 KV 與腳本還原。

- **邪法（A28W_old）**：名稱取自 `npc/A_Oda/A28.txt:606` 註解。說明依 `npc/A_Oda/A28.txt:607-692`（單體、modifier_A28W 移速 -100% 與 OnCreated 1 點傷害）與 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:25-34`（modifier_voodoo_lua 青蛙、AMHC 1 點傷害）。
- **治療之風（A28E_old）**：名稱取自 `npc/A_Oda/A28.txt:693` 註解。說明依 `npc/A_Oda/A28.txt:694-811` 與 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:60-186`：只能對友軍，每個目標治療兩次 AbilityDamage（KV Heal 與腳本 Heal 各一次），傳導總數 jump_count。「治療兩次」是依程式推得，推定不是原意；D22 照意圖修成每個目標治療一次（quirk 改 fixed）。
- **火炎彈（A28R_old）**：名稱取自 `npc/A_Oda/A28.txt:812` 註解。說明依 `npc/A_Oda/A28.txt:813-897` 與 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:500-572`：施法方向上 castRange/3 的三處同時爆炸。
- **火神‧迦具土（A28T_old）**：名稱取自 `npc/A_Oda/A28.txt:898` 註解。說明依 `npc/A_Oda/A28.txt:899-964`、`vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:574-609`、`npc/npc_units_custom_load.txt:3877-3930`（火神單位）與 `npc/npc_abilities_custom_load.txt:5073-5088`（for_magic_immune）。
- **靈魂燃燒（A28TW_old，火神技能）**：名稱取自 `npc/A_Oda/A28.txt:965` 註解。說明依 `npc/A_Oda/A28.txt:966-1069`。
- **熾烈焚身（A28TE_old，火神被動）**：名稱取自 `npc/A_Oda/A28.txt:1070` 註解。說明依 `npc/A_Oda/A28.txt:1071-1195`、`vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:611-671`（每秒範圍傷害）與 `vscripts/AON_Cleave.lua:17-40`（%attack_damage × 0.5 的 600 範圍物理濺射）。
