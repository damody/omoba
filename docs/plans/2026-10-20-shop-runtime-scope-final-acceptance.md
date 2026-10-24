# 商店Runtime生命週期與完整本機驗收

2026-10-20。本輪補齊4.4／7.3在直接Runtime Stop／Start時，舊商店撤銷關聯與經濟資格未退休的實作缺口。父項維持33/44、11待驗；沒有新對局、重設10/10、commit或push。

## 實作與回歸

既有Runtime共用scope reset只清一般操作；商店撤銷的pending／receipt watch／request id／purchase pin及已結算文字仍能跨Runtime存在，舊經濟投影也未失效。red-v1以真port-zero Runtime Stop重現唯一失敗：Direct runtime stop retires shop undo correlation immediately，預期0實為987。原report／engine.log／六張PNG保留。

新增widget共用ResetRuntimeInputScope，沿現有Runtime scope reset訂閱生命期同步執行：先清一般操作、退休undo pending／watch／id／pin與結算文字，再套既有空經濟投影，讓buy／sell追蹤與recipe資格也依原共用規則退休。保留widget與輸入owner，等新公開經濟投影恢復操作；已存在Runtime的冪等Start不清除有效pending。沒有重送、改動權威交易、推測退款或更動receipt匹配規則。

十六專項全部Success／零warning。新增整合斷言涵蓋停止即時清除987、watch與purchase pin退休、舊資格失效、重啟仍等待新投影、fresh投影恢復資格、舊receipt不顯示退款、新undo request 1可接收、冪等Start保留live pending，以及重複Stop退休新請求。既有一般操作低ID重啟、來源隔離、退休與single draw等回歸同時通過。admission／receipt／result為受控fixture，不是權威ACK。

## 完整驗收

- 普通UE5.8 NoEngineChanges／local／無action cache成功，UBT 37.56秒。
- 205 native全成功，保留12筆既有warning；458實際操作／222 wire／157 control stage／13游標觀察，全部owned程序及profile清理成功。
- 69既有Blueprint零compiler error／warning／load failure，資產未變；五份實際submix PCM成功。
- 原始視窗錄影3571幀／11片段／114726276 bytes、593.385秒，RIFF／JPEG／index／SHA／timeline嚴格驗證。860秒工作期限、900秒publisher壽命、事件順序、證明frame及8fps取樣保留，沒有降低接受標準。
- 127 Lua契約通過；3334來源／資產集合、SHA、size及mtime驗收前後一致。527 Rust／ABI與93內容Lua未變，完整Rust／TD／all-target明示沿用2026-10-19-new-machine；本輪fresh公開map exporter單項。
- 長步驟依程序creation identity每180000ms檢查並記錄final；六張HUD與兩張公開map原圖已直接檢視。一般操作PNG不冒稱新增商店狀態截圖；商店生命週期由真Runtime與實際widget狀態斷言驗證。

22份可攜證據：`docs/acceptance/2026-10-20-shop-runtime-scope/index.json`。raw log／AVI保留Saved。錯誤E1373記錄路徑搜尋失誤與防範，E1374保留red→green及完整接受；E1375記錄文件分段更新的中止與恢復。分項計時不代表固定效能接受；先前E1369的速度差仍未歸因單一原因。

## 接受界線

剩餘11父項仍缺真權威ACK／完整能力對局、LAN、固定效能或100%／150%實際OS DPI配對等證據，目前只實測150%實體monitor。不新增對局，不把fixture control.ack當原gameplay ACK，沒有把本輪子範圍當完整MVP完成。
