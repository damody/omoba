# A26 濃姬（舊版）說明還原註記

原作 `resource/addon_tchinese.txt` 沒有 A26W_old／A26E_old／A26R_old／A26T_old 的技能名稱與說明。招式名取自 KV 註解，說明依 KV 與腳本還原。

- **四連彈（A26W_old）**：名稱取自 `npc/A_Oda/A26.txt:482` 註解。說明依 `npc/A_Oda/A26.txt:483-532`（傷害、射程、MAGIC_IMMUNE_ENEMIES 旗標）與 `vscripts/heroes/A_Oda/A26.lua:432-471`（以施法距離為半徑，對前四個敵人 ApplyDamage 並 PerformAttack）。
- **三味火（A26E_old）**：名稱取自 `npc/A_Oda/A26.txt:533` 註解。說明依 `npc/A_Oda/A26.txt:534-626`（傷害、減益的攻速與移速、持續時間）與 `vscripts/heroes/A_Oda/A26.lua:473-527`（nobu_modifier_rooted、Physics 擊退、減益、傷害）。擊退距離沿用 current 版 A26E 由 Physics 摩擦推得的約 453。
- **二丁銃（A26R_old）**：名稱取自 `npc/A_Oda/A26.txt:627` 註解。說明依 `vscripts/heroes/A_Oda/A26.lua:529-551`：暴擊 modifier 被註解掉，實際效果是機率追加一次 PerformAttack，連續四次未觸發後第五次必定觸發。說明寫成「追加一次普攻」，不寫暴擊倍率。
- **一擊必殺（A26T_old）**：名稱取自 `npc/A_Oda/A26.txt:687` 註解。說明依 `npc/A_Oda/A26.txt:688-780`（傷害、半徑、前搖鎖定）與 `vscripts/heroes/A_Oda/A26.lua:553-635`（投射物落地範圍傷害、非英雄單位再受十次傷害、摧毀樹木）。英文名 One-Shot Kill 是本專案譯名。
