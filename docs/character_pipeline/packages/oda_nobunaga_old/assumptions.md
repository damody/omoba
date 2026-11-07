# A25_old 織田信長（舊版）說明還原紀錄

原作沒有任何 `A25*_old` 的說明，四招說明都依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`。

- a25_old_distant_thunder（A25W_old 遠雷遙）：名稱取 KV 註解（npc/A_Oda/A25.txt:483）；英文名自譯。效果依 A25.txt:526-558（只有 KV 魔法傷害，沒有本體的物理部分）。
- a25_old_calamity_thorns（A25E_old 厄災之棘）：名稱取 KV 註解（A25.txt:588）。效果依 vscripts/heroes/A_Oda/A25_Oda_Nobunaga.lua:82-130（兩次各半傷害與暈眩，相隔 1 秒）與 modifier_A25E_attack（A25.txt:637-647）。
- a25_old_heshikiri（A25R_old 壓切）：名稱取 KV 註解（A25.txt:680）。效果依 A25.txt:704-748 與 A25_Oda_Nobunaga.lua:346-363（20% 暴擊、無保底，不對建築）。
- a25_old_crimson_barrier（A25T_old 障壁深紅）：名稱取 KV 註解（A25.txt:770）。效果依 A25_Oda_Nobunaga.lua:257-341（與本體共用 A25T 腳本），數值取 A25T_old 的 KV。
- 沒有 D：heromap_skill ["11"] 有 D，但 KV 沒有 A25D_old 技能（見 v3.lua 檔頭）。
