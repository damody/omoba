# 正式技能冷卻屬性：最終驗收

`complete-cooldown-modifiers` 完成4/4 tasks。既有 `cooldown_percentage`／`cooldown_percentage_stacking` 原只在共用屬性API存在，正式交易預留與技能成功後啟動CD都直接使用作者基礎秒數。本批 `UnitStats::checked_cooldown` 沿用既有兩項相加政策與family strongest聚合，以checked_sum_add／checked multiplier／i128乘除檢查非法值與overflow；乘數至少102raw、正冷卻至少1raw、最多1,000,000秒，zero維持zero。這是既有百分比schema接線，沒有改成另一種ability-haste公式。

正式dispatch在任何handler之前解析一次，managed Hero ledger與最後ScriptStartCooldown使用完全相同值；不讓串行事件預留與實際settlement不一致，不在script內另乘一次。無Mana的legacy路徑也使用同一解析。前搖入場做合法性檢查，成功釋放時依當時屬性重新解析，已運行的冷卻不再因buff變更縮放。非法release沒有召喚／Mana／CD；既有成功交易與失敗回滾保留。沒有英雄／營地名稱分支、新內容ID、wire／C ABI或新對局。

新增2core測試驗證無modifier、兩項疊加、family strongest、legacy float、fractional floor、正值最小／zero、非法型別、聚合／跨項加法overflow及duration超限。新增3base正式60Hz測試走真PlayerInput：20秒技能受-50%、+100%、-50%+25%、過強haste後為10240／40960／15360／2040raw；成功扣45Mana一次，pause凍結，後續modifier漂移不縮放running CD，冷卻中重施不扣Mana。非法兩項各種輸入拒絕且無自療／Mana／timer；前搖召喚技能釋放時從+100%改-50%得到13824raw並僅生成2個召喚／扣90Mana，非法release零效果。

## 集中驗收

1808passed/0failed/9existingignored；World71全執行；3433正式輸入一致；121份證據／34PNG；health8 samples／max171秒。

|套件|通過|既有ignored|
|---|---:|---:|
|abi|15|0|
|content|42|0|
|templates|116|0|
|codegen|69|0|
|core|962|0|
|base_content|317|0|
|bridge|136|1|
|client|151|8|

正式backend、UE5.8、stage DLL與import library SHA驗證成功；既有World71全執行、0Failed/NotRun，保留既有warning案例與4個RuntimePresentationReset警告。34PNG與上一批比較：28完全相同可繼承已審閱像素，6實際逐張檢視；完整viewport／paintcrop／座標證據保留。這是既有呈現回歸驗收，沒有新增冷卻modifier專用UI，也不宣稱切換OS DPI。已知first-kill HUD右下裁切、部分map hero/focus名牌重疊仍未修復。

正式來源exactset／SHA／bytes／mtime一致，原始stdout/stderr／ready／owner/watch receipts與退休紀錄封存，獨立核對封存bytes/SHA與原始來源。原native runner成功輸出於CooldownWorldV3，但監督器錯找CooldownWorldV1，故原false receipt、runner與raw logs保留；沒有將這個驗收腳本錯誤改稱玩法失敗。新增獨立owned確認步驟核對實際report的70success+1warningcase／0failed/notrun與原始零stderr成功輸出，未重跑native cohort。final6 workers包含該確認程序，另封存1份歷史失敗監督收據。全部owned worker/watchers及170秒health timer已退休reap。證據：[index](D:/omoba/docs/acceptance/2026-10-23-cooldown/index.json)；錯誤：[errors](D:/omoba/docs/plans/2026-10-23-cooldown-errors.md)。

11外部父項、MVP33/44、完整combat replica、持續引導與其他玩法／美術缺口仍pending；本批不宣稱整個遊戲等同LOL。無networkgames／quota reset／commit/push／omfx維護。無人值守期限仍為2026-10-25 00:33:38 Taipei。
