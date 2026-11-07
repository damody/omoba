# A15 寧寧（舊版）推定紀錄

四招原作都沒有繁中說明；名稱取自 KV 註解，說明依 KV 與腳本還原。

- a15_old_vanishing_step（A15W_old）：名稱取自 npc/A_Oda/A15.txt:515「忍法．迷蹤步」。效果依 A15.txt:531-592 與 A15.lua:316-336。
- a15_old_moon_watch（A15E_old）：名稱取自 A15.txt:615「忍法‧觀月」。效果依 A15.lua:338-366 與 A15.txt:642-674；dummy 基礎視野 0（npc_units_custom_load.txt:5141-5142），所以視野半徑就是 A15E_vision。AbilityCastRange 0 推定為不限距離。
- a15_old_shadow_windmill（A15R_old）：名稱取自 A15.txt:697「忍法．影風車」。A15R_OnAttackLanded（A15.lua:114-120）以 FindAbilityByName("A15R") 取技能，舊版沒有 A15R，A15R_bounce_init（:131-136）對 nil 呼叫方法報錯，被動沒有效果。D22 照設計意圖修：做成與本體 A15R 相同的彈跳，數值用 A15R_old 的 KV（bonus_damage −0.30、jump_max 2/4/6/8、jump_radius 700）；本體與 A15T 連動的半徑加成、翻倍與吸血在舊版沒有對應技能，推定不生效；侍大將半傷照本體 FC-050。說明依本體寫法改寫。
- a15_old_nightmare（A15T_old）：名稱取自 A15.txt:772「忍法密傳‧夢魘」。效果依 A15.lua:368-413 與 A15.txt:788-836；AbilitySpecial 兩個 "03"（A15.txt:850-859）推定都讀得到。RequiredLevel 未寫，推定 6/12/18。
