# A20_old 九鬼嘉隆（舊版）說明還原紀錄

原作沒有任何 `A20*_old` 的說明，四招說明都依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`。

- a20_old_sea_devour（A20W_old 海噬）：名稱取 KV 註解（npc/A_Oda/A20.txt:504）；英文名自譯。效果依 vscripts/heroes/A_Oda/A20.lua:484-544（每 0.5 秒 10 次、建築 ×0）與光環 modifier_A20W_old（A20.txt:532-561，緩速 10%）。
- a20_old_pirate_domain（A20E_old 海賊領域）：名稱取 KV 註解（A20.txt:593）。效果依 A20.lua:549-610；modifier_A20E_old_self 的 %bonus_atk 不存在（A20.txt:661-664），說明不寫攻擊加成。
- a20_old_pirate_armor（A20R_old 海賊具足）：名稱取 KV 註解（A20.txt:727）；英文名自譯。效果依 A20.txt:735-771（被攻擊 70% 套 1 秒格擋、失敗移除一層）。
- a20_old_seasick_frenzy（A20T_old 船醉不知）：名稱取 KV 註解（A20.txt:786）；英文名自譯。效果依 modifier_A20T_old（A20.txt:817-845）與 A20.lua:610-630（受傷 0.01 秒後移除敵方施加的所有 modifier，flame_armor_debuff 例外）。
