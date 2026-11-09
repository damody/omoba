# C 線：信使 UE 呈現準備（2026-11-09）

依據 courier-spec-2026-11-09.md §5。後端 land 前先定好模型與 UI 接點；這份是盤點與版面，還沒有改任何 live 檔。

## 1. 模型

- omfue 裡**沒有**馬或坐騎模型：TripoLodV1 只有人形英雄、小兵、天狗／鬼／河童；WorldModelsV1 是石頭、樹、塔、武器；BP_Wolf 用的是小兵材質，不是動物 mesh。
- 做法：照金錢袋與變青蛙的形態，在 `scripts/prop_meshes.lua` 加一個 `courier_horse`，用引擎基本形狀（Sphere／Cylinder／Cone／Cube）拼一匹低多邊形的馬，由 OmPropMeshCommandlet 合成 `/Game/Omoba/Props/courier_horse/SM_courier_horse`。
  - 不花 Tripo 點數，來源是文字（符合 text-first），面數遠低於 5k。
  - 預計部件：身體、胸、臀、頸、頭、口鼻、兩耳、四腿加四蹄、鬃毛、尾巴、馬鞍、鞍布。朝 +X，與其他單位相同。
  - 材質槽：馬身棕、深色鬃毛／蹄、鞍皮，以及一塊鞍布。鞍布要帶隊伍色的話，靜態 mesh 的 `ally_material_sections`／`enemy_material_sections` 需要有 ally／enemy 兩份材質；prop commandlet 目前一個槽只產生一個材質，所以第一版用中性色鞍布，用血條顏色區分敵我。
  - 靜態 mesh 沒有走路動畫，與金錢袋相同。第一版能辨識就好。
- 接線：在 `scripts/lua_data/templates/world_units.lua` 加一列 presentation（`native_only`、`static_mesh_asset`、高度約 60），catalog id 要和 E 的信使單位模板相同。這是 live 輸入，要走 land_to_live 並跑 OmPropMesh commandlet（跑之前先看 target/locks/live_match）。

## 2. 選取與「送貨」「回家」按鈕

UE 已經有「選取自己的召喚物」整套流程：
- `OmSummonInput.cpp`：`SelectionIsCommandable` 要求 Kind 為 `EntitySummon`、`bCommandableSummon`、擁有者是自己。
- 選取後可以右鍵移動（operation 1）、釋放（operation 2）、施放召喚物的技能（operation 3，支援無目標、點、單位目標）。
- 技能按鈕由 `UOmCommandBarWidget::BuildSummonAbilityButtonText` 從 `FOmSelectedUnitStatePayload.Abilities` 畫出，冷卻、熱鍵都已處理。

**建議後端（E／B 決定）：**
1. 信使用 frame kind `EntitySummon`，只對主人是 `bCommandableSummon`。
2. 「送貨」「回家」做成信使身上的兩個無目標技能，例如 `courier_deliver`、`courier_return`。這樣 UE 選取信使時，兩個按鈕和熱鍵會直接出現，不用新增輸入種類，bot 也能用同一個 SummonOrder operation 3。
3. 信使不應該能被「釋放」（operation 2）。需要一個標記，例如 selected-unit payload 的 `bReleasable=false`，或後端直接拒絕；UE 依此隱藏釋放鍵。
4. 右鍵移動照用 operation 1。spec 說沒有指令時停在原地，這和召喚物相同。

若後端改成新的指令種類，UE 就要另做兩顆按鈕與新的 input action，工作量比較大。

## 3. 商店面板加 stash 6 格

- 現在頁尾是「背包」6 張小卡（`OmCommandBarShop.cpp:1947-2010`）。點選後按「出售」，出售走 `SubmitShopAction(false, slot)`。
- 版面：同一列在背包 6 格後面加「儲藏處」6 格，樣式相同，tag 用 `OmShopStash.<i>`。開關關閉時，或後端沒有送 stash 時，整組隱藏。
  ```
  金幣 1234 | 背包 [1][2][3][4][5][6] | 儲藏處 [1][2][3][4][5][6] | [出售] [復原]
  ```
- 點選 stash 卡片可以出售（只在商店範圍內，由後端判斷），tooltip 寫「儲藏處欄位 n，信使送貨或回到商店時取出」。
- 合成與最低價計算目前用 `FOmShopModel::InventoryIds`。開關打開、人在範圍內時要把 stash 也算進去，範圍外只算 stash。但 UE 不知道是否在範圍內，**需要後端在經濟狀態送一個「目前在商店範圍內」的布林值**，不然價格提示會不準（購買本身仍以後端判斷為準）。
- **需要後端提供：**
  - 經濟／商店狀態加 `StashCatalogIds[6]`；若 stash 道具有充能或冷卻，也要比照 `ItemSlots` 送。
  - `in_shop_range` 布林值。
  - 出售指令能指定來源是背包或 stash，例如 sale 加上 container 欄位。
  - 購買結果能分辨道具進了 stash，UE 顯示「已放入儲藏處」；`StashFull` 拒絕要有中文訊息。
- UE 不擋範圍外購買（沒有任何 range 判斷），所以規則改變不需要改購買路徑。

## 4. 血條與小地圖

- 血條：`OmWorldBridgeActor.cpp:3852` 對 Hero 到 Summon 的 kind 都有血條，信使用 EntitySummon 就會有，不用改。
- 小地圖：`SOmMinimap.cpp:455` 只分英雄（半徑 4）和其他（半徑 2.5），frame 沒有「信使」標記。
  - 打算用 frame entity 的 `catalog_id` 對照信使的 presentation catalog id（codegen 產生常數），在小地圖畫一個專用的小馬蹄形或菱形標記，用主人的隊伍色。
  - 不需要後端改 frame，只需要信使有固定的 catalog id。
- 選取信使時，頭像和名稱由現有選取面板顯示（名稱「馬」來自模板）。

## 5. 等待後端的項目

| 需要 | 來源 |
|---|---|
| 信使單位模板 id／catalog id、frame kind | E |
| 送貨／回家的形式（建議做成信使技能） | E／B |
| 不可釋放的標記 | B |
| stash 在經濟狀態的欄位名稱、`in_shop_range` | E（型別）、B（填值） |
| stash 出售的指令欄位、購買進 stash 的結果碼 | B |

收到型別後的順序：先做模型與 world_units 列，再做 stash 格子，然後是選取按鈕（若做成技能就幾乎不用改），最後是小地圖標記。每段都走 land_to_live 加 UE automation 測試，最後錄一段實機：買進 stash、按送貨、信使跑到英雄身邊交付。
