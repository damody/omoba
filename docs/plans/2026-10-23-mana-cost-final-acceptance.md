# 魔力消耗modifier：集中驗收

complete-mana-cost-modifiers完成4/4。共用checked_mana_cost_mult使用既有manacost_percentage與checked_sum_add family strongest聚合，checked one-plus後將合法負值clamp到zero；非法型別／聚合或加法overflow明確None，不會被當免費施法。正式managed admission和role bot預算共用此結果，再交給既有checked_mana_cost向上取整乘法與ManaPool交易。既有legacy getter保留API，正式兩個消費端改用checked版本；無newwire/CABI或英雄／營地名稱特判。

新增2core驗證折扣／加價／free／family／legacy float／i64MIN合法zero與非法／overflow。新增3base正式60Hz測試：45Mana技能-50%／+100%／-100%／更低有效modifier分別扣23040／92160／0／0raw且效果/CD成功；string／i64MAX／u64MAX拒絕且零自療/Mana/CD。準備技能不早扣款，釋放時改為-50%僅扣45Mana並生2召喚；非法release零召喚/CD/費用。Bot面對非法成本不產生cast input，合法折扣仍可選擇，planning不改mana。

1813passed/0failed/9existingignored；World71全執行；3433正式輸入一致；145份證據／34PNG；health8 samples／max172秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|964|0|
|base_content|320|0|
|bridge|136|1|
|client|151|8|

backend／UE5.8／stage DLL與import library SHA成功。既有71native測試全執行，0failed/notrun；1既有warningcase／4既有RuntimePresentationReset警告保留。34PNG中29與前批像素相同、5逐張實際view；本批沒有新增UI，不把既有firstkillHUD右下裁切／部分map hero-focus名牌重疊算已修復。OS DPI及11外部/MVP33/44、完整combat replica、持續引導仍pending。

全部正式inputs exactset/SHA/bytes/mtime一致，所有reader/watchers與health已退休reap，封存證據數量如上，獨立核對archive bytes/SHA及原始來源。V1新增bot測試漏ManaPool顯式匯入，E0433編譯失敗；程序退休後保留4份精確來源與manifest、原始logs/ready/false receipt，再修正fixture並完整V2驗收。沒有把V1編譯失敗抹除或當作已執行測試。所有失敗原始輸入／logs與修正預防見errors MD。無真networkgames／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。

[證據](D:/omoba/docs/acceptance/2026-10-23-mana-cost/index.json)；[錯誤](D:/omoba/docs/plans/2026-10-23-mana-cost-errors.md)。
