## Context

目前 player lockstep path 對所有 `lockstep_joined` session 廣播相同的 `TickBatch` 與 global `StateHash`，bootstrap 也提供 global `WorldSnapshot` 與 `master_seed`。因此 renderer-only fog 無法防止封包、client memory 或 deterministic replay 洩漏 hidden state。

本改造跨越 `proto/game.proto`、`omoba-core::runtime`、omb Specs ECS、KCP transport、script outcome contract、omfx sim/render boundary、安全模型、效能與 release migration。權威設計來源為 `docs/superpowers/specs/2026-08-26-server-authoritative-selective-lockstep-design.md`。

現有約束：

- Production cadence 為 shared 120Hz，所有 tick-based window 必須由 shared timing helper 推導。
- `scripts/script-abi` 只能放 `abi_stable` 友善型別，不承載 replication runtime。
- Gameplay mutation 以 `Outcome` 集中提交，新的 projection fact 不能破壞 deterministic ordering。
- omfx 不依賴 `omobab`；共用 runtime 必須位於 `omoba-core`。
- Secure fog match 必須真正保密，不能把完整世界送到 client 後只隱藏 renderer。
- 完整測試與檢查集中到所有實作整合完成後的 final verification phase。

## Goals / Non-Goals

**Goals:**

- 讓 server 成為唯一完整世界與最終 authority。
- 讓每個 team 只取得可合法得知的 deterministic subworld，並保留 local step。
- 在 Specs tick 內以 Wave A 平行產生 `Outcome`/`ObservableFact`，commit 後以 Wave B 平行產生 per-team projection。
- 以 scheduled reveal/hide/forget、team-scoped ID、filtered snapshot、external effect、repair/rebase 支援 visibility churn。
- 讓 omfx 與 server observer 共用 `SelectiveReplicaRuntime`。
- 讓 encoded frame 直接進 outbound queue，再由同 process worker thread 非阻塞驗算。
- 以 non-interference、anti-probing、randomness isolation、padding 與 redaction 建立資訊安全邊界。
- 在 10,000 entity、所有 team observer 啟用時滿足 real-time 與 bandwidth gates。

**Non-Goals:**

- 不對 gameplay 已公開的資訊提供額外密碼學保護。
- 不為每位玩家常駐 replica；replica 與 visibility 以 team 為單位。
- 不讓 remembered record 參與 simulation 或 hash。
- 不允許 active secure match 降級回 global-world protocol。
- 不在每個 implementation phase 重跑完整 test/security/stress suite。

## Decisions

### Decision 1：採用 server-authoritative selective lockstep

Server 保存完整 authoritative world；client 只執行 disclosed subworld。當 hidden dependency 不能安全 disclosure 時，server 將結果轉成 `SanitizedExternalEffect`。Client 與 server 發生 revision/hash conflict 時，永遠以 server repair/rebase 為準。

**理由：** 只有此模式能同時滿足真正保密、低 steady-state 頻寬與 client deterministic step。

**替代方案：**

- Global lockstep + renderer hide：實作簡單，但封包與 memory 仍洩漏 hidden state，拒絕。
- 純 server-authoritative state delta：安全且較簡單，但放棄已要求保留的 visible subworld local step，拒絕。

### Decision 2：visibility 以 team 共享並由 deterministic rule + override 合成

ECS 新增 `ReplicationScope`、`VisionSource`、`StealthProfile`、`VisibilityOverride`、`RememberPolicy` 與 `TeamVisibilityIndex`。Script/gameplay 只能透過 explicit outcome 改變 visibility rule。Override 依 priority、expiration tick 與 stable rule ID resolve。

**理由：** Team sharing 符合 gameplay 規則，且讓同隊 session 共用 projection/encoding 成本；explicit outcome 可維持現有 mutation boundary。

**替代方案：** 沿用 viewport/AOI；其 camera/wall-clock/floating-point 語義不 deterministic，也不是 gameplay authorization，拒絕。

### Decision 3：以 team-scoped `ReplicaEntityId` 隔離 canonical identity

每個 team 使用獨立、monotonic、match-local、不可重用的 ID namespace，mapping 包含 disclosure epoch。Remembered 期間可沿用同一 ID；authoritative forget 後永久 retire。

**理由：** 防止 raw ECS ID、entity count 與跨 team correlation 洩漏，也能拒絕 stale input/transition。

### Decision 4：Specs tick 採兩個 parallel wave 與 deterministic barrier

