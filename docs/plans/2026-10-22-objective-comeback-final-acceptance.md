# 2026-10-22 目標經濟補償最終驗收

OpenSpec complete-moba-objective-comeback **5/5完成**。依整體17系統盤點補上全隊經濟補償，沿正式玩法實作並完成單機接受。

## 實作

共用 ObjectiveComebackRules 由 Lua 提供，包含開始時間、啟用與退出經濟差、塔與史詩每人額外金、每次啟用總池，整數有界／未知欄位拒絕。全零保留舊設定停用；規則進內容hash與compiled catalog／hot reload重建守門，正式server／role match設定接入。core明確依賴既有本機內容模型，compiled-only也可使用同一規則，不更動script ABI。

正式訓練180秒後，累計實得差1500啟用、750退出；敵方lane tower每人100、具目標獎勵的史詩根怪每人125，每次池1000。這是專案訓練平衡，不宣稱特定LOL版本。

隊伍收益使用u64飽和ledger；擊殺／助攻／尾刀／野怪／建築／首塔／首殺／塔皮／被動金統一經credit_moba_player，一般GainGold正向實付也記錄。購買／賣出／退款不改累計，死亡保留，飽和錢包只計實付，XP0避免無效成長工作。已認證死亡錢包可取得補償；清空entity但無合法respawn身分不得領取。

資格於tick開始依上一批提交收益鎖定；兩道門檻避免抖動，池耗盡但差距仍大時不自動補滿。塔／史詩致死路徑只記一次claim，tick結束依tick／完整entity生命身分排序，等分且餘數捨去／無足額平均份額則收池。保守扣除已提交份額，即使錢包飽和也不讓同批後續claim越過總額上限。普通野怪、匿名、友軍與強制退休不製造補償；pause不推進。終局清除資格與待發claim。

公開native Buff objective_comeback追加於目錄末尾，既有ID不位移；正式快照／既有ABI與Unreal owned list顯示「目標補償啟用（持續生效）」，不帶戰鬥屬性，復活重新同步，退出／池耗盡／死亡／終局按生命期移除。完整ledger／規則／claim納入既有replay digest。

## 集中驗收

Rust最終 **1788通過／10既有ignored／0失敗**；template75、core902、codegen58、base266、backend190／1、bridge132／1、client150／8、ABI15，含generation九工作均成功退休。新增內容案例涵蓋界線／型別／未知欄位／legacy停用／hash與重建。核心涵蓋一般GainGold、spending不影響、飽和、死亡／重生、時間／雙門檻／pause／耗盡不重開／終局、逆序多目標／重複claim／死者平分、實際tower與epic致死與普通／匿名／強制退休排除。

正式60Hz使用generated單體技能擊殺英雄形成300實得差，清現金不改資格，遊戲時間後啟用敵隊補償，實際重生與永久public snapshot、generated技能摧塔取得100額外金、pause凍結與終局清Buff。整場雙map／雙隊filtered lifecycle及既有技能／塔／野怪／兵線一併回歸。

UE5.8 build-v1與正式後端build-v1成功，Lua三契約成功。WorldV1 **41成功**，0 Failed／NotRun；新增ObjectiveComebackStatus驗證generated目錄與原生owned list顯示、owner隔離、舊generation拒絕與完整缺項退休。既有警告保留。兩map各74影格／6片段，兩map、首殺、塔皮與護盾五張原生圖片均目視，屬回歸；不宣稱新增補償專用真連線畫面。

V1因core缺直接內容模型依賴而編譯失敗；九工作退出後修正。V2 core全通過、base新fixture誤將英雄TAttack helper套塔而失敗；全批與world均退出後，只修該Rust測試，再完整重跑Rust，正式生產碼、UE與後端產物保留。失敗原始log與receipt全部封存，不加總不同批通過數。

**3367來源／資產**接受前後集合、SHA／size／mtime一致，七項DLL staging與import library另驗。**88份證據**逐byte／SHA封存並獨立複核，三分鐘監測 **11次／最大180秒**；7最終worker／watcher與先前失敗cohort、全程monitor均退出。

## 剩餘

錯誤紀錄：2026-10-22-objective-comeback-errors.md，總登錄E1478。仍有更多目標公告／平衡資料、持續引導／特殊技能幾何與動畫、召喚完整複本生命期、建築／場景美術與外部ACK／LAN／OS DPI／效能接受。原MVP33/44與11外部缺證不變。沒有新增真網路對局、重設配額、commit／push或omfx維護。

證據索引：../acceptance/2026-10-22-objective-comeback/index.json。
