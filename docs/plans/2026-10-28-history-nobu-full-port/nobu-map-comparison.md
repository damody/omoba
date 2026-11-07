# 原作地圖 nobu.vmap 與現行 moba_maps.lua 的比較，以及用原作座標建圖的方案

建立日期：2026-10-29。負責：工作線 A（資料與工具）。原作資料是 [docs/character_pipeline/maps/nobu.lua](../../character_pipeline/maps/nobu.lua)，摘要與限制見 [nobu.md](../../character_pipeline/maps/nobu.md)。

遊戲模式 session 正在改 `scripts/lua_data/templates/moba_maps.lua`，所以這份只提出資料、報告與方案，沒有改 live 的地圖檔。

## 1. 怎麼取得的

- 工具：`tools\lua\lua.exe scripts\gen_nobu_map.lua`，讀原作 `content/dota_addons/nobu/maps/nobu.vmap`，格式是 binary DMX 9、vmap 29。
  - `scripts/nobu_map/dmx.lua`：Lua 5.4 `string.unpack` 寫的唯讀解析器，讀到檔尾每個位元組都用完。
  - `scripts/nobu_map/world.lua`：從 CMapWorld 逐層累積位置、角度、縮放，並展開 156 個 prefab 擺放（38 份 prefab vmap，含巢狀）。
- 只讀原作檔案，沒有執行也沒有複製任何資產。
- 每筆資料都附出處 `<vmap>#<nodeID>`。
- 座標換算集中在一個係數 `world_units_per_source_unit = 1`。它和 `dist` 數值的規則一致：射程、半徑目前就是原作原值，例如塔射程 800。

## 2. 差異

| 項目 | 原作 nobu.vmap | 現行 moba_maps.lua（three_lane_training） |
|---|---|---|
| 地圖範圍 | 可玩範圍是地形網格 70×70 tile × 256：(−8995, −8924)～(8925, 8996)，約 17,920 見方，minimap 邊界 (−9216, −9216)～(8960, 9216)。world_bounds (−7680, −7936)～(7680, 6912) 比較小，聯軍方的泉水、出生點與 3 個營地都在它的 y 上限之外，所以它不是可玩範圍（很可能是鏡頭範圍） | 0～4800 的正方形（設計座標 × `layout_scale` 2） |
| 主堡 | 織田方 (5777, −5722)、聯軍方 (−5480, 5473)（`dota_{goodguys,badguys}_fort`，單位 com_soldiercamp_Oda／Unified） | 兩方基地在 (0, 0) 與 (4800, 4800) |
| 路線 | 每方各 3 條、共 6 條有方向的路徑點鏈（`chubinluxian_location_of_{nobu,wl}_{top,middle,button}N`，6～9 點）；邊路長約 20,900～21,700，中路約 14,000 | 3 條雙向共用路線，邊路約 9,000、中路約 6,800 |
| 塔 | 每方 16 座：邊路 3 層 × 2 座並排，中路 2 層 × 2 座（共 32 座） | 每方 9 座：每路 3 層、每層 1 座，位置由 `tower_layers` 沿路距離推算 |
| 兵營 | 每方 6 座：每路 2 座（`com_soldiercamp_{oda,unified}_{top,mid,down}`） | 每路 1 座 inhibitor |
| 大將塔 | 每方 5 座（com_general_*、com_general2_*，npc_dota_base 單位） | nexus tower（`nexus_tower_offset`） |
| 基地單位 | 每方 12 個 com_soldiercamp npc_dota_base，用途地圖看不出來 | 無 |
| 野怪營地 | 24 營、68 隻中立單位，共 14 種；左右對稱各 12 營 | 16 營：5 種 × 2 方，加上 2 守衛、2 河道斥候、2 魔王 |
| 魔王 | 2 隻：npc_dota_cursed_warrior_souls (−3671, −3892)、npc_dota_the_king_of_robbers (3548, 3692) | 2 隻 epic guardian（數值已照原作） |
| 泉水 | 2 處（Dota 內建 prefab，只有位置）：(7445, −7518)、(−7391, 7203) | 由基地位置推算 |
| 商店 | 6 處：shoptype 0（家）2、1（路邊）2、2（神秘）2；其中 3 處在 world_bounds 之外 | 地圖未定義 |
| 樹木 | 1,564 棵（地形網格的物件格點，5 種樹） | 無；以抽象牆區塊表示叢林 |
| 不可通行 | gridnav 280×280 格（每格 64），其中 4,629 格不可通行 | 抽象牆區塊與 2 個小島 |
| 高度 | 每個頂點的整數 cliff 等級 0～5；每級多高，地圖沒有記錄 | 平面 |
| 水面 | 240 個頂點標為水面（河道） | `central_river` 呈現帶 |
| 草叢 | **原作沒有**：vmap 沒有草叢實體或觸發區，原作腳本也沒有 bush／grass | 6 塊作者自訂的草叢 |
| 裝飾 | 658 個 prop_static（柵欄 348、軍旗 190……）、植物格點 1,168 | 呈現用的 surfaces |

