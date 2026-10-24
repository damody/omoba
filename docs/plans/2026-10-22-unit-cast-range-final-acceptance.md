# 2026-10-22 指定單位技能射程提示最終驗收

OpenSpec complete-moba-unit-cast-range **5/5完成**。接續全遊戲盤點，補上單機可驗證的指定單位技能射程與當前公開資料生命周期。

## 實作

Bridge新增只讀TryGetPublishedCastUnitPositions，以明確player、source hero與source／target完整generation，從同一份目前PublishedDirectTargets讀取存活可見中心。owner不一致、來源死亡／替換、目標隱藏／死亡／退休／新generation、無效座標皆拒絕，輸出歸零。沒有Actor掃描或last-known推測。

Owner僅保存目前marker的Entity，初始化與pointer move走共用SetCastMarkerSample，支援world與disclosed card；普通UI與退休清空。繪製時經嚴格反射契約即時讀公開中心，依2D距離顯示射程內外、數字與顏色，目標marker使用單位中心而非cursor。沒有有效目標時距離未知，仍保留原有施法者射程圈；未知range、未學習／冷卻／魔力提示優先。點地技能既有落點／半徑維持。通用於已有entity預覽的技能、道具與召喚師，不改命令提交或權威admission，不把提示當ACK。

OmRuntime保持不依賴OmGenerated，不建立全場單位座標cache，也不新增每entity每幀操作。Bridge API與新enum只在Unreal內部，未變更CABI34／protobuf。

## 建置完整性

V1 build成功但UBT來源快取未收集新增.cpp，World案例完整性守門發現只有45項而拒絕。正式Lua ordinary／bootstrap與既有restart旗標資料加入UE5.8原始碼明確支援的-NoUBTMakefiles，讓每次建置發現來源集合；保留NoEngineChanges，不刪cache、不改引擎或全域設定。Lua規劃契約與restart測試驗兩種入口一致。

BuildV2真的編譯新測試後，抓到fixture讀protected縮放及錯用seq欄位；等所有消費者退出後，僅修測試CPP為UPROPERTY反射與CABI實際sequence。BuildV5成功，WorldV4實際找到46項。WorldV2／V3進一步定位：direct公開查詢成功，只有未初始化Actor的測試World略過ProcessEvent。沿UE5.8 Actor.cpp與UObject/Script.h，fixture使用FEditorScriptExecutionGuard限定作用域，保持正式查詢守門；WorldV4真反射與完整生命周期成功。來源guard新增restart crate，RustV2所有guarded Rust／內容輸入mtime均早於該owned工作起點；後續僅改原生fixture，因此保留已通過的RustV2，不混加數字。V1不完整report／log與BuildV2錯誤均封存。

## 集中驗收

RustV2 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功退出。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v5成功（UBT **5.67秒**）。

WorldV4 **46成功**、0 Failed／NotRun，4既有清理warnings保留。新增實際World公開frame→Bridge→owner反射→preview測試，驗source／target中心與cursor分離、舊本機hero座標不影響、精確射程邊界、移動即時超距、未知range、卡片、錯owner／source generation、目標隱藏／死亡／新generation、source死亡／非hero、目標缺席、UI清空與owner退休，並保留未學習／冷卻／魔力、legacy candidate與NaN range守門。

既有45項地圖、召喚動畫、技能提示、HUD與資源生命周期一起回歸。兩map各74影格／6片段；7張既有PNG已目視，作消費端回歸，沒有冒稱新增射程專用截圖或真連線ACK。

**3378來源／資產**最終接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**150份證據**逐byte／SHA封存並獨立複核。健康監測 **12次**、最大間隔 **171秒**，7最終及11先前worker／watcher與全程monitor皆退出。

## 剩餘

持續引導／特殊技能幾何、完整combat replica、退休來源callback完整上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術與音效仍待擴充。MVP33/44、11項外部ACK／LAN／實際OS DPI／固定效能條件維持pending，本機射程提示不替代權威接受。

錯誤與決策見2026-10-22-unit-cast-range-errors.md，總登錄E1484。零新增真網路對局／quota reset／commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-unit-cast-range/index.json。
