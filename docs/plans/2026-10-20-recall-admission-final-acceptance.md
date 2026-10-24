# 目前回城持續特效配置恢復接受

2026-10-20。共用修正、二十二專項及完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗。未新增權威對局、重設10/10、commit或push。

## 實際缺口與通用修正

SynchronizeCombatAnimation在邏輯回城旗標改變時才呼叫void SetRecall；native pool滿載配置失敗後，邏輯旗標已true，後續相同有效快照不再補圈框。這不是權威回城未啟動，也不是要重播已丟棄的oneshot。

SetRecall現在回報是否達成目前呈現狀態；actor以bRecallVisualPending分開記錄邏輯回城與尚未配置的持續圈框。只有狀態改變或後續目前有效動畫仍pending才處理。O(1) CanRetryPersistentAllocation檢查scope／frame／free slot及resources，池仍滿時不重掃效果陣列。成功後清pending，正常已呈現快照沒有新增pool查找或重啟age；取消、隱藏、死亡、epoch／presentation reset同步清pending。沒有新timer、網路重試、tick掃描或提高512上限，Rust／FFI schema不變。

既有NativeCombatVfxLifecycle真生成hero／world pool填滿512格persistent effects，使目前回城圈框被拒絕；RetireOwner釋放容量後重送同一有效動畫，red唯一失敗Current recall restores its declined persistent visual。green驗滿載capacity gate為false、重複pending快照仍512，釋放後恰好一個圈框，Tick後重複目前狀態保持同age，pending取消後釋放容量不復活，fresh回城仍可建立。原死亡／重生／隱藏／scope、Buff世代與容量／更新退休、shader pixel與content rebinding均保留。使用明示受控公開frame／animation，未新增權威對局或ACK。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | UBT44.95秒成功，stage SHA一致 |
| 專項 | 22成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒、90 proof frames與8fps。strict union重新驗原始native／wire／history／UE index／cleanup、七binary cohort及案例清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。兩公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖片僅為呈現回歸，配置恢復由實際pool／age斷言證明。

38份SHA／size封存保留有效red，E1421記錄缺口與修正。所有歷史接受與失敗包保持原狀。execution_mode=phased、single_session_full_success=false；不將受控資料或跨session覆蓋冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對，十一父項缺證不勾選。
