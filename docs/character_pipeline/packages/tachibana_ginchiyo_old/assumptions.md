# C09_old 立花誾千代（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C09*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C09"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:325`），沒有 D；`heromap_autoskill["C09"]["11"] = ""`（428 行）。
2. **冰霜斬擊（C09W_old）**：依 `npc/C_Neutral/C09.txt:444-567`、`vscripts/heroes/C_Neutral/C09.lua:196-262` 還原。舊版真的佔用 nobu 法球槽（`caster.nobuorb1`），以 `orb_slot = { claim, overridable, release = status_end }` 表達。對建築多打一次的 PerformAttack 照本體 FC-090 修正。移速減益不看魔免、攻速減益跳過魔免，照腳本。
3. **白熱（C09E_old）**：依 `C09.txt:571-628`、`C09.lua:264-276` 還原。對友方單位施放，只給攻速；`C09E_heal` 為 0、回血已註解。
4. **冰靈舞魂（C09R_old）**：依 `C09.txt:629-707`、`C09.lua:278-336` 還原。沒有施法距離（任何距離施放、截斷到 C09R_distance），瞬移後下攻擊移動指令，沒有 ProjectileDodge。魔免期間被移除時補回的機制照本體寫成移除當下立即補回。
5. **破甲術（C09T_old）**：依 `C09.txt:709-770`、`C09.lua:338-352` 還原。只有主動；KV 沒寫 AbilityCastPoint，推定 0（且 IMMEDIATE）。不在 `ultimate_modifiers`，受狀態抗性縮短。
