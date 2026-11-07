# A01_old 德川家康（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `A01*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/A_Oda/A01.txt:537,597,662,740`）。

1. **槽位與沒有 D**：`heromap_skill["A01"]["11"] = "WERT"`、`heromap_autoskill["A01"]["11"] = ""`（`vscripts/addon_game_mode.lua:243,346`），`tofar_goback.lua:105-133` 移除全部技能後依序 AddAbility，所以舊版沒有天下之勢，也沒有 magical_resistance。
2. **東照權現（A01W_old）**：說明依 `npc/A_Oda/A01.txt:538-595` 還原：單體友方治療 `heal`；modifier_A01W_old 只有特效。KV 沒寫 AbilityCastPoint，推定前搖 0。
3. **天羅地網（A01E_old）**：說明依 `npc/A_Oda/A01.txt:598-660` 與 `vscripts/heroes/A_Oda/A01.lua:279-340` 還原：追蹤網（速度 1000、可閃避）命中後定身 `time` 秒，0.1 秒 Timer 在被驅散時重套（寫成 reapply_if_removed）。KV 的魔法傷害型別沒有任何傷害使用。
4. **雷霆刃（A01R_old）**：說明依 `npc/A_Oda/A01.txt:663-738` 與 `A01.lua:242-277` 還原：被動，普攻命中 60% 機率對自身周圍 `radius` 造成 `dmg` 魔法傷害。radius2 沒有被使用。
5. **三河之勇（A01T_old）**：說明依 `npc/A_Oda/A01.txt:741-929` 還原：被動 passive_A01T_old 只給自己每秒最大生命 `regen3`%；啟動給自己 20 秒 +20 護甲並每 0.5 秒給 `radius` 內友軍 1 秒的 `regen`% 回復與 `spd`% 移速。modifier_A01T_old2 與 regen2 沒有被使用。
