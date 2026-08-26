# 實作任務與證據規則

所有 resolved L3 leaf 都必須在 `openspec/changes/server-authoritative-selective-lockstep/evidence/index.jsonl` 寫入唯一 `task_id` record；record 至少包含 `task_id`、`status`、`artifact_or_command`、`expected`、`actual`、`exit_status_or_reviewer`、`hashes`、`related_gates`、`adjustment_id`、`timestamp`，共享 artifact 另加 unique `subcheck`。`status` 只能是 `passed`、有證據的 `not-applicable` 或 `superseded`；failed、blocked、stale、未執行不得勾選。

Phase 1–5 只做實作、artifact review、最低限度 compile/focused smoke，不執行完整 acceptance suite。完整 unit/property、differential、cross-platform、fault、security、packet/client-memory inspection、10,000 entity 與 30 分鐘 soak 全部集中在 Phase 6。

## 1. Contract、分類與證據基礎

### 1.1 建立 state、event 與 projection inventory

**目的：** 產生完整且可追蹤的 gameplay state/event disclosure 分類，避免 hidden dependency 遺漏。
**輸入：** 核准設計、`omoba-core/src/runtime/**`、`omb/src/comp/**`、`omb/src/tick/**`、`scripts/base_content/**`、現有 snapshot/hash/protocol。
**產出：** `docs/selective-lockstep/state-classification.md`、`docs/selective-lockstep/projection-policy-matrix.md`。
**依賴：** 無。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-CONTRACT-STATE`；`evidence/index.jsonl` task records。
**完成門檻：** 所有 deterministic component/resource/input/event/script outcome/snapshot/hash field 都有唯一分類與四象限 projection policy disposition。

- [ ] 1.1.1 盤點 `omoba-core::runtime` 與 omb ECS 的 deterministic components/resources，記錄 owner、mutation phase 與 hash/snapshot usage。
- [ ] 1.1.2 盤點 `PlayerInput`、`Outcome`、script event、render cue 與 retained network event，記錄 producer/consumer 與 authoritative phase。
- [ ] 1.1.3 將每個 inventory item 分類為 `Public`、`TeamPrivate`、`VisibilityBound` 或 `ServerOnly`。
- [ ] 1.1.4 為每個 gameplay/script action 填寫 visible-visible、hidden-visible、visible-hidden、hidden-hidden projection policy。
- [ ] 1.1.5 將未分類、重複分類與缺少 projection policy 的項目整理成 blocking migration list。

### 1.2 固定 protocol、schema、gate 與 evidence contract

**目的：** 在實作前固定 V2 public contract、blocking threshold 與證據格式。
**輸入：** 1.1 inventory、proposal、design、delta specs。
**產出：** `docs/selective-lockstep/protocol-v2-contract.md`、`docs/selective-lockstep/evidence-schema.md`、`evidence/index.jsonl`。
**依賴：** 1.1。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-CONTRACT-PROTOCOL`、`G-EVIDENCE-SCHEMA`。
**完成門檻：** V2 message/phase/version/bounds、security invariant、A/B/C adjustment 與 evidence lineage 均有無歧義定義。

- [ ] 1.2.1 固定 `TeamGameStart`、`TeamTickFrame`、transition、random tape、repair/rebase 欄位與 canonical order。
- [ ] 1.2.2 固定 `visibility_commit_delay_ticks` default/bounds、`replica_buffer_ticks` default/bounds 與 120Hz 換算規則。
- [ ] 1.2.3 固定 team-scoped ID、disclosure epoch、authority revision、snapshot/chunk/manifest versioning 規則。
- [ ] 1.2.4 建立 append-only evidence JSONL schema、hash 規則、stale/superseded lineage 與 gate mapping。
- [ ] 1.2.5 寫入 A/B/C adjustment 流程，確保 B-level correction 重開 affected task/evidence，C-level change 要求使用者核准。

### 1.3 建立 baseline 與 final-verification harness 骨架

