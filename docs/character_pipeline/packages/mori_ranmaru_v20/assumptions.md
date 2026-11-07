# A31_v20 森蘭丸（2.0）假設

原作 `resource/addon_tchinese.txt`、`addon_english.txt`、`addon_japanese.txt`、`addon_schinese.txt` 都沒有任何 `A31*_20` 的技能名稱或說明。招式名取自 KV 檔的註解，說明全部依 KV 與腳本還原。英文名稱沿用本體譯名，觀雲（Cloud Watch）為本檔自譯。

- **響雨（A31W_20）**：名稱取 `npc/A_Oda/A31.txt:843` 註解。效果依 `vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:93-141`：1 秒後起每秒一跳、共 11 跳的範圍魔法傷害（`abilityDamage` 18～65），森蘭丸自身 10 秒魔抗 +35%（`npc/A_Oda/A31.txt:878-887`）。11 跳（而不是特效的 10 秒）照原作 `second <= 10` 的判斷。
- **瞬步（A31E_20）**：名稱取 `npc/A_Oda/A31.txt:937` 註解。與本體共用 `vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:7-29`，數值依 `npc/A_Oda/A31.txt:947-992`。
- **嵐矢（A31R_20）**：名稱取 `npc/A_Oda/A31.txt:1007` 註解。全由 KV 動作完成（`npc/A_Oda/A31.txt:1036-1122`）。
- **五月雨（A31T_20）**：名稱取 `npc/A_Oda/A31.txt:1127` 註解。被動攻速依 `npc/A_Oda/A31.txt:1177-1186`，分裂箭依共用的舊版 `A31T_old_split_shot`（`vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:389-431`）與 `SplitShotDamage`（`:225-240`）。`modifier_a31t_20` 沒有被套用，不寫。
- **觀雲（A31D_20）**：名稱取 `npc/A_Oda/A31.txt:1226` 註解。效果依 `npc/A_Oda/A31.txt:1239-1276` 與 `vscripts/heroes/A_Oda/A31_Mori_Ranmaru.lua:454-462`。視野的 0.5 秒重複改寫成狀態的 periodic（第一跳另以 0.1 秒 delay 補上）。
