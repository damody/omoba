# A02_old 稻姬（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `A02*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/A_Oda/A02.txt:450,590,689,759`，標「11.2B」）。

1. **槽位**：`heromap_skill["A02"]["11"] = "WERT"`、autoskill 空（`vscripts/addon_game_mode.lua:244,347`），`tofar_goback.lua:105-133` 依序 AddAbility。
2. **蠍毒箭（A02W_old）**：說明依 `npc/A_Oda/A02.txt:451-588` 與 `vscripts/heroes/A_Oda/A02.lua:3-20,88-131` 還原。A02W_old_OnAttackStart 的 `caster.nobuorb1 ~= "A02W"` 永遠成立（寫入那行已註解，全專案沒有別處寫 "A02W"），所以原作每次攻擊前搖開始就自動關閉；只有前搖中才開啟的那一擊會上毒。D22 比照 D13 照設計意圖修：拿掉這個檢查（本體同一段已註解），開關持續開著，每擊對非建築、非魔免目標扣 5 法力、法力不足才關閉；建築與魔免不扣魔也不多打一下（FC-078 比照本體）。說明照修正後的行為寫。禁療（DISABLE_HEALING）因產生器尚未支援 cannot_be_healed 旗標，以減療 100% 加回復增幅 −100% 表達。
3. **風之力（A02E_old）**：說明依 `npc/A_Oda/A02.txt:591-687` 還原：友方單體 15 秒攻速與移速加成，一級移速加成為 0。
4. **自我恢復（A02R_old）**：說明依 `npc/A_Oda/A02.txt:690-757` 與 `A02.lua:133-155` 還原：雖名為自我恢復，KV 是友方單體目標，給 10 秒每秒回血。
5. **融影術（A02T_old）**：說明依 `npc/A_Oda/A02.txt:760-860` 還原：隱形（modifier 狀態，攻擊不破隱）、無碰撞、基礎攻擊間隔 0.9。KV 沒寫 RequiredLevel，大招等級推定 6/12/18（同本體）。
