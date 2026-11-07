# A05_old 酒井忠次（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `A05*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/A_Oda/A05.txt:422,510,581,624`）。

1. **槽位與沒有 D**：`heromap_skill["A05"]["11"] = "WERT"`、autoskill 空（`vscripts/addon_game_mode.lua:247,350`），`tofar_goback.lua:105-133` 依序 AddAbility。忠之極意在舊版是大招 A05T_old（被動、三級）。
2. **三河之勇（A05W_old）**：說明依 `npc/A_Oda/A05.txt:423-508` 與 `vscripts/items/Addon_Items/item_muramasa_katana.lua:109-137` 還原：15 秒攻擊力加成與吸血。共用的 A05W_RunScript 以 GetCastRange()／GetAbilityDamage() 打周圍，舊版 KV 兩者都沒寫（0），沒有效果，說明不提。
3. **武者之道（A05E_old）**：說明依 `npc/A_Oda/A05.txt:511-579` 與 `vscripts/heroes/A_Oda/A05.lua:202-212` 還原：閃避加成，20% 暴擊並在連續四次沒暴擊後必中。原作暴擊 modifier 掛到下一次命中才移除，照設計意圖寫成只影響當次攻擊。
4. **怪力擊（A05R_old）**：說明依 `npc/A_Oda/A05.txt:582-622` 與 `A05.lua:168-200` 還原：常駐必中；命中非建築 25% 機率（連續四次沒觸發後必中）暈眩 0.35 秒並造成純粹傷害（魔免只暈眩）。常駐必中以 on_learn 套上的狀態表達（產生器尚未支援 passive.states）。
5. **忠之極意（A05T_old）**：說明依 `npc/A_Oda/A05.txt:625-686` 與 `A05.lua:215-240` 還原：與本體 A05D 同一段邏輯，套用本體的 FC-024 修正（致死傷害前取消並觸發）；保護狀態只有 0.5 秒無敵與隱藏血條。是否沿用 FC-024 由工作線 D 確認。
