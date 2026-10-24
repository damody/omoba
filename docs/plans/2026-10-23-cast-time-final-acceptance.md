# 施法準備時間屬性接線：最終驗收

`complete-cast-time-modifiers` 完成4/4 tasks。共用CastTimePercentage（作者key為casttime_percentage）原已存在，但正式preparation只讀基礎cast_time。現在UnitStats::checked_cast_time與正式dispatch共用接線：base0維持instant；正base限定0..60秒，checked_sum_add沿用family strongest政策、檢查1+bonus、既有102raw乘數下限、i128乘法／1024，正duration最少1raw、最多60秒。非法／不可表示聚合或超出時長拒絕開始，不把overflow變成加速。既有f64 legacy stat輸入仍由共享checked aggregator處理。

開始時鎖定duration為該instance的total/remaining；之後buff改變或失效不重新計時，也不在release再次套乘數。原有pause／移動／stop／控制／死亡中斷與實際釋放資源交易保持。Mana／cooldown／召喚只在成功release結算；拒絕不產生技能效果。本批沒有新增英雄／營地特判、wire／C ABI、新內容ID或持續引導脈衝，持續引導仍待實作。

Unreal繼續消費權威effective total/remaining，新增同instance總時長不可漂移守門，正值剩餘時間小於0.1秒顯示「施法 <0.1s」，避免被四捨五入成0.0s。Tooltip明示作者數值為「基礎施法前搖」，實際準備時間依開始施法時的屬性決定。共用HUD密度重排原會清空cast進度／文字列、只重建快捷鍵與CD；現在保留獨立cast row並於每種模式重新掛接，文字納入可讀字體、逐字換行、量測fit與內容失效，避免消失或裁切；真技能readout增加穩定tag供節點驗收。

## 集中結果

1803passed/0failed/9existingignored；World71全執行；3433正式輸入一致；260份證據／34PNG；health16 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|960|0|
|base_content|314|0|
|bridge|136|1|
|client|151|8|

新增2core測試驗證無modifier、haste／slow、102raw下限、正base一raw下限、fractional floor、family strongest、legacy float、非法型別／聚合與加法overflow／60秒超限、instant不被modifier改變。新增2base正式60Hz測試，對既有準備技能走真PlayerInput：-50%、+100%、過強haste、無modifier得到256／1024／51／512raw；公開preparation total/remaining等於實際instance，途中改成非法modifier與pause仍保持已鎖定總時間；釋放前零召喚／零ManaCD，跨deadline生成2召喚並扣90Mana、啟動cooldown恰好一次；非法初值拒絕，instant自療仍立即成功。這些角色名稱只出現在測試fixture，執行路徑沒有名稱分支。

完整Rust、正式backend與UE5.8 build／stage／import通過；World71全執行、0Failed/NotRun，保留既有4個RuntimePresentationReset警告。新增Om.Runtime.CastTimeModifiers從真正owned game world/player viewport測試effective比例、同instance總時長漂移拒絕、newinstance／subdecimal提示與退休；800x600/1280x720、TextScale1/2確實執行NativeTick響應fit後，cast進度與文字兩個children仍在、文字desiredwidth/height在arrangedbounds內。保存未修改的1280x720原始viewport PNG，再依實際paint geometry拷貝技能cast row原像素矩形、保存capture.json座標並檢查字形像素，完整畫面與局部PNG均實際檢視成功。這些是viewport／UI文字縮放，不宣稱OS DPI已切換。公共ABI compiled-map export另行執行成功。

34PNG：28張與前批完全相同，6張逐張實際view，包含newcast-time/cast-time.png及cast-time-frame.png。新文字逐字換行後完整保留所有字形；既有first-kill下／右HUD裁切與部分map hero/focus名牌重疊保持known，不算本批已修。

## 證據與範圍

原始錯誤／根因／預防見errors MD。完整Rust讀取程序退休後，封存兩份C++修正前來源，再補上基礎前搖說明與native斷言。UE／WorldV1測試成功，但實際PNG文字靠右裁切，繼而查出density清空cast row；這是視覺拒收，未改稱測試失敗。所有V1程序退休後保存6份pre-readout來源、原始visual JSON／logs／report／PNG，再補persistent row、fit與真正owned viewport測試。V2實際fit與71測試通過，但局部TakeScreenshot的安排座標未對應本fixture的paint座標，字形像素證據仍拒收；再次封存6份pre-capture來源與全部V2報告／PNG，V3只改native捕捉方式為完整viewport原像素＋paint矩形，不變更Runtime／Rust。V1/V2視覺JSON不覆寫，V3獨立輸出。最終UE／WorldV3集中接受，所有owned reader/watchers與170秒health timer退休reap，正式來源exactset/SHA/bytes/mtime與封存bytes/SHA獨立驗證。[證據索引](D:/omoba/docs/acceptance/2026-10-23-cast-time/index.json)；[錯誤紀錄](D:/omoba/docs/plans/2026-10-23-cast-time-errors.md)。

11外部父項、MVP33/44、完整combat replica及持續引導仍pending。無真網路對局／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。
