# 已有native Buff更新退休接受

2026-10-20。共用修正、二十二專項及完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗。未新增權威對局、重設10/10、commit或push。

## 實際缺口與通用修正

OnBuffRefreshed原本忽略SyncBuff結果，zero remaining會移除renderer卻保留ActiveBuffEffects中的native handle；OnBuffUpdated在非zero但pool拒絕時也保留舊renderer／handle。這會讓目前visual與ownership不一致。

AOmUnitActor新增private UpdateTrackedNativeBuff，供default Refreshed／Updated共用，只接管已追蹤UOmCombatBuffVisual。zero remaining、無pool或SyncBuff拒絕，皆沿既有UntrackBuffEffect退休renderer及ownership，回傳handled避免落入legacy文字。合法有限值與-1無限期保持既有視覺，不自動重試、不在update建立新visual、不提高容量。fresh完整State仍可沿既有admission建立；不是native handle的custom／legacy不由helper接管，既有target／generation guard仍先執行。只增加非反射private C++方法，不修改Rust／FFI schema或逐entity tick。

既有NativeCombatVfxLifecycle以真生成hero／world pool重現：先有目前Buff，terminal refresh後renderer已0但ownership仍1；有效RedV1唯一失敗為Current terminal refresh retires native ownership。green驗terminal refresh、current update/remove同步退休，NaN update／refresh拒絕後renderer與handle皆0，hidden update清已被pool退休的handle，合法infinite refresh／update保持一個native renderer；custom terminal refresh／update仍保留自己的visual。原舊id／generation隔離、512容量、拒絕不配置、回復、shader pixel與content rebinding斷言保留。受控公開frame／Buff不冒稱新權威對局或ACK。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | UBT63.09秒成功，stage SHA一致 |
| 專項 | 22成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒、90 proof frames與8fps。strict union重新驗原始native／wire／history／UE index／cleanup、七binary cohort與案例清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。兩公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖片僅為呈現回歸，更新退休由實際renderer／handle斷言證明。

38份SHA／size封存保留有效red，E1420記錄缺口與修正。相依建置→red→修正建置→green皆在前一步session明確退出後啟動，所有歷史接受與失敗包保持原狀。execution_mode=phased、single_session_full_success=false；不將受控資料或跨session覆蓋冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對，十一父項缺證不勾選。
