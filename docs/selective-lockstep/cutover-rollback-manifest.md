# Secure V2 切換與回滾清單

狀態：`prepared`。本文件只準備 Phase 6.8 cleanup；在 final verification 通過前不套用不可逆移除。

## Match 模式與開關

- `MATCH_LOCKSTEP_MODE = "legacy"`：只允許 V1，僅能在建立 match 前選擇。
- `MATCH_LOCKSTEP_MODE = "secure_v2_opt_in"`：產生 V2 team stream，V2 client 可 opt-in；V1 僅存在於 non-secure session。
- `MATCH_LOCKSTEP_MODE = "secure_v2_required"`：match 建立後固定 V2，任何 V1 join 或 runtime downgrade 必須拒絕。
- `SELECTIVE_LOCKSTEP_SHADOW = true`：產生 V2 frames 並送同 process observer replica 驗算，不要求 player 切換。
- `SELECTIVE_LOCKSTEP_DOGFOOD = true`：internal match 強制 secure V2，且必須具備 server-owned team binding。

active secure match 的 mode、authenticated team binding、view epoch 與 capability 在建立後不可變。回滾只能停止尚未開始的 match，將下一個 match 設為 `legacy`；不得將進行中的 secure match 降級、改送 global snapshot 或切回 V1。

## Phase 6.8 cleanup patch set

| ID | 目標 selector | 預備動作 | 套用條件 |
|---|---|---|---|
| CLN-01 | `omb/src/transport/kcp_transport.rs::LockstepFrame::TickBatch` secure player fan-out | 移除 player global TickBatch 分支 | final gates 全綠 |
| CLN-02 | `omb/src/transport/kcp_transport.rs::LockstepFrame::StateHash` secure player fan-out | 移除 player global StateHash 分支 | final gates 全綠 |
| CLN-03 | `SnapshotResp` / `SnapshotStore` player bootstrap | 移除 secure player WorldSnapshot bootstrap | final gates 全綠 |
| CLN-04 | legacy `GameStart.master_seed` player delivery | 移除 secure player seed delivery | final gates 全綠 |
| CLN-05 | global snapshot／event raw ECS identifiers | 移除 secure player raw-ID serialization | packet gate 全綠 |
| CLN-06 | `State::client_visibility` | 刪除 dead storage 與 disconnect cleanup | frontend cutover 完成 |
| CLN-07 | `State::last_visibility_tick` | 刪除 dead storage | frontend cutover 完成 |
| CLN-08 | legacy viewport/`VisSet` gameplay authority | quarantine 為 non-secure presentation-only adapter | boundary gate 全綠 |
| CLN-09 | `omb/src/vision/**` nondeterministic authority | quarantine 為 diagnostics/presentation，不得寫 authoritative visibility | parity gate 全綠 |

每個 cleanup commit 必須保留 pre-match legacy mode 的獨立路徑；任何 fallback 都不得由 active secure session 觸發。
