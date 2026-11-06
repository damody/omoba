# B16_old 望月千代女（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B16*_old` 技能說明（只有 modifier 名稱與短說明，2135-2154 行），七招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解（`npc/B_Unified/B16.txt:778,933,994,1061,1116,1147,1194`）。

1. **槽位與沒有 D**：`heromap_skill["B16"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:295`），choose_11 逐一 `AddAbility("B16?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）；`heromap_autoskill["B16"]["11"] = ""`（398 行），沒有自動學習。W/E/R/T → Q/W/E/R。
2. **忍法．召喚忍犬（B16W_old）**：說明依 `npc/B_Unified/B16.txt:779-932` 與 `vscripts/heroes/B_Unified/B16_Mochidsuki_Chiyome.lua:418-491,579-615,645-656` 還原。舊版的召喚在 W，有 90 秒存在時間（modifier_kill），沒有牽引、主人死亡不連動。生命 = baseHP + 等級 × 100（上限 2200）只在召喚時算一次。`%attack_damage`（855 行）推定為 OnAttackLanded 的攻擊傷害事件值，寫成 `event.attack.damage`。純粹傷害分支（612-614 行）檢查千代女身上的 buff2，永遠不成立，記為 no_effect。
3. **主從相依（modifier_B16W_old_buff2）**：狀態名稱是 v3 自訂（原作只有 modifier 名，沒有說明）；由 `modifier_B16W_old_master`（Passive，B16.txt:821-833）每 0.5 秒補套。
4. **忍法．落葉散華（B16E_old）**：說明沿用本體 B16E 的寫法，數字依 `B16.txt:934-993`。datadriven `Stun` 動作是引擎 modifier_stunned，照慣例 `dispel = "strong"`。
5. **忍法．迅雷（B16R_old）**：攻擊速度 Property 被註解（1040 行），只寫移動速度；modifier 說明「提升攻擊與移動速度」與實際不符。
6. **忍法．煙霧彈（B16T_old）**：說明依 `B16.txt:1062-1112` 與腳本 `517-522`。datadriven `Stun` 的 Target 是 CASTER，千代女自己暈眩 5 秒，明顯是誤寫；D22 比照 D13 已修（fixed）成暈眩周圍敵方英雄與部隊 5 秒。KV 沒有範圍，推定沿用落葉散華的 500。`SetTimeUntilRespawn(0)` 對活著的英雄無效，不列入。
7. **一擊斬（B16MMW_old）**：說明依腳本 `524-577`。保底：`noncrit_count > 4` 時必中，等於連續四次沒中後第五次必中（`pity_after = 4`）；計數在忍犬實體上，以 `counter = "ability"` 表達。原作用持續「攻速+0.1 秒」的暴擊 modifier，照設計意圖改為只影響本次攻擊的 `modify_attack`。英文名 Decisive Slash 為意譯。
8. **撕裂攻擊（B16MME_old）**：說明依腳本 `617-631` 與 `B16.txt:1148-1186`。暈眩只在忍犬當時有 buff2（千代女在 1000 內）時套上。
9. **永久隱形（B16MMT_old）**：說明沿用本體 B16MMF 的寫法，計時 2 秒（`B16.txt:1211`）。
10. **B16MMR_old**：KV 空殼、AddAbility 被註解（腳本 482 行），寫在文件層 `deferred`（reason = unused）。
11. **忍犬模板**：`npc/npc_units_custom_load.txt:4879-4953`；`owner_death = "persist"`（舊版沒有 B16D_Interval 的主人死亡連動）。
12. **升級事件**：`vscripts/library/events/eventforlevelup.lua:14-19` 對 naga_siren 英雄每次升級呼叫本體 `B16_AbilityAdjust`；舊版找不到 B16W／B16R／B16D，`caster.moonMoon` 有效時在 249-250 行對 nil 呼叫 `GetSpecialValueFor` 報錯中斷，沒有效果，不列入。
