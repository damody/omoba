# B02_old 風魔小太郎（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B02*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/B_Unified/B02.txt:668,795,914,1030`）。

1. **槽位與沒有 D**：`heromap_skill["B02"]["11"] = "WERT"`、autoskill 空（`vscripts/addon_game_mode.lua:281,384`），`vscripts/items/Addon_Items/tofar_goback.lua:105-133` 依序 AddAbility。舊版迷蹤步是 Q、可升四級；沒有忍法．朧。
2. **忍法．迷蹤步（B02W_old）**：依 `npc/B_Unified/B02.txt:668-796` 與 `vscripts/heroes/B_Unified/B02.lua:428-438` 還原：隱形、加速、無碰撞；攻擊**命中**時額外物理傷害並現身（不排除建築），使用技能或道具現身。
3. **忍法．熾烈之徑（B02E_old）**：依 `npc/B_Unified/B02.txt:796-915` 與 `B02.lua:440-465` 還原：移動每超過 100 距離留一團火（光環半徑 140，存在 10 秒），燃燒每 0.25 秒魔法傷害。光環更新間隔（0.1 秒）與離開後殘留（0.5 秒）是 Dota 引擎行為，值為推定（`assumed:`）。多團火焰的燃燒推定不疊加（同本體）。
4. **忍法．十八方手裏劍（B02R_old）**：與本體共用 `B02.lua:148-269`，只有數值不同（`npc/B_Unified/B02.txt:915-1031`）。原作實際二十枚，比照本體 FC-071 寫十八枚（quirk `fixed`），是否沿用請工作線 D 確認。
5. **忍法密傳．四方導雷（B02T_old）**：依 `npc/B_Unified/B02.txt:1031-1200` 與 `B02.lua:467-706` 還原：標記 10 秒並持續給視野；5 秒後暈眩 2 秒，4/8/12 道閃電各打原目標 300 並各自彈跳（每跳 +5% 基礎、最多 100 跳、半徑 1000、間隔 0.3）。彈跳結束時折返原目標的最後一擊無法用 `chain` 表達，列為 pending（gap：chain 的 on_end）。
