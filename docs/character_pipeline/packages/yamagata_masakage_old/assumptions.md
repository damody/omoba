# B20 山縣昌景（舊版）推定紀錄

原作 `resource/addon_tchinese.txt` 沒有任何 `B20*_old` 的說明，四招說明都依 KV 與腳本還原。中文招式名取 KV 註解（`npc/B_Unified/B20.txt:279`「岩衝」、`:326`「壞毀波」、`:377`「強韌」、`:457`「赤備猛攻」），英文名沿用本體。

- **b20_old_rock_rush（B20W_old 岩衝）**：說明依 `npc/B_Unified/B20.txt:279-324` 與 `vscripts/heroes/B_Unified/B20.lua:4-68`。腳本與本體共用 `B20W_OnSpellStart`，只有數值不同，說明句型照本體。
- **b20_old_ruin_wave（B20E_old 壞毀波）**：說明依 `npc/B_Unified/B20.txt:326-375` 與 `B20.lua:71-105`。腳本與本體共用 `B20E_OnSpellStart`。
- **b20_old_tenacity（B20R_old 強韌）**：舊版獨有的暴擊被動，說明依 `npc/B_Unified/B20.txt:378-455`（`modifier_B20R_old_Crit` 的 OnAttackStart Random Chance 36、`modifier_B20R_old_Crit2` 的 PREATTACK_CRITICALSTRIKE 與 OnAttackLanded）與 `B20.lua:172-194`（暴擊命中非建築時 `B20R_old_damageBonus` 純粹傷害）。名稱與本體相同，但效果完全不同。
  - Random Chance 推定為獨立機率（沒有 PseudoRandom）。
  - 原作在 OnAttackStart 掛暴擊 modifier，資料改用 `modify_attack` 記在這次攻擊，另以隱藏狀態 `b20r_old_crit` 標記，命中時加純粹傷害並移除。
- **b20_old_red_charge（B20T_old 赤備猛攻）**：說明依 `npc/B_Unified/B20.txt:457-551` 與 `B20.lua:134-170`。舊版 `modifier_B20T` 多了 `MOVESPEED_BONUS_PERCENTAGE %B20T_msBonus`；減益 `modifier_B28T_debuff` 以 Properties 同時降移速百分比與攻速常數（`B20T_msasDebuff`），不走 `dummy.lua` 緩速池，所以寫成 `move_speed_pct` 負值而不是 `slow_pct`。
  - KV 沒寫 RequiredLevel，照引擎大招預設 6/12/18。
