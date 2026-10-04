# 商店原始結果保存與查詢（2026-10-04）

## 計畫與實作決定

1. 在已有 admission journal／ID 恢復上保存原始 terminal receipt，先建立查詢恢復，再接正常購買 UI。secure buy／sell gate 本輪保持關閉。
2. `InputBuffer` 的每筆 shop record 保存 optional receipt；只有正式 authority filtered frame 的 kind 23、subject=None 結算事件可以 finalize。必須匹配 player、input ID、admitted effective tick、Buy 的 compiled catalog ID 或 Sell 的原 slot，禁止以 Accepted／DuplicateShop 當成交。
3. 首個 terminal receipt 固定不變；相同 receipt 重複 finalize 冪等，改 result／tick／catalog／slot／player 拒絕。最多 1024 筆，逐出仍保存 watermark，舊 request 不會再次排入。這是有界記憶體記錄，不是持久化帳本或 exactly-once 保證。
4. 新增獨立 KCP tags 0x2B／0x2C 與 `ShopReceiptQuery`／`ShopReceiptReplay`。query 只有 request ID 與 input ID，沒有 client-selected player；server 要求已加入的 secure V2 session，player 取自該 session，回覆單播到同一 socket。query 不經 InputSubmit、不分配 ID、不執行玩法。
5. status 明確為 unknown=0、pending=1、terminal=2、expired=3、admission-late-rejected=4。只有 terminal 有原始 40-byte typed receipt；其他結果不得夾帶 receipt，更不以未知或尚未結算冒充成功。
6. runtime 驗證 schema、owner、request／input ID 與 typed payload，保留原 settled tick，僅寫 presentation history。主 select 與 catch-up pump 均處理 recovery；未套用交易、Gold／Inventory／replica tick／hash。history 同一 input ID 的不同內容不新增第二筆。下一個 snapshot 沿用既有 IPC／C ABI 5 呈現，不改 Unreal C++ 或 Blueprint。
7. 沒有 pending shop 不做額外 frame decode；history 保存 pending 計數，finalize／eviction 更新，不每 tick 掃 1024 筆已結算結果。尚未建立新的效能量測，不宣稱 overhead 已量化。

## 驗證

- core 全 lib：321 passed；replay schema／owner／ID／status／missing payload／nonterminal payload 驗證通過。
- server 全 lib：151 passed／1 ignored。新測試涵蓋 immutable receipt、重送不重排、unknown／pending／expired／late、逐出後不能 finalize、Sell slot／player／retarget tick。
- generated catalog 交易核心／投影／journal 整合單元測試：fixture Gold 1000 購買 moba_sword 後 650／一件裝備；正式 receipt result=0。完全相同重送與 read-only recovery 各五次，pending queue 始終為空、Gold 650、一件裝備。這是 unit fixture，不對真實對局注入餘額；也不冒充完整 ECS／KCP／UE 正向購買驗收。
- runtime：46 lib＋3 bin passed；PresentationHub 測試接受歷史 settled tick 70、重複回覆只保留一筆、錯 owner 拒絕，既有 economy Gold 550／slot 不因 replay 改變，persistent renderer baseline 保留結果。
- bridge：44 passed／1 real-capture opt-in ignored。
- Fyrox `cargo check --manifest-path omfx/Cargo.toml --tests` 通過；legacy 路徑不發 recovery query，收到新 variant 只明確警告，不送入 gameplay。
- targeted rustfmt／scoped whitespace check 通過。既有 dead-code、unused-world、protoc fallback 與 CRLF 提示仍存在。

### 真實查詢與正常重連

以固定 Lua 執行 `scripts/run_moba_runtime_smoke.lua`，設定 `OMOBA_SHOP_QUERY_SMOKE=1`、`OMOBA_RUNTIME_RECONNECT_SMOKE=1`。測試旗標必須明確轉交 child env；正常遊戲沒有自動測試查詢。實際 server／DLL／兩個 runtimes 不開放商店，只查自己不存在的 input ID u32::MAX。

成功 run：`target/interactive-runs/moba-runtime-1791048834/moba-runtime-smoke-report.json`。

- success=true／cleanup_verified=true；兩隊 runtime 日誌確認 `status=0 terminal=false`，新玩家 1 runtime 亦收到相同 unknown 回覆。
- read-only query 沒有消耗 MAX ID：原移動 ID=1、重連 floor=1、新移動 ID=2。
- 實際移動與重連後 4 個 checkpoint PASS；雙隊三方 pre/post hash 14／12 PASS，最後 tick 1680，零 FAIL／無 repair。
- server 49336、舊 runtime 60056、隊伍 2 runtime 86096、重連 runtime 35476 均退出；報告與程序查詢核對。
- 本輪未重建 Unreal／執行 PIE／實際 UE 商店，也沒有真實 KCP terminal 購買回覆；不能把 unknown query 視為所有交易成功／失敗網路驗收。

## 錯誤與限制

- Fyrox 新 inbound variant 漏更新 exhaustive match，已修正。
- run 1791048551 因 child env 漏傳測試旗標失敗；run 1791048712 因 catch-up drain 忽略 replay 訊息失敗。失敗報告保留，三程序清理均成功；修正版真實重跑通過。細節見 error register E092。
- 查詢 API 不等於自動重試／pending request 持久化或 runtime 新程序自動找回所有已丟失 correlation。沒有近期結果列舉／bootstrap 恢復，也沒有獨立 query capability 握手與 query rate limit；正式商店開放前一併處理。
- 若 pending request 在 journal 滿後被逐出，交易可能已執行而查詢只能回 expired；不能據此換新 ID 自動重購。原始同 ID replay 仍因 watermark 拒絕。
- server 重啟清空記憶體記錄；crash／斷網 session timeout 仍未封關，正常重連才有本輪證據。

## 下一步

1. 在正式 shop intent 接上時，同步加入 query capability／節流與 pending recovery 規則；明確處理 pending／expired，禁止以新 ID 自動重購。
2. 通過真實 KCP Buy 拒絕／成功、receipt finalize／重查、自己 Gold／六格與完整 filtered hash；成功交易需要正式收入規則，不靠測試餘額。
3. 接原生 UE 商店操作，完成真實非空裝備、出售與 renderer／runtime 重連呈現。
4. 完成金錢收入／擊殺助攻／回城，再推進三路、Bot 及完整 UI。

OpenSpec 仍 17/30；4.1／5.3／6.2 未完成，不勾選。
