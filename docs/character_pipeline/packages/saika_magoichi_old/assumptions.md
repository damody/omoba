# B01_old 雜賀孫市（舊版）推定事項

原作 `resource/addon_tchinese.txt` 沒有 B01*_old 的說明，招式名沿用 KV 註解（與本體同名），說明依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`（history_nobu f966720）。

1. **狙擊模式 B01W_old**：說明依 `npc/B_Unified/B01.txt:97-211`（OnToggleOn／OnToggleOff、modifier_B01W 的 ATTACK_RANGE_BONUS、BONUS_NIGHT_VISION、MOVESPEED_BONUS_PERCENTAGE −70）。`bonus_atk` 沒有被任何屬性引用，不寫進說明。`modifier_B01W_locker`（2/3/4/5 秒停用技能，B01.lua:416-422）短於開啟時的 10 秒冷卻，推定沒有額外效果而不列入；切換技冷卻在開啟時起算是推定。
2. **雜賀眾 B01E_old**：說明依 `npc/B_Unified/B01.txt:382-455`（SpawnUnit B01W_UNIT、UnitCount %count、Duration 10000、無 UnitLimit）。OnSpawn 的 `B01E_old_CHECK`（B01.lua:151-163）在 `table.insert(caster.B01E, …)` 因 `caster.B01E` 為 nil 報錯，原作攻擊力維持 KV 67～73；D22 比照 D13 依腳本原意修正為 70 + 15 × 孫市等級（生成時取樣，fixed）；`B01E`（B01.lua:41-60）的充能 modifier 綁不到技能，推定沒有效果。生成位置取施法者身邊（SpawnUnit 預設）。單位操控與賞金比照本體改編。
3. **雨鐵炮 B01R_old**：說明依 `npc/B_Unified/B01.txt:266-307` 與 `vscripts/heroes/B_Unified/B01.lua:363-408`：RandomInt(0,100) ≤ 45 觸發（46/101），連續兩次失敗後第三次必中，傷害為攻擊傷害 × 類別百分比的物理傷害。
4. **三段 B01T_old**：說明依 `npc/B_Unified/B01.txt:542-618`（modifier_B26T BASEDAMAGEOUTGOING_PERCENTAGE 200，持續 7/11/13 秒）。
