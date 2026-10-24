# 戰鬥特效動畫快照時序接受

2026-10-20。當前功能修正及完整分段本機驗收成功；OpenSpec仍33/44，十一父項待權威／LAN／效能或實際OS DPI證據。未新增對局或重設10/10，未commit／push。

## 缺口與通用修正

正式bridge先呼叫SynchronizeCombatAnimation，再OnAnimationState；骨架動畫已保護自身latest tick，戰鬥特效卻只比較最後HP frame。當動畫比HP更新時，較舊動畫仍可取消目前回城圈框或改暫停。有效red透過實際生成hero及world pool，唯一失敗Older animation cannot cancel current recall rendering。

AOmUnitActor共用CombatAnimationTick在任何副作用前拒絕外來entity、負tick、低於HP frame或已接受動畫tick的快照。合法同tick refinement維持；隱藏顯示保留時序，presentation reset及新epoch清時序。零新timer／掃描／網路重試，Rust／FFI schema不變。green驗stale cancel／unpause、實際圈框age凍結、同tick cancel／resume、stale recall／pause不能復活、foreign generation不能污染時鐘、concealment及新epoch低tick。既有512容量、回城容量回復、Buff世代／配置／更新退休與實際shader回歸全保留。

## 驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local無action cache | UBT44.31秒成功，stage SHA一致 |
| 專項 | 22成功，四筆既有preview-world warning保留 |
| 完整native | fresh206成功，既有warning保留 |
| 分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／音訊 | 69零compiler／load診斷，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源／資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor；原860／900秒、90 proof frames、8fps不變。strict union重新驗原始報告、七binary cohort、案例清單及清理，兩段AVI驗RIFF／索引／timeline／幀數／SHA。三分鐘監測至root退出，publisher／editor／profile均清理。兩公開map另驗144幀／12片段，六HUD與兩map原圖逐張檢視；圖像僅作呈現回歸，時序保護由真pool／age及state斷言證明。

38份SHA／size證據保留有效red；E1423記錄查找操作錯誤，E1424記錄產品缺口與修正。所有歷史接受及失敗包原狀保留。execution_mode=phased、single_session_full_success=false；受控frame／animation及跨session案例覆蓋不冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對。
