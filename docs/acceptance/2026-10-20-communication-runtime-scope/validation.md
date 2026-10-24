# 溝通Runtime生命週期接受

2026-10-20。本輪補齊Runtime直接Stop／Start仍保留聊天草稿、溝通gesture與UI快取的缺口。19專項及整套分段本機驗收成功，OpenSpec仍33/44、11父項待驗；沒有新增權威對局、重設10/10、commit或push。

## 共用修正

UOmCommandBarWidget::ResetCommunicationScope集中退休草稿、實際edit文字、歷史／objectives面板與rows、minimap標記、communication音訊去重／roster、catalog／scope／room／run、mute pending token及notice。Runtime scope reset、widget terminal退休、無bridge的UI刷新共用同一清理；Runtime停止只在自己的草稿持有焦點時沿既有guard歸還自己的viewport，不偷其他焦點。保留使用者偏好及可重用widget／input owner。一般有效通訊view更新的草稿規則未改。

InputOwner::ResetRuntimeInputScope先CancelCommunicationGesture，清除舊gesture／overlay，沿原有QueuedCommunicationReleases退休，不在Runtime回呼中提交訊息。新Runtime可重用token；冪等Start不重置live草稿／gesture。

## 重現與驗證

擴充既有ShopUndoProjectionAndPin真port-zero Runtime整合fixture，以friend測試接點把合法受控communication view的草稿／ping model、實際widget edit／rows、history／objectives、pending mute／notice放入owned物件。沒有繞過正式Runtime停止或回呼；沒有生造server ACK或宣稱這是實際聊天事件入口。修正前red唯一失敗為Runtime stop immediately cancels communication draft，原報告保留。

green驗無UI tick直接Stop立即取消草稿／gesture、清pending tokens／rows／文字／notice、關history／objectives、釋放自己的chat focus、保留all-chat偏好；新Runtime可建立fresh受控狀態，冪等Start保留live草稿／gesture／token，重複Stop再退休。完整458實際viewport事件另外回歸聊天／ping入口、正常view更新、輪盤、舊scope與focus仲裁。兩層證據分開，不把受控初始化說成實際網路通訊。

| 驗收 | 結果 |
|---|---|
| 普通UE5.8 local／無action cache建置 | UBT35.79秒成功，stage SHA一致 |
| 專項 | 19成功，四筆既有RuntimePresentationReset preview-world warning |
| 完整native | fresh206成功，既有warning保留 |
| 完整分段輸入 | 458案例／222 wire／157 stages／13 cursor |
| Blueprint與音訊 | 69既有Blueprint零compiler error／warning／load failure，五PCM成功 |
| Lua | fresh140 owned＋69 local acceptance契約成功 |
| source／asset | 3335檔前後SHA／size／mtime及集合一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust明示沿用2026-10-19新機接受 |

兩段各保留原860／900秒、90 proof frames、8fps，原始native／wire／history／UE index／cleanup及七binary cohort由正式strict union驗證。所有publisher／editor／profile清理成功，三分鐘監測至root退出，兩段AVI再次嚴格驗RIFF／索引／timeline／幀數／SHA。兩公開map另有144幀／12片段錄影；本輪六HUD與兩map圖已逐張檢視，圖是在注入communication scope前擷取，退休行為由actual owned widget/model斷言證明。

38份SHA／size封存保留有效red，錯誤與處置見unreal-moba-error-register.md E1394。execution_mode=phased、single_session_full_success=false；不將合成feed或跨session覆蓋當成權威ACK、單session效能、真LAN或實際OS DPI配對。11父項缺證不勾選，前次接受及失敗包均保持原樣。
