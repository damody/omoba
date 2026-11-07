# A14 前田利家（舊版）推定紀錄

五招原作都沒有繁中說明；名稱取自 KV 註解，說明依 KV 與腳本還原。

- a14_old_instinct_strike（A14W_old）：名稱取自 npc/A_Oda/A14.txt:460「本能之打擊」。效果依 A14.lua:222-302；擊飛距離 1070 與 1.87 秒沿用本體依 Physics 預設摩擦 0.05（libraries/physics.lua:1731）的推算。兩次傷害依 :251 與 :282/:292 共用同一個 damageTable。
- a14_old_wild_slash（A14E_old）：名稱取自 A14.txt:514「野生之斬擊」。效果依 A14.lua:306-378 與 A14.txt:535-567（DelayedAction flyTime）。
- a14_old_war_spirit（A14R_old）：名稱取自 A14.txt:594「戰魂」；只有攻速與移速百分比屬性（A14.txt:603-630）。
- a14_old_fire_phoenix（A14D_old）：名稱取自 A14.txt:793「火鳳凰」。效果依 A14.txt:816-864；只在大招期間可用依 A14.lua:388-389,419。
- a14_old_rage_spear（A14T_old）：名稱取自 A14.txt:633「狂怒．槍之又左」。效果依 A14.txt:659-748 與 A14.lua:382-428；推定 A14T_old_OnSpellStart（不存在）報錯不影響之後的 RunScript。減益「不受狀態抗性」依 ApplyDataDrivenModifier(it, it, ...) 以單位自己為施加者推定。