**目的：** 保存改造前 baseline，並先建立最後階段會使用的 harness 入口，但不提前執行完整 suite。
**輸入：** 現有 `run_10000.bat`、lockstep diagnostics、1.2 evidence schema。
**產出：** `docs/selective-lockstep/baseline.json`、`tools/selective_lockstep/` harness skeleton、fixture manifest。
**依賴：** 1.2。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-BASELINE-RECORDED`；raw logs/hash index。
**完成門檻：** 現況 CPU/memory/bandwidth 有可重現 baseline；final harness 有明確 entrypoint 與 fixture schema，但未被誤標為 acceptance pass。

- [ ] 1.3.1 執行一次現況 10,000 entity baseline capture，保存 command、config、raw log、machine profile 與 content hash。
- [ ] 1.3.2 建立 non-interference paired-world fixture schema 與 fixture generator entrypoint。
- [ ] 1.3.3 建立 packet/redaction scan、fault injection、observer slowdown 與 stress report harness skeleton。
- [ ] 1.3.4 建立 final-verification evidence manifest template，明確標示 Phase 1–5 smoke 不屬於 acceptance evidence。

## 2. Protocol V2 與 shared selective replica foundation

### 2.1 實作 V2 wire schema 與 compatibility boundary

**目的：** 提供 team-specific bootstrap/frame/recovery wire type 與 match-level negotiation。
**輸入：** 1.2 protocol contract、`proto/game.proto`、既有 KCP framing。
**產出：** 更新的 `proto/game.proto`、generated Rust types、framing tags/version guards。
**依賴：** 1.2。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-V2-SCHEMA`；proto/generated hashes。
**完成門檻：** 所有 proposal-listed V2 message 可 encode/decode，V1/V2 match negotiation 有明確拒絕路徑，player V2 type 不含 global seed/raw ECS ID。

- [ ] 2.1.1 在 `proto/game.proto` 定義 `TeamGameStart`、`TeamTickFrame` 與 `PreStep`/`Step`/`PostStep` payload。
- [ ] 2.1.2 定義 `RevealEntity`、`HideEntity`、`ForgetEntity`、`ComponentRepair`、`EntityReplace` 與 `TeamViewRebase` schema。
- [ ] 2.1.3 定義 replica ID、disclosure/view epoch、authority revision、random tape、external effect 與 filtered snapshot schema。
- [ ] 2.1.4 新增 V2 framing tags、protocol/schema version constants 與 match capability negotiation types。
- [ ] 2.1.5 以既有 code generation 流程更新 `omoba-core/src/generated/game.rs`，不手改 generated output。

### 2.2 實作 team identity 與 filtered snapshot primitives

**目的：** 建立不暴露 canonical identity 的 team world bootstrap。
**輸入：** 2.1 schema、1.1 state classification。
**產出：** `omoba-core::runtime` team identity、filtered snapshot encode/decode、manifest/chunk support。
**依賴：** 2.1。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-IDENTITY-ISOLATION`、`G-FILTERED-SNAPSHOT`。
**完成門檻：** 每隊 mapping 獨立、ID 不重用、stale epoch 可拒絕；snapshot 只含 classified safe state。

- [ ] 2.2.1 實作 monotonic non-reused `ReplicaEntityId` 與 per-team canonical mapping。
- [ ] 2.2.2 實作 disclosure epoch、remembered interval ID retention 與 authoritative forget retirement。
- [ ] 2.2.3 實作 filtered snapshot builder，只讀取 classification allowlist 與 team view。
- [ ] 2.2.4 實作 snapshot ID、chunk hash、manifest 與 interrupted rebase discard contract。
- [ ] 2.2.5 實作 snapshot/schema compatibility guard，拒絕 global `SnapshotStore` bytes 進入 V2 player path。

### 2.3 實作 `SelectiveReplicaRuntime`

**目的：** 提供 omfx 與 server observer 共用的 deterministic replica runtime。
**輸入：** 2.1/2.2、既有 `SimulationDriver` 與 runtime initialization。
**產出：** `omoba-core::runtime::SelectiveReplicaRuntime`、canonical team hash、transition/repair/rebase application。
**依賴：** 2.1、2.2。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-SHARED-REPLICA-RUNTIME`。
**完成門檻：** Runtime 可從 filtered snapshot bootstrap，依 phase 推進、停在 gap barrier、套用 authority revision 並輸出 filtered render snapshot/hash。

- [ ] 2.3.1 實作 `PreStep` reveal/hide/forget transition application 與 idempotence。
- [ ] 2.3.2 實作 `Step` accepted input、public event、external effect 與 bounded random tape injection。
- [ ] 2.3.3 實作一個 disclosed-world fixed tick，確保不讀 server-only 或 remembered cache。
- [ ] 2.3.4 實作 `PostStep` component/entity/rebase authority revision 與 server-wins conflict resolution。
- [ ] 2.3.5 實作 expected sequence/tick barrier、late-frame stall 與 replay/rebase resume。
- [ ] 2.3.6 實作 canonical team hash 與 filtered render snapshot extraction。

### 2.4 準備 synthetic fixtures 與最低限度 build

