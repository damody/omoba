# 玩家視角 Lockstep 最小差異恢復設計

## 目標

當玩家端 team replica 因未知單位、component 差異、hash 不一致或無法補齊的 sequence gap 拒絕增量幀時，不關閉遊戲。Client 保留最後一個已驗證畫面，由 server 依該 team 當下可公開資訊送出最小修復；只有最小修復無法安全收斂時，才升級為 filtered rebase。

## 核心原則

- Server authoritative 永遠是唯一正確狀態。
- Client 只使用 replica ID 回報錯誤，不得要求 canonical entity，也不得指定想取得的 component。
- Server 必須重新檢查 team visibility、disclosure epoch 與 allowlist，才決定修復內容。
- 修復封包不得包含該 team 當下不可見或已忘記的資訊。
- 發生可恢復錯誤時，renderer、client runtime 與連線保持存活。
- Server 內的 Team 1／Team 2 observer replica 使用相同修復封包驗算。

## 恢復層級

### 第一級：ComponentRepair

Server 能證明 entity 仍對該 team 公開，且只有 allowlist 內的 component 不一致時，只送該 component 的 authoritative replacement fields。

### 第二級：EntityReplace 或 DependencyBundle

Entity 缺失、epoch 不一致或安全 baseline 無法由 component repair 恢復時，server 送完整的 team-safe entity baseline。若 entity 依賴其他目前可公開 entity，bundle 以依賴優先的拓撲順序送出。不可公開的依賴必須從投影事件移除或轉成 sanitized external effect，不能因修復而揭露。

### 第三級：FilteredRebase

Server 無法唯一判定最小修復、差異超過門檻、修復重試失敗、sequence replay 已過期，或 observer replica 仍不收斂時，才建立該 team 的 filtered snapshot。快照成功驗證後，client 丟棄舊增量佇列並從 manifest 指定的 resume sequence 繼續。

## Client 狀態機

正常狀態收到可恢復的 frame apply error 時，client 保存失敗幀與最後已驗證 sequence，進入 `AwaitingRepair`。此狀態停止套用後續增量幀，但持續接收並有界暫存資料、維持 renderer IPC，畫面顯示最後一個安全狀態。Client 送出一次具節流與 request ID 的 repair report。

收到修復後先驗證 team、view epoch、authority revision、allowlist、可見性證明及 request ID，再套用修復並重試原失敗幀。成功後依序清空暫存佇列。收到 filtered rebase 時以既有 staging 與 manifest 驗證流程原子替換 replica。

同一故障不可無限重試。重複修復沒有進展時依序升級；只有簽章／manifest 驗證失敗、跨 team 資料、協定版本不相容或恢復次數耗盡，才把該 session 標記為 unsafe 並結束該 client。

## Server 與 observer 行為

Server 將 client report 當成診斷提示，不信任 client 對錯誤原因的判斷。AuthorityRepairCoordinator 根據 authoritative team projection 產生修復。每份修復先送入同 process 對應 team 的 observer thread；observer 成功套用並收斂後才允許送往玩家。

Observer 遇到一般 `UnknownEntity` 不永久停止。它回報含 phase、operation、replica ID、disclosure epoch、sequence 的結構化差異，進入相同分級恢復流程。若 server 自己產生的原始幀在 observer 上立即失敗，server 必須先修正或捨棄該幀，不得把非法幀送給 client。

## Outbound 與生命週期

修復控制訊息與一般 team frame 使用同一條有序 team stream。修復期間該 session 的增量幀保留在有界 replay buffer；buffer 滿載時阻塞 authoritative outbound，而不是遺失順序。`run_2player.bat` 只有在 client/runtime process 真正退出時才執行整場清理；`AwaitingRepair` 不視為程序失敗。

## 診斷

所有 frame rejection 與修復決策記錄：team、session、request ID、server tick、replica tick、sequence、phase、operation、replica ID、disclosure epoch、authority revision、選擇的恢復層級、封包 bytes、嘗試次數與結果。不得把 canonical ID 寫入 client 可見 log 或封包。

## 測試策略

實作完成後才集中測試：

1. Unit tests 覆蓋 component repair、entity replace、dependency ordering、hide/forget 與過期修復。
2. Contract tests 證明 repair report 不能取得視野外資料，且 server 對惡意 replica ID 回報採 fail-closed。
3. Fault injection 在 `pre-step`、`step`、`post-step` 各製造 `UnknownEntity`，確認視窗不關閉並能恢復。
4. 三程序測試啟動 server、Team 1 runtime、Team 2 runtime，穿越視野與遮蔽物並持續移動。
5. 驗證一般錯誤只使用小型修復；只有門檻條件才 filtered rebase。
6. 長時間 release soak 重現 sequence 5946 類型的視野邊界變化，確認兩個 observer、兩個 client 與 renderer 都持續運作。
7. 最後執行 workspace 既有完整測試與 `run_2player.bat` lifecycle 驗證。

## 成功條件

- 可恢復的 replica 錯誤不再造成遊戲視窗或整場程序關閉。
- 修復後 client hash 與 server team observer 收斂，sequence 連續。
- 修復內容嚴格受 team visibility 與 allowlist 限制。
- 常見單 entity 問題不傳完整快照，且修復不造成可感知卡頓。
- 無法安全修復時能自動升級 filtered rebase，而不是繼續使用錯誤狀態。