Wave A 中 gameplay system 在產生 `Outcome` 時同步產生 `ObservableFact`；parallel output 依 `(tick, phase, canonical_source_order, local_ordinal, fact_kind)` 合併排序。Commit 後建立 `State[T+1]`，Wave B 再按 team 平行計算 `V[T+1]`、transition、projection、encoding 與 enqueue。

**理由：** 避免三次 full-world serial scan，同時確保 visibility 讀到 post-step committed state。

**替代方案：** Wave A/B 完全無 barrier；這只能取得 stale `V[T]` 或複製 next-state logic，容易 desync，拒絕。

### Decision 5：reveal 使用 scheduled commitment 與 fresh baseline

Default `visibility_commit_delay_ticks = 3`，允許範圍 2–4。Candidate 到期時若條件仍成立，server 才從當下 authoritative state 擷取 fresh baseline。Client default `replica_buffer_ticks = 12`（120Hz 下 100ms），允許範圍 3–24 且不得小於 visibility delay。

**理由：** Client 可在指定 barrier 原子 reveal，無需 rollback；fresh baseline 避免把 T 的 stale state 套到未來 tick。

### Decision 6：Protocol V2 使用 `PreStep`/`Step`/`PostStep`

`TeamGameStart` 只包含 filtered team snapshot 與 safe metadata。`TeamTickFrame` 包含 tick/sequence/view epoch、transition、accepted input、public event、random tape、external effect、authority repair/hash。Frame 依 canonical key 排序。

**理由：** 固定 phase 可讓 server observer 與 remote client 以完全相同順序重播，也能將 transition 與 authority revision 放在明確 barrier。

### Decision 7：randomness 採 bounded tape 或 decided outcome

Global seed/PRNG state 不離開 server。Client 只取得 disclosure-epoch/tick-window scoped random tape，或 server 已決定 outcome。

**理由：** 避免 client 推導 entity 進 fog 後的行為。

### Decision 8：server observer validation 為非阻塞 sidecar

Encoded `Arc<[u8]>` 立即進 outbound queue，同時 tap 到 bounded validation channel。獨立 worker thread 為每個 active team 維護 observer replica，且只能透過 filtered bootstrap 與 encoded stream 取得資料。Mismatch 經 control channel 回報，repair/rebase 在後續 frame 發送。

**理由：** 能驗證真實 wire/runtime parity，又不把 validator 放進 network critical path。

**替代方案：** 先驗算再送出；會增加 latency 並讓 validator backpressure 阻塞 player stream，依使用者決策拒絕。

### Decision 9：validator overflow 記為 coverage gap

Validation channel 滿時 outbound 繼續；stale observer 丟棄後以 filtered snapshot rebootstrap。Coverage gap 必須告警並進 evidence，不得視為 pass。

### Decision 10：server authority 以三層 correction 收斂

依偏差範圍使用 `ComponentRepair`、`EntityReplace` 或 `TeamViewRebase`。所有 correction 都走相同 projection/redaction boundary，且帶更高 authority revision。

### Decision 11：完整驗證集中到最後

Phase 1–5 只執行保持 branch 可編譯與可整合所需的最低限度 compile/focused smoke；所有 unit/property、differential、cross-platform、fault、security、packet/memory inspection、10,000 entity 與 30 分鐘 soak 集中在 Phase 6。

**理由：** 避免每完成一小步就重跑昂貴完整 suite，同時以最終 end-to-end system 作為唯一 acceptance evidence。

### Decision 12：規劃調整分級

- **A — task refinement：** 可調整 task split/order、檔案位置、command 與 evidence mechanics，但不得改 scope、requirement、gate 或 public contract。
- **B — design/spec correction：** 在核准 scope 內修正假設時，受影響工作暫停，更新 design/spec/tasks，將依賴 evidence 標為 stale 後重新驗證。
- **C — material change：** 改變 scope、public contract、blocking gate、threshold、required evidence、platform、permission、external write 或 destructive action 前，必須取得使用者核准。

任何 adjustment 都不得靜默降低 blocking gate；task ID 與 evidence lineage 必須保留。

## Data Flow

```text
InputSubmit + V[T] history
        |
        v
Wave A Specs systems
  Outcome + ObservableFact (stable-key buffers)
        |
        v
Deterministic reduce/commit -> State[T+1]
        |
        v
Wave B per-team jobs (parallel)
  V[T+1] -> transition -> projection -> encode
        |
        +--------------------> outbound queue -> team clients
        |
        +--------------------> validation tap -> observer worker
                                                |
                                                v
                                      mismatch control channel
                                                |
                                                v
                                      later repair/rebase frame
```

