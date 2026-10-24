# 偵隱飾品與共用放置說明：最終驗收

`complete-detection-trinket` 完成4/4 tasks。原創偵隱燈以append-only catalog2加入共用資料：放置距離600、視野600、偵測2、可見本體stealth0、HP4、生命期60秒、180秒回復1電荷、最多放置1個。保留catalog1偵察燈所有舊值。沿用權威loadout、admitted owner queue、正式放置／視野／時間與退休機制；請求不能自行聲明catalog。沒有英雄／營地名稱特判、C ABI或wire layout變更。

Generated native catalog現在攜帶作者說明、放置距離、視野與生命期。共用BuildWardCatalogDetails供既有成功owner view的ward tooltip使用，動態電荷／回復與結果仍來自原投影；未知／未裝備catalog沒有虛構說明，單純catalog存在不會啟用放置權限。Generated bridge透過既有owner HUD view傳遞目錄資料，Runtime不反向依賴OmGenerated。數字格式保留非整數。沒有新增對局內切換飾品UI或專用圖示；本批為可由既有權威loadout選用的原創內容，不宣稱完整LOL控制守衛規則。

## 集中結果

1799passed/0failed/9existingignored；3429正式輸入一致；246份證據／31PNG；health12 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|958|0|
|base_content|312|0|
|bridge|136|1|
|client|151|8|

新增template／codegen測試驗證穩定ID、tradeoffs與數值／文字轉義。實際ECS測試驗證偵測1/2揭露、3不揭露、精確圓邊界與超出一raw unit、地形遮蔽、敵我隔離、可見本體與摧毀後即時移除偵測。另一測試走真admitted WardPlacement queue和Production60Hz driver，確認恰好消耗一電荷生成一body/source、settled receipt和owner私有catalog2投影；pause不推進生命期／回復，原生game-time推進到expiry前一raw unit仍存在，正式driver跨越deadline後退休，recharge不復活body。60/180秒之間使用既有native時間入口快進，不宣稱逐frame長局或真網路驗證。

正式backend、UE5.8 build／stage／import成功；World69全執行、0 Failed/NotRun，既有RuntimePresentationReset警告保留。Om.Runtime.DetectionTrinket驗證generated identity、shared detail中文文字、未知catalog與未綁定widget權限，真正初始化UObject及Slate並保存共享tooltip畫面。這是正式共用formatter的實際Slate驗收，不是假裝已完成live network loadout選擇。compiled-map公共ABI export成功。

31PNG：26張與前批完全一致，5張逐張實際view，含新detection-trinket/detection-trinket.png。原有first-kill HUD下／右裁切與部分hero/focus名牌重疊仍為已知限制。

## 證據與範圍

失敗版本原始logs、每次Rust修正前9個來源、UE修正前14個來源與SHA manifest全部保留；最終完整Rust revision重新驗收。所有owned reader/watchers與單一170秒health timer退休reap，來源exact set/SHA/bytes/mtime與封存bytes/SHA獨立驗證成功。[證據索引](D:/omoba/docs/acceptance/2026-10-23-detection-trinket/index.json)；[錯誤紀錄](D:/omoba/docs/plans/2026-10-23-detection-trinket-errors.md)。

11外部父項、MVP33/44、完整combat replica與持續引導保持pending。無真網路對局／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。
