# 2026-10-22 指定單位技能射程提示盤點

本輪接續17系統完整度盤點，補上單機能驗證的技能操作提示。既有point落點／半徑、內容施法前搖、受傷中斷、冷卻／魔力提示保留。

## 缺口與實作

原有entity技能只顯示「已指向單位」，FOmCastPreviewFeedback刻意不把ground cursor當單位中心，因此沒有單位射程距離。現在Bridge提供只讀TryGetPublishedCastUnitPositions：精確驗證owner、當前source hero、source／target完整generation、公開relation／alive與有限座標，從PublishedDirectTargets讀取同一份當前公開中心；拒絕時兩個輸出歸零。無Actor掃描，也不讀隱藏或last-known位置。

Owner追蹤目前marker的完整Entity，不保存全場單位座標。初始化與pointer move走同一SetCastMarkerSample，支持world與disclosed card；普通UI清空。繪製時經嚴格反射參數契約讀當前source／target中心，計算2D距離與射程內／外、數字、顏色與目標marker。隱藏、死亡、退休、新generation、source失效或owner錯配都不能繼承舊位置。未知range保持「射程未知」，未學習／冷卻／魔力提示優先。地點技能維持既有ground point規則。

此機制通用於已開啟entity預覽的QWER、召喚師與道具，不改輸入提交、command ID、權威admission或ACK，沒有英雄名字特判。OmRuntime保持不依賴OmGenerated；每次有效預覽只查source／target兩筆map，沒有新增每entity UI更新。

## 接受設計

新增真World／Bridge synthetic公開frame→owner實際反射查詢→preview計算的原生測試：不同cursor與target center、stale本機hero位置、精確邊界、目標移動、unknown range、card、owner／source generation、隱藏／死亡／新generation、來源死亡／種類替換、目標缺席、UI清空與owner退休；保留legacy candidate、未學習／cooldown／mana與非有限range守門。

Rust與完整原生46項集中回歸；七張既有map／角色／HUD圖是消費端回歸，不冒稱真網路技能接受或新增射程專用畫面。最終來源／staging、程序退休與證據核對另見unit-cast-range-final-acceptance.md。

## 後續

完整combat replica、退休來源callback完整上下文、召喚整組commit原子回滾、持續引導／特殊技能幾何，以及更多原創內容／美術音效仍待實作。MVP33/44與11項外部ACK／LAN／DPI／效能缺口不因本機提示勾完。

建置完整性補強：本輪發現UBT快取會漏掉新增.cpp，因此正式Lua ordinary／bootstrap及既有restart旗標資料加入-NoUBTMakefiles，保持NoEngineChanges與每次資源限制，不刪cache或改引擎／全域設定。新增Lua規劃契約及restart九測試，最終來源guard也納入restart crate。保留未納入新測試的WorldV1，以及發現新測試編譯錯誤的BuildV2。
