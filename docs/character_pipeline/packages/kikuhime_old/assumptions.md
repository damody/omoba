# B11_old 菊姬（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B11*_old` 說明，六招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/B_Unified/B11.txt:534,587,622,699,753,834`）。

1. **槽位**：`heromap_skill["B11"]["11"] = "WERDFT"`（`vscripts/addon_game_mode.lua:290`），choose_11 逐一 `AddAbility("B11?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；W/E/R/T → Q/W/E/R，D、F 維持。`heromap_autoskill["B11"]["11"] = "DF"`（`addon_game_mode.lua:393`），換版時 D、F 直接設為一級（`tofar_goback.lua:129-132`），兩者寫在 `hero.auto_learn`、`learnable = false`。
2. **體術．真空斬（B11D_old）**：依 `npc/B_Unified/B11.txt:535-585` 還原。datadriven Damage 加 Stun（引擎 modifier_stunned，`dispel = strong`）。MaxLevel 1、沒有 NOT_LEARNABLE，但已由自動學習滿級。
3. **忍法．櫻雨（B11W_old）**：依 `B11.txt:588-619` 與 `vscripts/heroes/B_Unified/B11.lua:287-320` 還原。只有一段魔法傷害（本體另有敏捷倍率純粹傷害，舊版沒有）。
4. **忍法．土蜘蛛鎖（B11E_old）**：依 `B11.txt:623-697` 與 `B11.lua:323-350` 還原。KV 選到魔免敵人但腳本 341 行跳過，結果是排除魔免。一點魔法傷害在 modifier 的 OnCreated（`B11.txt:673-680`）；依本體慣例推定重複套用只刷新、不再跑 OnCreated，所以寫在狀態 `on_created`，說明註明已被定身的目標不再受傷害。
5. **忍法．調息（B11R_old）**：依 `B11.txt:700-751` 與 `B11.lua:479-487` 還原。被動 HEALTH_REGEN_PERCENTAGE 與移速百分比；KV 上的冷卻、耗魔、前搖在被動上無作用，不列入。OnUpgrade 把分身術設為 R 等級加一。
6. **忍法．分身術（B11F_old）**：依 `B11.txt:754-830` 與 `B11.lua:354-444` 還原。
   - 移除敵方施加的效果用 `remove_status dispel = all, team_of_source = enemy`（同 hattori_hanzo_old 的 A13R_old）；原作依名稱 RemoveModifierByName，同名友方實例也可能被移除，未細分。
   - 分身法力：原作是新英雄滿法力，`copy.mana = caster.max_mana`（同本體截殺術的推定）。
   - `modifier_B11F_old`（基礎攻擊間隔 1.25）從未套到本體，給分身的分支是死碼，不列入（quirk no_effect）。
   - 上一個分身用 `unit_state b11_old_clone`（scope hero）記錄，再次施放時 `kill`。
7. **極限體術．鴉襲（B11T_old）**：依 `B11.txt:835-893` 與 `B11.lua:447-476` 還原。
   - 保底：計數先加一再判斷 `> 100/crit_chance`，17/23/29% 分別是連續 5/4/3 次未觸發後必中（`pity_after`）。
   - 暴擊 modifier 沒有時限、下次出手才移除；照設計意圖寫成 `modify_attack { crit_multiplier, cannot_miss }` 只作用於這一次攻擊（quirk design_intent）。
   - 純粹傷害在 OnAttackStart 立即結算（攻擊被取消也已造成），照原作寫在 `on_attack_windup`。
   - RequiredLevel 未寫，沿用大招引擎預設 6/12/18（同本體）。
