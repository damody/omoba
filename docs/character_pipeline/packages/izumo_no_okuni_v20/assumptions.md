# C04 阿國（2.0）izumo_no_okuni_v20 假設紀錄

原作 `*_20` 招式沒有任何語系的技能說明（`resource/addon_*.txt` 轉碼後查無 `C04*_20`）。名稱取自 KV 註解，說明依 KV 與腳本還原。

1. **C04W_20 神速之舞**：`npc/C_Neutral/C04.txt:707-770`。沒有腳本，ApplyModifier 半徑 10000 友方 HERO|BASIC。
2. **C04E_20 凋零舞步**：`C04.txt:771-885`、`vscripts/heroes/C_Neutral/C04.lua:37-97`（與本體共用 C04E_OnIntervalThink）。
   腳本讀 special `damage`，C04E_20 沒有這個鍵（取到 0）；KV 有 `AbilityDamage 70` 沒人讀。依主 session「原作 bug 照設計意圖修」，
   推定 70 是設計的固定傷害，寫成 fixed（FC-156）。若判定 AbilityDamage 只是殘留，應改回 0（刪掉 damage 與說明的 `{damage}`）。
3. **C04R_20 聚曳之舞**：`C04.txt:886-930`、`C04.lua:99-135`。數值與本體相同；Modifiers 區塊空白，modifier_flood_walk 套不上，修成本體行為（fixed）。
4. **C04T_20 巫魅之舞**：`C04.txt:931-1069`、`C04.lua:137-175`。冷卻 105、暈眩 5 秒、可指定部隊。C04T2_OnAbilityExecuted 不存在（no_effect）。
5. **C04D_20 護身之舞**：KV 註解 `C04.txt:1071` 寫「巫魅之舞」（與 T 重名，推定複製時沒改）。效果是 `item_devil_supressor_armor.lua:61-68`
   的魔法護盾（制魔腹卷同款），名稱「護身之舞／Guardian Dance」是依效果自擬。ShockTarget 用 ApplyDataDrivenModifier 套 Lua modifier
   失敗後報錯，照舊版 C04T_old 與本體 FC-062 修成 6 秒、45 點魔法護盾。KV 沒寫 NOT_LEARNABLE，但只有一級數值，寫成 max_level 1、不可學。