## 3. 方案：用原作座標建 nobu 地圖

目標是新增一張 `nobu` 地圖，與 `three_lane_training` 並存，全部座標都取自原作資料。只換數值不夠，原作的佈局需要引擎多幾項能力。

### 3.1 資料流

1. **原作資料**：`docs/character_pipeline/maps/nobu.lua` 由工具產生，不手改。
2. **轉換**（已完成，見 §5.0）：`scripts/gen_nobu_map_content.lua` 產生 `scripts/lua_data/templates/nobu_map.lua`。它負責：
   - 座標乘上 `world_units_per_source_unit`；
   - 依隊伍與路線整理，單位保留原作名稱（`MapUnitName`）；對應到遊戲單位模板屬於 §5.4；
   - 把 gridnav 不可通行格合併成矩形；
   - 為每筆資料保留出處。
3. **地圖定義**：`moba_maps.lua` 加 `id = "nobu"`，讀 `nobu_map.lua`。這一步由遊戲模式 session 做，避免兩邊同時改同一個檔。

### 3.2 引擎要補的能力

| 能力 | 原因 | 建議負責 |
|---|---|---|
| 每方各自、有方向的路線 | 原作兩方的路徑點不同，不是同一條路反向 | 遊戲模式 |
| 塔、兵營、大將塔的明確座標與成對配置 | 原作同一層 2 座並排，不能由 `tower_layers` 沿路推算 | 遊戲模式 |
| 每路 2 座兵營、5 座大將塔 | 現行每路 1 座 inhibitor，只有 nexus tower | 遊戲模式 |
| 24 個營地與 14 種中立單位模板 | 現行只有 5 種營地 | 遊戲模式（數值照原作 KV） |
| 樹木實體（可被砍倒、會擋路與視野） | v3 技能已有 `destroy_trees`、`destroy_tree`，原作有 1,564 棵 | 規則層與遊戲模式 |
| 地形碰撞讀 gridnav | 4,629 個不可通行格；合併成矩形後可以沿用現有 terrain 區塊 | 遊戲模式 |
| 高地視野（cliff 等級） | 原作有 0～5 級；Dota 的高地會擋低處的視野。每級高度不明，先當平面 | 之後再議 |
| 地圖尺寸 | 可玩範圍約 17,920 見方，邊長是現行的 3.7 倍：fog 網格、minimap、鏡頭範圍與效能（run_10000）都要重新量 | 遊戲模式與 UE |
| 商店位置 | 6 處，其中 3 處在 world_bounds 外，要確認原作是否真的用到 | 遊戲模式 |

### 3.3 比賽時間

- 原作邊路約 21,000 單位，小兵移速 325，走完一條邊路約 65 秒；現行約 28 秒。
- 正式的 `nobu` 地圖照原作，quick 地圖仍以 `QUICK_TIME_SCALE` 調整節奏。
- 不建議用縮小座標來縮短比賽，因為射程與半徑是 1:1，縮小座標會讓技能相對變大。要縮的話，應該改 `world_units_per_source_unit`，並同時套用到 `dist` 數值，兩者一起變。

