# A27 柴田勝家（舊版）說明還原註記

原作 `resource/addon_tchinese.txt` 沒有 A27 舊版四招的技能說明，只有 modifier 名稱（`modifier_A27T_old_buff`「鬼柴田」、`modifier_A27R_old_passive`「武者之道」，:3121-3125）。招式名取自 KV 註解，說明依 KV 與腳本還原。

- **鬼柴田（A27W_old）**：名稱取自 `npc/A_Oda/A27.txt:467` 註解與 modifier tooltip。說明依 `npc/A_Oda/A27.txt:468-573`（範圍內友軍加攻擊力、護甲，持續 18 秒）。
- **動地鎚（A27E_old）**：名稱取自 `npc/A_Oda/A27.txt:574` 註解。說明依 `npc/A_Oda/A27.txt:575-643`（ActOnTargets 範圍傷害加暈眩）。
- **武者之道（A27R_old）**：名稱取自 `npc/A_Oda/A27.txt:644` 註解。說明依 `npc/A_Oda/A27.txt:645-713`（閃避、暴擊倍率）與 `vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:301-311`（20% 機率，連續四次未暴擊後第五次必中）。
- **破竹之勢（A27T_old）**：名稱取自 `npc/A_Oda/A27.txt:714` 註解。說明依 `npc/A_Oda/A27.txt:715-781`（分身數、時間、傷害比例 0）與 `vscripts/heroes/A_Oda/A27_Shibata_Katsuie.lua:325-442`（MirrorImage／CreateMirror）。英文名沿用 current 版同名招式的譯名 Unstoppable Advance。
