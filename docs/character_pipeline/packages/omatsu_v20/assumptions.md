# A34 阿松（2.0）omatsu_v20 假設紀錄

原作 `*_20` 招式沒有任何語系的技能說明（`resource/addon_*.txt` 搜不到 `A34*_20`，UTF-16 轉碼後確認）。
名稱取自 KV 檔的區段註解，說明依 KV 與腳本還原。

1. **A34W_20 寒霜彈**：名稱取自 `npc/A_Oda/A34.txt:1144` 註解。說明依 KV `npc/A_Oda/A34.txt:1145-1243`（追蹤彈速度 2000、Dodgeable、
   Damage %A34W_Damage）與 `vscripts/heroes/A_Oda/A34_Matsu.lua:182-193`（非英雄結凍 duration + 1.5）。
2. **A34E_20 冰山之襲**：名稱取自 `A34.txt:1245` 註解。說明依 KV `A34.txt:1246-1383`（主目標 aoe_damage、範圍 AbilityDamage、
   modifier_frost_nova_slow 移速 -30%、攻速 -20，ApplyModifier Duration %AbilityDuration 5）。
3. **A34R_20 冰霜之息**：名稱取自 `A34.txt:1385` 註解。說明依 KV `A34.txt:1386-1533`（LinearProjectile、命中 AbilityDamage、
   modifier_A34R_datadriven 每秒 debuff_damage，持續 %damage_duration 6）。
4. **A34T_20 雪女的祝福**：名稱取自 `A34.txt:1535` 註解（與舊版同名）。說明依 KV `A34.txt:1536-1606`（回血、增傷、護甲三項與 0.5 秒無敵）。
   KV 沒有 RequiredLevel、MaxLevel、AbilityCastRange：推定大招 6/12/18 級、三級、任何距離可施放。
5. **A34D_20 加賀百萬石**：名稱取自 `A34.txt:1608` 註解。說明依 `A34_Matsu.lua:162-180`（OnAbilityExecuted，排除開關型與
   attribute_bonusx，對 A34W/E/R/T_20 中正在冷卻的每一招剩餘 −1 秒）。原作 modifier_A34D_20 沒有 tooltip。
   OnAbilityExecuted 時剛施放那招的冷卻是否已開始無法從腳本判定，推定已開始（也會減自己）。
