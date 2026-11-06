# B30_old 柿崎景家（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B30*_old` 說明，四招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解。

1. **槽位**：`heromap_skill["B30"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:309`），choose_11 逐一 `AddAbility("B30?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），沒有 D，也不自動學（`addon_game_mode.lua:412`）。
2. **突擊（B30W_old）**：說明依 `npc/B_Unified/B30.txt:377-471` 與 `vscripts/heroes/B_Unified/B30.lua:117-145`（衝刺）、`14-25`（普攻減冷卻）還原。knockback 的距離固定為 `B30W_distance`，與施放點遠近無關，以 `dash` 表達。被動減冷卻量 = 等級 × 0.5 秒（21 行）。
3. **熾烈劍（B30E_old）**：說明依 `B30.txt:472-549` 還原。7 秒內每秒一跳；最後一跳是否發生依 Dota 的 think 與移除時序，資料交給 `periodic`。
4. **煉獄之刃（B30R_old）**：說明依 `B30.txt:551-626` 與 `B30.lua:148-153` 還原。`B30R_agiBonus` 和 `B30R_duration` 都用鍵名 `"03"`（609-622 行）。推定 KV 保留重複鍵，`GetSpecialValueFor` 依變數名稱查找，兩個數值都會生效（同 B08R_old）。被動與主動是兩個不同的 modifier，屬性在主動期間疊成兩倍。
5. **邪王煉獄斬（B30T_old）**：說明依 `B30.txt:627-697` 還原。`B30T_old_OnSpellStart`（`B30.lua:155-185`）套的 modifier 不在 B30T_old 的 KV，推定 ApplyDataDrivenModifier 失敗而沒有效果（no_effect）。KV 沒寫 RequiredLevel，推定 AbilityType ULTIMATE 照 Dota 預設 6/12/18 級學習。
