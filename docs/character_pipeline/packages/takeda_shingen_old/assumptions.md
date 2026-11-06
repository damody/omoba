# B31_old 武田信玄（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B31*_old` 說明，四招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解（風林火山）。

1. **槽位**：`heromap_skill["B31"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:310`），choose_11 逐一 `AddAbility("B31?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），沒有 D，也不自動學（`addon_game_mode.lua:413`）。
2. **疾如風（B31W_old）**：說明依 `npc/B_Unified/B31.txt:496-585` 還原。兩個被動光環（英雄含自身、BASIC 不含自身）。第一個光環用 `passive.aura`，第二個以學會時掛上的永久隱藏狀態帶光環（同 A33R_old）。BASIC 光環以 `where = "not it.is_hero"` 排除英雄，因為產生器的光環不接受 creep+neutral+summon 這個組合。推定 Dota 的 BASIC 不含英雄，結果相同。攻擊速度與移動速度讀同一個 KV 變數，所以拆成兩個數值。兩者 source 相同，unit 不同。
3. **徐如林（B31E_old）**：說明依 `B31.txt:586-707` 與 `vscripts/heroes/B_Unified/B31.lua:260-367` 還原。KV speed 800 沒有被使用，每支箭的速度取 RandomInt(300, 1200)（342 行）。發射線在身後 radius × 2（309 行）。引導結束時是否多射一支取決於 Timer 順序，記為 design_intent。
4. **侵略如火（B31R_old）**：說明依 `B31.txt:710-774` 還原。DAMAGEOUTGOING_PERCENTAGE 照本體 B29R 對應 `attack_damage_pct`。被動上的 OnSpellStart 是死碼。
5. **不動如山（B31T_old）**：說明依 `B31.txt:775-916` 與 `B31.lua:372-431` 還原。名稱照風林火山推定（KV 沒有註解）。驅散以 `remove_status dispel = "all", team_of_source = "enemy"` 表達（同 A13 本體）。每 0.9 秒只看 FindUnitsInRadius 的第一個結果（`break`，427 行）。FIND_ANY_ORDER 的順序推定為任意，以 `limit = 1` 表達。bonus_attack 在 Properties 裡被註解掉，不列入。
