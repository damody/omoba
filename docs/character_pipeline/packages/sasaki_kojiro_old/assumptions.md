# C22_old 佐佐木小次郎（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C22*_old` 說明，招式名取自 KV 註解（`npc/C_Neutral/C22.txt:543、625、751、860`），說明依 KV 與腳本還原。

1. **槽位**：`heromap_skill["C22"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:338`），舊版沒有 D；自動學習為空（`addon_game_mode.lua:441`）。
2. **秘劍．燕（C22W_old）**：`C22.txt:543-624`，全部 datadriven：傷害、引擎暈眩、自身 3 秒攻速。
3. **秘劍．返（C22E_old）**：`C22.txt:625-750`、`vscripts/heroes/C_Neutral/C22_Sasaki_Kojiro.lua:280-297`。命中 0.1 秒後拉回。原作用 FindClearSpaceForUnit 放到面前 100；產生器不支援對其他單位 `find_clear_space = true`（`action:teleport.find_clear_space of victim`），資料同本體 C22D 寫 `false`，可能卡進地形或單位（差異）。modifier_C22E_old（攻速）沒有被套用，屬死碼。
4. **巖流（C22R_old）**：`C22.txt:751-859`、腳本 299-318。保底換算為連續 3/2/2/2 次；暴擊 modifier 原作留到下一次命中才移除，照意圖寫成這一擊的 modify_attack；回血在出手時。
5. **隱秘劍．地燕斬（C22T_old）**：`C22.txt:860-965`、腳本 320-372。原作半徑用 GetLevel() 當索引（讀到下一級），照 D22 修為 KV 的 600/625/650（`policy = "fixed"`，FC 待編號）。三級原作讀超出範圍的索引，推定取最後一個值 650。
