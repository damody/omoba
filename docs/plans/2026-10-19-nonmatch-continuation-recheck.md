# 非新增對局範圍接續複核

2026-10-19。依使用者要求繼續檢查未完成工作，優先處理不新增對局的部分；維持既有10/10額度，不重設、不建立新權威對局。

## 結論

目前可在上述限制內完成的實作與本機驗收已完成，OpenSpec仍為33/44完成、11項待驗。本次沒有改動正式程式、資產、Rust或Lua工作流，也沒有新增可據以勾選父項的權威／OS DPI證據。以下複核支持沿用既有完整驗收，不冒稱重新執行完整UE或Rust測試。

## 本次實際執行

- `scripts/tests/owned_viewport_ipc_contract_test.lua`：127 cases passed。新log：`omfue/Saved/BuildTemp/nonmatch-continuation-contract-v1.log`。
- 對照已接受的source-v6，3331份來源／資產逐一核對SHA-256、大小、mtime及檔案集合，全部一致。新結果：`omfue/Saved/BuildTemp/nonmatch-continuation-source-verified-v1.json`。
- 41份可攜證據逐一核對SHA-256與大小，全部一致；同時檢查完整接受summary、203 native／69 Blueprint、458 owned cases／13 cursor及33/11任務狀態。
- 指紋程序進度由400、1600持續到3331，CPU有累計前進；期間沒有Unreal／Rust／C++編譯consumer殘留。本次沒有啟動新UE、後端或對局程序。
- 原驗收仍為`docs/plans/2026-10-19-nonmatch-final-acceptance.md`，可攜索引仍為`docs/acceptance/2026-10-19-nonmatch/index.json`。所有歷史成功／失敗資料保留，本次log為`omfue/Saved/BuildTemp/nonmatch-continuation-recheck-v1.log`。

## 11項仍待驗的具體條件

| 任務 | 尚缺證據 |
|---|---|
| 3.1 | 實際移動／攻擊命令的伺服器ACK及權威結果。 |
| 3.3 | AttackMoveClick／MoveOnlyClick／AttackOnlyClick的實際權威行為。 |
| 3.4 | Stop／Hold的權威區別及queue結果。 |
| 3.5 | 目標取得與auto attack政策的實際權威結果。 |
| 3.9 | 眼位放置、充能與生命周期、隊伍／fog在實際對局中的整合。 |
| 4.3 | move／attack／cast成功及拒絕回饋的權威ACK鏈。 |
| 4.4 | 購買／出售／undo與權威receipt完整鏈。 |
| 4.7 | 實際100%／150% OS DPI配對；既有環境證據只有150%。 |
| 7.1 | native及原Blueprint所有action連到原命令ACK的完整矩陣。 |
| 7.3 | 實際100%／150% OS DPI配對；其餘既有本機矩陣沿用。 |
| 7.4 | 可用條件下的真對局／LAN與固定效能證據。 |

本機未找到足以補齊上述新功能接受的既有權威錄製。OwnedViewportIpc的`control.ack.json`是測試控制階段回覆，不能充當gameplay ACK；synthetic公開frame也不能充當權威對局。實際PIE的DPI觀測保留原報告，不用text scale、改render size或合成DPI取代真OS DPI配對。

本次唯讀盤點錯誤及避免重犯的處理已記錄於`docs/plans/unreal-moba-error-register.md`的E1358。沒有新增功能失敗。沒有commit／push。
