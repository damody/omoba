# HUD已揭露身分Runtime生命週期接受

2026-10-20。修正HUD的英雄頭像與已揭露名稱仍跨Runtime保留的缺口。十九專項與完整分段本機驗收成功，OpenSpec仍33/44、十一父項待驗；沒有新增權威對局、重設10/10、commit或push。

## 共用修正與實際界線

UOmPlayerHudOwnerComponent::ResetGameplayInputScope沿既有ClearIdentityCaches清除已揭露英雄content／name／player、實際widget頭像／名稱／details／team與overlay身分，標示legacy composition需要重新套用，再走既有widget scope清理。HUD自己負責Runtime scope退休，不依賴world actor發空投影、下一tick或來源選用順序；不銷毀可重用widget／owner，不放寬來源guard。冪等Start沒有reset廣播，同Runtime的死亡仍保留已揭露身分。

既有ShopUndoProjectionAndPin真port-zero Runtime整合fixture新增合法bound但未選用的既有bridge，使用正式ApplyHeroHud初始化明示受控公開投影，確認頭像成立，直接Stop驗無tick立即清頭像／名稱。此初始化不是實際網路快照或authority ACK。green另驗新Runtime無身分死亡不能借舊身分、fresh公開英雄可用、冪等Start與同Runtime死亡保留當前身分、重複Stop安全，以及退休widget不復活。

red-v1用普通AActor，被六個HUD delegate契約guard拒絕，未到停止斷言，不算有效red。改用契約完整但未選用的既有來源後，red-v2唯一失敗為Expected 'Runtime stop immediately clears typed HUD identity' to be None, but it was saika_magoichi。兩份原報告均保留。錯誤紀錄E1396是唯讀查找時猜錯檔名，E1397是fixture來源契約失敗，E1398是實際Runtime身分缺口。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache建置 | UBT6.42秒成功，stage SHA一致 |
| 專項 | 19成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔前後SHA／size／mtime及集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，兩段保留原860／900秒期限、90 proof frames、8fps及完整清理。strict union重新核對原始native／wire／history／UE index／cleanup、七binary cohort與原始案例清單，兩段AVI重新驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖是注入scope狀態前的呈現回歸，生命週期行為由actual owned widget及Runtime斷言證明。

39份SHA／size封存保留有效red及無效fixture red。execution_mode=phased、single_session_full_success=false，不將合成feed或跨session覆蓋冒稱權威ACK、真LAN、單session效能或實際100%／150% OS DPI配對。十一父項缺證不勾選，歷史接受與失敗包保留原樣。
