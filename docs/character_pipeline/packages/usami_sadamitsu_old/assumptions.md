# B19_old 宇佐美定滿（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有 `B19*_old` 招式說明，只有單位名 `B19T_old`「宇佐美騎兵精英」（2713 行）。五招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`），招式名取 KV 註解（`npc/B_Unified/B19.txt:304,349,438,500,540`）。

1. **槽位**：`heromap_skill["B19"]["11"] = "WERDT"`、`heromap_autoskill["B19"]["11"] = ""`（`vscripts/addon_game_mode.lua:298,401`）；`vscripts/items/Addon_Items/tofar_goback.lua:96-140` 依序 AddAbility 五招。`B19D_old` 在 KV（`npc/B_Unified/B19.txt:305-348`），不能學，由 `vscripts/library/events/eventforpichero.lua:143-152` 在 18 級時設為一級，寫成 `hero.auto_learn` 加上 `requires.hero_level`。
2. **水龍攻（B19W_old）**：說明依 `npc/B_Unified/B19.txt:349-437` 與 `vscripts/heroes/B_Unified/B19.lua:338-452` 還原。和本體不同的是，命中標記在 0.8 秒後清除（397-401 行），所以水柱可以彈回已擊中的單位。v3 用持續 jump_delay + 0.8 秒的隱藏標記表達。標記到期和搜尋落在同一刻時，先後順序可能差一跳。
3. **冰茅圍籬（B19E_old）**：說明依 `npc/B_Unified/B19.txt:438-498` 與 `B19.lua:170-220` 還原。結算延後 0.6 秒（204 行），半徑是 A25E_range + 50。
4. **越後流兵法（B19R_old）**：和本體共用 `B19R_OnSpellStart`（`B19.lua:222-247`），只有 KV 數值不同（`npc/B_Unified/B19.txt:500-539`）。KV 沒寫 MaxLevel，照 Dota 預設四級。
5. **駿河守（B19D_old）**：說明依 `npc/B_Unified/B19.txt:305-348`（datadriven Heal %heal）與 `B19.lua:479-491` 還原：先治療目標友軍，再對它周圍半徑 500 的敵人造成 AbilityDamage 魔法傷害。這個半徑在腳本裡寫死，等於 KV 的 AOERadius。KV 沒寫 AbilityCastRange 與 AbilityCastPoint，射程取 Dota 預設 0（要走到與目標相接才施放），前搖取 0。這是推定，實際接觸距離依 host 對 range 0 的處理而定。
6. **車懸（B19T_old）**：說明依 `npc/B_Unified/B19.txt:540-620` 與共用的 `B19T_OnSpellStart`（`B19.lua:249-311`）、`B19T_old_OnAttackLanded`（465-476 行）還原。
   - 沒有引導，施放瞬間暈眩並開始召喚。
   - 實際召喚 cavalry + 1 隻。
   - 騎兵存在 8 秒（從施放起算）。
   - KV `atk` 沒有腳本讀取，`add_damage` 舊版 KV 沒有這個鍵，所以騎兵沒有加攻擊力。
   - 普攻命中非建築時，以 `%attack_damage` 再造成一次物理傷害，另加 20 魔法傷害。`%attack_damage` 推定為減免前的攻擊傷害（`event.attack.damage`，同 `hashiba_hideyoshi_old`）。長宗我部（`chosokabe_motochika`）與 A06 則推定為護甲後；如果護甲後才對，要改成 `event.attack.damage * armor_multiplier(event.target.armor)`。
7. **騎兵模板**：原作單位 `B19T_old`（`npc/npc_units_custom_load.txt:429-490`）和本體共用。舊版文件自帶一份模板 `b19_old_cavalry`，數值照本體 `b19t_cavalry`。
