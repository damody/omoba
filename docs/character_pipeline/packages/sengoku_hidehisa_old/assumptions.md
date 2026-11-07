# A22_old 仙石秀久（舊版）說明還原紀錄

原作沒有任何 `A22*_old` 的說明，五招說明都依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`。

- a22_old_taunt（A22D_old 嘲弄）：名稱取 KV 註解（npc/A_Oda/A22.txt:420）；英文名自譯。效果依 vscripts/heroes/A_Oda/A22.lua:93-134 與 modifier_A22D_old／old2（A22.txt:457-483）。開場自動學（vscripts/addon_game_mode.lua:367）。
- a22_old_tiger_swing（A22W_old 虎振）：名稱取 KV 註解（A22.txt:510）；英文名自譯。效果依 A22.lua:137-184（三道 ±30 度劍氣，KV Damage）。
- a22_old_counterattack（A22E_old 反擊）：名稱取 KV 註解（A22.txt:567）。效果依 A22.txt:611-653 與 A22.lua:187-240（被動魔法反擊、主動 15 秒純粹反擊）。
- a22_old_swordsmanship（A22R_old 刀術）：名稱取 KV 註解（A22.txt:686）；英文名自譯。效果依 A22.lua:243-264（20%，連續五次未暴擊後必暴）。
- a22_old_cyclone_slash（A22T_old 氣旋斬）：名稱取 KV 註解（A22.txt:741）；英文名自譯。效果依 A22.lua:267-456：投射物只是特效，傷害與推開由隱形 dummy 每 0.5 秒五個圓結算，每單位最多 8 次（12 秒歸零）。
