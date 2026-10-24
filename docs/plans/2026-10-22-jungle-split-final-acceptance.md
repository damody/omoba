# 2026-10-22 野怪死亡分裂最終驗收

OpenSpec complete-moba-jungle-splitting **5/5完成**。依17系統完整度盤點，新增單機可驗證的野怪死亡分裂；正式維護範圍維持omb／共用Rust／scripts／omfue。

## 實作

共用Lua catalog的主體與初始成員皆可指定death_children。最多兩代分裂、每body四子體、每營生命期總數32；遞迴驗證HP／傷害／獎勵／ID／offset、累積home與root leash、座標界線及地形碰撞。規則與後代內容均參與catalog hash，舊內容省略欄位維持空子體；無營地名稱判斷的runtime分裂規則。

致死ledger提交後，退役body僅排入一次子體定義與屍體附近的碰撞安全位置，mutable營地tick建立原生Unit／Faction／Pos／CProperty。子體保留原營群體仇恨、patience／soft／hard回頭及攻擊／回血規則；子體自己的HP／攻擊／gold／XP由內容指定，不繼承目標／個人Buff。強制退役與終局不產生子體。

有待生或存活後代時不啟動重生倒數；最後子體退休才開始deadline。整營重生重新建立初始人口，清除舊後代。pending定義與位置進既有deterministic digest。兩張map的blue／red石怪營皆啟用兩代分裂：2初始→4→4，共10個生命期個體，每營總124金／193XP；此為原創訓練平衡。

## 集中驗收

V1因模組helper引用區域valid_id而編譯失敗；整批consumer／watcher退休後提升共用函式。V2 UE與後端成功，Rust新增移動fixture缺ObservableFactBuffer而1失敗；等整批退休後補正式資源，再跑V3。原始失敗與先前成功紀錄皆封存，不加總多批通過數。

最終RustV3 **1774通過／10既有ignored／0失敗**：template73、core894、codegen57、base264、backend190／1、bridge131／1、client150／8、ABI15；包含generation九工作exit0且退休。

新增內容測試驗證樹深／分支／總人口／累積home／非法後代／deep hash／舊欄位預設。新增core測試驗證待生queue／重複死亡／各代數量／最後清空／deadline／初始人口重建、強制與終局退役、屍體附近位置、Buff不繼承、原營仇恨與hard回頭。新增正式60Hz測試對兩map雙方營地實際Damage／Death、重複致死、每body錢包／XP、公開entity snapshot、全清重生與相同seed digest重跑；既有整場雙隊filtered lifecycle等一併回歸。

UE5.8 build-v3成功（UBT **3.21秒**），正式後端v3與Lua三契約成功。WorldV1 **38成功**（37 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有測試警告保留。兩map各74影格／6片段，兩map與既有首殺／塔皮名牌四張原生PNG已目視，屬回歸；本輪不宣稱取得分裂專用實際對局畫面。

**3360來源／資產**接受前後集合、SHA／size／mtime一致；七項staging及import library另驗。**93份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **10次／最大181秒**。7個最終worker／watcher、6個先前worker／watcher與全程monitor均退出。

## 剩餘與界線

錯誤與預防記於2026-10-22-jungle-split-errors.md，總登錄E1475。仍需經濟補償、更多目標公告、持續引導／特殊技能幾何與動畫、召喚複本生命期與建築／場景美術。原MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗不變；沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-jungle-split/index.json。
