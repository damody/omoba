# 記分板Runtime生命週期接受

2026-10-20。修正直接Stop／Start仍保留記分板輸入hold的缺口。十九專項與完整分段本機驗收成功，OpenSpec仍33/44、十一父項待驗；沒有新增權威對局、重設10/10、commit或push。

## 共用修正與重現

InputOwner::ResetRuntimeInputScope使用既有CloseOwnedScoreboard，同步清除bScoreboardHeld、所有hold mask與keys，並關閉owned HUD請求。widget自己的ResetRuntimeInputScope另同步關閉requested，不依賴world actor清rows、下一tick或事件註冊順序。保留控制profile與可重用owner；冪等Start仍保留live hold。

既有ShopUndoProjectionAndPin整合fixture透過真PlayerInput／ProcessOwnedInput按Tab，先確認hold實際成立，再無world tick直接StopRuntime。有效red只有Runtime stop immediately retires scoreboard hold失敗。green驗立即清旗標／槽位／keys／requested，重啟後舊release不復活狀態、新press可用、冪等Start保留live hold、重複Stop退休。fixture沒有捏造權威資料或ACK；公開記分板呈現與正常按鍵仲裁由完整viewport事件另回歸。錯誤與防重犯記錄為E1395。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache建置 | UBT17.18秒成功，stage SHA一致 |
| 專項 | 19成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔前後SHA／size／mtime及集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，兩段保留原860／900秒期限、90 proof frames、8fps及清理。strict union重新核對兩段原始native／wire／history／UE index／cleanup、七binary cohort與原始案例清單；兩段AVI重新驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。另驗公開map144幀／12片段，六HUD與兩map圖逐張檢視；圖是受控scope狀態前的呈現回歸，不能代替生命週期斷言。

38份SHA／size封存保留有效red。execution_mode=phased、single_session_full_success=false，不將合成feed或跨session覆蓋冒稱權威ACK、真LAN、單session效能或實際100%／150% OS DPI配對。十一父項缺證不勾選，歷史接受與失敗包保留原樣。
