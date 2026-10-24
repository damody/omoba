# 正式Buff特效配置邊界接受

2026-10-20。共用修正、二十二專項及完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗。未新增權威對局、重設10/10、commit或push。

## 實際缺口與通用修正

AOmUnitActor::OnBuffState在native VFX啟用時，SyncBuff因容量／隱藏／狀態非法拒絕，原本會落入legacy UTextRenderComponent；錯目標／世代也因外層條件不成立落入legacy。正式共享池上限並沒有阻止額外文字元件與ownership配置，且錯目標可走到錯誤呈現。

現在native路徑完整封閉：先拒絕錯目標／世代，zero remaining即退休；既有自訂handler保留；SyncBuff成功才建立或保留輕量native ownership handle，拒絕則清過期native／text bookkeeping並返回。局部狀態的持續Buff可隨新snapshot在容量回復後再次配置；沒有延後重播oneshot cue、提高512上限或增加per-entity掃描。明確停用native的既有legacy路徑保留。

擴充既有NativeCombatVfxLifecycle，使用真world subsystem與正式生成hero，實際填滿512格persistent pool。有效red唯一失敗為Rejected native buff never allocates a legacy text component，預期registered text為0、實際1。green驗滿載及連續拒絕不增component／handle、不超共享上限，RetireOwner回收後fresh目前狀態以native handle成功配置；隱藏／錯世代／NaN duration不建立fallback，自訂handler不被取代，明確legacy仍可建立並清理文字。原實際instanced renderer／shader pixel、動畫／死亡／重生／scope／geometry回歸保留。資料為明示受控公開frame／Buff，不冒稱真對局或權威ACK。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | 正式修正UBT6.17秒，最終fixture版本10.13秒成功，stage SHA一致 |
| 專項 | 22成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔SHA／size／mtime與集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒、90 proof frames、8fps及完整清理。strict union重新核對原始native／wire／history／UE index／cleanup、七binary cohort與案例清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。publisher／editor／profile清理成功，三分鐘監測至root退出。兩公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖片只作呈現回歸，容量與拒絕由實際pool／registered component斷言證明。

42份SHA／size封存保留有效red、failed-green及失敗全輪；E1413記錄唯讀查找猜錯檔名，E1414記錄實際產品缺口與防重犯，E1415記錄custom探針錯用抽象UObject而觸發engine ensure，改為具體USceneComponent後保留相同產品斷言。E1416記錄ActorContentRebinding在沒有native pool的EditorPreview依賴舊fallback而失敗；改支援native pool的Editor world及有效hp/max_hp，保留原baseline、重用、control-only及退休斷言，再加入專項。失敗全輪success=false及Blueprint未啟動狀態保留，不把其viewport成功當全輪接受。source-v1/v2保留，最終fixture另以source-v3前後核對；E1417記錄相依工具輸出舊檔名被防覆寫guard拒絕，改以新版本輸出並重驗。所有歷史接受與失敗包保持原狀。execution_mode=phased、single_session_full_success=false；不將受控輸入或跨session覆蓋冒稱權威ACK、真LAN、固定單session效能或實際100%／150% OS DPI配對；十一父項缺證不勾選。
