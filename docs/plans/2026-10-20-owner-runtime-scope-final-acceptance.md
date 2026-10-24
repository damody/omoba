# 輸入owner Runtime生命週期與完整本機驗收

2026-10-20。補齊3.9／7.3直接Runtime Stop／Start跨兩個World tick的輸入owner退休缺口。父項仍33/44、11待驗；沒有新對局、重設10/10、commit或push。

## 實作及回歸

輸入owner原本只在direction refresh訂閱命令admitted，沒有Runtime scope reset。red-v1以真port-zero Runtime Stop重現唯一失敗：Direct runtime stop retires ward admission immediately，預期0實為997。原report／engine.log／六張PNG保留。

共用EnsureRuntimeInputBindings在owner建立／重用／BeginPlay即綁定命令與Runtime重設；direction refresh沿相同helper維護，Runtime替換及RetireInput一起解除兩種訂閱。scope reset只退休本機暫態：方向model以原Retire保留實體按鍵中立規則，丟棄end intent，不在停止回呼送命令；清除analog／pointer hold／確認與溝通release、瞄準／cast hold、舊hero／ward／能力及道具資格、舊方向scheme與combat preference觀察。保留owner及使用者profile，等待fresh公開投影恢復。

十七專項Success／零warning，包含完整既有WardAdmissionRetirement。新增真Runtime整合斷言驗997停止即時歸零、舊ward投影失效、owner仍可重用、重啟不能以舊資格接998、新hero／ward公開投影後接受request 1、冪等Start保留live ward admission、重複Stop清除新關聯。商店及一般操作生命週期、來源隔離、退休、single draw、原BP／native PIE回歸同時通過。ward renderer admission／result仍為受控fixture，不是權威ACK或真正落地守衛證據。

## 完整驗收

- 普通UE5.8 NoEngineChanges／local／無action cache成功，UBT 36.78秒。
- 205 native全成功，保留12筆既有warning；458實際視窗操作／222 wire／157 control stage／13游標，全部owned程序及profile清理成功。
- 69既有Blueprint零compiler error／warning／load failure，資產未變；五份實際submix PCM成功。
- 原始錄影3583幀／13片段／114805748 bytes、594.610秒，RIFF／JPEG／index／SHA／timeline嚴格驗證。保留860秒工作期限、900秒publisher壽命、事件順序、證明frame與8fps取樣。
- 127 Lua契約通過；3334來源／資產集合、SHA、size與mtime驗收前後一致。527 Rust／ABI及93內容Lua未變，完整Rust／TD／all-target明示沿用2026-10-19-new-machine；本輪fresh公開map exporter單項。
- 長步驟依程序creation identity每180000ms檢查及記錄final；六張HUD與兩張公開map原圖已直接檢視。一般操作PNG不冒稱新增ward落地／退休截圖；新增生命週期由真Runtime與實際owner／widget狀態斷言驗證。

22份可攜證據：`docs/acceptance/2026-10-20-owner-runtime-scope/index.json`。raw log／AVI保留Saved。E1376記錄red→green與共用修正。分項計時不是固定效能接受；先前E1369速度差仍未歸因單一原因。

## 接受界線

11父項仍缺真權威ACK／完整能力對局、LAN、固定效能或100%／150%實際OS DPI配對等證據，目前只實測150%實體monitor。沒有新增對局或將fixture control.ack當原gameplay ACK，不把本輪子範圍當完整MVP完成。
