# C01_old 明智光秀（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C01*_old` 說明，五招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解（`npc/C_Neutral/C01.txt:412,526,617,691,729`）。

1. **槽位**：`heromap_skill["C01"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:317`）。`heromap_autoskill["C01"]["11"] = "D"`（420 行）：C01D_old 自動學一級，寫成 `hero.auto_learn`，learnable = false。
2. **血刃亂舞（C01W_old）**：依 `C01.txt:412-525` 與 `vscripts/heroes/C_Neutral/C01_Mitsuhide_Akechi.lua:326-345` 還原。datadriven OnAttack 推定為攻擊出手（`on_attack_launch`）。範圍照腳本寫 400，KV 的 340 沒有被讀取。
3. **火雨術（C01E_old）**：依 `C01.txt:526-616` 與腳本 347-433 行還原。每波 0.86 秒（由 `wave_interval` 推導）。第一波在施放當下（Timer 0）。
4. **劍術（C01R_old）**：依 `C01.txt:617-690` 與腳本 435-477 行還原。照本體 C01R 的寫法，暴擊記在這次攻擊，保底 `pity_after = 5`、`counter = "hero"`。
5. **灌頂斬擊（C01D_old）**：依 `C01.txt:691-728` 還原。實際只有 1 點魔法傷害加 1.7 秒暈眩。KV AbilityDamage 100（PURE）沒有被使用，資料照實際寫，列在報告的不確定判斷裡（可能是原作想造成 100 純粹傷害）。
6. **本能寺燄上（C01T_old）**：沿用本體 C01T 的寫法。火球傷害鍵缺失導致原作傷害為 0，依 D22「原作 bug 照設計意圖修」改用本體 C01T_Damage 84/94/104（quirk fixed）。舊版沒有 OnAbilityPhaseStart，所以沒有前搖範圍提示。
