# 2026-10-22 選取目標戰鬥資訊卡最終驗收

OpenSpec complete-moba-selected-unit-combat-details **5/5完成**。接續全遊戲盤點，選取目標資訊卡整合目前公開護盾、控制效果、塔皮與野怪返回狀態。

## 實作

FOmSelectedUnitStatePayload新增typed Shield／CrowdControl／HealthSegments／JungleReturnMode，從同一公開frame既有摘要取得。沿既有player、source／target generation、living／visibility與runtime scope守門；不可受傷的公開單位仍可inspect，選取不等同attack admission。不新增私人stats／敵方技能冷卻查詢，不改後端、內容catalog、CABI34或protobuf。

資訊卡與名牌共用OmShieldLabel、OmCrowdControlLabel、OmHealthSegmentsLabel、OmJungleReturnLabel，維持既有文字、bounded標籤／數字與倒數。非法子狀態獨立省略，不污染HP主卡。HP不變但護盾／控制／塔皮／返回或顯示秒數改變會發布HUD；同顯示秒數保留節流，但Bridge selected getter仍保存最新raw公開值。HUD自身的顯示snapshot只需保持正確文字，不宣稱每fractional秒發布。

UOmPlayerHudOwnerComponent維持原有source／epoch／player檢查並傳遞完整typed payload。取消selection、target死亡／隱藏／generation替換、source conflict及runtime／HUD退休仍清理整張卡。不新增全場cache或每entity每frame Slate更新，OmRuntime不依賴OmGenerated。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **68.51秒**）。

WorldV1 **51成功**、0 Failed／NotRun，保留4個既有清理warnings。新增真公開frame→Bridge selection→實際native HUD測試，驗四項資料、名牌格式一致、malformed子狀態、固定HP時獨立更新盾／控制／返回／塔皮、同秒raw快照、完整缺席、取消選取／重選、世代／source conflict／隱藏／死亡與HUD退休。OmSelectedUnitCard實際Slate截圖已目視，四項狀態及HP完整可讀；原有控制／返回名牌與七張消費端PNG保留。兩張地圖各74影格／6片段；本機synthetic呈現不當作真網路ACK。

**3387來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**78份證據**逐byte／SHA封存並獨立複核，10張PNG已目視。健康監測 **7次**、最大間隔 **171秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與錯誤

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策：2026-10-22-selected-combat-errors.md，總登錄E1489。P01猜測選取測試檔名造成讀取不存在路徑，改用實際測試ID與rg定位；後續建置／測試無新增失敗。所有圖片等World receipt成功及worker／watcher退休後才目視；未放寬正式守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-selected-combat/index.json。
