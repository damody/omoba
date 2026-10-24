# 骨架動畫生命值frame時序接受

2026-10-20。當前功能修正及完整分段本機驗收成功；OpenSpec仍33/44，十一父項待權威／LAN／效能或實際OS DPI證據。未新增對局、重設10/10或commit／push。

## 缺口與通用修正

AOmAnimatedUnitActor::OnAnimationState原先只比較上一筆動畫tick；生命值frame已前進時，舊但高於上一筆動畫的快照仍會改目前姿勢，或以相同動畫tick解除暫停。七正式hero共用AnimationSnapshotOrdering擴充真骨架／instance回歸，有效red唯一失敗Animation older than current health frame preserves recall pose，實際由recall變idle。

共用動畫actor以既有NativeHealthTick在副作用前拒絕落後frame的動畫，與戰鬥特效frame邊界一致。不新增欄位、timer、掃描、網路重試或ABI；同tick refinement與新epoch／pool時間軸保留。四個既有fixture將代表目前狀態的default tick 0改成明確目前HP tick，所有原斷言保留。green涵蓋七hero姿勢、暫停／resume、current/equal tick、新epoch低tick及pool reset，另納入英雄生命週期、小兵與原創動畫三個完整測試；前輪VFX容量／時序及Buff回歸保留。

## 驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local無action cache | UBT7.84秒成功，stage SHA一致 |
| 專項 | 25成功，既有preview-world warning保留 |
| 完整native | fresh206成功，既有warning保留 |
| 分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／音訊 | 69零compiler／load診斷，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源／資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor；原860／900秒、90 proof frames、8fps不變。strict union重新驗原始報告、七binary cohort、案例清單與清理，兩段AVI驗RIFF／索引／timeline／幀數／SHA。三分鐘監測至root退出，publisher／editor／profile均清理。兩公開map另驗144幀／12片段，六HUD與兩map原圖逐張檢視；圖片僅作呈現回歸，動畫時序由實際slot／instance斷言證明。

38份SHA／size證據保留有效red；E1425記錄查找錯誤、E1426記錄產品修正、E1427記錄UE報告BOM解析錯誤。歷史證據原狀保留。execution_mode=phased、single_session_full_success=false；受控資料及跨session案例覆蓋不冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對。

## 封存最終核對

獨立v2明確清單核對成功：本輪38份及歷史551份逐份SHA／size一致，strict完整操作union、3335來源及33/44任務狀態均通過。v1因多層文字替換保留舊數量而失敗，原log保留；修正未更動已封存index、產品來源或驗收門檻。E1428記錄原因與防重犯。最終核對結果位於omfue/Saved/BuildTemp/animation-frame-final-audit-v2.json。
