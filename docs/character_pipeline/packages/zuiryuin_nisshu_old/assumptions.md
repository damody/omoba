# A21_old 瑞龍院日秀（舊版）說明還原紀錄

原作沒有任何 `A21*_old` 的說明，五招說明都依 KV 與腳本還原。路徑相對 `game/dota_addons/nobu/scripts/`。

- a21_old_will_corruption（A21D_old 意志腐化）：名稱取 KV 註解（npc/A_Oda/A21.txt:381）；英文名自譯。效果依 modifier_A21D_old（A21.txt:416-427，MANA_REGEN_CONSTANT -60、30 秒）。開場自動學（vscripts/addon_game_mode.lua:366）。
- a21_old_blood_barrier（A21W_old 嗜血結界）：名稱取 KV 註解（A21.txt:443）；英文名自譯。效果依 vscripts/heroes/A_Oda/A21.lua:250-294 與 modifier_A21W／A21W_2／Passive_A21W_old3（A21.txt:497-531）。結界單位 A19W_old_SUMMEND_UNIT（npc/npc_units_custom_load.txt:925-1001）。
- a21_old_pearl_disc（A21E_old 珍珠圓盤）：名稱取 KV 註解（A21.txt:561）。效果依 A21.lua:296-371；舊版沒有回血光環、有雙方視野。
- a21_old_decay_aura（A21R_old 退化靈氣）：名稱取 KV 註解（A21.txt:657）；英文名自譯。效果依 A21.txt:676-695（900 光環、DAMAGEOUTGOING -20～-35%）。
- a21_old_false_intel（A21T_old 偽情報）：名稱取 KV 註解（A21.txt:708）。效果依 A21.lua:390-440；目標法力上限 mana（A21.lua:390-399）寫進說明。