**目的：** 讓後續 server/omfx 可整合 shared runtime，不提前執行完整 determinism/fault suite。
**輸入：** 2.1–2.3。
**產出：** Synthetic encoded-frame fixture、server observer/client fixture constructors、Phase 2 build log。
**依賴：** 2.3。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-PHASE2-BUILDABLE`，非 acceptance gate。
**完成門檻：** Shared crates 與 fixture code 可編譯；完整測試明確 deferred 到 Phase 6。

- [ ] 2.4.1 建立消費相同 encoded bytes 的 synthetic client/observer fixture constructors。
- [ ] 2.4.2 建立 reveal/hide/repair/rebase fixture data，僅供 Phase 6 suite 使用。
- [ ] 2.4.3 執行一次最低限度 `omoba-core`/protocol compile check 並記錄為非 acceptance evidence。

## 3. Deterministic Specs projection pipeline

### 3.1 建立 stable `Outcome`/`ObservableFact` buffers

**目的：** 在 gameplay 計算期間同步產生 projection fact，且不受 thread completion order 影響。
**輸入：** 1.1 policy matrix、現有 `Outcome`/runtime event flow。
**產出：** Stable key types、sharded/thread-local buffers、deterministic reducer。
**依賴：** 1.1、2.1。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-OBSERVABLE-FACT-CONTRACT`。
**完成門檻：** 每筆 outcome/fact 有 stable key，commit 不依賴 insertion/arrival order，沒有 post-hoc full-world effect scan。

- [ ] 3.1.1 定義 `ObservableFact` variant、safe metadata 與 stable ordering key。
- [ ] 3.1.2 實作 Specs-safe sharded/thread-local output buffers。
- [ ] 3.1.3 實作 deterministic merge、sort、dedupe 與 malformed key rejection。
- [ ] 3.1.4 將既有 runtime event bridge 改為由 ordered facts/outcomes 驅動。

### 3.2 遷移 gameplay 與 script projection policy

**目的：** 讓所有現有 gameplay action 具有完整四象限 visibility behavior。
**輸入：** 1.1 migration list、3.1 buffer contract。
**產出：** 更新的 omb/omoba-core gameplay systems、script outcome bridge、policy registry。
**依賴：** 3.1。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-PROJECTION-POLICY-COMPLETE`。
**完成門檻：** Inventory 中每個 action 都有 code-owned policy；缺少 policy 時 secure startup/content validation fail closed。

- [ ] 3.2.1 遷移 movement、spawn、death 與 ownership actions 產生 `ObservableFact`。
- [ ] 3.2.2 遷移 combat、projectile、AOE、buff/debuff actions 產生 projection facts/external-effect inputs。
- [ ] 3.2.3 遷移 hero/tower/item/ability actions 與 retained HUD/terminal events。
- [ ] 3.2.4 擴充 script host boundary，讓 `base_content` 透過 Outcome/fact contract 聲明 policy，不把 runtime-heavy type 放入 `script-abi`。
- [ ] 3.2.5 實作 projection policy registry completeness check 與 actionable error report。

### 3.3 實作 Wave A commit 與 Wave B visibility

**目的：** 在同一 Specs tick pipeline 中保留平行計算與 post-step visibility correctness。
**輸入：** 3.1/3.2、既有 dispatcher/process_outcomes。
**產出：** Wave A reduce/commit barrier、fixed-point visibility systems、per-team Wave B jobs。
**依賴：** 3.1、3.2。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-TWO-WAVE-PIPELINE`。
**完成門檻：** `State[T+1]` 只在 deterministic commit 後可見；team jobs 讀 committed state 並可平行執行。

- [ ] 3.3.1 將 stable outcome/fact reduce 插入 authoritative tick 的 deterministic commit barrier。
- [ ] 3.3.2 實作 fixed-point `ReplicationScope`、`VisionSource`、`StealthProfile`、`VisibilityOverride`、`RememberPolicy` ECS types。
- [ ] 3.3.3 實作 team-shared geometry/detection resolve 與 stable override precedence。
- [ ] 3.3.4 實作 reveal/hide candidate、2–4 tick commitment delay、cancellation 與 fresh baseline capture。
- [ ] 3.3.5 實作 `TeamVisibilityIndex`、visibility history 與 per-team read-only Wave B scheduling。

### 3.4 實作 `TeamViewProjector` 與 frame builder

**目的：** 將 committed state/facts 轉成 canonical、安全且可重播的 team frame。
**輸入：** 2.x shared protocol/runtime、3.3 visibility。
**產出：** Per-team projector、external effect sanitizer、frame encoder、padding buckets。
**依賴：** 2.3、3.3。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-TEAM-PROJECTION`、`G-NONINTERFERENCE-READY`。
**完成門檻：** Projector 不輸出 forbidden state；同 logical payload canonical bytes 穩定；hidden dependency 轉成 sanitized outcome。

- [ ] 3.4.1 實作 per-team fact audience/redaction 與 disclosed dependency closure。
- [ ] 3.4.2 實作 hidden-visible external damage/buff/projectile/AOE sanitizer。
- [ ] 3.4.3 實作 `PreStep`/`Step`/`PostStep` canonical frame assembly。
- [ ] 3.4.4 實作 fixed cadence empty frame、size bucket/padding 與 mass reveal/rebase chunk policy。
- [ ] 3.4.5 實作 authoritative expected team hash projection 與 safe mismatch metadata。

### 3.5 Phase 3 最低限度 integration build

**目的：** 確認 pipeline 可供 transport 整合，不提前執行完整 test matrix。
**輸入：** 3.1–3.4。
**產出：** Phase 3 compile/focused smoke log。
**依賴：** 3.4。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-PHASE3-BUILDABLE`，非 acceptance gate。
**完成門檻：** Authoritative tick 可產生 synthetic team frames，相關 workspaces 可編譯。

