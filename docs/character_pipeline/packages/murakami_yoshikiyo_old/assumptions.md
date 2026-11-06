# B29_old 村上義清（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B29*_old` 說明，五招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["B29"]["11"] = "WERDT"`（`vscripts/addon_game_mode.lua:308`），choose_11 逐一 `AddAbility("B29?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["B29"]["11"] = "D"`（`addon_game_mode.lua:411`）出生自動學 D。W/E/R/T → Q/W/E/R，D 照慣例放 D 並寫進 `hero.auto_learn`。
2. **武王丸（B29D_old）**：名稱取 KV 註解（`npc/B_Unified/B29.txt:485`）。說明依 487-559 行還原；註解寫「30%攻擊速度」，實際是 ATTACKSPEED_BONUS_CONSTANT 30，照 KV。沒有 NOT_LEARNABLE，但 MaxLevel 1 且自動學到 1 級，寫 `learnable = false`。
3. **動地斧（B29W_old）**：說明依 `B29.txt:561-617` 與 `vscripts/heroes/B_Unified/B29.lua:179-204` 還原。半徑照腳本 550（KV AOERadius 400 只是指示圈）。
4. **震地斧（B29E_old）**：說明依 `B29.txt:618-698` 與 `B29.lua:207-238` 還原。搜尋半徑是 `ability:GetCastRange()`，取 KV AbilityCastRange 400；減益直接寫在 modifier Properties，以 `attack_speed`、`move_speed_pct` 負值表達，不經緩速池。
5. **士氣提升（B29R_old）**：說明依 `B29.txt:699-764` 與 `B29.lua:360-381` 還原。crit_chance 100 使每次攻擊都暴擊，保底計數沒有作用，不寫 chance。暴擊照 A18R 慣例直接記在本次攻擊（design_intent）。光環沒寫殘留時間，推定 Dota 預設 0.5 秒；對友軍不排除魔免（推定 Aura_Flags NONE 不影響友軍魔免）。
6. **大地震撼（B29T_old）**：說明依 `B29.txt:765-873` 與 `B29.lua:282-358` 還原。KV `interval` 只用在第一次 Timer（343 行），之後固定 0.1（353 行）；每次兩擊（348-349 行）；散布半徑 (700 + 100) × 0.5 = 400（335、339 行）；打擊半徑 240（283 行）。暈眩 0.2 是腳本寫死（308 行），與 KV `B29T_old_Duration` 同值。施放時的 100 秒力量 buff（KV 807 行 ApplyModifier）照原作保留，引導中斷不移除。
