# 正式 60Hz 商店網路入口（2026-10-04）

## 本輪範圍與決定

依使用者最新指示，先完成 60Hz；不把 120Hz 效能或 Unreal 商店按鈕加進本輪驗收。

- Join／TeamGameStart 新增 shop protocol version 1 與完整 Lua rules hash。物品 catalog agreement、交易協定、玩家身分綁定是三個不同條件。只有 secure V2、已加入 player、single_lane 及相容規則才開交易；Story／舊版不開。
- 買賣沿正式 InputSubmit、InputBuffer shop journal、PlayerInput、authority ordered queue、ShopReceipt 與 OwnerEconomy 執行；不另造商店 world，不把 wire ID 放進 gameplay／script ABI。
- IPC 新增 numeric catalog ItemBuyIntent／six-slot ItemSellIntent。runtime 檢查本地 owner／view epoch／能力／catalog／slot，再轉成正式 PlayerInput。Unreal C ABI 與按鈕尚未接上，原生 UI 仍唯讀。
- 同 renderer shop request ID 同 payload 只觸發原 wire ID 的結果查詢；改 payload 拒絕。1024 筆歷史滿額時拒絕新交易而不逐出防重放記錄。此記憶只限目前 runtime lifetime，沒有假稱跨程序持久化。
- 結果查詢從已加入 session 推導 player，最大 32-byte query、每 session 每 monotonic 牆鐘秒最多 8 次。暫停玩法不凍結 query budget，client timestamp 無法增加額度。
- runtime 最多 64 個待查 ID、每秒最多查最久未查的 4 筆，不重新提交交易。不確定 write error 保留原 ID／回報 SHOP_TRANSPORT_UNCERTAIN，不假設沒成交。terminal／expired／late 停止查詢；expired 標示不確定，不換 ID 補買。unknown／pending 仍待查，滿額背壓，不冒充成功。
- 主／catch-up inbound pump 都驗證 replay、保留原 settled tick、恢復原 renderer request 結果；只有 presentation／correlation 改變，不扣錢、不改 Inventory／hash。

## 驗證方式

`OMOBA_SHOP_TRANSACTION_SMOKE=1` 明確 opt-in 到既有 Lua smoke。正常開局不自動買賣。network fps 預設60，可明確選正式允許的90／120；本輪只驗60。presentation30、出生0Gold、暖場2秒、每active秒2Gold。

兩隊依序不足金錢買入拒絕 → 累積350後買 moba_sword → 賣出；每次正式wire input以完全相同ID／tick／payload額外重送5次。成功後查原始buy terminal結果。210秒截止不延長，門檻依negotiated fps，不用牆鐘冒充玩法時間。

opt-in `real_single_lane_shop_transaction_capture` 串流解析原始 protobuf capture，逐snapshot核對自己的player／team、immutable receipt、Gold=正式收入−成功buy350＋成功sell175、六格／最多一件物品。缺少三筆原始拒絕／買／賣receipt、沒有非空inventory或錯誤餘額均失敗。

## 已完成回歸

- core 322 passed；server152 passed、1 ignored；runtime48 lib＋3 bin passed、2真實capture opt-in ignored；bridge44 passed、1 external capture ignored；Fyrox cargo check --tests通過。
- 新測試涵蓋 rules／catalog／protocol錯配、query額度及clock倒退、bound owner／capability、IPC catalog／slot／owner／epoch、renderer重複／衝突不配新ID、待查上限／節流與公平性。
- scoped diff whitespace check通過。proto fallback透過既有vendored protoc重新生成，沒有手寫generated Rust。

## 真實60Hz驗收通過

- run `moba-runtime-1791051305`，tick_rate_hz=60、success=true、cleanup_verified=true。server27264、runtime93612／72724均已清理。
- 兩隊三方canonical hash各89個checkpoints PASS，最後驗證tick10680、safe tick10751，無FAIL／repair。物品改裝在server／filtered runtime／observer間保持一致。
- 兩隊input1原始不足金錢拒絕(code6)、input2成功買sword(code0)、input3成功sell(code0)；每筆額外完全相同wire input重送5次。日誌確認原input2查詢status2／terminal=true，不重新交易。
- 原始protobuf capture checker通過：team1共5554 snapshots、team2共5518；各3筆immutable receipts，確實觀察到裝備一件sword，再出售回空格。每snapshot金錢均精確等於玩法時間收入−成功買入350＋出售175；最後皆193Gold（交易後繼續累積收入），不是把重送ACK當成功或注入測試餘額。
- checker額外核對兩隊原input2terminal replay日誌、三筆各5次重送與無conflicting receipt警告。IPC3與C ABI5版本不變，新intent為additive欄位。
- 正式runtime binary已重新建置，最新read-only recovery與不確定write處理有48＋3回歸測試。完整60Hzrun是在原terminal query路徑完成；後加的renderer重送／recovered critical result／不確定write處理屬單元／建置驗證，沒有假稱故障注入網路測試。

OpenSpec保持17/30，整項4.1／5.3／6.2仍不勾選。此次封關的是正式60Hz商店交易／重送／terminal查詢／完整金錢与inventory投影。Unreal商店按鈕、C ABI輸入export、真實UE非空item／receipt畫面、跨runtime重啟pending恢復／斷線故障注入未完成；120Hz依使用者本輪指示延後。本輪沒有重啟Editor／stage新bridge DLL或執行PIE，不引用前輪畫面回歸為本輪網路證據。

## 保留的失敗證據

- 1791050713：120Hz nominal在210秒截止只走到tick18311，尚無足夠Gold；capture只有不足金錢receipt，checker失敗，三程序清理。不能假稱120Hz成功。
- 1791051137：誤用headless15Hz作network，server validator拒絕；單一server清理。
- 1791051262：誤用presentation10Hz，runtime validator拒絕；server／runtime清理。
- 後續查實際validators改正式60Hz／30Hz，不鬆綁驗證、不注入Gold。完整錯誤與預防見E094。