### 3.4 UE 底稿

- `tools\lua\lua.exe scripts\gen_nobu_map.lua --heightmap target\maps` 產生 `nobu_height_253.r16`：253×253、16-bit little endian，可以直接用 Landscape 的 Import from file 匯入。另附說明檔 `nobu_height_253.txt`。
- 高度是 cliff 等級線性對應到 0～65535，每級的實際高度要在 UE 匯入時決定，因為地圖沒有記錄。
- 樹木、prop_static 的座標與模型路徑都在 `nobu.lua`，美術導入時可以用。
- 原作模型不複製；美術是後續工作。

## 4. 無法確定、需要確認的事

- **高度**：cliff 等級一級的高度，地圖沒有記錄；Dota 地形編輯器的慣例是 128（推定）。
- **網格索引方向**：從資料推定為「x 先、逐列」。依據是這個順序下沒有任何遊戲實體落在不可通行格，對調時有 5 個。
- **基地單位 com_soldiercamp**：每方 12 個的作用，要查原作單位 KV 與腳本。
- **商店**：3 處在 world_bounds 外的商店是否實際可用。
- **草叢**：原作沒有。現行遊戲的草叢是移植時加的設計；`nobu` 地圖要不要保留，屬於遊戲設計決定。

## 5. 交辦規格：引擎要補的 9 項能力

每項列出要改的模組、資料格式與驗收方式。資料一律來自 §5.0 的 template，不另寫座標。

### 5.0 已有的輸入：`scripts/lua_data/templates/nobu_map.lua`

- 由 `tools\lua\lua.exe scripts\gen_nobu_map_content.lua` 從 `docs/character_pipeline/maps/nobu.lua` 產生，不手改。`--check` 確認檔案是最新的。
- **還沒被 `templates.lua` include**，遊戲內容不變。接上時，在 `templates.lua` 加 `nobu_map = ctx.include("templates/nobu_map.lua")`，並由 `moba_maps.lua` 的 `nobu` 地圖讀取。
- 形狀：

  ```lua
  {
    id = "nobu", world_units_per_source_unit = 1,
    bounds = { min, max },
    world_bounds = {...}, minimap_bounds = {...},  -- 可玩範圍是地形網格；world_bounds 只保留
    teams = {
      goodguys = {   -- 織田方（原作 nobu_*）；badguys 是聯軍方（wl_*）
        fort = { position, unit, source }, fountain = { position, source },
        lanes = { top = { waypoints = { {x, y, source}, ... }, source_name }, mid = {...}, bot = {...} },
        towers = { { lane, tier, position, unit, name, source }, ... },  -- 每層 2 座
        barracks = { { lane, position, unit, source }, ... },            -- 每路 2 座
        general_towers = { { position, unit, source }, ... },            -- 大將塔 5 座
        base_units = {...}, spawns = {...},
      },
      badguys = {...},
    },
    neutral_camps = { { id, center, units = { { unit, position, source } } } },  -- 24 營
    bosses = { { unit, position, source } },
    shops = { { kind = "home"|"side"|"secret", position, outside_bounds?, source } },
    trees = { { x, y }, ... },                  -- 1,564 棵
    terrain = { { min, max }, ... },            -- gridnav 不可通行格合併成的 305 個矩形
  }
  ```

- 產生時就會驗證，任何一項不通過就失敗：
  - 每方三路的路徑點都落在可通行格，而且在 gridnav 上互相連通；
  - 每條路從己方主堡 3,000 內出發，到敵方主堡 3,000 內；
  - 每座塔、兵營距離自己隊伍的同名路線 1,200 以內；
  - 所有位置都在可玩範圍內（商店依原作標出 `outside_bounds`）；
  - 數量與 nobu.lua 一致；
  - 矩形面積剛好等於不可通行格。
- 測試：`scripts/tests/nobu_map_test.lua`。

### 5.1 每方各自、有方向的路線

