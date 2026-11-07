# nobu 場景換 Tripo 元件：前後效能對照

## 結論

驗收以 CPU 低的時段為準（`nobu-after9`，CPU 19～43%，423 個樣本）。

| 項目 | before | after9 | 目標 | 結果 |
|---|---|---|---|---|
| render thread 中位數 (ms) | 4.58 | 3.86 | ≤5 | 達標 |
| frame interval p95 (ms) | 16.92 | 16.72 | ≤17.5 | 達標 |
| 三角形中位數 | 101,123 | 119,593 | ≤150k | 達標 |
| 三角形 p95 | 549,889 | 628,555 | ≤605k | **未達標** |

三角形 p95 超過目標約 4%，但資料顯示峰值不是新元件造成的，見「三角形 p95」。

## 換了什麼

- **地形**：305 個 gridnav 方塊（Cube＋石材）換成崖壁模組。
  - 長條方塊沿長邊排 `cliff_straight`。每段長度約為厚度的 6 倍，限制在 640～1280 單位之間，相鄰段重疊 6%。
  - 近方形方塊（長邊小於短邊的 1.5 倍）用一塊 `cliff_corner`。
  - 高度 140 單位，共 395 個模組。
- **樹**：1,564 棵依樹種分開。
  - 竹 518 棵：`bamboo_tree`。
  - 闊葉 603 棵：`broadleaf_tree`。
  - 松 443 棵沿用 Kenney `SM_pine_tree`。
  - 竹、闊葉樹和崖壁模組各用一個 HISM，松樹沿用原本的 ISM。竹和闊葉樹不投影。
- **泉水和商店**：依地圖的 fountains／shops 位置放 `fountain_honjin`（寬 520）與 `shop_stall`（寬 260），朝向地圖中心。泉水位置上的本陣商店不重複放。
- **大將塔**：`moba_nexus_tower` 改用 `general_tower`（高 260），和路上的箭樓分開。
- **共用材質**：`OmSceneArtMaterials` commandlet 在 `/Game/Omoba/Structures/TripoStaticShared/` 建立以下資源，匯入的原始 package 不動：
  - `M_TripoStatic`：master material，Opaque、DefaultLit、非 two-sided，帶 instanced static mesh usage。
  - `MI_<id>`：每個模型一個 instance，只換 base color。
  - `SM_<id>`：帶 LOD 的 mesh 副本，LOD1、LOD2 為 25%、10%。
  - `T_<id>`：貼圖上限副本，樹 512，其他 1024。
- **shader 預編**：commandlet 把 master 的 SM5 shader 預編進共享 DDC，載入畫面以 `reason=ready` 關閉，不再等 shader grace。
- **bridge ABI 43→44**：
  - `OmTreeOccluder.kind`（0 其他、1 竹、2 闊葉、3 松）。
  - `OmFrameHeader.map_landmarks`（1 泉水、2 商店，上限 64）。
- **退路**：任一美術資源找不到時，該類退回原本的方塊和松樹，log 會印 `Scene art unavailable`。

各模型的 LOD 三角形數：

| 模型 | LOD0 | LOD1 | LOD2 |
|---|---|---|---|
| cliff_straight | 1,360 | 340 | 136 |
| cliff_corner | 1,416 | 354 | 142 |
| bamboo_tree | 774 | 194 | 78 |
| broadleaf_tree | 694 | 174 | 70 |
| fountain_honjin | 7,171 | 1,792 | 718 |
| shop_stall | 7,313 | 1,828 | 732 |

## 量測方法

- **局**：`run_moba_split_local --recipe scripts/lua_data/moba_nobu_jungle_check.lua --autopilot lane_v0 --performance-lock`。nobu 地圖，1 人＋9 bot，英雄 12 級，兵線開啟。每局的英雄陣容相同。
- **數字來源**：UE 每 60 幀印一行 `OM_RENDER_STATS`，包含上一幀的 draw call、三角形數（含陰影 pass）與 GPU、render thread、game thread 的 ms。frame interval 取同一時段的 `OM_PERF game_frame_interval` 視窗平均。每局略過前 10 個視窗。
- **對照局**：`nobu-after9-ab` 設 `OM_SCENE_ART_AB_SECONDS=20`，每 20 秒隱藏一組元件，stats 記下隱藏哪一組。`all_new` 隱藏全部新元件，但松樹還在，舊方塊也沒有放回去，所以它不等於重跑 before。
- **負載**：after9 與對照局都用 `--prebuilt`，等 CPU 連續約 1 分鐘低於 35% 才開跑，並每 2 秒記錄 CPU。量測期間調度請其他工作線不開新的大型建置。

## render thread：是機器負載，不是新元件

相同內容在不同負載下差很多：

| 局 | 狀況 | render thread 中位數 | frame p95 |
|---|---|---|---|
| after9 | CPU 19～43%，全部顯示 | 3.86 | 16.72 |
| after9-ab | CPU 17～45%，輪流隱藏 | 3.85 | 16.71 |
| after8 前 85 秒 | 輪流隱藏 | 4.43 | 17.06 |
| after8 之後 | 輪流隱藏，同時有建置 | 6.77 | 23.17 |
| after7 | 全部顯示，未記錄 CPU | 6.58 | 18.81 |

