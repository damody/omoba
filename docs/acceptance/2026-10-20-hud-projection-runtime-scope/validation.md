# 整組HUD投影Runtime生命週期接受

2026-10-20。修正舊結算modal、回城時鐘及其他HUD投影跨Runtime保留的缺口，採集中清理與既有呈現setter。十九專項與完整分段本機驗收成功；OpenSpec仍33/44、十一父項待驗，沒有新增權威對局、重設10/10、commit或push。

## 共用修正

UOmCommandBarWidget::ResetRuntimeInputScope集中退休結算model與實際Slate panel、回城payload／clock／match文字、shop／scoreboard／stats請求、英雄身分／資源／details／team／summoner、四格技能logical／visual／tooltip、選取／Buff／minimap及inspect、通訊、一般feedback、商店transactions／undo與最後提交狀態，再將模式設為等待新投影。沿用既有setter更新模型與已建widget，保留控制偏好及可重用widget，沒有送出遊戲指令。

HUD owner的ResetGameplayInputScope改用既有ClearPresentedCaches，同時退休已揭露身分、mode／epoch、legacy composition及measured-rate快取。只把widget模式設回0但保留owner舊mode，可能讓同模式的新Runtime被誤認未變而無法恢復；整組cache清理消除這個依賴，不放寬來源契約或權威gate。清理只在Runtime生命週期發生，不新增逐entity／frame操作。

## 重現、失敗與修正

既有ShopUndoProjectionAndPin使用真port-zero Runtime及契約完整、bound但未選用的來源，以正式typed API初始化明示受控投影。確認實際結算視窗已顯示、技能visual／minimap／selection／Buff均存在，直接Stop且不經world tick。有效red唯一失敗為Runtime stop immediately retires the old match result。受控payload不冒稱實際權威結算、網路快照或ACK。

green驗結果model及實際panel立即退休，技能logical空且readout精確只剩目前profile快捷鍵，舊rank／cooldown／minimap／selection／Buff／match摘要清除，既有身分隔離仍成立；fresh hero不能復活舊result、建立自己的回城clock，冪等Start保留live clock，重複Stop退休clock與舊label。

第一次正式建置因Slot區域變數遮蔽UWidget::Slot的C4458失敗，改SlotIndex，不關warning。第一次green發現清label時仍用舊visual組rank／cooldown，調整為先visual後label；第二次green仍只有空字串斷言失敗，查明setter本來就保留當前快捷鍵，改驗logical空且readout精確等於目前hotkey，不隱藏快捷鍵迎合測試。兩份failed-green均未接受，原始報告及失敗建置保留。錯誤與防重犯見E1399–E1402。

## 完整驗收

| 項目 | 結果 |
|---|---|
| UE5.8普通local、無action cache建置 | 最終UBT6.32秒成功，stage SHA一致 |
| 專項 | 19成功，四筆既有preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段viewport | 458案例／222 wire／157 stages／13 cursor |
| Blueprint／實際音訊 | 69零compiler error／warning／load failure，五PCM成功 |
| Lua契約 | fresh140 owned＋69 local acceptance成功 |
| 來源與資產 | 3335檔前後SHA／size／mtime及集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

foundation294案例／120wire／103stages，tail164／102／54及13cursor，兩段保留原860／900秒期限、90 proof frames、8fps及完整清理。strict union重新核對原始native／wire／history／UE index／cleanup、七binary cohort與原始案例清單；兩段AVI重新驗RIFF／索引／timeline／幀數／SHA。所有publisher／editor／profile清理成功，三分鐘監測至root退出。公開map另驗144幀／12片段，六HUD與兩map圖逐張檢視；圖是在scope注入前擷取的呈現回歸，生命週期由actual widget及Runtime斷言證明。

41份SHA／size封存保留有效red、失敗建置與兩份failed-green。execution_mode=phased、single_session_full_success=false，不將合成feed或跨session覆蓋冒稱權威ACK、真LAN、單session效能或實際100%／150% OS DPI配對。十一父項缺證不勾選，歷史接受與失敗包保留原樣。
