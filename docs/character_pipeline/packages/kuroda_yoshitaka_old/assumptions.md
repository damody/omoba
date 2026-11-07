# A19_old 黑田孝高（舊版）說明還原紀錄

原作 resource/addon_tchinese.txt 沒有任何 `A19*_old` 的說明，四招說明都依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`。

- a19_old_fire_stratagem（A19W_old 鬼謀．火攻）：名稱取 KV 註解（npc/A_Oda/A19.txt:499）。效果依 vscripts/heroes/A_Oda/A19.lua:295-366（0～4 秒 dmg1 五次、5～9 秒 dmg2 五次、建築 ×0.2）。KV 有兩個 "02" 鍵（A19.txt:528-537），推定兩個都讀得到。
- a19_old_chain_fire（A19E_old 連環火）：名稱取 KV 註解（A19.txt:566）。效果依 A19.lua:369-392 與 modifier_A19E_old（A19.txt:647-668）。
- a19_old_clinging_flame（A19R_old 焰之附著）：名稱取 KV 註解（A19.txt:672）；英文名為自譯。效果依 A19.lua:395-423（層數 × dmg 純粹傷害、魔免第二層起減半）與 A19.lua:437-455（死亡爆炸 400、dmg2）。
- a19_old_siege_fire（A19T_old 籠城之火）：名稱取 KV 註解（A19.txt:744）。效果依 A19.lua:458-539：只有中間一道火牆有點（pos[2]、pos[3] 為空），39 點、間距 100，0.1 秒起每秒結算共 12 次。
