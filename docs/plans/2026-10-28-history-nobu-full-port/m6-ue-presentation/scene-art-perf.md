# nobu 場景換 Tripo 元件：前後效能對照

## 換了什麼

- **地形**：305 個 gridnav 方塊（Cube＋石材）換成崖壁模組。
  - 長條方塊沿長邊排 `cliff_straight`。每段長度約為厚度的 4 倍，限制在 384～768 單位之間，相鄰段重疊 6%。
  - 近方形方塊（長邊小於短邊的 1.5 倍）用一塊 `cliff_corner`。
  - 高度 140 單位，共 497 個模組。
- **樹**：1,564 棵依樹種分開。
  - 竹 518 棵：`bamboo_tree`。
  - 闊葉 603 棵：`broadleaf_tree`。
  - 松 443 棵沿用 Kenney `SM_pine_tree`。
  - 竹、闊葉樹和崖壁模組各用一個 HISM，松樹沿用原本的 ISM。
- **泉水和商店**：依地圖的 fountains／shops 位置放 `fountain_honjin`（寬 520）與 `shop_stall`（寬 260），朝向地圖中心。泉水位置上的本陣商店不重複放。
- **大將塔**：`moba_nexus_tower` 改用 `general_tower`（高 260），和路上的箭樓分開。
- **bridge ABI 43→44**：
  - `OmTreeOccluder.kind`（0 其他、1 竹、2 闊葉、3 松）。
  - `OmFrameHeader.map_landmarks`（1 泉水、2 商店，上限 64）。
- **退路**：任一美術資源找不到時，該類退回原本的方塊和松樹，log 會印 `Scene art unavailable`。
- **commit**：omfue f7778e8、主 repo d0bbfef8；大將塔 a6231b41；效能 log 9dd5bf71。

## 量測方法

- **局**：`run_moba_split_local --recipe scripts/lua_data/moba_nobu_jungle_check.lua --autopilot lane_v0 --performance-lock --max-seconds 300`。nobu 地圖，1 人＋9 bot，英雄 12 級，兵線開啟。
  - before：`target/lineC-perf/nobu-before`
  - after：`target/lineC-perf/nobu-after2`
- **數字來源**：UE 每 60 幀印一行 `OM_RENDER_STATS`（新增，9dd5bf71），包含：
  - 上一幀的 `GNumDrawCallsRHI`、`GNumPrimitivesDrawnRHI`（三角形數，含陰影 pass）。
  - GPU、render thread、game thread 的 ms。
  - frame interval 取同一時段的 `OM_PERF game_frame_interval` 視窗平均。
- 兩局都略過前 10 個視窗，各 151 個樣本。

## 結果

| 項目 | before 中位數 | after 中位數 | before p95 | after p95 |
|---|---|---|---|---|
| frame interval (ms) | 16.67 | 16.67 | 16.92 | 16.90 |
| GPU (ms) | 16.77 | 16.51 | 19.48 | 21.06 |
| render thread (ms) | 4.58 | 4.48 | 6.07 | 5.75 |
| game thread (ms) | 10.15 | 9.74 | 14.39 | 13.90 |
| draw call | 168 | 183 | 214 | 213 |
| 三角形 | 101,123 | 109,493 | 549,889 | 480,603 |
| 地形 instance | 305 | 497 | | |
| 樹 instance | 1,564 | 1,564 | | |

- frame interval 前後都是 60 fps（16.67 ms），沒有變差。
- draw call 中位數多 15（+9%），來自多出來的 5 個 instanced component 和它們的材質。p95 沒有變。
- 三角形中位數多 8%；p95 反而少 12%。p95 的峰值出現在鏡頭拉遠、陰影 pass 涵蓋大片樹林的時候，竹和闊葉樹每棵的面數（774／694）比 Kenney 松樹少。
- GPU 中位數持平，p95 多 1.6 ms，仍在 60 fps 以內。這台機器同時有其他 session 在建置，GPU 數字偏保守。

## 載入時間

- 第一次載入新元件（cold shader cache）時，從 UE 啟動到進地圖花了 3 分 28 秒（`nobu-after`，局因此被 300 秒時間上限收掉，沒有樣本）。before 只花 56 秒。
- cache 熱了以後（`nobu-after2`），進地圖花 42 秒。
- `nobu-after2` 第 5 張截圖右上仍顯示 `Preparing Shaders (1)`，畫面裡的竹子是灰色的，表示竹子的材質在遊戲中才編完。
- 下一步依調度要求處理：
  - 把新元件的材質納入預編／預熱流程，比照 E549 的 Niagara precompile。
  - 能共用 master material 的就共用。
  - 量 cold cache 載入時間，目標回到 1 分鐘左右。

## 截圖

- after：`target/lineC-perf/nobu-after2/p1/autopilot-p1-shot-005-combat-text.png`，左下是竹林，材質還在編譯，所以是灰色。
- before：`target/lineC-perf/nobu-before/p1/autopilot-p1-shot-001-facing.png`。
- autopilot 的截圖都是跟著英雄的近景，看不到整片崖壁和泉水。做預熱時會補一組固定鏡頭的遠景截圖（泉水、商店、崖壁、三種樹）。
