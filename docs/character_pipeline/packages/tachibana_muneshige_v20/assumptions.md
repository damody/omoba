# C11 立花宗茂（2.0）tachibana_muneshige_v20 假設紀錄

原作 `*_20` 招式沒有任何語系的技能說明（`resource/addon_*.txt` 轉碼後查無 `C11*_20`）。名稱取自 KV 註解（`npc/C_Neutral/C11.txt:879,1070,1158`），
說明依 KV 與腳本還原。英文名「Lightning Flash」「Swift Thunder Fang」為意譯。

1. **只有三招**：`heromap_skill C11 ["20"] = "WERDT"`、`heromap_autoskill ["20"] = "D"`（`vscripts/addon_game_mode.lua:327,430`），
   但全部 KV 只定義 C11W_20、C11E_20、C11R_20。choose_20（`vscripts/items/Addon_Items/tofar_goback.lua:22-33`）對 C11D_20、C11T_20 的
   AddAbility 失敗，接著 `FindAbilityByName("C11D_20"):SetLevel(1)` 對 nil 報錯中止。腳本殘留的 `C11D_20_OnAttackLanded`
   （`C11.lua:352-361`，讀 special `out`）與 `C11T_20_OnUpgrade`（`C11.lua:560-566`）沒有 KV 數值，無法還原設計意圖，所以不補；
   2.0 版沒有大招（R 槽空著）。
2. **C11W_20 雷光電閃**：KV `C11.txt:879-1068`、腳本 `C11.lua:414-558`。彈跳結構照本體 v3 的 bolt_routines，去掉消去法力與附加普攻。
   爆擊視窗用的 modifier_C11W_old 宣告在 C11W_20 自己的 Modifiers 裡，數值讀 C11W_20 的 crit_chance／crit_bonus。
   投射物未命中時走本體的 C11W_miss_target → C11W_hit_unit，該函式最後讀 caster.isHit（nil）報錯，但下一段投射物已發射，跳躍照常。
3. **C11E_20 疾雷伏牙**：KV `C11.txt:1070-1156`，與本體雷之疾同構、移速不同。
4. **C11R_20 雷之眼**：KV `C11.txt:1158-1237`、腳本 `C11.lua:179-248`（與本體共用）。
