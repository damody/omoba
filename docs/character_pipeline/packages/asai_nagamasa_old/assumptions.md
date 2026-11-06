# B08_old 淺井長政（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B08*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位與沒有 D**：`heromap_skill["B08"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:287`），choose_11 逐一 `AddAbility("B08?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）。KV 沒有 `B08D_old` 技能（`npc/B_Unified/B08.txt:74` 的 `B08D_old` 是本體 B08D 的腳本函式名），所以 AddAbility 失敗，舊版只有四招。`eventforpichero.lua:153-160` 的 16 級自動學習只找本體 `B08D`，舊版不會發生。舊版 AI 單位 `npc_units_custom_load.txt:15795-15799` 也是這四招。
2. **一刀．牙（B08W_old）**：說明依 `npc/B_Unified/B08.txt:289-418` 還原。可以點單位也可以點地（UNIT_TARGET | POINT，295 行），都朝施放點發射。和本體不同，沒有施放後的指令限制。
3. **一刀．閃瞬（B08E_old）**：說明依 `npc/B_Unified/B08.txt:530-618` 與 `vscripts/heroes/B_Unified/B08_Asai_Nagamasa.lua:173-187`（B08E_Action2）還原。只有瞬移與傷害，沒有暈眩和燒魔。可以點魔免敵人（538 行 MAGIC_IMMUNE_ENEMIES），但腳本 183 行會跳過傷害。
4. **活心．魂（B08R_old）**：說明依 `npc/B_Unified/B08.txt:680-743` 與腳本 `250-278` 還原。AbilitySpecial 的兩個條目都用 `"01"` 當鍵（730-742 行）。推定 KV 保留重複鍵，`GetSpecialValueFor` 依變數名稱查找，所以 `b08r_attack_speed` 與 `b08r_run_speed` 都會生效。hCaster 未定義（FC-135）與 AphoticShieldRemaining 被清零（FC-005）都照本體的 D13 修正。
5. **一刀．空牙（B08T_old）**：說明依 `npc/B_Unified/B08.txt:877-1005` 還原。KV 沒寫 AbilityCastRange，推定 Dota 預設 0 代表點地不受距離限制，所以 cast_range 照 oichi 取 10000（design）。`dragon_slave_distance` 30000 照原作。沒有 AbilityType ULTIMATE，但因為 RequiredLevel 和 LevelsBetweenUpgrades 都是 6，學習等級仍是 6/12/18，所以照 T 鍵當大招放 R。
