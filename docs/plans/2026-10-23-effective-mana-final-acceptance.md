# 有效魔力消耗與前端預檢：集中驗收

complete-effective-mana-hud完成4/4。後端原已支援checked cost modifiers，但Unreal最後送出與游標資源提示仍拿generated base cost比較，折扣可能被誤擋。現在emit_owner_mana_costs沿用共用checked UnitStats乘數／checked_mana_cost與當前AbilityRegistry，隨每tick單次owner score發布：active PostStep、inactive PreStep；每player只發布tick stamp與最多4slot的template catalog/rank binding和raw cost。FactAudience::Team，不公開私有buff、canonical entity或運算過程。raw -1unknown／-2invalid／0free，active source驗證alive、owner、playing與mana support。

client要求同team、同tick、唯一metric、slot/catalog/rank符合當前Hero，再用owner ManaPool raw比較，產生0未提供／1可負擔／2不足／3非法。原protobuf AbilityHud新增狀態與raw price；bridge同步ABI35，OmAbilityHud48bytes、statusoffset40/price44，Rust/header/型別smoke斷言一致，舊ABI拒絕。shared typed sanitizer再驗證價格非負、rank與mana support、分類不矛盾；非法或負價不會變成可負擔的zero。ABI數字價格只供文字呈現，原生不足提示保留權威分類，避免large raw轉float後相等造成漏判。

Generated actor刪除base-cost admission，改讀當前status；known short／invalid明確拒絕，unknown不猜base而交後端最終判定；scope退役清除。PublishedManaCost現在提供effective authority price，既有owner preview／tooltip直接共用，未知與非法有中文提示；無C++重算buff或英雄名稱特判。後端仍在真正release重驗成本，不保證快照後mana/buff不會改變。

新增1base正式60Hz測試驗證45base在折扣／加價／free／invalid得到23040／92160／0／-2raw，binding/tick一致且只向owner team。新增4client測試驗證raw分類、zero/missing差別、stale/duplicate/wrongteam/player/ability/rank/slot、owner HUD price與清除、typed負數／分類矛盾／無mana支援／非法state拒絕。既有bridge typed-HUD→ABI測試擴充22.5價格／status與空frame清除；現有native測試驗證實際formatter22.5、非法提示、authority-short在float equality仍正確，以及既有重設/退休路徑。

1824passed/0failed/9existingignored；World71全執行；3433正式輸入一致；300份證據／34PNG；health15 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|967|0|
|base_content|322|0|
|bridge|136|1|
|client|157|8|

正式backend、UE5.8、stage DLL/import library SHA與公共ABI map export成功；World71全執行0failed/notrun，保留既有1warningcase／4RuntimePresentationReset警告。34PNG中27與前批像素相同、7逐張實際檢視；本批有效價格以實際HUD formatter／原生model斷言驗證，不宣稱另有滑鼠hover價格專用截圖。舊firstkillHUD右下裁切／部分map hero-focus名牌重疊仍known，OS DPI與外部LAN/ACK/perf未驗證。

V1 E0605誤將template AbilityId wrapper直接cast成i64，程序退休後保留15 exact sources／manifest、logs／ready／false receipt；改用已驗證.raw() API。V2全部1823+9ignored成功，但預檢發現負價格會被ABI轉成0並保留affordable分類；全部V2程序退休後再保留18 sources與完整成功cohort，補typed sanitizer／native exact分類與測試，最終完整RustV3接受。V2不改稱失敗測試。UE／nativeV1有1個實際失敗：既有unknown tooltip斷言禁止所有「魔力消耗」文字，誤拒新的明確未提供提示；其餘70case含warningcase成功。全部讀取程序退休後保留18 pre-native-v2來源與manifest、原始失敗report/logs及全部34PNG，再將斷言精確改成拒絕數字前綴「魔力消耗：」並要求未知提示。未變更Rust或產品邏輯；UE/nativeV2重新集中接受。所有原始證據均保留。

正式inputs exactset/SHA/bytes/mtime一致；final18 sources、兩個Rust history scope及pre-native-v2 scope／原native失敗證據封存。所有readers/watchers與health退休reap後，獨立核對archive bytes/SHA與原始來源。11外部/MVP33/44、完整combat replica／持續引導等仍pending。無真networkgames／quota reset／commit/push／omfx維護；原無人值守期限2026-10-25 00:33:38 Taipei不變。

[證據](D:/omoba/docs/acceptance/2026-10-23-effective-mana/index.json)；[錯誤](D:/omoba/docs/plans/2026-10-23-effective-mana-errors.md)。