- **現況**：`MobaLaneEntry { id, waypoints }`（`omoba-template-ids/src/lua_content.rs`）是兩方共用的一條路，一方反向走。
- **改法**：
  - 允許 `lanes = { { id, team = "goodguys"|"badguys", waypoints } }`；沒寫 `team` 時維持現狀。
  - `MobaLaneConst` 加 `team`。
  - `omoba-core` 的小兵出生與尋路（`runtime/native/moba_match.rs`）依自己隊伍的路線走，不再反轉另一方的路線。
- **資料**：`teams.<team>.lanes.{top,mid,bot}.waypoints`。
- **驗收**：
  - lua_content 測試：兩方路線點數不同也能載入；同一隊同名的路線不能有兩條。
  - headless 一局：每方小兵依序經過自己路線的每個路徑點（記錄路徑後比對）。

### 5.2 塔的明確座標與成對配置

- **現況**：`tower_layers` 依沿路距離推算每層一座塔的位置。`combat_acquisition.rs` 的 `lane_tower_layers` 已經是每層兩個槽位（`[Option; 2]`）。
- **改法**：
  - `MobaMapEntry` 加 `towers = { { team, lane, tier, position, unit } }`，有寫時取代 `tower_layers`。
  - 同一 `team`、`lane`、`tier` 的塔放進同一層的兩個槽位。
  - 解鎖規則照現行：外層都倒了，內層才可被攻擊。原作是否有這條規則，要對照原作的 `chubing.lua` 與塔的 KV；不確定的話，先照現行規則並記 quirk。
  - 塔的數值仍讀 `structure_stats`。
- **資料**：`teams.<team>.towers`。原作單位名稱 `com_towera` 等，數值已在 `moba_maps.lua` 的 `structure_stats`。
- **驗收**：
  - 每方 16 座塔，位置與 template 完全一致。
  - 同層兩座都倒了，下一層才開放。
  - tower_plating、building_rewards 對每座塔各自結算。

### 5.3 每路 2 座兵營與 5 座大將塔

- **現況**：`base_structures.rs` 每路 1 座 inhibitor，加上依 `nexus_tower_offset` 擺放的 nexus tower。
- **改法**：
  - 加 `barracks = { { team, lane, position, unit } }`，每路可以有多座；該路的兵營都倒了，才觸發原作「兵營破壞強化」（`lane_down_percent`）。
  - 加 `general_towers = { { team, position, unit } }`，取代 nexus tower 的推算位置，數值照 `nexus_tower_*`。
  - `base_unlock` 照現行規則。
- **資料**：`teams.<team>.barracks`、`teams.<team>.general_towers`。
- **驗收**：
  - 每方 6 座兵營、5 座大將塔，位置一致。
  - 一路 2 座兵營都倒了才強化。
  - 大將塔倒了不重建（原作不重建）。

### 5.4 24 個營地與 14 種中立單位

- **現況**：`jungle_camps` 有 16 營、5 種，加上守衛、斥候與魔王，`members` 用相對 offset。
- **改法**：
  - 營地取自 `neutral_camps`：`position = center`，`members` 用各單位位置減去 center。
  - 14 種原作中立單位（`npc_dota_neutral_130_*`、`npc_dota_neutral_160_*`）加進單位模板，HP、攻擊、金錢、經驗、重生時間照原作 `npc/npc_units_custom_load.txt`。現行 `moba_maps.lua` 已經以原作數值寫了 5 種，可以沿用同一套寫法。
  - 魔王取自 `bosses`，數值已在 `moba_maps.lua`（north／south epic guardian）。
- **資料**：`neutral_camps[].units[].unit` 是原作單位名稱，需要一張「原作名稱 → 遊戲模板 id」對照表，放在 `moba_maps.lua` 或新的 template。
- **驗收**：
  - 開局後 24 營、68 隻中立單位都出生，位置與組成符合 template。
  - 重生與經驗、金錢照原作。
  - `jungle.rs` 的牽引範圍與重置照現行規則。

### 5.5 樹木實體

