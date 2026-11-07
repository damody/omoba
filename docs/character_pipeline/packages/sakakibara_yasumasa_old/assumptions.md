# A08_old 榊原康政（舊版）假設

原作沒有任何語言的舊版說明（`npc/A_Oda/A08.txt:290-291` 註解：「遊戲裡面技能都沒說明」；`resource/addon_*.txt` 都找不到 A08*_old）。四招的名稱取 KV 註解，說明依 KV 與腳本還原，英文名稱是本檔自譯。

- **狂熱（A08W_old）**：名稱取 `npc/A_Oda/A08.txt:292`。效果依 `npc/A_Oda/A08.txt:318-369`（modifier_A06W：移速 %、攻速常數、基礎攻擊力 %，持續 12 秒）。
- **狂嗥（A08E_old）**：名稱取 `npc/A_Oda/A08.txt:371`。效果依 `npc/A_Oda/A08.txt:372-454` 與借用的 `vscripts/heroes/C_Neutral/C03.lua:35-66`（半徑 = 施法距離 500，攻速與移速降低 6 秒）。腳本的 ApplyDamage 因為沒有 AbilityDamage 而是零傷害，說明不寫傷害。
- **力量強化（A08R_old）**：名稱取 `npc/A_Oda/A08.txt:455`。效果依 `npc/A_Oda/A08.txt:456-489`。
- **剛毅大將（A08T_old）**：名稱取 `npc/A_Oda/A08.txt:489`。效果依 `npc/A_Oda/A08.txt:490-590` 與 `vscripts/AON_Cleave.lua:357-379`（以被攻擊者為圓心 400，其他敵人受到 60% 物理傷害，連攻擊建築也擴散）。
