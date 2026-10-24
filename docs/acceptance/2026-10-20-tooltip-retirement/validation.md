# 失效HUD閱讀提示退休接受

2026-10-20。共用host修正、二十一專項及完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗，未新增權威對局、重設10/10、commit或push。

## 共用修正與實際重現

SOmHudToolTipHost::Tick原先在State::Visible=false時直接返回；長提示因HUD範圍、來源、文字或client使IsEmpty後只隱藏，未執行閱讀焦點退休，也保留舊Tip。現在持有仍存在的tip局部shared pointer，IsEmpty時呼叫既有OnClosed，再以CloseIfCurrent移除自己的舊提示。OnClosed沿原user／descendant焦點guard歸還原焦點，不呼叫全域CloseToolTip、不關其他host提示，不修改字型、scroll、Gameplay快捷鍵仲裁或tooltip可讀功能；正常可用提示定位與clamp維持原路徑。只處理單一host目前提示，沒有per-entity新工作。

擴充既有ReadableHudToolTip，沿真GameViewport及實際Slate hover→wheel→click取得長提示閱讀焦點。800x600／text2.0案例切換shop使該hint失效，執行host正常Tick，刻意在global UpdateToolTip前驗退休以排除全域更新時序相依。有效red唯一失敗為Unavailable tooltip host restores prior reading focus。green驗立即歸還原閱讀焦點，回原HUD scope也不能復活舊hint；其餘800x600／400x300、text1／2、滑鼠捲動、PageDown、消耗遊戲鍵、Esc、owner退休及GC斷言保留。輸入資料是明示受控公開HUD，實際hover／focus／scroll由Slate產生，不冒稱新網路對局或權威ACK。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache | UBT5.57秒成功，staged DLL SHA一致 |
| 專項 | 21成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔SHA／size／mtime及集合前後一致 |
| Rust／內容Lua | 527／93維護輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，保留原860／900秒、90 proof frames與8fps。strict union重新驗原始native／wire／history／UE index／cleanup、七binary cohort與原始案例清單；兩段AVI驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。兩張公開map另驗144幀／12片段；六HUD、兩map及九tooltip原圖逐張檢視，圖片僅證明呈現，焦點退休由actual Slate斷言證明。

48份SHA／size封存保留有效red及提早執行相依輸入比對的失敗。E1410記錄再次錯用rg位置glob；E1411為實際產品缺口；E1412記錄來源基準未完成就比對的流程錯誤，等session退出後重新執行才接受。全部歷史接受／失敗包保持原狀。

execution_mode=phased、single_session_full_success=false；受控HUD與跨session覆蓋不冒稱權威ACK、真LAN、單session效能或實際100%／150% OS DPI配對；十一父項缺證不勾選。
