# Runtime共用操作回覆與完整本機驗收

2026-10-20。本輪修正直接Runtime提交繞過HUD的缺口，完成可在不新增對局下實作的送出回覆接線。OpenSpec維持33/44、11父項待驗；沒有新對局、重設10/10、commit或push。

## 實作

成功SubmitGameplayInputEvent統一發布實際wire player、原request id與transport status，涵蓋直接Stop／Hold、legacy controller、世界／Blueprint及小地圖。原方向admission事件保留。移除WorldBridge與小地圖的分散通知，避免新增路徑時再次漏接；HUD owner只接受自身registered local controller、同world、目前widget及OmMatchSessionSource的captured controller／epoch權杖，重綁／退休先解除訂閱。Stop／Hold的本機送出失敗也接回既有拒絕提示。

沿用十種操作、64筆／10秒的有界關聯；送出不是權威ACK或遊戲效果。商店、ward、方向與偏好保持各自的正式結果投影，Busy／未知結果不偽裝成功，不重送。

## 驗收

- 普通UE5.8 NoEngineChanges／local／無action cache建置成功，UBT 10.30秒。
- 十六專項全部Success、零warning：包含Runtime事件到實際Slate文字、外來owner拒絕、Stop／Hold同request結果關聯、未選中source拒絕／Acquire後才通知、退休不復活、四種resize單次繪製與兩張公開map。
- 完整205 native成功，保留12筆既有warning；458實際操作、222 wire、157控制stage、13游標案例通過。正式owned測試新增每次Stop／Hold真正按鍵送出到live HUD的文字斷言，不能以模型kind測試代替。
- 69既有Blueprint零compiler error／warning／load failure，原package不變；五份實際submix PCM與全部owned清理成功。
- 原始owned錄影3561幀、11片段、114007720 bytes，實際595.246秒；原期限、事件順序與8fps取樣保留，RIFF／JPEG／index／SHA／timeline嚴格驗證。
- 127 Lua契約通過；3334來源／資產檔案集合、SHA、size及mtime驗收前後一致。527 Rust／ABI與93內容Lua未變，完整Rust／TD／all-target沿用2026-10-19-new-machine；僅公開map exporter單項fresh執行，不冒稱重跑全Rust。
- 長步驟按原程序creation identity每180000ms檢查進度，完成後記錄final；本輪錯誤與修復記E1366–E1369。六張HUD原圖與兩張map原圖已直接檢視。

中間v1的205 native與69 Blueprint子項通過，但owned操作超時，整輪仍為失敗；原報告／AVI保留。新增每600個實際engine frame的World tick、驗證工作及其餘引擎工作分項計時，未增加tick或減少證明frame。獨立溝通段83案例、十三筆計時及錄影通過，再以最終來源完成v2整輪。兩輪速度差異尚未歸因單一原因，不宣稱已完成固定效能接受；原860秒工作期限與900秒publisher壽命不變。clock紀錄的maxTick是以1/60秒作查詢的probe，不能當成實測FPS上限。

可攜證據：`docs/acceptance/2026-10-20-runtime-feedback/index.json`。原始log／AVI留在Saved，證據包保存摘要、來源指紋與未修改PNG，不納入暫存二進位。

## 接受界線

本輪是實作及完整本機回歸，沒有新增權威對局；fixture的control.ack不是原gameplay ACK。11父項仍需要實際權威ACK／完整能力對局、LAN、固定效能及100%／150%實際OS DPI配對等證據，目前只有150%實體monitor。不得把這份接受改寫成完整MVP完成。
