# A26_v20 濃姬（2.0）假設

原作 `resource/addon_tchinese.txt`、`addon_english.txt`、`addon_japanese.txt`、`addon_schinese.txt` 都沒有任何 `A26*_20` 的技能名稱或說明。招式名取自 KV 檔的註解，說明全部依 KV 與腳本還原。英文名稱沿用本體／舊版譯名，槍林彈雨、隱雷為本檔自譯。

- **連銃（A26W_20）**：名稱取 `npc/A_Oda/A26.txt:780` 註解。效果依共用的 `vscripts/heroes/A_Oda/A26.lua:3-36`，數值依 `npc/A_Oda/A26.txt:794-799`。
- **槍林彈雨（A26E_20）**：名稱取 `npc/A_Oda/A26.txt:832` 註解。效果依 `vscripts/heroes/A_Oda/A26.lua:748-780`：五次 PerformAttack、0.5 秒暈眩、Physics 擊退、自身 0.6 秒輸出 −50%（`npc/A_Oda/A26.txt:870-880`）。擊退距離約 320 是依本體 A26E 同一套 Physics 摩擦推導（初速 500）。KV `duration` 0.8～1.7 沒有被讀取。
- **二丁銃（A26R_20）**：名稱取 `npc/A_Oda/A26.txt:907` 註解。效果依共用的舊版 `vscripts/heroes/A_Oda/A26.lua:529-551`：機率追加一次必中普攻、連續四次失敗後必中；說明不寫暴擊倍率（同舊版）。
- **灼熱爆裂投擲（A26T_20）**：名稱取 `npc/A_Oda/A26.txt:1099` 註解。效果依 `vscripts/heroes/A_Oda/A26.lua:317-372,637-698`：物理傷害 = AbilityDamage + 等級 × 10，摧毀樹木；暈眩與加倍傷害的條件被註解關掉，不寫。
- **隱雷（A26D_20）**：名稱取 `npc/A_Oda/A26.txt:966` 註解。效果依 `vscripts/heroes/A_Oda/A26.lua:133-177,700-746` 與 `npc/A_Oda/A26.txt:1050-1095`。地雷單位 `A26_MINE_hero` 依 `npc/npc_units_custom_load.txt:5961-6028`。地雷觸發爆炸用光環給的狀態 `on_created` 呼叫 routine（同百地三太夫舊版 B13R_old 的寫法）。
