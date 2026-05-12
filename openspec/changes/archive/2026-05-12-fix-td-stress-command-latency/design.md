## Context

目前 KCP transport 使用單一 `out_tx/out_rx` 承載 lockstep frames 與 legacy `GameEvent` 廣播。`TickBatch` 被標為 urgent，但如果前面已累積大量 `creep.M`、`Creep.H`、`entity.F` 等普通事件，單一 FIFO 仍可能讓 `TickBatch` 排在 backlog 後面；反過來，urgent batch 也會 drain 已排隊的普通事件，造成真正影響 input 回放的資料被高量 rendering/event traffic 拖慢。

現有 `input-latency-metric` 已有 `input_render_latency:` 與 `input_latency_phase:`，但 HUD 的 `Lag` 只由成功 paired samples 計算；若某些 input 長時間卡在 pending、未 pair 或等待 snapshot handoff，HUD 可能仍顯示最近成功樣本的低 p50/p99，與玩家看到的秒級延遲不一致。

這次變更應優先修正「誤會」：讓指標能反映卡住的輸入，並讓 TD_STRESS 下 lockstep input path 不被高量非 input event starvation。所有 metadata 必須留在 wire-edge / client diagnostics，不能進 gameplay ECS、script ABI 或 state hash。

## Goals / Non-Goals

**Goals:**

- 讓 HUD `Lag` 與 debug log 能指出移動指令真正卡在哪一段，而不是只顯示成功樣本的樂觀數字。
- 保證 `TickBatch`、`InputSubmit` handling 與 applied input pairing 在 TD_STRESS event flood 下有優先處理路徑。
- 降低或消除 legacy high-volume events 對 lockstep frame broadcast 的反壓影響。
- 增加 regression 測試，覆蓋 400+ creeps 或等價 high-volume outbound backlog 下的 input latency。

**Non-Goals:**

- 不重新設計整個 networking protocol。
- 不移除 TD_STRESS 的 creep / tower gameplay 或改變壓測規模。
- 不改變 deterministic sim 的輸入語意、state hash 或 script ABI。
- 不把 client-local 或 server-local absolute timestamp 用於跨機相減。

## Decisions

### Decision 1: lockstep outbound 使用優先路徑，不再只依賴 single FIFO urgency

`TickBatch`、`StateHash`、`GameStart` 與 `SnapshotResp` SHALL 走 lockstep-priority outbound path，或在既有 broadcaster 中以等價方式先處理 lockstep queue。普通 `GameEvent` 批次仍可 dedupe 與 batching，但不得讓已排隊的普通事件阻塞下一個 `TickBatch`。

替代方案是繼續放大 `out_tx` buffer。這只能延後飽和，且會把玩家感知延遲藏在更深的 queue 裡，不解決 1 秒 input delay。

### Decision 2: urgent batch 不應無限制夾帶普通事件

目前 urgent first 會 `try_recv` drain 已排隊項目。修正後 urgent / lockstep flush 應限制普通事件夾帶量，或將普通事件留給下一個 normal batch。這可以避免一個 urgent `TickBatch` 在送出前先處理大量與 input 無關的 entity updates。

替代方案是完全停用 legacy events。這可能是長期方向，但範圍較大；本 change 只要求它們不得 starvation lockstep input path。

### Decision 3: HUD Lag 納入 pending age / stale input 診斷

`InputLatencyMeter` SHALL 保留 paired sample 的 p50/p99，但 HUD 與 log 也要顯示或納入 oldest pending input age、stale pending count、evicted unpaired count 或等價欄位。當玩家有 input pending 超過 paired p99 時，HUD 不得只顯示低 p50/p99 而讓使用者誤判。

替代方案是只看 `input_latency_phase:` log。這對開發者有用，但無法修正玩家看到 `Lag: 46ms` 與實際 1s 延遲矛盾的問題。

### Decision 4: phase trace 保持 metadata-only

新增或調整的 latency metadata 只能存在 transport、lockstep wire structs、client pending input book、latency meter 與 logs。server queue duration 繼續由 server-local duration 提供；client 只做 client-local timestamp 相減，避免 clock sync 假設。

替代方案是把 timing 寫進 ECS 或 gameplay resources 方便查詢。這會污染 deterministic state 與 hash 邊界，因此不採用。

## Risks / Trade-offs

- [Risk] lockstep priority path 可能讓普通 event 更新在極端壓測下被延後。→ Mitigation: 普通事件已多數可 dedupe，rendering 也可由 snapshots / local sim 補足；只要維持 bounded batch 與 drop/disconnect policy，玩家 input 應優先。
- [Risk] HUD 顯示 pending age 可能讓數字比 p50/p99 更跳動。→ Mitigation: 明確標示 paired `p50/p99` 與 `pending max` 或 `blocked` 狀態，避免混成單一難解讀數字。
- [Risk] 分離 queue 會碰到現有 `TransportHandle` API。→ Mitigation: 先用最小 API 變更；若需要新增 lockstep sender，保持舊 game event sender 行為不變。
- [Risk] regression test 若依賴完整 TD_STRESS 會慢。→ Mitigation: 優先建立 transport / broadcaster 層級的 synthetic backlog test，再補 smoke log 驗證。

## Migration Plan

1. 先新增 failing tests / diagnostics，重現 ordinary event backlog 下 `TickBatch` 被延遲與 HUD paired-only 低估延遲。
2. 實作 lockstep outbound priority 或等價的 bounded urgent handling。
3. 擴充 `InputLatencyMeter` 與 HUD/log，使 pending input age 與 evictions 可觀測。
4. 跑 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` 與 omfx 相關 tests。
5. 用 `run_stress.bat` 或較短 TD_STRESS smoke 檢查 `input_render_latency:`、`input_latency_phase:` 與 HUD `Lag` 不再互相矛盾。
