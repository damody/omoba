# B17_old 霧隱才藏（舊版）假設

原作沒有任何 `B17*_old` 的繁中說明（resource 查無），四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。名稱取自 KV 註解。

1. **槽位與沒有 D**：`heromap_skill["B17"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:296`），choose_11 拔掉全部技能後逐一 `AddAbility("B17?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:100-139`）；自動學習 `heromap_autoskill["B17"]["11"] = ""`（`addon_game_mode.lua:399`）。所以舊版只有四招，W/E/R/T → Q/W/E/R。
2. **忍法．迷蹤步（B17W_old）**：名稱依 `npc/B_Unified/B17.txt:521`。說明依 `B17.txt:522-619` 與 `vscripts/heroes/B_Unified/B17.lua:232-252` 還原：隱形＋移速＋無碰撞＋低攻擊優先度；攻擊出手時對非建築造成額外物理傷害並現形；OnAbilityExecuted 無條件現形。英文名 Phantom Step 為自訂。
3. **忍法．劇毒手裏劍（B17E_old）**：名稱依 `B17.txt:621`。說明依 `B17.txt:622-713` 還原。減速是 modifier 的 `MOVESPEED_BONUS_PERCENTAGE` 直接屬性（不走 `dummy.lua` 緩速池），寫成 `move_speed_pct` 負值。
4. **忍法．迴旋手裏劍（B17R_old）**：名稱依 `B17.txt:715`。說明依 `B17.txt:716-783`、`B17.lua:254-314` 與單位 `npc/npc_units_custom_load.txt:1576-1652` 還原。
   - 終點是「施法者位置 + 2000 × 面向」（`B17.lua:263`），腳本讀的 `GetCursorPosition` 沒用；KV 前搖註解說多 0.08 秒讓英雄轉向（`B17.txt:727`），推定設計意圖是朝施法點方向，寫成 `direction(caster, point)`。
   - 生成後 0.06 秒才下移動指令（`B17.lua:266`），寫成立即下令。
   - 回收判斷原作用 `CalcDistanceBetweenEntityOBB`（包圍盒邊緣距離）< 100，以中心距離近似，手裏劍會比原作略晚消失。
   - 存在上限：第四秒起每 0.5 秒 `count += 0.5`，`count > 40` 才 ForceKill（`B17.lua:274-283`），即約第 44 秒；寫成 `lifetime = return_time + follow_limit`。
   - 魔免敵人傷害 ×0.5（`B17.lua:305-307`）照寫；傷害為魔法，對魔免實際由規則層歸零。
5. **忍法密傳．變化（B17T_old）**：名稱依 `B17.txt:785`。KV 有兩個 `OnSpellStart`（`B17.txt:802-819` 與 `820-832`）。照本專案「重複鍵後者生效」（同 `saito_dosan_old`）採第二個：`MirrorImage`（`B17.lua:424-538`）。佐證：AbilitySpecial 只有 MirrorImage 讀的鍵（`B17.txt:833-865`），`B17T_old` 函式讀的 `B17T_old_Duration`／`_people`／`_outgoingDamage` 都不存在，`modifier_B17T_old` 也沒有定義。若實際上兩段都執行，`B17T_old` 只會多一次淨化敵方來源 modifier 與小幅位移（people = 1），不影響分身。
   - 分身數 9、持續 40 秒、造成傷害 -75/-65/-55%、承受 0%；半徑 50/sin(π/10) ≈ 161.8。
   - 0.01 秒的逐個生成間隔（create_delay）寫成同時出現。
   - KV 沒寫 RequiredLevel，照大招預設 6/12/18。
6. **迴旋手裏劍單位（b17r_old_shuriken）**：名稱「迴旋手裏劍」為自訂（單位只有註解 `霧隱才藏 迴旋手裏劍`，`npc_units_custom_load.txt:1576`）。賞金 15 金因單位無敵不列入。
