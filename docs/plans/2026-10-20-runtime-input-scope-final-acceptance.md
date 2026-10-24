# Runtime輸入關聯生命週期與完整本機驗收

2026-10-20。本輪補齊直接Runtime Stop／Start在兩個World tick之間完成時的一般操作回覆缺口。OpenSpec維持33/44、11父項待驗；沒有新對局、重設10/10、commit或push。

## 實作

Rust Runtime的新實例從request counter 0初始化。舊HUD若只等actor的下一個無連線frame才清除關聯，在立即重啟時可能保留舊LatestId，壓掉新實例的低ID。

新增Runtime共用OnGameplayInputScopeReset：每次明確Stop及成功建立新Runtime時同步發送；對既有實例的冪等Start不重設。HUD沿原建立／重綁／退休同時訂閱與解除，清除一般操作文字及64筆有界關聯，無須World tick、延遲、重建widget或放寬單調ID規則。既有selected source、controller／epoch檢查保留；不改商店權威receipt、退款、重試或遊戲效果。

## 驗收

- red-v1以真port-zero Runtime Stop重現唯一失敗：Direct runtime stop clears ordinary feedback immediately。原report／log／PNG保留。
- 修正後十六專項全部Success、零warning：直接Stop即時清除、舊結果不復活、無World tick重啟接受request 1並得到對應結果、第二次重啟可再用1、冪等Start保留pending、重複Stop及來源隔離／退休均通過。admission／result仍為受控fixture，不冒稱權威ACK。
- 普通UE5.8 NoEngineChanges／local／無action cache建置成功，UBT 50.25秒。
- 完整205 native成功（保留12筆既有warning）、458實際操作／222 wire／157 stage／13游標、69既有Blueprint零compiler error／warning／load failure且package不變、五份實際submix PCM與全部owned清理成功。
- owned原始錄影3555幀／11片段／114501270 bytes、591.967秒，RIFF／JPEG／index／SHA／timeline嚴格驗證。原860秒工作期限、900秒publisher壽命、事件順序、證明frame及8fps取樣保留。
- 127 Lua契約通過；3334來源／資產檔案集合、SHA、size與mtime驗收前後完全一致。527 Rust／ABI與93內容Lua未變，完整Rust／TD／all-target沿用2026-10-19-new-machine；本輪僅fresh公開map exporter單項。
- 長步驟依原程序creation identity每180000ms檢查並記錄final；原圖六張HUD與兩張map已直接檢視。錯誤與red／green證據記E1370。

可攜證據：`docs/acceptance/2026-10-20-runtime-input-scope/index.json`。原始log／AVI留在Saved。額外分項計時是功能驗收診斷，不代表固定效能接受；先前E1369的速度差仍未歸因單一原因。

## 接受界線

11父項仍需要實際權威ACK／完整能力對局、LAN、固定效能及100%／150%實際OS DPI配對等證據，目前只有150%實體monitor。fixture的control.ack不是原gameplay ACK，沒有把本輪接受當成完整MVP完成。
