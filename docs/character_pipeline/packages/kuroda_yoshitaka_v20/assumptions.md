# A19_v20 黑田孝高（2.0）假設

原作 `resource/addon_tchinese.txt`、`addon_english.txt`、`addon_japanese.txt`、`addon_schinese.txt` 都沒有任何 `A19*_20` 的技能名稱或說明。招式名取自 KV 檔的註解，說明全部依 KV 與腳本還原。英文名稱沿用本體譯名，焰之附著為本檔自譯。

- **鬼謀．火攻（A19W_20）**：名稱取 `npc/A_Oda/A19.txt:935` 註解。效果依共用的 `vscripts/heroes/A_Oda/A19.lua:3-73` 與 2.0 KV `npc/A_Oda/A19.txt:946-1006`：爆炸兩次讀 `dmg`、餘火讀 `dmg2`，餘火 modifier 持續 5 秒（本體 1 秒）。腳本給建築套的 `modifier_A19W2` 在 2.0 KV 不存在，建築沒有餘火，說明寫「點燃非建築敵人」。
- **連環火（A19E_20）**：名稱取 `npc/A_Oda/A19.txt:1009` 註解。效果依 `npc/A_Oda/A19.txt:1060-1112` 與 `vscripts/heroes/A_Oda/A19.lua:75-96`；減甲與緩速是 modifier 的引擎屬性，不走 nobu 緩速池。
- **火焰返響（A19R_20）**：名稱取 `npc/A_Oda/A19.txt:1116` 註解。效果依 `vscripts/heroes/A_Oda/A19.lua:135-156`：以孝高為圓心、半徑讀施法距離 600、智力 ×1 的純粹傷害，沒有本體的道具／魔免減半。
- **籠城之火（A19T_20）**：名稱取 `npc/A_Oda/A19.txt:1177` 註解。效果依共用的 `vscripts/heroes/A_Oda/A19.lua:158-275`，數值依 `npc/A_Oda/A19.txt:1346-1372`（建築燃燒 16/24/32，本體 4/6/8）。
- **焰之附著（A19D_20）**：名稱取 `npc/A_Oda/A19.txt:1375` 註解。效果依 `vscripts/heroes/A_Oda/A19.lua:425-456`：普攻附著 8 秒，附著期間死亡在 400 內爆炸 `dmg2`。等級同步依 `vscripts/heroes/A_Oda/A19.lua:542-549`。爆炸寫在狀態的 `on_death`（狀態觸發，不是被動的 on_death），選取的 `team = "enemy"` 推定以施加者（孝高）為準，同舊版 A19R_old。