- [ ] 3.5.1 執行一次 omb/omoba-core 最低限度 compile check 並保存 non-acceptance log。
- [ ] 3.5.2 以單一 synthetic tick focused smoke 確認 Wave A/commit/Wave B data plumbing，不跑完整 scenarios。

## 4. Server team stream、recovery 與 observer sidecar

### 4.1 實作 team session routing 與 replay ring

**目的：** 將 V2 frame 只送給綁定 team，並提供 sequence recovery。
**輸入：** 2.1 negotiation、3.4 encoded frames、現有 KCP session map。
**產出：** Team-bound sessions、direct outbound enqueue、per-team encoded replay ring。
**依賴：** 2.1、3.4。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-TEAM-ROUTING`。
**完成門檻：** Secure session 只收自身 team frame；encoded frame 立即 enqueue；gap 可由 ring/rebase 路由恢復。

- [ ] 4.1.1 擴充 join/session state，綁定 protocol version、team、view epoch 與 secure-match capability。
- [ ] 4.1.2 將 `TeamTickFrame` 直接 enqueue 給同 team sessions，移除 secure path global fan-out。
- [ ] 4.1.3 實作 bounded per-team encoded-frame replay ring 與 idempotent resend。
- [ ] 4.1.4 實作 ring expiry 後 filtered rebase/catch-up routing。
- [ ] 4.1.5 拒絕 V1 client 加入 secure match，且拒絕 active match runtime downgrade。

### 4.2 實作非阻塞 observer validation worker

**目的：** 在同 process 另一條 thread 模擬每隊 observer，不阻塞 outbound。
**輸入：** 4.1 encoded stream、2.3 runtime。
**產出：** Validation tap/channel/worker、per-team observer lifecycle、audit metrics。
**依賴：** 2.3、4.1。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-OBSERVER-SIDECAR`、`G-OUTBOUND-NONBLOCKING`。
**完成門檻：** Observer 只消費 filtered bootstrap/actual bytes；validator backpressure 不影響 outbound；每隊 lifecycle 隔離。

- [ ] 4.2.1 建立 bounded validation channel，tap 與 outbound 相同的 encoded `Arc<[u8]>`。
- [ ] 4.2.2 建立獨立 worker thread 與每個 active team observer replica map。
- [ ] 4.2.3 讓 observer 經 V2 filtered bootstrap path 初始化並 decode 實際 frame bytes。
- [ ] 4.2.4 實作 observer tick/hash、audit lag、queue depth 與 coverage tracking。
- [ ] 4.2.5 實作 channel overflow coverage gap、stale observer discard 與 filtered rebootstrap。

### 4.3 實作 mismatch control 與 authority recovery

**目的：** 將 client/observer divergence 轉成後續 server-authoritative correction。
**輸入：** 2.3 repair/rebase、3.4 team hash、4.2 mismatch signal。
**產出：** `AuthorityRepairCoordinator`、safe diagnostic bundle、client gap/rejoin handlers。
**依賴：** 3.4、4.2。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-AUTHORITY-RECOVERY`。
**完成門檻：** First divergence 可定位；repair/rebase 在 later frame 發出；無法恢復時 secure match fail closed。

- [ ] 4.3.1 定義 observer/client mismatch control message 與 safe first-divergence record。
- [ ] 4.3.2 實作 component-level repair selection 與 authority revision allocation。
- [ ] 4.3.3 實作 entity replace 與 full filtered `TeamViewRebase` selection。
- [ ] 4.3.4 實作 join/rejoin、observer rebootstrap 與 interrupted rebase recovery。
- [ ] 4.3.5 實作持續 recovery failure 的 secure match safe termination，不送 global fallback。

### 4.4 實作 anti-probing、redaction 與 observability

**目的：** 封閉 player/session 與 diagnostic side channel。
**輸入：** 1.1 classification、3.4 projector、4.1/4.3 transport/recovery。
**產出：** Input visibility validation、generalized rejection、padding/redaction、metrics/traces。
**依賴：** 3.4、4.1、4.3。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-ANTI-PROBING-READY`、`G-REDACTION-READY`。
**完成門檻：** Player-visible outputs 不含 forbidden fields；invalid target 不形成 existence oracle；admin diagnostic boundary 明確隔離。

