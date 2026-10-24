# 2026-10-22 外塔塔皮最終驗收

OpenSpec complete-moba-tower-plating **5/5完成**。接續17系統完整度盤點，補上單機可驗證的塔皮玩法與防禦單位邊界修正。

## 實作

SingleLaneConfig新增optional TowerPlatingRules，預設None保留舊fixture；正式server與role plan均啟用同一training設定：5層、840秒、每層125附近金、1200半徑、每破層+20護甲。plates1..16、expiry1..3600、gold<=1m、radius1..10000、armor<=1000皆於setup原子驗證。數值為本專案平衡，不宣稱對應特定LOL patch。

權威只接受tracked外層lane塔，依entity generation保存max HP與已破層數。i128固定點門檻可一次跨多層；治療、重複Outcome不會重領。匿名、同隊或無附近英雄的已跨門檻仍消耗，不能晚點追領。沿既有英雄／召喚／NPC隊伍歸因，僅附近存活進攻英雄逐層均分捨餘，不因合併傷害多拿餘數；不發額外KDA或XP，與既有拆塔／首塔金獨立。

加固沿正常BuffStore，使用有限剩餘時間與pending tick補償，不改基礎護甲；下次實際傷害讀到已破層*20護甲。到期、HP0、強制退役、結束清除，ledger與規則加入digest。金額沿既有owner economy同tick送出，Buff沿ABI33 snapshot與生成Unreal catalog。專用塔血條分段與美術仍未新增。

檢查防禦路徑同時修正前輪魔抗單位接線：DamageInstance百分點與UnitStats normalized比例明確往返，保留25%基礎魔抗，20%額外魔抗組合後約40%；100魔傷實際約60。固定點結算與既有公式／旗標保留。

## 集中驗收

RustV1 **1768通過／10既有ignored／0失敗**：template72、core891、codegen57、base263、backend190／1、bridge130／1、client150／8、ABI15。含generation九工作exit0且退休。

新增core四個塔皮案例涵蓋exact HP門檻、逐層／跨多層分金一致、半徑包含／超界、匿名／同隊消耗、內塔排除、治療／重複致死、護甲聚合、pause／expiry、強制退役／結束、原子無entity副作用及digest。另新增魔抗百分點回歸。既有server與role plan測試確認正式啟用設定。

新增正式60Hz案例經generated lumen_bolt實際施放破層，驗證當tick owner economy含正確塔皮金、公開tower_plating payload層數與有限倒數；下一次100物傷實際按當前加固減傷一次，840秒後不再給金且清Buff；無額外擊殺／助攻。既有全map／雙隊digest／死亡重生／結束凍結與其他玩法一併回歸。

UE5.8 build-v1成功（UBT **45.18秒**）、正式後端與Lua三契約成功。WorldV1 **37成功**（36 Success＋1 SuccessWithWarnings，0 Failed／NotRun），4既有preview-world清理警告保留。兩map各74影格／6片段，兩map與既有首殺資訊列三張原生PNG已目視，屬既有場景與HUD回歸，不當成塔皮專用美術接受。

**3357來源／資產**接受前後集合、SHA／size／mtime一致；七項staging與import library另驗。**44份證據**逐byte／SHA封存並獨立複核；三分鐘監測 **8次／最大181秒**。7 worker／watcher與全程monitor均退出。

## 剩餘與界線

本輪沒有建置／測試失敗。P01..P04紀錄分金、退休、路徑查詢與前輪魔抗接線修正，總登錄E1473。完整MVP33/44與11項外部ACK／LAN／OS DPI／效能待驗維持；沒有新增網路對局、重設配額、commit／push或omfx維護。沒有把前輪native／IPC／Blueprint／PCM數字算成本輪接受。

後續仍需塔血條分段與美術、野怪分裂、經濟補償、更多目標公告、技能引導／特殊幾何與動畫、召喚複本生命期。這批完成不代表整個遊戲已達LOL全部規則與內容。

證據索引：../acceptance/2026-10-22-tower-plating/index.json。
