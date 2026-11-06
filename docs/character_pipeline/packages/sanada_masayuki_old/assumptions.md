# B09_old 真田昌幸（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）只有旋風單位名「狂風」（825 行），五招都沒有技能說明，說明依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱依效果與特效推定（落雷、地裂、衰弱、旋風、暴風），單位名照原作。

1. **槽位**：`heromap_skill["B09"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:288`），choose_11 逐一 `AddAbility("B09?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），自動學習為空（391 行）。沒有 D。`B09TW_old` 由旋風單位 `AddAbility`（`vscripts/heroes/B_Unified/B09.lua:367`），寫成隱藏的單位模板技能 `b09_old_gale`。
2. **落雷（B09W_old）**：說明依 `npc/B_Unified/B09.txt:351-411` 還原。傷害由 KV ActOnTargets 立即結算，腳本只放特效。
3. **地裂（B09E_old）**：說明依 `npc/B_Unified/B09.txt:413-473` 與腳本 292-347 行還原。
   - 三個爆點相隔 300、半徑 300。原作用 `it.b09e` 讓同一次施放每個單位只受一次，第二輪迴圈清回 nil。
   - 資料用 routine 依序結算三個爆點，以「身上還沒有 `b09e_old_stun`」篩選來去重。差異：被前一次施放的暈眩還留著的單位會被跳過。暈眩最長 2 秒、冷卻最短 10 秒，只有在大幅縮短冷卻時才會遇到。
   - AbilitySpecial 兩個條目都用 `"01"` 當鍵（460-472 行），推定兩個都生效。
4. **衰弱（B09R_old）**：說明依 `npc/B_Unified/B09.txt:475-537` 還原。`attack_bonus` 同時是攻速常數與傷害百分比，拆成 `attack_slow` 與 `weaken` 兩個數值（同一個 KV 來源）。
5. **旋風（B09T_old）**：說明依 `npc/B_Unified/B09.txt:541-581`、腳本 351-384 行與單位 KV `npc/npc_units_custom_load.txt:1737-1795` 還原。
6. **暴風（B09TW_old）**：說明依 `npc/B_Unified/B09.txt:583-724` 與腳本 388-438 行還原。
   - 無敵（modifier_B09TW_old 的 States）改用單位模板常駐狀態 `b09tw_old_invulnerable` 表達，因為產生器不支援 `passive.states`。
   - 建築傷害每秒 100，對象是敵方建築。原作用技能的目標旗標（未寫，0）。
   - 光環 `modifier_A28TE_old_building_debuff` 只播粒子，不列入。
   - 光環子狀態的殘留時間照慣例取 0.5 秒（推定）。
