## Context

目前 `omfx/game/src/sim_runner.rs` 的 `receive_tick_batch` span 包住 `tick_input_rx.recv_timeout`，因此健康 120 TPS cadence 下等待下一個 `TickBatch` 的 8.33ms idle 可能被看成 active work。這個發現不能直接推論「問題只是 span 標錯」；使用者的實際問題是 `omfx::sim_runner::tick` 看起來只花 1-3ms，整體卻無法穩定 120 FPS，代表瓶頸可能在 frame pacing、OS timer granularity、vsync/present、render-thread snapshot consume、sim-to-render mutex handoff、lockstep event drain jitter、logging/profiling overhead，或多個小 jitter 疊加。

既有 `add-omfx-perfetto-profiling` 已建立 opt-in Perfetto workflow 與粗粒度 spans。這個 change 應站在該基礎上，把 120 FPS stress 驗收需要的 critical path 補齊，並把量測結果用來修正實際掉幀原因，而不是只改 trace 名稱。

## Goals / Non-Goals

**Goals:**

- 建立可驗收的 120 FPS SLO：10k render-backed units stress 場景 release run，平均 FPS 達 target，1% low FPS >= 119。
- 將 frame-to-sim-to-present pipeline 拆成可歸因的 spans/counters：lockstep receive、event drain/forward、sim wait、active receive、tick work、snapshot publish、snapshot consume、render update、frame pacing sleep/yield、present interval。
- 分離 idle wait、blocking wait、sleep/vsync/present wait、mutex/channel contention、active CPU work 與 logging/profiling overhead。
- 找到 FPS/1% low 未達標的真因後實作修正；若第一個假設只是 span 標錯，仍必須繼續追下一個瓶頸。
- 在 bounded jitter 下維持穩定：TickBatch arrival 或 OS scheduling 有短暫 jitter 時，sim_runner/backlog/pacing 機制應吸收，不讓 render 1% low 掉到 119 以下。

**Non-Goals:**

- 不更動 deterministic gameplay timestep、KCP/lockstep wire protocol、script ABI 或 state hash。
- 不用跳 simulation tick、丟 TickBatch、停止 entity render update、降低 unit 數或降低 lockstep TPS 達成指標。
- 不要求 Perfetto always-on；正式 FPS/1% low 驗收必須 profiling disabled 或只用低 overhead frame-time counters。
- 不把 120 FPS 未達標單純歸因於 `receive_tick_batch` span；該 span 是診斷入口，不是結論。

## Decisions

1. 先建立 frame-time SLO 與 low-overhead histogram，再看 Perfetto deep trace。

理由：Perfetto trace 本身可能增加 overhead，不能作為最終 FPS 驗收來源。release stress run 應輸出 frame count、平均 FPS、p50/p95/p99 frame time、1% low FPS、max frame time、sim TPS、queue backlog 等低 overhead summary；Perfetto 用於定位，不用於最終判定。

替代方案：只用 Perfetto UI 目測。暫不採用，因為 1% low 需要可重複計算，且 trace overhead 會污染結果。

2. Span 要更細，但必須沿 critical path 對齊，不做 per-entity default instrumentation。

理由：要知道「明明 tick 很快但 FPS 不滿」需要知道時間花在等待、handoff、render update、present 還是 sleep overshoot。default trace 應以 frame/tick/pipeline 階層為主；10k units 下 per-entity spans 只允許 deep opt-in，避免 profiling 本身造成掉幀。

替代方案：全面加 per-unit spans。暫不採用，會讓 10k units trace 爆量且改變效能特性。

3. 將 sim_runner receive 拆成 wait、active receive、tick、publish，並量測 queue/backlog/catch-up。

理由：`recv_timeout` wait 是 cadence/idle，active receive 才是 CPU work；但如果 queue backlog 持續增加，代表 sim_runner 或 downstream 真的落後。兩者必須分開量測才能決定是 trace 命名問題還是 worker throughput 問題。

替代方案：只把 span 改名。暫不採用，因為使用者目標是找到 FPS 不滿的真因。

4. 將 render thread frame pacing 視為一等診斷對象。

理由：若 sim tick 1-3ms、render work 也足夠快，FPS 仍卡在 120 以下，最可疑的是 pacing/present/sleep/jitter 相關路徑，例如 sleep overshoot、vsync interval、engine frame cap、present blocking、snapshot 等待或主執行緒某段間歇性 work。必須在 `native.rs` frame root span 下明確標示這些區段。

替代方案：只優化 sim_runner。暫不採用，因為問題現象已顯示 sim_runner tick 不是唯一可能瓶頸。

5. 以「測量 → 修正 → 再測量」完成，不接受只有 instrumentation 的結果。

理由：這個 change 的成功條件是 10k units 120 FPS/1% low 達標。若新增診斷後發現問題在 sleep/present/lock contention/render update，tasks 必須包含修正該問題與回歸驗證。

替代方案：拆成 profiling change 與 optimization change。暫不採用，因為使用者明確要求找到問題並解決。

## Risks / Trade-offs

- [Risk] 10k units 1% low >= 119 對現有 renderer 可能暴露多個瓶頸 → Mitigation：先用 frame-time histogram 與 critical path spans 排序，逐一修最大 contributor，不用猜。
- [Risk] Perfetto deep tracing 會讓 1% low 變差 → Mitigation：正式 SLO 只用 profiling disabled 或低 overhead counters；deep tracing 只做短時間定位。
- [Risk] OS scheduler/timer jitter 在 Windows 上可能造成 sleep overshoot → Mitigation：明確量測 requested sleep vs actual sleep、present interval 與 busy/yield 策略，必要時調整 pacing strategy。
- [Risk] Backlog catch-up 過度 aggressive 可能讓 render thread 更抖 → Mitigation：量測 sim queue 與 render frame time，使用 bounded catch-up 或 thread coordination，不改 tick order。
- [Risk] 為了達標而跳過 render updates 會讓畫面不正確 → Mitigation：規格禁止跳 tick、丟 TickBatch、停更 entity 或降低 unit count；驗收需檢查 render state 仍更新。

## Migration Plan

1. 建立 baseline：在 release stress run 收集 profiling disabled 的 FPS、1% low、frame-time histogram、sim TPS、queue backlog 與現有 spans。
2. 補齊 critical path diagnostics：frame root、lockstep drain/forward、sim wait/receive/tick/publish、snapshot consume、render update、frame pacing sleep/yield、present interval。
3. 修正 `receive_tick_batch` span，使 blocking wait 不再混入 active receive；同時保留 wait/idle/starvation 資訊。
4. 用 baseline + trace 判斷真正掉幀來源，依結果修 frame pacing、present/sleep strategy、handoff contention、snapshot consume 或 render update hot path。
5. 重跑 TD_STRESS_10K release，確認平均 FPS、1% low、sim TPS、queue backlog 與 frame-time histogram 達標。
6. 若某次修正無法達標，保留量測結果並繼續下一個最大 contributor；rollback 只需還原對應 code change，無資料遷移。

## Open Questions

- `TD_STRESS_10K` 是否已存在固定 story / run script；若沒有，實作時需先建立等價、可重複的 10k units stress scenario。
- 1% low 的統計窗口採 30 秒或 60 秒，應在驗收 task 中固定，避免短樣本誤判。