## Failure Handling

- Missing/late frame：replica 停在 tick barrier，不猜測 gap。
- Duplicate frame：依 `team_sequence` 與 authority revision idempotently ignore。
- Replay ring miss：改送 filtered `TeamViewRebase` 與 catch-up frames。
- Interrupted rebase：snapshot ID、chunk hash、manifest 未完整驗證前不得套用。
- Observer mismatch：記錄 first divergence，再由後續 authority correction 收斂。
- Validation overflow：記 coverage gap、rebootstrap observer，不阻塞 outbound。
- Active secure match 無法 repair/rebase：安全中止並保存 diagnostics，不降級到 global world。

## Security and Privacy

- Team frame 不含 canonical ECS ID、其他 team visibility、global seed 或 server-only state。
- Invalid target 使用 generalized rejection 與 uniform timing，防止 existence probing。
- Fixed cadence 與 size bucket/padding 降低 payload length side channel。
- Log、trace、replay、crash bundle 使用 team redaction；full diagnostic 只限 server-admin capability。
- Non-interference property：hidden-only difference 在 allowed public effect 前產生 byte-identical team frame。

## Performance and Observability

Blocking gate：10,000 entity、production tick rate、兩個 team projection、兩個 observer replica、transport encoding 與 visibility churn 同時啟用；p99 authoritative tick + commit 不超過 tick period 80%，steady state 低於 5 KB/s/player，30 分鐘 soak 無 unintended rebase 或 authoritative deadline miss。

Metric 至少涵蓋 transition count、frame bytes/padding、build/encode/enqueue/replica duration、queue depth、audit lag、coverage gap、hash mismatch、repair/rebase、barrier stall、gap replay 與 redaction violation。

## Risks / Trade-offs

- **[Risk] Partial simulation closure 遺漏 hidden dependency** → 每個 gameplay/script action 必須有四象限 projection policy；缺少 policy 視為 blocking integration error，final verification 執行 non-interference 與 boundary matrix。
- **[Risk] Per-team projection 與 observer replica 增加 CPU/memory** → Team sharing、Wave B parallel、bounded queue；以 10,000 entity gate 驗證，不得以關閉 validator 過關。
- **[Risk] Payload length/timing 洩漏 hidden activity** → Fixed cadence、size bucket、padding、mass reveal/rebase chunking 與 packet audit。
- **[Risk] Observer lag 造成驗算空窗** → Coverage gap metric/alert、filtered rebootstrap、append-only evidence；不得標為 pass。
- **[Risk] Protocol/schema migration 造成 V1/V2 混用** → Match-level negotiation，secure match 禁止 mixed client 與 runtime downgrade。
- **[Risk] Repair 掩蓋長期 determinism bug** → 保存 first-divergence evidence、限制 repair/rebase rate，final gate 要求零 unintended rebase。
- **[Risk] Scheduled reveal 增加視覺延遲** → `D=3` 與 `L=12` 為 default，bounds 固定；threshold 變更屬 C-level change。
- **[Trade-off] Validator 不阻塞 outbound** → 玩家 latency 較低，但 mismatch 只能在後續 frame 修正；以 server authority 與 barrier correction 接受此取捨。

## Migration Plan

1. 建立 state/event inventory、classification、protocol/schema 與 baseline/harness。
2. 在 `omoba-core` 建立 protocol V2、filtered snapshot 與 `SelectiveReplicaRuntime`。
3. 整合 Specs Wave A/commit/Wave B、visibility、projection policy 與 team frame。
4. 整合 KCP team stream、replay ring、repair coordinator 與非阻塞 observer worker。
5. 整合 omfx barrier buffer、remembered cache、transition、repair/rebase。
6. Phase 1–5 完成後集中執行完整 final verification。
7. 通過後依序啟用 server shadow、internal dogfood、opt-in secure match、secure default。
8. 最後移除 player global `TickBatch`、`StateHash`、`WorldSnapshot`、`master_seed`、raw ECS ID 與 dead visibility path。

Rollback 只允許在 match 建立前選擇明確 non-secure legacy mode；active secure match 不允許降級。

## Open Questions

無。Default delay/buffer、authority、validation sidecar、測試集中時點與 security gates 均已由核准設計固定；若 implementation evidence 顯示假設不合理，依 A/B/C adjustment 規則處理。
