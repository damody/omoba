# A29 石田三成（舊版）說明還原註記

原作 `resource/addon_tchinese.txt` 沒有 A29 舊版四招的技能說明，只有 modifier 名稱（:3153-3157：老樹盤根「無法移動與攻擊，持續造成傷害」、機智消磨「持續減少法力」）。招式名取自 KV 註解，說明依 KV 與腳本還原。

- **袖毒矢（A29W_old）**：名稱取自 `npc/A_Oda/A29.txt:696` 註解。說明依 `npc/A_Oda/A29.txt:697-893`（單體追蹤投射物、命中傷害、8 秒中毒減速與每秒傷害）。
- **老樹盤根（A29E_old）**：名稱取自 `npc/A_Oda/A29.txt:894` 註解。說明依 `npc/A_Oda/A29.txt:895-999`（定身＋繳械、每秒傷害）。英文名 Ancient Roots 是本專案譯名。
- **機智消磨（A29R_old）**：名稱取自 `npc/A_Oda/A29.txt:1000` 註解。說明依 `npc/A_Oda/A29.txt:1001-1095`（光環降低法力回復）與 `vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:46-66`（補魔免單位、法力低於一半時套固定減速）。「法力低於一半」由 `5 - floor(mana/max_mana*10) > 0` 推得；減速 modifier 的狀態名「心力交瘁」是本專案命名（原作 modifier_A29R_old 沒有 tooltip）。
- **兀鷹群（A29T_old）**：名稱取自 `npc/A_Oda/A29.txt:1096` 註解。說明依 `npc/A_Oda/A29.txt:1097-1324` 與 `vscripts/heroes/A_Oda/A29_Ishida_Mitsunari.lua:68-487`（與 current 版共用的 Exorcism 程式），寫法照 current 版 A29T 的說明，數值換成舊版 KV；舊版 heal 0，不寫施放時回血。
