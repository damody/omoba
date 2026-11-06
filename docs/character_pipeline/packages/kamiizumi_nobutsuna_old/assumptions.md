# B35_old 上泉信綱（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B35*_old` 說明，四招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。

1. **槽位**：`heromap_skill["B35"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:314`），choose_11 逐一 `AddAbility("B35?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），沒有 D，也不自動學（`addon_game_mode.lua:417`）。
2. **劍之極意（B35W_old）**：說明依 `npc/B_Unified/B35.txt:387-474` 還原。KV 註解只寫「上泉信綱 劍聖」，屬性組合和本體 B35W 相同（多了回魔），名稱沿用本體的「劍之極意」。
3. **新陰流．一閃（B35E_old）**：說明依 `B35.txt:476-558` 與 `vscripts/heroes/B_Unified/B35.lua:215-276` 還原，名稱取 KV 註解（476 行）。突進用 `dash`（原作 modifier_knockback，速度 6000、距離 1150）。劍氣在 0.1 秒後從施放位置射出（264 行）。半徑照腳本寫 175，KV 的 150 沒有被讀取。命中時的 PerformAttack 寫成 `attack`（必中、使用法球、觸發特效、不受射程限制）。`modifier_B35E_old`（命中縮短冷卻）沒有任何地方套用，是死碼，不列入。
4. **新陰流（B35R_old）**：說明依 `B35.txt:560-632` 與 `B35.lua:291-305` 還原，名稱取 KV 註解（560 行）。暴擊照其他 OnAttackStart 暴擊的慣例寫成這一次攻擊的 `modify_attack`（design_intent）。保底「計數 > 9」寫成 `pity_after = 9`，計數存在 hero 範圍的 `unit_state`。
5. **絕‧陰流最終祕劍轉（B35T_old）**：說明依 `B35.txt:634-740` 與 `B35.lua:307-477` 還原，名稱取 KV 註解（634 行）。
   - `MODIFIER_PROPERTY_DISABLE_HEALING` 以 `heal_reduction_pct` 100% 表達，因為產生器不支援 `cannot_be_healed`。原作會連自然回復一起擋住；heal_reduction 是否也擋回復，依規則層而定，可能有差異。
   - 斬擊階段的位置變化與分身只是呈現，以 cue 表達；只保留會影響最終位置的移動：一開始移到目標 +x 方向 300 處（分身迴圈的最後一點，460-467 行），play_time − 1.1 秒移到目標前方 400，1 秒內跳到目標後方 400，再 1.1 秒後找空地。
   - 暈眩照 Dota 慣例 `dispel = "strong"`；`modifier_B35T_old_playing` 沒寫 IsPurgable，暫定 basic。
   - 結尾 1 點傷害打在無敵目標上，推定沒有效果。
   - KV 沒寫 RequiredLevel，照大招預設 6/12/18。
   - confidence 為 low，因為整段是 Timer 拼出來的演出，時序細節（0.15 秒重套、下一個 frame 才解除暈眩）只能近似。
