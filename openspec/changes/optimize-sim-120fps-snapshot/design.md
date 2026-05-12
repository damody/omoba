## Context

`omfx` 目前透過 lockstep client 接收 `TickBatch`，交給 `sim_runner` worker 推進本地 ECS replica。現有程式在每個 tick batch 完成 systems、script dispatch、outcome processing 與 `world.maintain()` 後呼叫 `omoba_core::runtime::extract_snapshot`，再把完整 `SimWorldSnapshot` 寫入 shared mutex。使用者貼出的 log 顯示 `[sim_runner] tick=2760/2880/3000/3120` 與 `[mirror-snapshot]` 每 120 tick 反覆出現，證明 runtime tick loop 正在持續執行完整 snapshot extraction。

正確契約是：`extract_snapshot` 不應被 sim_runner 呼叫。初始化/seed 階段只需要靜態 render bootstrap data，可以直接從 world resources 與 metadata registries 建立；runtime 階段則只推進本地 ECS 與發布必要的輕量 runtime deltas/mirror，不能每 tick 重建完整 snapshot。

## Goals / Non-Goals

**Goals:**

- 確保 sim_runner 不再呼叫 `extract_snapshot`。
- 讓 TD_1/TD_STRESS log 不再出現由 full snapshot extraction 產生的 `[sim_runner]` / `[mirror-snapshot]` 掃描輸出。
- 保留 sim_runner 以 shared 120 TPS cadence 消化 TickBatch 的能力。
- 為 render 所需的 runtime dynamic state 建立或接上輕量 publication path，避免完整 ECS snapshot 重建。
- 讓 diagnostics 可分辨 init static seed、runtime tick processing、runtime lightweight publish 與 render pacing。

**Non-Goals:**

- 不把 simulation 改成 variable timestep。
- 不改 KCP/lockstep protocol payload 格式。
- 不移除 `SimWorldSnapshot` type，也不一次性重寫所有 render consumers。
- 不用「停止更新 render state」假裝達成效能；若 UI/場景需要 runtime state，必須有替代輕量資料來源。

## Decisions

1. 從 sim_runner 移除 `extract_snapshot` 呼叫。

理由：這直接符合使用者契約，也移除 stress path 最大的完整世界掃描來源。初始化 seed 需要的 paths、blocked regions、tower templates、ability definitions 等資料可以從 world resources 與 metadata registries 直接建立，不需要完整 ECS dump。

替代方案：初始化時呼叫一次 `extract_snapshot`。暫不採用，因為它仍會把重型 full snapshot dump 留在 sim_runner path，且靜態 seed 沒有必要透過它取得。

2. Runtime dynamic state 改走 lightweight publication，而不是呼叫 `extract_snapshot`。

理由：render 仍需要 tick、entity movement/HP、removed ids、FX、round/lives 與 input receipt metadata。這些資料應由小而明確的 publisher 更新 shared state 或專用 mirror，避免每 tick rebuild full metadata、paths 與全量 snapshot。

替代方案：完全不發布 runtime state。暫不採用，因為會破壞 creep movement、HP bar、tower UI、VFX 與 input latency pairing。

3. 先以最小改動建立 runtime lightweight updater，沿用 `SimWorldSnapshot` 作為共享容器但不使用 `extract_snapshot`。

理由：大規模替換所有 `native.rs` consumers 風險高。較小的做法是在 sim_runner 內新增明確命名的 runtime update helper，只更新動態欄位，靜態欄位沿用 init seed 的 Arc/Vec；後續若要更乾淨再抽新 type。

替代方案：新增完整 `RuntimeRenderFrame` type 並改所有 consumers。暫不採用於第一步，除非 borrow/正確性迫使切分。

4. 加入防回歸檢查與低頻 diagnostics。

理由：這類回歸很容易被「臨時要拿 snapshot」重新引入。檢查可保證 sim_runner 不再 call `extract_snapshot`；diagnostics 應顯示 init seed 與 runtime lightweight publish count。

替代方案：只靠 code review。暫不採用，因為目前已發生契約偏離。

## Risks / Trade-offs

- [Risk] Lightweight updater 漏掉原本 `extract_snapshot` 會填的 dynamic 欄位 → Mitigation：逐項對照 `SimWorldSnapshot` consumers，先覆蓋 tick/entities/removed ids/FX/round/lives/applied input metadata/lua reload fields。
- [Risk] `extract_snapshot` 原本 drain queues，移除後 removed ids 或 FX 消失 → Mitigation：runtime updater 必須明確 drain 同一批 queues 或改由 outcome processing 直接推到 runtime publish queues。
- [Risk] 沿用 `SimWorldSnapshot` 容器會讓語意仍混淆 → Mitigation：helper 命名與 comments 明確標示 init static seed vs runtime lightweight update，後續可另開 change 拆 type。
- [Risk] 120 FPS 仍受 render 或 searcher 熱點限制 → Mitigation：保留 profile window diagnostics，移除 full snapshot 後再看下一個 bottleneck。

## Migration Plan

1. 新增 init seed path：初始化 world/metadata 後直接建立靜態 render seed，寫入 shared state。
2. 在 per-tick loop 移除 `extract_snapshot`，改呼叫 runtime lightweight updater。
3. 移除或改寫 `[sim_runner]` / `[mirror-snapshot]` 這類依賴 full snapshot entity scan 的 runtime log。
4. 加入測試或 diagnostic guard，確認 sim_runner 不會呼叫 `extract_snapshot`。
5. 跑 TD_1/TD_STRESS smoke，確認 runtime render state 仍更新且 log 不再顯示 full snapshot extraction。
6. 若需 rollback，可恢復 per-tick full snapshot call；沒有資料遷移。

## Open Questions

- Lightweight updater 第一版允許仍全量掃描 dynamic entities，但不能呼叫 `extract_snapshot`；後續再用 profiling 決定是否做 entity-level dirty/delta。
