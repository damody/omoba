# A28_v20 羽柴秀吉（2.0）假設

原作 `resource/addon_tchinese.txt`、`addon_english.txt`、`addon_japanese.txt`、`addon_schinese.txt` 都沒有任何 `A28*_20` 的技能名稱或說明。招式名取自 KV 檔的註解，說明全部依 KV 與腳本還原。英文名稱沿用本體／舊版譯名，鳳凰守護為本檔自擬。

- **邪法（A28W_20）**：名稱取 `npc/A_Oda/A28.txt:1195` 註解。效果依 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:36-58`：單位目標，以目標位置為圓心 500 變青蛙；數值依 `npc/A_Oda/A28.txt:1216-1257`。
- **創念（A28E_20）**：名稱取 `npc/A_Oda/A28.txt:1282` 註解。與本體 A28E 共用腳本與相同數值（`npc/A_Oda/A28.txt:1283-1423`），寫法照本體。
- **縛足之沼（A28R_20）**：名稱取 `npc/A_Oda/A28.txt:1424` 註解。與本體共用 `A28R` 腳本，冷卻固定 15 秒（`npc/A_Oda/A28.txt:1437`）。緩速是 modifier 的引擎移速百分比，不走 nobu 緩速池。
- **鳳凰召來（A28T_20）**：名稱取 `npc/A_Oda/A28.txt:1519` 註解。效果依 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:894-925`：鳳凰生命 1000 + 秀吉等級 × 100，攻擊 150～200 + 等級 × 10；鳳凰帶 A28_20TW 與舊版的 A28TE_old。存在時間不可驅散沿用本體 FC-084。
- **鳳凰守護（A28D_20）**：KV 註解只寫「羽柴秀吉」，名稱「鳳凰守護」／Phoenix Guard 是本檔依效果自擬。效果依 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:928-959` 與 `npc/A_Oda/A28.txt:1581-1631`。傷害量用減免後傷害（KV 註解寫 `%attack_damage` 是 mitigation 後的值）。
- **天火（A28_20TW）**：名稱取 `npc/A_Oda/A28.txt:1632` 註解。效果依 `vscripts/heroes/A_Oda/A28_Hashiba_Hideyoshi.lua:961-1012`，最大生命 2%；施法時間 13 秒（KV `A28T_Skill_Duration` 沒有被讀取）。
- **熾烈焚身（A28TE_old，鳳凰被動）**：不在 vlist 的 2.0 清單，但 A28T_20 把它加到鳳凰身上（`A28_Hashiba_Hideyoshi.lua:916`），所以本檔另以 `a28_v20_blazing_body` 收錄；名稱與效果照舊版文件 `hashiba_hideyoshi_old` 的還原（KV `npc/A_Oda/A28.txt:1071-1195`、`vscripts/AON_Cleave.lua:17-40`）。