- [ ] 4.4.1 將 target input 改為 replica ID/view/disclosure epoch，依 input tick visibility history 驗證。
- [ ] 4.4.2 實作 generalized rejection、uniform timing bucket 與 invalid reference rate limit。
- [ ] 4.4.3 實作 player log/replay/crash/trace redaction 與 server-admin capability boundary。
- [ ] 4.4.4 實作 transition/frame/queue/audit lag/coverage gap/repair/rebase/security metrics。
- [ ] 4.4.5 實作 packet padding diagnostics，分開 steady-state 與 reveal/rebase burst accounting。

### 4.5 Phase 4 最低限度 server build

**目的：** 確認 server integration 可供 omfx 對接，不提前執行完整 fault/security/stress suite。
**輸入：** 4.1–4.4。
**產出：** Server compile/focused connection smoke log。
**依賴：** 4.4。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-PHASE4-BUILDABLE`，非 acceptance gate。
**完成門檻：** omb V2 path 可編譯，synthetic client 可完成一次 filtered join 與一個 frame receive。

- [ ] 4.5.1 執行一次 omb 最低限度 compile check 並保存 non-acceptance log。
- [ ] 4.5.2 執行一次 synthetic filtered join/frame receive focused smoke，不跑完整 recovery/security matrix。

## 5. omfx selective replica 與 cutover preparation

### 5.1 將 omfx sim runner 遷移到 `SelectiveReplicaRuntime`

**目的：** 讓 native frontend 只持有 team disclosed deterministic world。
**輸入：** 2.3 shared runtime、4.1 V2 client stream。
**產出：** omfx V2 lockstep client/sim runner、team bootstrap、barrier buffer。
**依賴：** 2.3、4.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-OMFX-SELECTIVE-RUNTIME`。
**完成門檻：** omfx 不建立 global world、不讀 `master_seed`/raw ECS ID，並以 negotiated barrier 推進。

- [ ] 5.1.1 擴充 omfx KCP client decode `TeamGameStart`、`TeamTickFrame`、replay/rebase control。
- [ ] 5.1.2 以 filtered snapshot bootstrap `SelectiveReplicaRuntime`，移除 secure path global world bootstrap。
- [ ] 5.1.3 實作 12-tick default barrier buffer、expected sequence 與 late-frame stall。
- [ ] 5.1.4 將 accepted input/external effect/transition/authority correction 導入 shared runtime。
- [ ] 5.1.5 確保 UI/network/input handling 在 replica stall 時保持 responsive。

### 5.2 實作 filtered rendering 與 remembered cache

**目的：** Render disclosed state 並以獨立 cache 呈現允許的 last-known ghost。
**輸入：** 5.1 runtime snapshots、`RememberPolicy` transition。
**產出：** omfx render bridge、remembered cache、transition presentation/cache cleanup。
**依賴：** 5.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-FILTERED-RENDERING`。
**完成門檻：** Hidden entity 不在 deterministic snapshot/render cache；remembered data 不可 target/hash；reveal 可關聯 prior remembered ID。

- [ ] 5.2.1 將 render bridge identity 改為 team-scoped replica ID。
- [ ] 5.2.2 實作 hide/forget 時 deterministic scene cleanup。
- [ ] 5.2.3 實作 `LastKnown`/custom remembered presentation cache 與 lifecycle。
- [ ] 5.2.4 阻止 remembered cache 進入 target lookup、collision、simulation 與 team hash。
- [ ] 5.2.5 實作 re-reveal 與 remembered presentation 的安全關聯／替換。

### 5.3 實作 client recovery、authority 與 diagnostics

**目的：** 讓 omfx 在 gap/mismatch/rebase 後按 server 指示收斂。
**輸入：** 4.3 server recovery、5.1 runtime。
**產出：** Replay request、repair/rebase handling、redacted client diagnostics。
**依賴：** 4.3、5.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-OMFX-RECOVERY`。
**完成門檻：** Duplicate/gap/late/correction/rebase 有 deterministic terminal path；client 不嘗試 global snapshot fallback。

- [ ] 5.3.1 實作 duplicate/gap detection 與 replay request。
- [ ] 5.3.2 實作 `ComponentRepair`、`EntityReplace` 與 `TeamViewRebase` barrier application。
- [ ] 5.3.3 實作 rebase chunk/manifest verification 與 interrupted rebase discard。
- [ ] 5.3.4 實作 client team hash report、barrier stall、gap/rebase metrics 與 redacted diagnostic bundle。
- [ ] 5.3.5 移除 secure client path 的 global snapshot/hash/master-seed fallback。

### 5.4 準備 match negotiation、shadow 與 cleanup

