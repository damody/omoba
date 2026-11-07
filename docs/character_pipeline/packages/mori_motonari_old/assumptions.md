# B05_old 毛利元就（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B05*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/B_Unified/B05.txt:415,522,642,909`）。

1. **槽位與沒有 D**：`heromap_skill["B05"]["11"] = "WERT"`、autoskill 空（`vscripts/addon_game_mode.lua:284,387`），`vscripts/items/Addon_Items/tofar_goback.lua:105-133` 依序 AddAbility，W/E/R/T → Q/W/E/R。
2. **鏈鎖閃電（B05W_old）**：說明依 `npc/B_Unified/B05.txt:416-519` 與 `vscripts/heroes/B_Unified/B05_Moari.lua:10-130` 還原：與本體共用 LightningJump／NewInstance，數值換成舊版（250～550 傷害、9～15 個目標、0.1 秒間隔、800 範圍）。腳本的 ×1.25（`B05_Moari.lua:92-93`）要看 `modifier_thundergods_wrath_datadriven`，舊版沒有招式會套它，不寫入。
3. **天雷擊（B05E_old）**：說明依 `npc/B_Unified/B05.txt:523-640` 與 `B05_Moari.lua:136-159` 還原：周圍 1500 內非魔免敵方英雄與部隊受 AbilityDamage 魔法傷害，另有 100 範圍 2 秒視野（hidden）。舊版沒有攻速緩速。
4. **三矢之訓（B05R_old）**：說明依 `npc/B_Unified/B05.txt:643-755` 還原：舊版是主動，施放後 20 秒全地圖（99999）友方英雄與部隊（含自己）護甲 +5/10/15/20。光環子 modifier 的 `Duration 1`（`B05.txt:704`）推定為殘留時間（`aura_linger`），信心中等。sight／spread_aoe／AbilityDamage 都沒有腳本讀取。
5. **氣霸西國（B05T_old）**：說明依 `npc/B_Unified/B05.txt:910-1003` 與 `B05_Moari.lua:362-498` 還原：ActOnTargets 對 800/900/1000 內敵方英雄、部隊、建築（含魔免）先 Stun 1.5/2/2.5 秒再造成 700/900/1200 魔法傷害，摧毀樹木。同時呼叫的本體 B05T 腳本讀不到舊版沒有的 damage／blind 等 special，也找不到 `modifier_B05T_stunned1/2`、`modifier_B05T_blind`，只剩 1500 範圍 3 秒視野與粒子。暈眩是引擎 modifier_stunned（不在 `vscripts/events.lua:246-268` ultimate_modifiers），受狀態抗性縮短。
