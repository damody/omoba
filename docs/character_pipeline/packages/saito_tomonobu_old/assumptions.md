# B03_old 齋藤朝信（舊版）假設

原作 `resource/addon_tchinese.txt` 沒有任何 `B03*_old` 說明，四招的說明都依 KV 與腳本還原（路徑相對 `game/dota_addons/nobu/scripts/`）。招式名取 KV 註解（`npc/B_Unified/B03.txt:403,472,508`；B03T_old 沒有註解，沿用本體大招名「鍾馗之審判」）。英文名為自擬。

1. **槽位**：`heromap_skill["B03"]["11"] = "WERT"`、autoskill 空（`vscripts/addon_game_mode.lua:282,385`），`vscripts/items/Addon_Items/tofar_goback.lua:105-133` 依序 AddAbility，W/E/R/T → Q/W/E/R。舊版沒有本體的 B03D／B03F。
2. **鍾馗之力（B03W_old）**：依 `npc/B_Unified/B03.txt:404-470` 與 `vscripts/heroes/B_Unified/B03.lua:161-167` 還原：對友方單體施放，攻速、移速加成，持續時間逐級。可以對自己施放是 Dota 友方單體目標的預設行為（推定）。
3. **越後金鋼（B03E_old）**：依 `npc/B_Unified/B03.txt:473-506` 還原：純被動護甲。
4. **怪力擊（B03R_old）**：依 `npc/B_Unified/B03.txt:509-563` 與 `B03.lua:169-188` 還原：命中 25% 機率暈眩並造成物理傷害，建築除外。
5. **鍾馗之審判（B03T_old）**：依 `npc/B_Unified/B03.txt:565-609` 與 `B03.lua:190-218` 還原：對射程 200 的敵方單體施放，對目標周圍 600 範圍造成魔法傷害。KV 沒寫 RequiredLevel，需要等級推定 6／12／18（同本體）。
