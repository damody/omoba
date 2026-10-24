# 2026-10-22 可見單位控制效果名牌最終驗收

OpenSpec complete-moba-crowd-control-nameplates **5/5完成**。接續全遊戲盤點，補可見單位的暈眩／定身／沉默名牌。

## 實作

新增FOmCrowdControlState，三種控制各自保存公開剩餘時間。同類多來源取最長，永久優先；主效果優先暈眩、定身、沉默，只用該效果的時間，並標示其他並存效果，避免把長沉默誤當暈眩持續。永久不顯示負數倒數；極大有限時間呈現9999+秒，避免整數轉換溢位。沒有wall-clock推算。

Bridge從最多4096項公開Buff與100000 entity rows建摘要，使用typed catalog canonical ID，不依賴動畫、顯示名稱或UI64項截斷清單。非法duration、重複控制instance、重複entity、死亡、缺席／錯generation皆拒絕；未知Buff不憑空產生控制，沒有公開entity時Buff本身不能揭露狀態。未顯示控制不代表保證可操作。

接線為公開frame→Bridge→typed frame payload→Actor→名牌，沿health偏好與文字縮放；只在主要效果顯示秒數、組合或偏好改變時重畫，次要純倒數不引起Slate寫入。死亡、reset/rebind清空，hide清cache、reveal不可重播，fresh authority才恢復；control-only保留，完整replacement缺席清理。不改後端控制規則、內容catalog、CABI34／protobuf或命令裁決，OmRuntime不依賴OmGenerated。

## 集中驗收

RustV1 **1820通過／10既有ignored／0失敗**：template76、core914、codegen60、base270、backend190／1、bridge135／1、client151／8、ABI15、restart9；含generation十工作成功。四Lua契約成功，正式後端build-v1成功，UE5.8 build-v1成功（UBT **101.66秒**）。

WorldV1 **50成功**、0 Failed／NotRun，保留4個既有清理warnings。新增測試涵蓋主要時間與並存效果、root／silence區分、永久／極大時間、NaN／無限／非法負值、同類多來源、重複instance／entity、錯generation、死亡／不可見、輸入上限、名牌偏好／節流／顯示秒數邊界，以及實際Frame→Bridge→Actor→Slate、control-only／完整replacement、hide/reveal與退休。新控制名牌實際Slate PNG已目視；既有返回名牌及七張消費端PNG保留。兩張地圖各74影格／6片段，本機synthetic呈現不當作真網路ACK。

**3386來源／資產**接受前後集合、SHA／size／mtime一致；七DLL與import library另驗。**77份證據**逐byte／SHA封存並獨立複核，9張PNG已目視。健康監測 **6次**、最大間隔 **170秒**，7組worker／watcher及全程monitor皆退出。

## 剩餘與決策

持續引導／特殊幾何、完整combat replica、退休來源callback上下文、召喚整組commit原子回滾、更多英雄／道具／召喚師、美術音效仍待擴充。MVP33/44，11項外部ACK／LAN／實際OS DPI／固定效能接受維持pending，未宣稱全遊戲已與LOL一致。

錯誤與決策記錄：2026-10-22-crowd-control-errors.md；總登錄E1488。P01曾過早讀取尚未完成的地圖截圖，改為等World退休後再目視；本批防錯聚焦獨立控制時間、永久sentinel、顯示整數上限、完整公開清單與生命周期清理。驗收過程無新增建置／測試失敗，不重複前輪路徑glob／改名失誤，也沒有放寬守門。

零新增真網路對局、quota reset、commit／push，omfx未維護。證據索引：../acceptance/2026-10-22-crowd-control/index.json。
