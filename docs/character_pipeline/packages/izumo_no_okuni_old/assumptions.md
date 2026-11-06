# C04_old 阿國（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C04*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["C04"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:320`），choose_11 逐一 `AddAbility("C04?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["11"] = ""`（423 行），沒有 D。2.0 版（`C04*_20`，含 C04D_20）不在這份。
2. **神速之舞（C04W_old）**：說明依 `npc/C_Neutral/C04.txt:418-484` 還原。沒有腳本，KV ApplyModifier 以阿國為中心、半徑 10000 套給友方 HERO|BASIC；說明照本體寫「全地圖」，10000 以隱藏數值保留在選取半徑。
3. **凋零舞步（C04E_old）**：說明依 `npc/C_Neutral/C04.txt:485-606` 與 `vscripts/heroes/C_Neutral/C04.lua:177-210` 還原。命中後的 `modifier_C04E_2` 只有特效、沒有屬性，所以舊版沒有緩速。冷卻 2 秒推定在開啟時起算。
4. **死亡之舞（C04R_old）**：說明依 `npc/C_Neutral/C04.txt:607-653` 與 `C04.lua:214-268` 還原，機制和真田昌幸 B09E_old 相同（三個落點、`it.b09e` 去重）。AbilitySpecial 兩個條目都用鍵名 `"01"`，推定 damage 與 duration 都生效。資料用三圈聯集一次選取表達去重。
5. **巫魅之舞（C04T_old）**：說明依 `npc/C_Neutral/C04.txt:654-705` 與 `vscripts/items/Addon_Items/item_devil_supressor_armor.lua:70-77,119-155`（Shock_C04_old → ShockTarget(…, 90)）還原。
   - 推定原作**沒有效果**：`ability:ApplyDataDrivenModifier` 套的 `modifier_devil_supressor_armor` 是 LinkLuaModifier 的 Lua modifier，C04T_old 的 KV 沒有 Modifiers 區塊，套用失敗後 `FindModifierByName(...).caster`（128 行）對 nil 取值報錯。和制魔腹卷 FC-062 是同一個函式、同一個失敗原因；FC-062 已由使用者決定照意圖修正（`docs/plans/2026-10-28-history-nobu-full-port/fix-candidates.md:149`），依 2026-11-07 主 session 決定（原作 bug 照設計意圖修）寫成護盾，quirk `policy = "fixed"`、`FC-155 D22 舊版`。
   - 護盾量 `armor` 1700/2500/3300，持續時間照腳本寫死的 90 秒（KV duration 90 同值）。原作以減免前傷害扣盾（`original_damage`）；hero 產生器不支援 `absorb.phase = "pre_mitigation"`，改用預設的減免後扣盾，並在 quirks 註明差異。只擋敵方傷害、盾量足夠時回寫 0.2 秒前生命的副作用沒有重現。
   - Lua modifier 沒寫 IsPurgable，暫定 `dispel = "none"`。
   - 大招 KV 沒寫 RequiredLevel，照 Dota 大招預設 6/12/18。
