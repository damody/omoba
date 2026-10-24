# 世界Runtime生命週期：功能已驗，完整流程未通過

2026-10-20。當前兩項共用修正已通過18項專項；完整viewport事件驗收三輪都超出原期限，不能宣告整體完成。OpenSpec仍33/44、11父項待驗。沒有新增對局、重設10/10、commit或push。

## 實作與功能證據

world actor原公開StartRuntime／StopRuntime會清理呈現，直接subsystem生命週期卻未通知actor。red-v1以真port-zero Runtime及實際92:1公開actor／mode1快照重現唯一失敗：Direct runtime stop invalidates the world presentation immediately。

新增EnsureRuntimeScopeBinding，從PostInitializeComponents／BeginPlay訂閱Runtime scope reset，Start／Stop及Tick維護Runtime替換，EndPlay解除；沿既有ResetRuntimePresentation清除公開actor、route／occluder、target、cue／audio、frame／catalog游標與HUD。沒有在回呼重啟Runtime，冪等Start保留live actor。

green-v1的18項中17成功，ShopUndoProjectionAndPin三斷言失敗。兩個mode恢復斷言誤把未連線port-zero當已連線，改驗fresh公開actor恢復、owned HUD仍unavailable；第三個control-only結果被bSuppressOwnedHud早退略過。抽共用DispatchOwnedInputResults供正常及suppressed路徑，完整snapshot仍先套權威receipt再派送結果，不預測效果或退款。原失敗不覆寫。

green-v2十八專項全部Success，保留四筆與前一輪一致的RuntimePresentationReset preview-world no-context warning。真Runtime整合驗停止立即mode0／舊actor釋放、新Runtime等待新快照、同快照恢復公開actor、未連線owned HUD維持0、冪等Start保留actor及ward／undo關聯、新低ID結果可處理、舊結果不復活。來源隔離、single draw、原BP／native PIE等回歸也成功。frame／admission／result皆受控fixture，不是新權威ACK。

普通UE5.8 NoEngineChanges／local／無action cache建置成功，UBT 30.64秒；127 Lua契約及stage SHA成功。三輪各自205 native成功（12既有warning）、69既有Blueprint零compiler error／warning／load failure、五PCM成功。這些是成功分項，不能當成整套viewport驗收通過。

## 完整驗收尚未通過

| 完整嘗試 | 全程錄影 | 結果 | 最後control stages／wire |
|---|---|---|---|
| world-scope-final-v1 | 開 | 原860秒期限超時 | 147／209 |
| world-scope-final-v2 | 開 | 同期限超時 | 144／200 |
| world-scope-final-v3 | 關，工具原預設 | 同期限超時 | 153／215 |

三輪publisher／editor／profile都清理成功，過程持續前進，不是已證明死鎖。未延長860／900秒，未降低case／frame／8fps標準；正式來源沒有改動。關閉錄影仍逾時，因此不能將錄影當成唯一原因或宣稱效能已修復。尚無本輪fresh完整458案例成功證據；前一輪owner-runtime-scope的完整458只保留為歷史參考，不能冒稱本輪通過。

已讀UE5.8本機原始碼：Editor背景CPU節流在unattended路徑停用；Slate.bAllowThrottling=1僅表示允許節流，不證明active ThrottleCount。GetMaxTickRate(1/60)的60亦不是實際FPS。未以猜測修改Engine、使用者節流／FPS設定或降低畫面品質。

## 證據與接受界線

- 3334來源／資產的檔案集合、SHA、size與mtime全部嘗試前後一致。
- 527 Rust／ABI及93內容Lua未變，完整Rust／TD／all-target明示沿用2026-10-19-new-machine；本輪fresh公開map exporter單項。
- 兩張公開map的錄影144幀／12片段嚴格驗證成功，六張HUD與兩張map原圖已直接檢視。這是獨立fixture，不能替代失敗的全程錄影或完整事件流程。
- 三輪長步驟每180000ms按creation identity檢查並記錄final；沒有遺留受管UE／publisher程序。
- E1377–E1382保留world來源缺口、green失敗修正、搜尋路徑失誤及三輪期限失敗。原report／log／AVI均保留Saved。

27份封存見`docs/acceptance/2026-10-20-world-runtime-scope/index.json`。索引明確success=false、feature_success=true、full_acceptance_success=false。本輪僅接受已通過專項的實作，不稱整體完成。

11父項仍缺真權威ACK／完整能力對局、LAN、固定效能或100%／150%實際OS DPI配對等證據，目前只實測150%實體monitor。fixture control.ack不是原gameplay ACK，10/10不重設。
