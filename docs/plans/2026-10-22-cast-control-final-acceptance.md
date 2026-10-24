# 2026-10-22 控制效果施法提示最終驗收

OpenSpec complete-moba-cast-control-feedback **5/5完成**。接續全遊戲盤點，補上目前公開控制效果對QWER預覽的原因與時間提示。

## 實作

Bridge新增TryGetPublishedCastControl，從同一份完整公開frame有界掃描最多4096項Buff，僅保存目前source摘要。依typed catalog識別stun／silence；root僅禁移動，不誤當技能阻擋。暈眩優先沉默，同類取最長公開剩餘時間，永久效果不顯示負數倒數。不使用UI64項截斷清單，不掃Actor、不建立全場cache；查詢O(1)。

查詢沿正式公開source中心的玩家、hero、存活與完整generation守門。替換／清空／revoke／退休即失效；非完整presentation、未知metadata、malformed當前控制資料保持未知。owner嚴格五參數反射讀取，point／unit的QWER預覽顯示「暈眩中，無法施放技能」或「沉默中，無法施放技能」與當前公開剩餘秒數；未知顯示「施法控制狀態尚未確認」。未開啟預覽的instant self不宣稱新增浮層。

未學習／冷卻／魔力保留優先；無控制阻擋時繼續既有目標條件與射程提示。道具／召喚師未硬套技能控制規則。沒有使用牆鐘推估倒數，也沒有改命令提交、後端裁決、CABI34或protobuf；提示不代表ACK或未來釋放必然成功。OmRuntime保持不依賴OmGenerated。

## 集中驗收

RustV1 **1818通過／10既有ignored／0失敗**：template76、core912、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **54.42秒**）；新CPP確實編譯。

WorldV1 **48成功**、0 Failed／NotRun，保留4個清理warnings。新增真World frame→Bridge→owner反射→feedback測試驗空快照、root、silence、stun優先、同類最長時間、永久效果、第66項控制、他generation、無效時間／instance／metadata、快照替換移除、錯owner、死亡／source revoke／退休／空presentation、可讀標籤與原有提示優先序。fixture的FEditorScriptExecutionGuard限定測試scope，沒有放寬正式生命周期。

既有47項技能、地圖、召喚、HUD與資源回歸一起通過。兩張地圖各74影格／6片段，七張既有PNG已目視，僅作消費端回歸，不冒稱本功能專用截圖或網路ACK。

**3380來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**75份證據**逐byte／SHA封存並獨立複核。健康監測 **5次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾，以及更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，沒有宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-cast-control-errors.md，總登錄E1486。P01 Windows路徑glob、P02宣告入口與BuffIdName未知ID fallback、P03 Saved封存helper改名路徑均已修正。零新增真網路對局、quota reset、commit／push；omfx未維護。

證據索引：../acceptance/2026-10-22-cast-control/index.json。
