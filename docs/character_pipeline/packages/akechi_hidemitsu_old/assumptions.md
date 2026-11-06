# C02_old 明智秀滿（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C02*_old` 說明，五招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解（`npc/C_Neutral/C02.txt:450,526,699,774`）。

1. **槽位**：`heromap_skill["C02"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:318`）。`heromap_autoskill["C02"]["11"] = "D"`（421 行）：C02D_old 自動學一級，寫成 `hero.auto_learn`，learnable = false。
2. **海波浩蕩（C02W_old）**：依 `C02.txt:450-524` 與 `vscripts/heroes/C_Neutral/C02.lua:482-549` 還原。寬度漸增以 projectile 的 `radius`／`end_radius` 表達。7 支特效投射物以 cue 表達。
3. **吸魂（C02E_old）**：依 `C02.txt:526-652` 與 `C02.lua:553-651` 還原。
   - 原作以 EndChannel(true) 打斷引導，資料用 `issue_order stop` 表達：modifier 被移除且仍在引導時，對秀滿下停止命令。
   - 距離檢查掛在目標的 `on_moved`，同原作 OnUnitMoved。只有目標移動時才檢查，秀滿自己移動不檢查（引導中秀滿本來就不能移動）。
   - 目標魔力不足時，秀滿照樣回復完整 steal_mp（SetMana）。照原作寫。
4. **鬼武者劍術（C02R_old）**：依 `C02.txt:699-772` 與 `C02.lua:653-663` 還原。暴擊照其他 OnAttackStart 暴擊的慣例，記在這一次攻擊（design_intent）。
5. **鬼武者之力（C02D_old）**：KV 沒有名稱與說明，名稱依效果推定。等級由 C02T_old 的 OnUpgrade 設成大招等級 + 1（`C02.lua:826-828`），以大招的 `on_upgrade` 加 `set_ability_level` 表達。
6. **炎龍劍（C02T_old）**：依 `C02.txt:774-914` 與 `C02.lua:667-828` 還原。結構和 B35T_old 類似：
   - DISABLE_HEALING 以 `heal_reduction_pct` 100% 表達，因為產生器不支援 `cannot_be_healed`。
   - 斬擊階段的位置變化只是呈現；最後位置以「目標周圍 150 的隨機點」加找空地表達。
   - KV 沒寫 RequiredLevel，照大招預設。
   - confidence 為 low，因為時序細節（0.15 秒重套、下一個 frame 才解除暈眩）只能近似。
