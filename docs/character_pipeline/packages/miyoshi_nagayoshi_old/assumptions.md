# B10_old 三好長慶（舊版）假設

原作 `resource/addon_tchinese.txt`（UTF-16）沒有任何 `B10*_old` 說明，四招的說明與名稱都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名稱依效果推定：疾行、飛斧、戰魂、魔性狂化。

1. **槽位**：`heromap_skill["B10"]["11"] = "WERT"`（`vscripts/addon_game_mode.lua:289`），choose_11 逐一 `AddAbility("B10?_old")`（`vscripts/items/Addon_Items/tofar_goback.lua:96-140`），自動學習為空（392 行）。沒有 D。B10T_old 的 `HotKeyOverride "W"`（`npc/B_Unified/B10.txt:833`）只改快捷鍵，槽位照 T 放 R。
2. **疾行（B10W_old）**：說明依 `npc/B_Unified/B10.txt:587-656` 還原。`modifier_B10W` 的狀態區塊鍵名是小寫 `"states"`（650 行）。推定 KeyValues 鍵名不分大小寫，所以「穿過單位」照樣生效。
3. **飛斧（B10E_old）**：說明依 `npc/B_Unified/B10.txt:659-733` 與 `vscripts/heroes/B_Unified/B10.lua:1-35`（和本體 B10E 共用同一個函式）還原。速度照本體的換算寫成 3500（design_intent）。暈眩照本體慣例寫 `dispel = "strong"`。
4. **戰魂（B10R_old）**：說明依 `npc/B_Unified/B10.txt:735-819` 還原。
   - 光環本體 `modifier_B10R_aura` 自己有屬性，又以 `Aura_ApplyToCaster 1` 給自己一份 `modifier_B10R_buff`，所以長慶得到兩倍加成。寫成 `passive.stats` 加上含自己的光環。
   - `add_movespeed` 條目的 `var_type` 拼成 `vsr_type`（756 行）。推定數值照樣讀得到。
5. **魔性狂化（B10T_old）**：說明依 `npc/B_Unified/B10.txt:823-916` 還原。modifier 名稱 `modifier_A06W` 定義在 B10T_old 自己的區塊（873 行），數值讀 B10T_old 自己的 AbilitySpecial。
