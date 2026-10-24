# 世界Runtime生命週期與分段完整接受

2026-10-20。世界生命週期及suppressed HUD結果修正已通過當前功能驗證，完整本機清單改由兩個有界UE session完成。此次整套phased local acceptance成功；single_session_full_success=false，前三次單次全程860秒逾時仍保留，效能原因尚未證實。OpenSpec維持33/44、11父項待驗；沒有新增權威對局或重設10/10，未commit／push。

## 正式實作

world actor透過EnsureRuntimeScopeBinding訂閱／解除Runtime scope reset，直接subsystem Stop／Start亦會沿既有ResetRuntimePresentation釋放公開actor、target、cue／audio及frame／catalog游標。fresh快照可恢復公開actor，未連線owned HUD維持不可操作；冪等Start保留有效actor及輸入關聯。

抽出共用DispatchOwnedInputResults，suppressed HUD仍可處理合法control-only結果。正常snapshot保留先處理權威receipt再處理結果的順序，未預測退款或效果。修正前red與第一次green失敗已封存於前一包，不覆寫。

新增正式foundation phase，沿原full流程至shop-recipe-buy結束；既有tail從ward-ready-status接續。新ue_owned_phase_coverage逐段重新驗原生報告、逐筆wire／control history、Unreal index及清理證據，核對七個binary SHA一致、內容／ABI一致、run id不同，再確認原有順序聯集無遺漏／重複／增項。游標13筆另核對案例、觀察、座標及原生同一移動閾值。

入口為`tools\lua\lua.exe scripts\ue_local_acceptance.lua --ue-root D:/UE5.8 --run-id UNIQUE --owned-execution phased --record-video`；預設single保留。每段仍860秒操作／900秒publisher、90 proof frames與8fps，不降低案例或幀標準；不同session的狀態不宣稱連續，不能代替單session效能或LAN證據。

## 當前驗收

| 項目 | 結果 |
|---|---|
| 普通UE5.8 local／無action cache建置 | 成功，UBT11.27秒；stage SHA一致 |
| Lua契約 | 140 owned＋69 local acceptance成功 |
| 世界專項 | 18成功，RuntimePresentationReset四筆既有preview-world warning |
| 完整native | 205成功，保留既有warnings |
| 既有Blueprint | 69，零compiler error／warning |
| 實際音訊輸出 | 五PCM通過 |
| foundation | 294案例／120 intents／103 stages |
| tail | 164案例／102 intents／54 stages，13游標觀察 |
| 分段嚴格聯集 | 458／222／157，完整清單覆蓋 |
| 公開地圖錄影 | 兩map共144幀／12片段 |
| source／asset | 3335檔前後SHA／size／mtime一致 |
| Rust／內容Lua | 527／93輸入未變，完整Rust測試明示沿用前次接受 |

兩段完整錄影各自通過RIFF、frame timeline、8fps、索引、幀數及SHA檢查，仍是filtered synthetic feed，authority_ack=false。18專項的六張操作回饋與兩張公開map PNG已逐張檢視；六張HUD圖是在注入world snapshot前擷取，世界停止／恢復由實際actor及widget斷言證明。所有publisher／editor／profile已清理，三分鐘監測至root lifetime結束。

前三次單次全程失敗與當時feature-only接受仍在`docs/acceptance/2026-10-20-world-runtime-scope/index.json`，success=false不改；本包另列execution_mode=phased、single_session_full_success=false。分段清單覆蓋不會把authority ACK、100%／150%實際OS DPI配對、真LAN或固定效能的缺證父項勾選。所有錯誤與處置見unreal-moba-error-register.md E1377起。
