# B25_old 本願寺顯如（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B25*_old` 技能說明（只有 `DOTA_Tooltip_modifier_B25E_old` 名稱「除魔經」，2160 行）。四招的名稱取 KV 註解（`npc/B_Unified/B25.txt:71,201,441`）與本體同名招式，說明依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["B25"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:304`），choose_11 逐一 `AddAbility("B25?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；自動學習 `heromap_autoskill["B25"]["11"] = ""`（`addon_game_mode.lua:407`），沒有 D。
2. **退魔擊（B25W_old）**：依 `npc/B_Unified/B25.txt:71-121` 與 `vscripts/heroes/B_Unified/B25_Honganji_Kennyo.lua:56-101` 還原。傷害是燒魔量的 0.3 倍（81 行）。
3. **除魔經（B25E_old）**：依 `npc/B_Unified/B25.txt:202-325` 與腳本 `163-261` 還原。AbilitySpecial 的 `distance` 與 `damage` 共用鍵 `"06"`（310-320 行），推定兩個都生效（同 B08R_old）。發射次數照設計意圖寫成「引導期間每 0.14 秒一道」，原作 Timer 先射再檢查，結束時多一道。
4. **死之手指（B25R_old）**：依 `npc/B_Unified/B25.txt:442-583` 與腳本 `103-107` 還原。KV 沒寫 AbilityCastRange，推定 Dota 預設 0 代表不受距離限制，cast_range 照 B08T_old、B18W_old 取 10000（design）；若實測是近戰距離，需要改。傷害在 0.05 秒 modifier 的 OnDestroy，驅散只會讓傷害提早。
5. **阿修羅地獄（B25T_old）**：名稱沿用本體 B25T（同一個腳本 `B25T_start`），KV 沒有舊版名稱。依 `npc/B_Unified/B25.txt:717-838` 與腳本 `284-432` 還原；寫法與本體 B25T 相同，只換數值並加上計數狀態的 LOW_ATTACK_PRIORITY（834 行）。
