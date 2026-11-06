# A17 佐佐成政（舊版）推定紀錄

四招原作都沒有繁中說明；名稱取自 KV 註解（npc/A_Oda/A17.txt「11.2B」段），說明依 KV 與腳本還原。

- a17_old_sulfur_shot（A17W_old）：名稱取自 A17.txt:804「硫磺砲」。效果依 A17.txt:822-880（追蹤投射物命中後套 modifier_A17W_old_debuff：減甲、PROVIDES_VISION、每秒魔法傷害）。
- a17_old_rapid_cannon（A17E_old）：名稱取自 A17.txt:908「連擊砲」。效果依 A17.txt:919-970 與 A17.lua:282-309（RandomInt(1,100) <= 50，恰好 50%）。
- a17_old_rain_guns（A17R_old）：名稱取自 A17.txt:993「雨鐵砲」。效果依 A17.lua:311-331；三個倍率都在 "02" 鍵（A17.txt:1040-1054），推定都讀得到；add_chance 判斷已註解，必定觸發。
- a17_old_great_cannon（A17T_old）：名稱取自 A17.txt:1058「大筒」。效果依 A17.txt:1081-1159 與 A17.lua:206-260。