- after8 在同一局裡從 4.4 ms 跳到約 6.7 ms，和當時隱藏哪一組無關。當時有 land_to_live 在編譯，但那局沒有記錄 CPU，所以跳升的時間點和建置的對應是推論。after9 的低負載數字支持這個解釋。
- after5 與 after7 的數字不能拿來比較元件好壞，它們可能受負載影響。
- before 當時的負載不明。after9 的 render thread 比 before 低，可能代表 before 也有一些負載，不代表新元件比方塊便宜。

after9-ab 的分組中位數（低負載）：

| 隱藏 | 樣本 | render thread | game thread | GPU | draw call |
|---|---|---|---|---|---|
| 無 | 70 | 3.80 | 8.53 | 16.52 | 204 |
| 崖壁 | 80 | 3.93 | 7.88 | 16.41 | 196 |
| 竹 | 67 | 4.07 | 7.79 | 16.60 | 198 |
| 闊葉 | 60 | 4.07 | 8.87 | 16.50 | 199 |
| 松 | 60 | 3.72 | 8.00 | 16.61 | 192 |
| 地標 | 60 | 3.82 | 8.87 | 16.50 | 188 |
| 全部新元件 | 60 | 3.72 | 8.75 | 16.34 | 187 |

全部隱藏時 render thread 只比全部顯示少 0.08 ms，在雜訊之內。GPU 中位數約 16.5 ms 接近 60 fps 的一幀，應該是被幀率上限拉平，不能當作 GPU 實際負擔。

## 三角形 p95：峰值不是新元件

after9-ab 各組的三角形數：

| 隱藏 | p50 | p95 |
|---|---|---|
| 無 | 105,681 | 395,811 |
| 崖壁 | 120,229 | 557,583 |
| 竹 | 223,855 | 527,983 |
| 闊葉 | 364,673 | 668,881 |
| 松 | 105,717 | 666,979 |
| 地標 | 128,423 | 609,373 |
| 全部新元件 | 318,753 | 670,853 |

- 全部新元件隱藏時 p95 仍有 671k，比全部顯示還高。分組差異主要來自那 20 秒鏡頭和戰況的差別，不是元件本身。
- 峰值以約 58k 為一階上下，before 也有同樣的階梯（58k、59k）。這符合「一個約 29k 面的單位進出畫面，主 pass 和陰影 pass 各算一次」，**但還沒有確認是哪種單位**。
- 同一份內容的局間差異很大。取相同的前 151 個樣本，p95 分別是：

| 局 | p95 |
|---|---|
| before | 549,889 |
| after2 | 480,603 |
| after7 | 553,543 |
| after9 | 785,811 |

- before 只跑 300 秒，after9 跑 540 秒，涵蓋更多後期戰鬥，全局 p95 不完全可比。

因此再降場景元件應該降不了這個數字。要確實降低，要先確認 58k 的來源（可能是英雄模型的面數或陰影），並用固定鏡頭量測排除局間差異。

先前版本寫「p95 的峰值來自鏡頭拉遠、陰影 pass 涵蓋大片樹林」，資料不支持，已經撤回。

## 貼圖

6 張匯入貼圖都是 2048×2048、預設壓縮、World 群組、可串流、非 virtual texture，沒有 4K。上限副本已經建立，MI 指向副本。commandlet 在 `-NullRHI` 下讀不到實際尺寸（回報 0×0），所以**縮小後的實際尺寸還沒有實機確認**。

## 載入時間

UE 啟動、asset registry 和遊戲本身要分開看：

| 階段 | before | after7（新 ue-run 副本） | after9（副本已熱） | 屬於 |
|---|---|---|---|---|
| LoadMap | 1.05 s | 9.2 s | 0.43 s | 遊戲 |
| AssetRegistryGather | 18 s | 97 s | 15 s | ue-run 副本與磁碟，非遊戲 |
| 載入畫面顯示 | 1.9 s | 10.3 s | 1.3 s | 遊戲 |

- 先前的 3～4 分鐘主要是新 ue-run 副本的 asset registry 掃描（`nobu-after` 為 135 s）加上引擎啟動，不是 shader，也和遊戲內容無關。after7 的 CPU 負載未記錄，也可能拉長這些數字。
- after9 載入快，但用的是已熱的快取，不能證明是貼圖上限的功勞。
- **真正的 cold cache 載入（目標 1 分鐘左右）還沒有量。**

## 截圖

- 竹子已是正確材質，不再是灰色：`target/lineC-perf/nobu-after9/p1/autopilot-p1-shot-003-combat-text.png`。
- 箭樓與竹林：`target/lineC-perf/nobu-after9/p1/autopilot-p1-shot-007-periodic.png`。
- before：`target/lineC-perf/nobu-before/p1/autopilot-p1-shot-001-facing.png`。
- autopilot 的截圖都是跟著英雄的近景。**整片崖壁、泉水和商店的遠景還沒有截圖確認。**

## 尚未完成

- 三角形 p95 未達標，要確認 58k 階梯的來源，並決定是否用固定鏡頭量測。
- cold cache 載入時間。
- 貼圖上限的實際尺寸。
- 崖壁、泉水、商店的遠景截圖。
