# Buff事件entity世代隔離接受

2026-10-20。共用修正、二十二專項及完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗。未新增權威對局、重設10/10、commit或push。

## 實際缺口與通用修正

default OnBuffRemoved／Refreshed／Updated原先只靠visual key，錯entity或generation事件可移除目前handle／native renderer，或改目前legacy文字。OnBuffState的目標檢查原本只在native分支，legacy路徑未共用同一邊界。

OmUnitActor.cpp共用IsCurrentBuffTarget以O(1)驗目前EntityId>=0、EntityGen>0及payload target id／generation一致，default Added／State／Removed／Refreshed／Updated皆在視覺操作前拒絕錯事件。native pool、tracked custom、明確legacy及debug cue共用邊界；移除native分支重複比較，不新增header／ABI或逐entity掃描。合法目前事件仍可建立、更新、退休；明確的Cleanup／Untrack管理API維持原用途。自訂Blueprint override須遵守其自身契約，本輪驗的是預設handler，payload沒有Runtime epoch，不冒稱跨Runtime ACK或序號防重播。

擴充既有NativeCombatVfxLifecycle真生成hero／world pool：先建立目前native handle，再送同key、錯generation的remove；有效RedV2唯一失敗為A stale buff removal preserves current ownership。green對錯id及錯generation驗五類回呼，含zero remaining refresh不能清目前pool，舊added／state不能換handle；current terminal update與remove正常退休，fresh state仍可用。另驗stale remove不能破壞tracked custom，legacy五類舊事件不能改文字／移除元件。前輪512容量、拒絕不分配、回復、shader pixel與content rebinding斷言全部保留。使用明示受控公開frame／Buff，並非新權威對局。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | UBT6.30秒成功，stage SHA一致 |
| 專項 | 22成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒、90 proof frames與8fps。strict union重新驗原始native／wire／history／UE index／cleanup、七binary cohort與原始清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。兩公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖片僅為呈現回歸，entity隔離由實際handle／pool／文字斷言證明。

39份SHA／size封存保留有效RedV2及排除的RedV1。E1418記錄尚未確認建置session退出即啟動相依red，該輪不採證；等所有程序退出後獨立重跑才接受RedV2。E1419記錄實際產品缺口與修正，所有歷史接受／失敗包保持原狀。

execution_mode=phased、single_session_full_success=false；不將受控資料或跨session覆蓋冒稱權威ACK、真LAN、單session固定效能或實際100%／150% OS DPI配對，十一父項缺證不勾選。
