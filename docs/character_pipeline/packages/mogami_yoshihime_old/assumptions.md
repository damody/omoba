# B33_old 最上義姬（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）沒有 `B33*_old` 的技能說明，只有 `modifier_B33R_old_debuff`「毒氣之箭：持續造成傷害」（2241 行）。四招說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱取 KV 註解：腐蝕之箭、蛛網纏身、毒氣之箭、蜘蛛纏身。

1. **槽位**：`heromap_skill["B33"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:312`），choose_11 逐一 `AddAbility("B33?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`）。沒有 D，也不自動學（415 行）。
2. **腐蝕之箭（B33W_old）**：說明依 `npc/B_Unified/B33.txt:684-852` 還原。毒傷與 MULTIPLE 緩速是兩個 modifier。緩速的 modifier 隱藏，所以只在毒傷狀態上寫說明。
3. **蛛網纏身（B33E_old）**：說明依 `B33.txt:853-992` 還原。推定 SOFT_DISARMED 用 `disarm` 表達。命中傷害與每跳傷害讀同一個 `damage`。
4. **毒氣之箭（B33R_old）**：說明依 `B33.txt:993-1100` 與 `vscripts/heroes/B_Unified/B33.lua:198-252` 還原。
   - 原作的毒霧是無敵 dummy 加 1 秒 think 的 modifier，這裡改成不建單位的 `repeat`，在攻擊當下記住的位置刷新中毒三次。推定 dummy 的第三次 think 會發生，這一點記成 design_intent。
   - 推定中毒已存在時重套不會重跑 OnCreated。
   - 命中瞬間有兩次傷害：OnCreated 一次，AMHC 立即傷害一次。兩個分支相同（214-219 行）。
5. **蜘蛛纏身（B33T_old）**：說明依 `B33.txt:1101-1238` 與 `B33.lua:91-190` 還原，腳本與本體共用，寫法照本體的蜘蛛狀態機。
   - B33T_old 缺 Ult_Level、decrease_health，舊版沒有減療。
   - 目標可以是一般單位與魔免敵人（1109、1114 行）。
   - 單位模板 `b33t_old_spider_unit` 沿用原作 B33T_UNIT，數值照本體模板。v3_rust 回報它不是 content summon（`templates/v3_summons.lua` 尚未收錄），屬預期。
