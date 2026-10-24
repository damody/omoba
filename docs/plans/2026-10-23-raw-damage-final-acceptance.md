# 共用原始傷害結算：最終驗收

`complete-raw-damage-settlement` 完成4/4 tasks。明確區分RawDamage與既有已結算Damage/ProjectileDamage；直接腳本、持續傷害、新建原生／script彈體在serial settlement讀取當時護甲／抗性，使用與DamageInstance相同的固定點防禦公式，再進既有packet modifier、shield、HP與歸因流程，避免漏算或雙算。Hero取CProperty基礎、普通Unit取Unit基礎、只有CProperty的目標保留其基礎；純傷跳過基礎防禦，TD沿用profile／layer規則。無英雄／營地名稱特判。

Projectile明確序列化damage_stage，舊資料預設Settled，新factory明確Raw；死亡來源的既有投射物仍保留來源世代／credit。目標已列入本批退休清單時，後續RawDamage不能消耗其護盾。非法amount/profile或防禦加總失敗不改HP／shield。未修改C ABI或公開protocol layout。

## 集中結果

1781passed/0failed/9existingignored；3426正式輸入一致；221份證據／29PNG；health15 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|115|0|
|codegen|68|0|
|core|949|0|
|base_content|306|0|
|bridge|135|1|
|client|151|8|
正式backend與UE5.8 build成功，部署DLL／import library SHA一致。World66全執行，0 Failed/NotRun；既有RuntimePresentationReset warning保留。compiled-map公共ABI export另行成功。29PNG中25張與上一批完全一致，其餘4張逐張開啟檢視。既有首殺HUD下／右裁切與部分hero/focus名牌重疊保持限制，未宣稱本批新增視覺特效。

七項新ECS測試、序列化／彈體stage測試與兩項正式60Hz input測試驗證：雙路徑等價／不重複減傷、live防禦buff、混合傷害、pure、shield一次、錯誤輸入、同批退休target、已退休source／偽造origin、TD；實際遊俠箭＋四秒流血的pause、護盾與中途加護甲，以及正式魔法技能的抗性／direct override cap均成功。現有完整離線模擬／filtered replay通過。

## 原始失敗與預防

FocusedV1 core938通過9失敗：五個新fixture少TowerTemplateRegistry，四個舊pure projectile fixture用無效profile0。保存12原始來源後補完整fixture／明確TRUE mask，保留原wallet／generation／credit要求；FocusedV2 core948成功。正式V2全1780passed/9ignored後，新增test-first死亡target邊界重現0passed/1failed：同批Death後RawDamage仍扣shield。保存19個V2與兩個red原始來源，所有reader退休/reap後才加退休清單guard；最終V3全scope重新接受，未混用舊revision。

來源讀取前先發現實際路徑；PowerShell的rg path不放wildcard。原始量只做負值／總量可表示性檢查，不加任意一百萬上限；已減傷與未減傷不能依predeclared猜測。沿用既有正護甲、正魔抗75% cap及量子順序，本批不宣稱完整LoL平衡／負護甲放大或任意post-mitigation頻率不變性。

6final與4history worker／watcher核對creation token退休，全階段170秒health timer停止並reap。來源exact set／SHA／bytes／mtime及封存證據獨立檢查通過。詳見[index](D:/omoba/docs/acceptance/2026-10-23-raw-damage/index.json)與[錯誤紀錄](D:/omoba/docs/plans/2026-10-23-raw-damage-errors.md)。

仍待具體channels／zones、更多內容與完整combat replica；11外部父項、MVP33/44保持pending。未新增真網路對局／quota reset／commit/push／omfx維護。原無人值守期限2026-10-25 00:33:38 Taipei不變。
