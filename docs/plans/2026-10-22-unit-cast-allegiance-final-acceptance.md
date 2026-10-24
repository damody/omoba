# 2026-10-22 指定單位技能陣營提示最終驗收

OpenSpec complete-moba-unit-cast-allegiance **5/5完成**。接續全遊戲盤點，補上單機可驗證的QWER指定單位技能即時條件提示。

## 實作

Bridge新增只讀TryEvaluatePublishedCastTarget，沿目前玩家、source hero、source／target完整generation及公開存活中心守門，重用正式命令提交的IsPublishedLivingAlly／IsPublishedLegalHostile。未符合敵友需求顯示「目標陣營不符」，敵方缺少目前可攻擊條件顯示「目標目前不可施放」，條件未知顯示「目標條件尚未確認」。非敵對不直接等於友軍；中立關係以正式現有規則判定。未知metadata不猜測。

Owner經嚴格六參數反射契約讀目前公開表；目標變更陣營、失去可攻擊條件不需移動游標即可更新。目標死亡／隱藏／generation替換、source失效／owner退休即失效。未學習、冷卻、魔力、缺少目標保留優先，已通過條件繼續顯示當前中心射程內外。OmRuntime不依賴OmGenerated，不掃Actor、不建立全場cache、不新增每entity每幀操作。不改命令提交、CABI34或protobuf，也不把預覽當作ACK。未具陣營metadata的道具／召喚師維持既有射程提示，不聲稱本批新增其陣營檢查。

## 集中驗收

RustV1 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **58.12秒**）。正式-NoUBTMakefiles入口確實納入新增CPP。

WorldV1 **47成功**、0 Failed／NotRun。新增真實World公開frame→Bridge→owner反射→feedback測試：敵友需求、同目標關係即時改變、自用、中立不推測、敵方失去可攻擊條件、未知需求、owner不符、目標死亡／隱藏／新generation、source死亡與owner退休，以及提示優先序與legacy行為。測試World限定FEditorScriptExecutionGuard，不改正式生命周期。

既有46項地圖／召喚／技能／HUD／資源回歸一起通過；4既有清理warnings保留。兩張地圖各74影格／6片段，7張既有PNG已目視，屬消費端回歸，沒有新增功能專用截圖或真連線ACK宣稱。

**3379來源／資產**在接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**75份證據**逐byte／SHA封存並獨立複核。健康監測 **6次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多原創英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能條件維持pending。本批沒有宣稱全遊戲已與LOL一致。

錯誤MD：2026-10-22-unit-cast-allegiance-errors.md；總登錄E1485。Saved helper初版遮蔽Lua pairs，於啟動驗收前改名修復；未改正式工作流。零新增真網路對局、quota reset、commit／push，omfx未維護。

證據索引：../acceptance/2026-10-22-unit-cast-allegiance/index.json。