- **現況**：沒有樹木實體。v3 技能已經用到 `destroy_trees`、`destroy_tree`、`create_tree`；視野遮擋已有 `comp/vision_occluder.rs` 的 VisionTree。
- **改法**：
  - 新增樹木元件：位置、存活、重生計時；Dota 的樹被砍後約 300 秒重生，原作是否改過要再查。
  - 樹會擋路：動態的 BlockedRegions（`moba_map_layout.rs`）。
  - 樹會擋視野：沿用 VisionTree。
  - 讓 v3 的三個樹木 action 作用在這個元件上；這部分屬於 B 的 IR 直譯器。
- **資料**：`trees = { {x, y} }`。半徑沿用 Dota 樹的碰撞半徑（約 64），屬於推定，請 B 決定後寫進 rules。
- **驗收**：
  - 1,564 棵樹都載入。
  - `destroy_trees` 半徑內的樹被移除後，該處可以通行、視野打開；到時間後重生。
  - 效能：run_10000 不退步。

### 5.6 地形碰撞讀 gridnav

- **現況**：`terrain` 矩形經 `compiled_blocked_regions`（`moba_map_layout.rs`）成為不可通行區域，fog 也用它當遮擋。
- **改法**：
  - 直接使用 template 的 305 個 `terrain` 矩形，現有格式就能吃。
  - 要確認的是數量（現行地圖約 14 個）對尋路與 fog 的效能影響；需要的話，改成網格查表（280×280 bit）。
- **資料**：`terrain`，以及 `docs/character_pipeline/maps/nobu.lua` 的 `tile_grid.gridnav`（原始 bit 格）。
- **驗收**：
  - 單位不能進入不可通行格。
  - 路線全程可通行（template 已用 gridnav 驗證）。
  - 尋路與 fog 每 tick 的耗時在 run_10000 基準內。

### 5.7 高地視野（cliff 等級）

- **現況**：平面地圖。
- **改法**：
  - 地形網格每個頂點有 0～5 的 cliff 等級（`tile_grid.vertex_levels`，71×71）。
  - Dota 的規則是低處看不到高處，除非有飛行或特殊視野。只需要相對等級，不需要知道每級多高。
  - 先在 `visibility.rs`／`fog_grid.rs` 以「觀察者所在等級 ≥ 目標等級」作為額外條件。
- **資料**：要在 template 加 `levels`（目前只在 nobu.lua 的 `tile_grid`），接線時由產生器補上。
- **驗收**：低地的單位看不到高地上不在其他視野內的敵人；高地看得到低地。
- **優先度**：低。原作地圖大部分是等級 0，先當平面也能玩。

### 5.8 地圖尺寸

- **現況**：fog 網格（`FogGridGeometry`：origin、cell_size、columns、rows）、minimap 與鏡頭範圍都依現行約 4,800 見方的地圖設計。
- **改法**：
  - `FogGridGeometry` 依 `bounds` 產生；可玩範圍約 17,920 見方，邊長是現行的 3.7 倍。
  - UE 的 minimap 用 `minimap_bounds`，鏡頭限制用 `world_bounds`（原作的鏡頭範圍）或 `bounds`。
  - quick 地圖不縮小座標，見 §3.3。
- **驗收**：
  - fog 格數與每 tick 耗時在預算內，run_10000 不退步。
  - minimap 上塔與營地的位置與 template 一致。
  - 鏡頭到得了全部可玩範圍。

### 5.9 商店位置

- **現況**：地圖沒有定義商店。
- **改法**：
  - 加 `shops = { { kind, position } }`，購買限制在商店的購買半徑內（Dota 預設 720，請遊戲模式決定）。
  - secret 商店只賣原作標為 secret 的道具（`shops/nobu_shops.txt` 的 secretshop 分頁）。
  - `outside_bounds` 的 3 處（home 1、side 2）先不放，等確認原作是否真的使用。
- **驗收**：
  - 只能在商店範圍內購買；secret 道具只能在 secret 商店買。
  - 3 處界外商店的處理要寫進 quirk。
