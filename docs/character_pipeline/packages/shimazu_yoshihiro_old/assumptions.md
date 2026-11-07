# C03_old 島津義弘（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C03*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C03"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:319`），choose_11 逐一 `AddAbility("C03?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["11"] = ""`（422 行），沒有 D。
2. **鐵布衫（C03W_old）**：說明依 `npc/C_Neutral/C03.txt:329-411` 與 `vscripts/heroes/C_Neutral/C03.lua:19-32` 還原，效果和本體相同（移除敵方施加的狀態、魔免與三項屬性），只有數值與持續時間不同。
3. **震地斧（C03E_old）**：說明依 `npc/C_Neutral/C03.txt:413-495` 與本體腳本 `C03.lua:35-66` 還原。舊版緩速是 KV Properties 直接生效（480-491 行），不走 nobu 緩速池，所以寫成 `move_speed_pct` 負值而不是 `slow_pct`。
4. **暴雷怒擊（C03R_old）**：說明依 `npc/C_Neutral/C03.txt:496-547` 與 `C03.lua:68-112` 還原，和本體完全相同；KV 寫 PURE 但腳本用魔法傷害，以腳本為準。
5. **不動戰錘（C03T_old）**：說明依 `npc/C_Neutral/C03.txt:548-637` 與 `C03.lua:115-200`（KV 582 行呼叫的是本體 `C03T_OnSpellStart`）還原。7 秒那一跳和戰錘銷毀在同一個 think，原作執行順序不確定；推定照 `floor(7/0.5)+1 = 15` 跳（和本體同一套算法），差一跳的可能已記在 quirks。