**目的：** 準備可逆的 V2 shadow/dogfood 與最終 cleanup，不在 final verification 前 irreversible cutover。
**輸入：** 4.x server、5.1–5.3 client。
**產出：** Match-level config、shadow/dogfood switches、cleanup patch set/manifest。
**依賴：** 4.5、5.3。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-CUTOVER-PREPARED`。
**完成門檻：** V2 可 opt-in；legacy 僅限 non-secure match；global path cleanup 已準備但未在 final evidence 前啟用。

- [ ] 5.4.1 實作 match-level `secure_v2` negotiation/config 與 V1/V2 isolation。
- [ ] 5.4.2 實作 server shadow generation 與 internal dogfood switches。
- [ ] 5.4.3 準備移除 player global `TickBatch`、`StateHash`、`WorldSnapshot`、`master_seed` fan-out 的 patch set。
- [ ] 5.4.4 準備移除 dead `client_visibility`/`last_visibility_tick`、legacy viewport authority 與 nondeterministic vision path 的 patch set。
- [ ] 5.4.5 建立 cutover/rollback manifest，明確禁止 active secure match downgrade。

### 5.5 End-to-end 最低限度 build/smoke

**目的：** 在完整 Phase 6 前只確認整合可啟動，不宣稱 acceptance。
**輸入：** Phase 2–5 implementation。
**產出：** Workspace build logs、單一 V2 join/tick/render focused smoke log。
**依賴：** 5.4。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-E2E-BUILDABLE`，非 acceptance gate。
**完成門檻：** Required workspaces build，單一 secure V2 session 可 join/step/render；所有完整驗證仍未標 pass。

- [ ] 5.5.1 執行 scripts、omb、omfx required workspace compile/build commands 並保存 non-acceptance logs。
- [ ] 5.5.2 執行一次單一 V2 filtered join、one-frame step、render focused smoke。
- [ ] 5.5.3 Freeze Phase 6 config、binary/content hashes 與 evidence manifest；之後 B/C correction 需標記 affected evidence stale。

## 6. 集中式 Final Verification、cutover 與 cleanup

### 6.1 執行完整 unit/property 與 schema suite

**目的：** 一次驗證所有低階 contract，不在先前 phase 重複。
**輸入：** 5.5 frozen build/config、完整 test harness。
**產出：** `evidence/final/unit-property/` raw logs、JUnit/summary、hash index。
**依賴：** 5.5。
**Owner／Wave：** Primary integrator／Wave 6A。
**Gate／Evidence：** `G-FINAL-UNIT`、`G-FINAL-SCHEMA`。
**完成門檻：** 所有 required unit/property/schema tests exit 0，無 skip blocking scenario。

