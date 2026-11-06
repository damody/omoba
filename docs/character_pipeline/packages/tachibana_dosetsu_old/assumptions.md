# C07_old 立花道雪（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `C07*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`），招式名取自 KV 註解（`// 立花道雪 [禦魔之盾]` 等）。

1. **槽位**：`heromap_skill["C07"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:323`），choose_11 逐一 `AddAbility("C07?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["11"] = ""`（426 行），沒有 D，也沒有 C07T2。舊版的禦魔之盾是 W 鍵（可學四級），放 Q。
2. **禦魔之盾（C07W_old）**：說明依 `npc/C_Neutral/C07.txt:556-644` 還原。狀態是魔免＋定身（ROOTED），不是本體的移速固定。死亡後開關殘留照本體 FC-110 修正（on_death 關閉開關）。
3. **雷電桎梏（C07E_old）**：說明依 `npc/C_Neutral/C07.txt:645-787` 與本體腳本 `C07.lua:2-67,469-475` 還原。舊版沒有施放瞬間的傷害。`modifier_C07E` 有 IsPurgable 1 與 IsStunDebuff 1，推定只有強驅散能移除，寫 `dispel = "strong"`（本體文件寫 basic，兩者不一致，待確認）。`C07E_END` 不存在（報錯但移除暈眩已先執行）。
4. **雷霆閃電（C07R_old）**：說明依 `npc/C_Neutral/C07.txt:788-868` 與 `C07.lua:97-123`（C07R_old 函式）還原。KV `target_count` 8 被讀進變數但沒有使用，範圍內所有敵人都會被打。
5. **雷神之怒（C07T_old）**：說明依 `npc/C_Neutral/C07.txt:869-984`、`C07.lua:170-230`（C07T）與 `293-415`（C07_Effect）還原。
   - 舊版沒有 C07T2，`C07.lua:217` 對 nil 呼叫 SetLevel 報錯；此時雷雲、視野、`modifier_kill` 都已建立，只少了到期銷毀粒子的 timer（呈現），所以不算影響玩法的 bug，照原作寫（雷雲不會移動）。
   - 減療 50% 由 `events.lua:352-360` 依名稱 `modifier_C07T_2` 判斷，舊版 modifier 同名所以也有；`ultimate_modifiers`（`events.lua:262`）同理，靜電不受狀態抗性縮短。緩速是 KV Properties 直接生效，寫 `move_speed_pct`。
   - 多朵雲時的判斷照本體 FC-016 修正。雷雲模板 `c07t_old_storm_cloud`（Dummy_Ver1）還沒收進 `templates/v3_summons.lua`，`v3_rust` 回報 not a content summon 屬預期。
