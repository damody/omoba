# 原作地圖 nobu.vmap 摘要

由 `tools\lua\lua.exe scripts\gen_nobu_map.lua` 從原作 history_nobu（commit f966720）`content/dota_addons/nobu/maps/nobu.vmap` 唯讀解析產生，不要手改。資料在 [nobu.lua](nobu.lua)，每筆都附出處 `<vmap>#<nodeID>`。

- 格式：binary 9, vmap 29。解析器 `scripts/nobu_map/dmx.lua` 讀到檔尾剛好用完每個位元組。
- prefab：共讀了 39 份 vmap（主地圖與 38 份 prefab，含巢狀），逐層累積位置、角度、縮放後展開。
- 有 2 個 prefab 不在 addon 裡，是 Dota 內建資產，只記錄它的擺放位置：`maps/prefabs/structure_fountain_radiant_basic.vmap`、`maps/prefabs/structure_fountain_dire_basic.vmap`。
- 座標：原作單位 × `world_units_per_source_unit` = 1。這是唯一的換算係數，射程等 `dist` 數值目前也是 1:1。

| 項目 | 數量 | 備註 |
|---|---|---|
| 地圖邊界 world_bounds | 1 | min (-7680, -7936)、max (7680, 6912) |
| 出兵路徑 | 6 | nobu_button 7 點、nobu_middle 9 點、nobu_top 6 點、wl_button 7 點、wl_middle 7 點、wl_top 7 點 |
| 塔 | 32 | 隊伍：badguys 16、goodguys 16；路：bot 12、mid 8、top 12 |
| 兵營 | 12 | badguys 6、goodguys 6 |
| 主堡 | 2 | badguys 1、goodguys 1 |
| 泉水 | 2 | Dota 內建 prefab，只有位置 |
| 商店 | 6 | shoptype：0 2、1 2、2 2 |
| 出生點 | 16 | any 2、badguys 7、goodguys 7 |
| 大將塔（com_general*） | 10 | badguys 5、goodguys 5，是基地內的 npc_dota_base 單位 |
| 野怪營地 | 24 | 中立單位 68 個，相距 450 以內的單位歸為同一營 |
| 魔王 | 2 | npc_dota_cursed_warrior_souls、npc_dota_the_king_of_robbers |
| 基地單位 com_soldiercamp | 24 | badguys 12、goodguys 12，用途地圖看不出來（見下） |
| 裝飾用的假主堡 | 2 | *_fake |
| 地形網格 | 70×70 tile | 原點 (-8995, -8924)，tile 邊長 256；頂點高度等級 5041 個；水面頂點 240 個 |
| 樹木 | 1564 | 物件格點（每 64 單位）上的樹種 |
| 植物 | 1168 | 稀疏／中等／茂密，屬於呈現，不是遊戲的草叢 |
| gridnav 不可通行格 | 4629 | 280×280 格，每格 64 單位 |

## 無法確定或沒有的資料

- **草叢**：地圖裡沒有草叢實體或觸發區。原作的草叢視野若由腳本或其他資料定義，不在 vmap 裡；植物格點是裝飾。
- **高度**：只有每個頂點的整數等級（0～5）。每一級代表多少世界單位，檔案裡沒有記錄；Dota 地形編輯器的慣例是每級 128（推定，未驗證）。
- **網格方向**：以「x 先、逐列」排列，這是從資料推定的。依據：這個順序下，242 個遊戲實體沒有一個落在不可通行格上；x、y 對調時有 5 個。
- **泉水**：用的是 Dota 內建 prefab，addon 沒有它的內容，只知道擺放位置。
- **野怪的生成規則**：vmap 只有擺放的單位；重生與成長在原作腳本裡，不在地圖。
- **網格數值**：有一個頂點的高度是 -1，原樣保留。
- **基地單位 com_soldiercamp**：每方 12 個 npc_dota_base，放在基地裡，單位名稱與兵營建築（npc_dota_barracks 的 com_soldiercamp_*_top 等）不同。作用要看原作的單位 KV 與腳本，這裡只列位置。

## 野怪營地

| 營地 | 中心 | 組成 |
|---|---|---|
| camp_01 | (7359, 7451) | npc_dota_neutral_130_japanese_pirates_leader, npc_dota_neutral_130_japanese_pirates_sea_people x2 |
| camp_02 | (4465, 7374) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_douchebag |
| camp_03 | (-1679, 4402) | npc_dota_neutral_130_giant_tortoise x3 |
| camp_04 | (7330, 4370) | npc_dota_neutral_130_wild_bear x3 |
| camp_05 | (890, 3995) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_douchebag |
| camp_06 | (2146, 3591) | npc_dota_neutral_160_bandit_military_adviser, npc_dota_neutral_160_executor x2 |
| camp_07 | (-195, 2902) | npc_dota_neutral_130_fear_molang x3 |
| camp_08 | (3198, 2535) | npc_dota_neutral_130_wild_warrior x2 |
| camp_09 | (4580, 1783) | npc_dota_neutral_130_giant_tortoise x3 |
| camp_10 | (-4299, 1465) | npc_dota_neutral_130_fear_molang x3 |
| camp_11 | (3739, 956) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_bandit |
| camp_12 | (3616, 18) | npc_dota_neutral_130_pour_odd_person x2 |
| camp_13 | (-3576, -395) | npc_dota_neutral_130_pour_odd_person x2 |
| camp_14 | (-3535, -1331) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_bandit |
| camp_15 | (4872, -1805) | npc_dota_neutral_130_fear_molang x3 |
| camp_16 | (-4586, -2075) | npc_dota_neutral_130_giant_tortoise x3 |
| camp_17 | (-3315, -2669) | npc_dota_neutral_130_wild_warrior x2 |
| camp_18 | (121, -3314) | npc_dota_neutral_130_fear_molang x3 |
| camp_19 | (-2582, -3801) | npc_dota_neutral_160_bandit_military_adviser, npc_dota_neutral_160_executor x2 |
| camp_20 | (-7562, -3967) | npc_dota_neutral_130_wild_bear x3 |
| camp_21 | (-912, -4184) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_douchebag |
| camp_22 | (2092, -4933) | npc_dota_neutral_130_giant_tortoise x3 |
| camp_23 | (-7717, -7379) | npc_dota_neutral_130_japanese_pirates_leader, npc_dota_neutral_130_japanese_pirates_sea_people x2 |
| camp_24 | (-4056, -7521) | npc_dota_neutral_160_assassin x2, npc_dota_neutral_160_douchebag |