- [ ] 6.1.1 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` 並保存完整 log。
- [ ] 6.1.2 執行 `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi` 與 `-p base_content` 並保存完整 log。
- [ ] 6.1.3 執行 omoba-sim/omoba-core determinism、visibility、identity、transition、repair、random-tape property suites。
- [ ] 6.1.4 執行 protocol encode/decode、schema version、canonical ordering、malformed transition/rebase suites。
- [ ] 6.1.5 執行 projection-policy completeness 與 remembered-state exclusion suites。

### 6.2 執行 differential、cross-platform 與 non-interference suite

**目的：** 驗證 authoritative projection、server observer 與 omfx replica 的 deterministic parity 與 hidden-state isolation。
**輸入：** 6.1 passing build、paired-world fixtures。
**產出：** `evidence/final/differential/` hash traces、platform reports、paired frame bytes。
**依賴：** 6.1。
**Owner／Wave：** Primary integrator／Wave 6B。
**Gate／Evidence：** `G-FINAL-PARITY`、`G-FINAL-NONINTERFERENCE`。
**完成門檻：** 三方 hash parity 全通過；Windows/Linux pins 相同；所有 hidden-only paired frames byte-identical 到 public causal effect。

- [ ] 6.2.1 執行 authoritative projector vs server observer vs synthetic/omfx replica checkpoint hash differential suite。
- [ ] 6.2.2 在 Windows Rust 1.95.0 執行 pinned determinism suite並保存 toolchain/platform metadata。
- [ ] 6.2.3 在 Linux Rust 1.95.0 執行相同 pinned determinism suite並比較 hashes。
- [ ] 6.2.4 執行 hidden movement/state/RNG/death paired-world non-interference suite並保存 frame byte hashes。
- [ ] 6.2.5 執行 parallel completion-order permutation suite，確認 canonical encoded bytes 不變。

### 6.3 執行 visibility boundary 與 gameplay integration matrix

**目的：** 驗證所有已核准的 visibility state machine 與跨界 projection。
**輸入：** 6.2 passing parity。
**產出：** `evidence/final/boundary/` scenario logs、state/hash timelines。
**依賴：** 6.2。
**Owner／Wave：** Primary integrator／Wave 6C。
**Gate／Evidence：** `G-FINAL-BOUNDARY`。
**完成門檻：** 每個 projection matrix scenario 有 passed evidence，無未分類/未執行 blocking case。

- [ ] 6.3.1 執行 team-shared vision、override precedence、expiry、candidate cancel、reveal/hide/re-reveal/forget scenarios。
- [ ] 6.3.2 執行 hidden attacker damage、buff/debuff、projectile enter/leave、AOE cross-boundary scenarios。
- [ ] 6.3.3 執行 remembered ghost、fog death、custom remember policy 與 target rejection scenarios。
- [ ] 6.3.4 執行 owner/team/public/server-only resource 與 event audience scenarios。
- [ ] 6.3.5 執行 input tick visibility-history、stale epoch、ownership 與 accepted/rejected action scenarios。

### 6.4 執行 network fault、recovery 與 validator suite

**目的：** 驗證 gap、replay、rebase、observer sidecar 與 non-blocking guarantee。
**輸入：** 6.3 passing integration。
**產出：** `evidence/final/fault-recovery/` injected fault manifest、queue/latency traces、recovery results。
**依賴：** 6.3。
**Owner／Wave：** Primary integrator／Wave 6D。
**Gate／Evidence：** `G-FINAL-RECOVERY`、`G-FINAL-NONBLOCKING`。
**完成門檻：** 每種 fault 有 deterministic terminal disposition；validator slowdown/overflow 不阻塞 outbound；coverage gap 不被誤標 pass。

- [ ] 6.4.1 執行 duplicate、reorder、late、missing、corrupt、oversized frame scenarios。
- [ ] 6.4.2 執行 replay-ring hit、ring expiry、filtered rebase、interrupted rebase、rejoin scenarios。
- [ ] 6.4.3 執行 component repair、entity replace、team rebase 與 persistent mismatch safe termination scenarios。
- [ ] 6.4.4 故意放慢 validation worker，量測 outbound latency 並證明不等待 observer。
- [ ] 6.4.5 觸發 validation queue overflow，驗證 coverage gap、observer discard/rebootstrap 與 evidence disposition。

### 6.5 執行 security、anti-probing 與 side-channel inspection

**目的：** 證明 player packet、memory 與 diagnostics 不含 hidden/global data。
**輸入：** 6.4 passing recovery。
**產出：** `evidence/final/security/` packet captures、memory/export scans、fuzz reports、redaction review。
**依賴：** 6.4。
**Owner／Wave：** Primary integrator／Wave 6E。
**Gate／Evidence：** `G-FINAL-HIDDEN-DATA`、`G-FINAL-ANTI-PROBING`、`G-FINAL-REDACTION`。
**完成門檻：** 零 canonical/global/hidden data finding；invalid target 無 existence oracle；padding/cadence 滿足 spec；無 unresolved P0/P1 security finding。

- [ ] 6.5.1 對 secure match packet capture 掃描 canonical ID、global seed、other-team mask、hidden component value。
- [ ] 6.5.2 檢查 omfx replica memory/export、remembered cache、log、replay、crash bundle 與 trace redaction。
- [ ] 6.5.3 執行 hidden-existing/nonexistent/stale replica ID anti-probing timing/shape comparison與 rate-limit scenarios。
- [ ] 6.5.4 執行 protocol transition/snapshot/rebase fuzzing 與 replay/malformed epoch attacks。
- [ ] 6.5.5 分析 fixed cadence、padding buckets、hidden-only payload sizes 與 mass reveal/rebase chunk behavior。
- [ ] 6.5.6 驗證 admin diagnostic capability/transport 與 player session 完全隔離。

### 6.6 執行 10,000 entity performance、bandwidth 與 soak gates

**目的：** 在完整架構啟用時驗證 real-time、memory、bandwidth 與長時間穩定性。
**輸入：** 6.5 security pass、frozen production config、baseline。
**產出：** `evidence/final/performance/` raw traces、30-minute soak logs、summary/gate verdict。
**依賴：** 6.5。
**Owner／Wave：** Primary integrator／Wave 6F。
**Gate／Evidence：** `G-FINAL-TICK-BUDGET`、`G-FINAL-BANDWIDTH`、`G-FINAL-SOAK`。
**完成門檻：** p99 tick+commit ≤ 80% period、steady-state <5 KB/s/player、零 authoritative deadline miss、零 unintended rebase、memory stable、observer coverage 完整。

- [ ] 6.6.1 以 10,000 entities、2 teams、2 observer replicas、visibility churn 啟動 production-cadence stress run。
- [ ] 6.6.2 擷取 authoritative tick/commit、Wave B projection/encode/enqueue 與 observer step p50/p95/p99 raw traces。
- [ ] 6.6.3 擷取 per-player steady-state bandwidth 與 reveal/rebase burst 分布，判定 5 KB/s gate。
- [ ] 6.6.4 執行 repeated mass reveal/hide、projectile/AOE boundary churn 與 observer rebootstrap memory test。
- [ ] 6.6.5 執行 30 分鐘 soak，判定 deadline miss、unintended rebase、disconnect、coverage gap 與 memory slope。
- [ ] 6.6.6 將結果與 Phase 1 baseline 比較並產生 immutable performance verdict。

### 6.7 收斂 failure、更新 evidence lineage 與重跑受影響 group

**目的：** 修正 final verification failure，而不重跑無關完整 suite或隱藏降低 gate。
**輸入：** 6.1–6.6 results。
**產出：** Adjustment records、fixed artifacts、stale/replacement evidence links、all-green final index。
**依賴：** 6.1–6.6。
**Owner／Wave：** Primary integrator／Wave 6G。
**Gate／Evidence：** 所有 final gates。
**完成門檻：** 每個 failure 有 A/B/C disposition；affected evidence 已 stale/replaced；所有 blocking gate 最終 passed，無 failed/blocked/stale terminal record。

- [ ] 6.7.1 彙整 Phase 6 failures，為每項建立 A/B/C adjustment record；沒有 failure 時以 shared immutable no-failure record 加 unique subcheck 結案。
- [ ] 6.7.2 對 A-level refinement 修正 task mechanics/artifact，保留 scope/gate 並標記 affected evidence stale。
- [ ] 6.7.3 對 B-level correction 暫停 affected branch、同步更新 design/spec/tasks/code 並標記 dependent evidence stale。
- [ ] 6.7.4 對 C-level material change 停止 affected work並取得使用者核准；若無 C-level item，以 evidence-backed `not-applicable` 結案。
- [ ] 6.7.5 只重跑受影響的 final-verification groups，建立 replacement evidence 與 lineage links。
- [ ] 6.7.6 確認所有 blocking gates 最終 passed，且 threshold/required evidence 未被靜默降低。

### 6.8 Shadow、dogfood、secure cutover 與 legacy cleanup

**目的：** 以已通過的完整 evidence 啟用 secure V2，並移除 player global disclosure path。
**輸入：** 6.7 all-green evidence、5.4 cutover manifest。
**產出：** Shadow/dogfood acceptance、secure-default config、legacy cleanup commits、rollback record。
**依賴：** 6.7。
**Owner／Wave：** Primary integrator／Wave 6H。
**Gate／Evidence：** `G-RELEASE-SHADOW`、`G-RELEASE-DOGFOOD`、`G-SECURE-DEFAULT`。
**完成門檻：** Shadow/dogfood 無 blocker；secure default 啟用；player global snapshot/hash/seed/raw-ID path 移除；rollback 只允許 pre-match non-secure mode。

- [ ] 6.8.1 以 frozen binaries/config 執行 server shadow acceptance並保存 parity/latency/coverage evidence。
- [ ] 6.8.2 執行 internal dogfood secure matches並保存 player-visible與 diagnostic acceptance。
- [ ] 6.8.3 啟用 match-level secure V2 default，保留明確 non-secure legacy pre-match selection。
- [ ] 6.8.4 套用移除 player global `TickBatch`、`StateHash`、`WorldSnapshot`、`master_seed`、raw ECS ID 的 cleanup。
- [ ] 6.8.5 移除／quarantine dead viewport/`VisSet` authority 與 superseded nondeterministic vision path。
- [ ] 6.8.6 驗證 active secure match 無 runtime downgrade path，並保存 final rollback manifest。

### 6.9 最終 traceability 與 release review

**目的：** 證明 proposal → design → requirement/scenario → task/gate/evidence 全鏈路完整。
**輸入：** 所有 artifacts、6.8 release evidence。
**產出：** `evidence/final/traceability.md`、release verdict、archive-ready status。
**依賴：** 6.8。
**Owner／Wave：** Primary integrator／Wave 6I。
**Gate／Evidence：** `G-FINAL-TRACEABILITY`、`G-RELEASE-READY`。
**完成門檻：** 每個 blocking requirement/scenario 有 task/evidence；無 placeholder、contradiction、unresolved P0/P1 或 incomplete apply-required artifact。

- [ ] 6.9.1 建立 proposal/design/spec requirement/scenario 到 permanent task ID 與 evidence record 的 traceability matrix。
- [ ] 6.9.2 掃描 artifacts/code/evidence 的 `TODO`、`TBD`、`待補`、contradiction 與 forbidden global disclosure reference。
- [ ] 6.9.3 審查每個 L2 input/output/dependency/owner/gate/evidence/completion threshold 與每個 L3 atomicity。
- [ ] 6.9.4 確認 conditional path 皆以 passed、evidence-backed `not-applicable` 或 superseded replacement 結案。
- [ ] 6.9.5 產生 final release verdict，列出 exact binary/config/content hashes、gate summary 與 rollback boundary。
