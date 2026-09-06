## Context

Protocol v2 已有 `ComponentRepair`、`EntityReplace`、filtered rebase、team replay buffer 與 server observer replica，但 client runtime 目前把一般 frame apply error 升級成 `UnsafeSession` 並退出。sequence 5946 的實際紀錄顯示 Team 2 external runtime 與 server observer 同時得到 `UnknownEntity`；launcher 隨後依 fail-fast 規則清理所有程序。系統需要把既有 recovery primitives 接成不中斷 renderer 的完整閉環，同時修正 server 在送出前未攔截非法 team frame 的缺口。

## Goals / Non-Goals

**Goals:**

- 以最小安全差異恢復可恢復的 team replica 分歧。
- 保持 renderer、client runtime、連線與最後安全畫面存活。
- 由 server 依 team projection 決定修復資料，維持戰爭迷霧隔離。
- 讓 server observer 在玩家收到資料前驗證同一份 frame／repair。
- 讓沒有進展的修復自動升級至 filtered rebase。
- 提供足以定位 entity lifecycle 錯誤的結構化診斷。

**Non-Goals:**

- 不讓 client 自行預測缺失的 server 狀態。
- 不建立 canonical ID 查詢 API。
- 不修改兩隊固定拓撲、120 Hz authoritative tick 或 renderer 視覺設計。
- 不以 repair 取代正常 steady-state lockstep replication。

## Decisions

### 1. 使用 server-decided 三級恢復

恢復順序固定為 `ComponentRepair`、`EntityReplace`／dependency bundle、filtered rebase。Client report 只描述套用位置與 opaque replica identity；server 以 authoritative projection 重新計算差異。相較於每次完整 rebase，此方案降低流量與停頓；相較於 client 指定 component，此方案不會建立戰爭迷霧 probing oracle。

### 2. Frame apply 必須具備交易性

Client 在 clone／staging replica 上套用一個完整 frame。成功才提交 world、tick 與 sequence；失敗時保留原 world 與失敗幀。這避免 `pre-step` 已刪除 entity、`post-step` 才失敗所造成的半套用狀態，也讓修復後能安全重試同一幀。

### 3. 可恢復錯誤進入 `AwaitingRepair`

`UnknownEntity`、`StaleDisclosureEpoch`、允許修復的 hash mismatch 與 replay 過期進入等待狀態。Client 暫停套用後續增量幀並使用既有有序接收佇列做有界緩衝，renderer IPC 持續發布最後已提交 snapshot。跨 team、未 allowlist component、manifest 驗證失敗及 protocol incompatibility 仍立即 fail-closed。

### 4. 沿用既有 recovery protocol

可由三方 checkpoint 定位的 component／entity 差異繼續使用既有 `ComponentRepair` 與 `EntityReplace`，不增加 steady-state 流量。Frame apply error 會在 client log 記錄 phase、operation、opaque replica ID 與 epoch，並以既有 `ClientTeamHashMismatch` 要求 authoritative recovery。Server 無法從安全資訊唯一建立小型修復時才送 filtered rebase。

### 5. Server observer 與 external runtime 使用相同驗證

兩隊 observer thread 與 external runtime 套用相同 frame。Projector 在建立 frame 時先移除已失效的 repair／replace 與 reveal tick 的重複事件，避免送出已知非法 lifecycle 組合。Observer mismatch 繼續使用既有 authority recovery；client 即使先收到未預期錯誤，也會保持視窗並等待 filtered rebase。

### 6. 視野生命週期採 dependency-first 與 stale-reference pruning

同一幀的 reveal 依 dependency graph 拓撲排序。Hide／Forget 前，projector 必須移除或轉換同幀中指向即將隱藏 entity 的 accepted input、external effect、repair 與 replace。排隊 repair 在送出時重新驗證 mapping、epoch 與目前 visibility；不再有效的 repair 直接丟棄或升級，不得引用已刪除 replica entity。

### 7. 有界升級與流量控制

同一 sequence 的第一個明確 component 差異使用 repair；entity／epoch 問題使用 replace bundle。兩次沒有 hash 進展、差異超過設定門檻、dependency closure 無法安全建立或 replay window 過期時使用 filtered rebase。每個 session 同時只能有一個 active recovery request，避免 repair storm。

## Risks / Trade-offs

- [失敗幀緩衝增加記憶體] → 使用既有 replay window 上限；超限直接升級 filtered rebase。
- [Repair report 被用來探測] → Client 只能回報 opaque ID；server 對不存在或不可見 ID 回傳不含 entity 資料的 generic recovery decision。
- [Observer gate 增加 outbound latency] → 兩隊 observer 維持獨立 thread，驗算與投影平行；steady-state 不產生 repair payload。
- [錯誤分類不完整導致循環] → 記錄 progress token／hash，沒有進展即升級，達上限才安全終止。
- [舊 client 不認識新控制訊息] → 以 protocol v2 可選 tag 擴充；server 只對宣告 targeted recovery capability 的 session 啟用，否則沿用 filtered rebase／disconnect 行為。

## Migration Plan

1. 先擴充共用 schema、錯誤上下文與交易性 frame apply，不改 wire 預設行為。
2. 接上 server repair decision 與 observer outbound gate。
3. 接上 client `AwaitingRepair` 與修復重試。
4. 對 protocol v2 capability negotiation 開啟 targeted recovery。
5. 最後以 fault injection、三程序與 release soak 驗證，再成為 `run_2player.bat` 預設。

回滾時可停用 capability negotiation，讓 server 直接使用既有 filtered rebase；不需要改回 renderer 或 authoritative world 格式。

## Open Questions

沒有待使用者決定的項目。實作細節以安全優先：無法證明最小修復不洩漏資訊時，一律升級 filtered rebase。
