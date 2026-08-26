# 實作任務與證據規則

所有 resolved L3 leaf 都必須在 `openspec/changes/server-authoritative-selective-lockstep/evidence/index.jsonl` 寫入唯一 `task_id` record；record 至少包含 `task_id`、`status`、`artifact_or_command`、`expected`、`actual`、`exit_status_or_reviewer`、`hashes`、`related_gates`、`adjustment_id`、`timestamp`，共享 artifact 另加 unique `subcheck`。`status` 只能是 `passed`、有證據的 `not-applicable` 或 `superseded`；failed、blocked、stale、未執行不得勾選。

Phase 1–5 只做實作、artifact review、最低限度 compile/focused smoke，不執行完整 acceptance suite。完整 unit/property、differential、cross-platform、fault、security、packet/client-memory inspection、10,000 entity 與 30 分鐘 soak 全部集中在 Phase 6。

## Luna 原子化執行契約

- 每個 L3 leaf 只允許一個主要行為、一個可獨立失敗的驗證，或一份明確 artifact；不得自行合併相鄰 leaf。
- Implementation leaf 原則上只修改 1–3 個緊密相關檔案。若實際影響超出此範圍，先以 A-level refinement 增加 leaf，不得在單一 leaf 內擴張。
- Leaf 中列出的型別、欄位、路徑、規則與 threshold 都是已核准輸入；執行者不得自行改 architecture、wire contract、authority policy 或 gate。
- 每次只領取一個 L3 task。完成時必須寫入該 task 的唯一 evidence record，再領取下一項。
- Phase 1–5 的 compile/focused smoke 只證明整合可繼續；不得提前執行或宣稱 Phase 6 acceptance suite 通過。
- 驗證 leaf 一律一條主要 command、一個平台或一個 fault/scenario；共享啟動成本可重用 immutable run，但每個 leaf 仍要有 unique `subcheck`。

## 1. Contract、分類與證據基礎

### 1.1 建立 state、event 與 projection inventory

**目的：** 產生完整且可追蹤的 gameplay state/event disclosure 分類，避免 hidden dependency 遺漏。
**輸入：** 核准設計、`omoba-core/src/runtime/**`、`omb/src/comp/**`、`omb/src/tick/**`、`scripts/base_content/**`、現有 snapshot/hash/protocol。
**產出：** `docs/selective-lockstep/state-classification.md`、`docs/selective-lockstep/projection-policy-matrix.md`。
**依賴：** 無。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-CONTRACT-STATE`；`evidence/index.jsonl` task records。
**完成門檻：** 所有 deterministic component/resource/input/event/script outcome/snapshot/hash field 都有唯一分類與四象限 projection policy disposition。

- [x] 1.1.1 盤點 `omoba-core/src/runtime/native/comp/**` 的 deterministic component，將 type 與來源檔寫入 state inventory。
- [x] 1.1.2 盤點 `omoba-core/src/runtime/native/comp/**` 的 deterministic resource，將 type 與來源檔寫入 state inventory。
- [x] 1.1.3 在 state inventory 定義唯一 classification 欄位與四個允許值。
- [x] 1.1.4 為 movement action 填寫四象限 projection policy。
- [x] 1.1.5 將未分類的 inventory item 寫入 blocking migration list。
- [x] 1.1.6 盤點 `omb/src/comp/**` 的 server-only component/resource，將 type 與來源檔寫入 state inventory。
- [x] 1.1.7 為 deterministic component inventory 補齊 owner、mutation phase、hash 與 snapshot 欄位。
- [x] 1.1.8 為 deterministic resource inventory 補齊 owner、mutation phase、hash 與 snapshot 欄位。
- [x] 1.1.9 為 input inventory 補齊 owner、authoritative phase、hash 與 snapshot 欄位。
- [x] 1.1.10 為 outcome/script event inventory 補齊 owner、authoritative phase、hash 與 snapshot 欄位。
- [x] 1.1.11 盤點 `PlayerInput` variant，記錄 producer、consumer 與 authoritative phase。
- [x] 1.1.12 盤點 `Outcome` variant，記錄 producer、consumer 與 authoritative phase。
- [x] 1.1.13 盤點 render cue，記錄 producer、consumer 與 retention rule。
- [x] 1.1.14 為 spawn action 填寫四象限 projection policy。
- [x] 1.1.15 為 death action 填寫四象限 projection policy。
- [x] 1.1.16 為 ownership action 填寫四象限 projection policy。
- [x] 1.1.17 為 direct combat action 填寫四象限 projection policy。
- [x] 1.1.18 為 projectile action 填寫四象限 projection policy。
- [x] 1.1.19 為 AOE action 填寫四象限 projection policy。
- [x] 1.1.20 為 buff/debuff action 填寫四象限 projection policy。
- [x] 1.1.21 為 hero ability action 填寫四象限 projection policy。
- [x] 1.1.22 為 tower action 填寫四象限 projection policy。
- [x] 1.1.23 為 item action 填寫四象限 projection policy。
- [x] 1.1.24 將重複分類的 inventory item 寫入 blocking migration list。
- [x] 1.1.25 將缺少 projection policy 的 action 寫入 blocking migration list。
- [x] 1.1.26 盤點 script event variant，記錄 producer、consumer 與 authoritative phase。
- [x] 1.1.27 盤點 retained network event，記錄 producer、consumer 與 retention rule。
- [x] 1.1.28 對 deterministic component inventory 套用 disclosure classification。
- [x] 1.1.29 對 deterministic resource inventory 套用 disclosure classification。
- [x] 1.1.30 對 input inventory 套用 disclosure classification。
- [x] 1.1.31 對 outcome/script event inventory 套用 disclosure classification。
- [x] 1.1.32 對 render/network event inventory 套用 disclosure classification。
- [x] 1.1.33 確認每個 inventory row 恰有一個 disclosure classification。
- [x] 1.1.34 為 render/network event inventory 補齊 owner、retention、hash 與 snapshot 欄位。

### 1.2 固定 protocol、schema、gate 與 evidence contract

**目的：** 在實作前固定 V2 public contract、blocking threshold 與證據格式。
**輸入：** 1.1 inventory、proposal、design、delta specs。
**產出：** `docs/selective-lockstep/protocol-v2-contract.md`、`docs/selective-lockstep/evidence-schema.md`、`evidence/index.jsonl`。
**依賴：** 1.1。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-CONTRACT-PROTOCOL`、`G-EVIDENCE-SCHEMA`。
**完成門檻：** V2 message/phase/version/bounds、security invariant、A/B/C adjustment 與 evidence lineage 均有無歧義定義。

- [x] 1.2.1 在 protocol contract 固定 `TeamGameStart` 欄位與 canonical order。
- [x] 1.2.2 在 timing contract 固定 `visibility_commit_delay_ticks` 的 default 3 與 bounds 2–4。
- [x] 1.2.3 在 identity contract 固定 team-scoped `ReplicaEntityId` 的配置與 retire 規則。
- [x] 1.2.4 定義 append-only evidence JSONL 的必填欄位與型別。
- [x] 1.2.5 在 evidence contract 定義 A-level refinement 的允許範圍。
- [x] 1.2.6 在 protocol contract 固定 `TeamTickFrame` envelope 欄位。
- [x] 1.2.7 在 protocol contract 固定 `PreStep` payload 與 canonical order。
- [x] 1.2.8 在 protocol contract 固定 `Step` payload 與 canonical order。
- [x] 1.2.9 在 protocol contract 固定 `PostStep` payload 與 canonical order。
- [x] 1.2.10 在 protocol contract 固定 transition message 欄位。
- [x] 1.2.11 在 protocol contract 固定 bounded random tape 欄位與 lifetime。
- [x] 1.2.12 在 protocol contract 固定 repair message 欄位與 revision rule。
- [x] 1.2.13 在 protocol contract 固定 rebase manifest 欄位與 version rule。
- [x] 1.2.14 在 timing contract 固定 `replica_buffer_ticks` 的 default 12 與 bounds 3–24。
- [x] 1.2.15 在 timing contract 固定 tick window 到 shared 120Hz helper 的換算規則。
- [x] 1.2.16 在 identity contract 固定 disclosure/view epoch 的遞增與 stale rejection 規則。
- [x] 1.2.17 在 authority contract 固定 server revision precedence 與 monotonic allocation 規則。
- [x] 1.2.18 定義 snapshot ID 的版本與唯一性規則。
- [x] 1.2.19 定義 chunk hash 的算法與輸入範圍。
- [x] 1.2.20 定義 evidence artifact hash 的算法與 canonical path 規則。
- [x] 1.2.21 定義 evidence `stale` 到 replacement record 的 lineage 欄位。
- [x] 1.2.22 建立 requirement/scenario、task ID、gate ID 與 evidence record 的 mapping schema。
- [x] 1.2.23 在 evidence contract 定義 B-level correction 的 pause、reopen 與 stale 流程。
- [x] 1.2.24 在 evidence contract 定義 C-level change 的使用者核准欄位與停止條件。
- [x] 1.2.25 在 protocol contract 固定 rebase chunk 欄位與 version rule。
- [x] 1.2.26 定義 manifest hash 的算法與輸入範圍。

### 1.3 建立 baseline 與 final-verification harness 骨架

**目的：** 保存改造前 baseline，並先建立最後階段會使用的 harness 入口，但不提前執行完整 suite。
**輸入：** 現有 `run_10000.bat`、lockstep diagnostics、1.2 evidence schema。
**產出：** `docs/selective-lockstep/baseline.json`、`tools/selective_lockstep/` harness skeleton、fixture manifest。
**依賴：** 1.2。
**Owner／Wave：** Primary integrator／Wave 1。
**Gate／Evidence：** `G-BASELINE-RECORDED`；raw logs/hash index。
**完成門檻：** 現況 CPU/memory/bandwidth 有可重現 baseline；final harness 有明確 entrypoint 與 fixture schema，但未被誤標為 acceptance pass。

- [x] 1.3.1 以 `TD_STRESS` 與 frozen 10,000-entity headless fixture 執行一次未改造 baseline run（A-20260826-001：`run_10000.bat` 現況只設定 10,000 gold，不能作為 entity-count evidence）。
- [x] 1.3.2 建立 non-interference paired-world fixture schema。
- [x] 1.3.3 在 `tools/selective_lockstep/` 建立 packet capture scan 入口。
- [x] 1.3.4 建立 final-verification evidence manifest template。
- [x] 1.3.5 將 1.3.1 的 exact command 與 config 寫入 baseline metadata。
- [x] 1.3.6 將 1.3.1 的 raw log 路徑與 content hash 寫入 baseline metadata。
- [x] 1.3.7 將 baseline machine profile 寫入 baseline metadata。
- [x] 1.3.8 從 baseline raw log 擷取 CPU 指標。
- [x] 1.3.9 從 baseline raw log 擷取 memory 指標。
- [x] 1.3.10 從 baseline raw log 擷取 per-player bandwidth 指標。
- [x] 1.3.11 在 `tools/selective_lockstep/` 建立 paired-world fixture generator 入口。
- [x] 1.3.12 在 `tools/selective_lockstep/` 建立 redaction scan 入口。
- [x] 1.3.13 在 `tools/selective_lockstep/` 建立 network fault injection 入口。
- [x] 1.3.14 在 `tools/selective_lockstep/` 建立 observer slowdown 入口。
- [x] 1.3.15 在 `tools/selective_lockstep/` 建立 stress report 入口。
- [x] 1.3.16 在 manifest template 標示 Phase 1–5 smoke evidence 不可滿足 acceptance gate。

## 2. Protocol V2 與 shared selective replica foundation

### 2.1 實作 V2 wire schema 與 compatibility boundary

**目的：** 提供 team-specific bootstrap/frame/recovery wire type 與 match-level negotiation。
**輸入：** 1.2 protocol contract、`proto/game.proto`、既有 KCP framing。
**產出：** 更新的 `proto/game.proto`、generated Rust types、framing tags/version guards。
**依賴：** 1.2。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-V2-SCHEMA`；proto/generated hashes。
**完成門檻：** 所有 proposal-listed V2 message 可 encode/decode，V1/V2 match negotiation 有明確拒絕路徑，player V2 type 不含 global seed/raw ECS ID。

- [x] 2.1.1 在 `proto/game.proto` 定義 `TeamGameStart` message。
- [x] 2.1.2 在 `proto/game.proto` 定義 `RevealEntity` message。
- [x] 2.1.3 在 `proto/game.proto` 定義 `ReplicaEntityId` 欄位型別。
- [x] 2.1.4 在 `omoba-core/src/kcp/framing.rs` 新增 V2 framing tag。
- [x] 2.1.5 以既有 code generation 流程更新 `omoba-core/src/generated/game.rs`，不手改 generated output。
- [x] 2.1.6 在 `proto/game.proto` 定義 `TeamTickFrame` envelope。
- [x] 2.1.7 在 `proto/game.proto` 定義 `PreStep` payload。
- [x] 2.1.8 在 `proto/game.proto` 定義 `Step` payload。
- [x] 2.1.9 在 `proto/game.proto` 定義 `PostStep` payload。
- [x] 2.1.10 在 `proto/game.proto` 定義 `HideEntity` message。
- [x] 2.1.11 在 `proto/game.proto` 定義 `ForgetEntity` message。
- [x] 2.1.12 在 `proto/game.proto` 定義 `ComponentRepair` message。
- [x] 2.1.13 在 `proto/game.proto` 定義 `EntityReplace` message。
- [x] 2.1.14 在 `proto/game.proto` 定義 `TeamViewRebase` manifest message。
- [x] 2.1.15 在 `proto/game.proto` 定義 `TeamViewRebase` chunk message。
- [x] 2.1.16 在 `proto/game.proto` 定義 view epoch 欄位型別。
- [x] 2.1.17 在 `proto/game.proto` 定義 authority revision 欄位型別。
- [x] 2.1.18 在 `proto/game.proto` 定義 bounded random tape message。
- [x] 2.1.19 在 `proto/game.proto` 定義 sanitized external effect message。
- [x] 2.1.20 在 `proto/game.proto` 定義 filtered snapshot message。
- [x] 2.1.21 在 `omoba-core/src/kcp/framing.rs` 新增 protocol/schema version constants。
- [x] 2.1.22 在 `omoba-core/src/transport.rs` 定義 match capability negotiation type。
- [x] 2.1.23 在 V2 player schema allowlist 中排除 global seed 欄位。
- [x] 2.1.24 在 V2 player schema allowlist 中排除 raw ECS ID 欄位。
- [x] 2.1.25 在 `proto/game.proto` 定義 disclosure epoch 欄位型別。

### 2.2 實作 team identity 與 filtered snapshot primitives

**目的：** 建立不暴露 canonical identity 的 team world bootstrap。
**輸入：** 2.1 schema、1.1 state classification。
**產出：** `omoba-core::runtime` team identity、filtered snapshot encode/decode、manifest/chunk support。
**依賴：** 2.1。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-IDENTITY-ISOLATION`、`G-FILTERED-SNAPSHOT`。
**完成門檻：** 每隊 mapping 獨立、ID 不重用、stale epoch 可拒絕；snapshot 只含 classified safe state。

- [x] 2.2.1 在 `omoba-core/src/runtime/` 新增 monotonic non-reused `ReplicaEntityId` allocator。
- [x] 2.2.2 在 team identity state 實作 disclosure epoch increment。
- [x] 2.2.3 建立 filtered snapshot builder 的輸入與輸出 shell。
- [x] 2.2.4 實作 snapshot ID allocator。
- [x] 2.2.5 實作 filtered snapshot schema-version compatibility guard。
- [x] 2.2.6 實作每個 team 獨立的 canonical-to-replica mapping。
- [x] 2.2.7 實作每個 team 獨立的 replica-to-canonical server lookup。
- [x] 2.2.8 實作 replica ID retire set，阻止同 match 內重用。
- [x] 2.2.9 實作 remembered interval 的 replica ID retention。
- [x] 2.2.10 實作 authoritative forget 的 replica ID retirement。
- [x] 2.2.11 實作 stale disclosure epoch lookup rejection。
- [x] 2.2.12 將 filtered snapshot builder 的 component 來源限制為 classification allowlist。
- [x] 2.2.13 將 filtered snapshot builder 的 entity 來源限制為 resolved team view。
- [x] 2.2.14 實作 snapshot chunk encoder。
- [x] 2.2.15 實作單一 chunk hash 計算。
- [x] 2.2.16 實作 snapshot manifest encoder。
- [x] 2.2.17 實作 manifest hash 驗證。
- [x] 2.2.18 實作 incomplete snapshot staging area discard。
- [x] 2.2.19 將 V2 filtered snapshot type 與 global `SnapshotStore` type 分離。
- [x] 2.2.20 在 V2 player encoder 拒絕 global `SnapshotStore` type。

### 2.3 實作 `SelectiveReplicaRuntime`

**目的：** 提供 omfx 與 server observer 共用的 deterministic replica runtime。
**輸入：** 2.1/2.2、既有 `SimulationDriver` 與 runtime initialization。
**產出：** `omoba-core::runtime::SelectiveReplicaRuntime`、canonical team hash、transition/repair/rebase application。
**依賴：** 2.1、2.2。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-SHARED-REPLICA-RUNTIME`。
**完成門檻：** Runtime 可從 filtered snapshot bootstrap，依 phase 推進、停在 gap barrier、套用 authority revision 並輸出 filtered render snapshot/hash。

- [x] 2.3.1 在 `SelectiveReplicaRuntime` 實作 `RevealEntity` application。
- [x] 2.3.2 在 `SelectiveReplicaRuntime` 實作 accepted input injection。
- [x] 2.3.3 在 `SelectiveReplicaRuntime` 執行一個 disclosed-world fixed tick。
- [x] 2.3.4 在 `SelectiveReplicaRuntime` 實作 `ComponentRepair` application。
- [x] 2.3.5 實作 expected `team_sequence` barrier。
- [x] 2.3.6 實作 canonical team hash。
- [x] 2.3.7 在 `SelectiveReplicaRuntime` 實作 `HideEntity` application。
- [x] 2.3.8 在 `SelectiveReplicaRuntime` 實作 `ForgetEntity` application。
- [x] 2.3.9 為 transition application 實作 revision-based idempotence guard。
- [x] 2.3.10 在 `SelectiveReplicaRuntime` 實作 public event injection。
- [x] 2.3.11 在 `SelectiveReplicaRuntime` 實作 sanitized external effect injection。
- [x] 2.3.12 在 `SelectiveReplicaRuntime` 實作 bounded random tape injection。
- [x] 2.3.13 將 fixed tick 的 component/resource access 限制為 disclosed-world allowlist。
- [x] 2.3.14 將 remembered presentation cache 排除於 runtime resource set。
- [x] 2.3.15 在 `SelectiveReplicaRuntime` 實作 `EntityReplace` application。
- [x] 2.3.16 在 `SelectiveReplicaRuntime` 實作 complete `TeamViewRebase` swap。
- [x] 2.3.17 在所有 `PostStep` correction 套用 monotonic authority revision check。
- [x] 2.3.18 在 revision conflict 時實作 server-wins overwrite。
- [x] 2.3.19 實作 expected replica tick barrier。
- [x] 2.3.20 實作 missing frame 的 deterministic stall state。
- [x] 2.3.21 實作 replay frame 抵達後的 resume transition。
- [x] 2.3.22 實作 verified rebase swap 後的 resume transition。
- [x] 2.3.23 實作 filtered render snapshot extraction。
- [x] 2.3.24 將 remembered presentation 排除於 canonical team hash。

### 2.4 準備 synthetic fixtures 與最低限度 build

**目的：** 讓後續 server/omfx 可整合 shared runtime，不提前執行完整 determinism/fault suite。
**輸入：** 2.1–2.3。
**產出：** Synthetic encoded-frame fixture、server observer/client fixture constructors、Phase 2 build log。
**依賴：** 2.3。
**Owner／Wave：** Primary integrator／Wave 2。
**Gate／Evidence：** `G-PHASE2-BUILDABLE`，非 acceptance gate。
**完成門檻：** Shared crates 與 fixture code 可編譯；完整測試明確 deferred 到 Phase 6。

- [x] 2.4.1 建立從 encoded bytes 初始化 synthetic client fixture 的 constructor。
- [x] 2.4.2 建立單一 reveal frame fixture。
- [x] 2.4.3 執行一次最低限度 `omoba-core` compile check，將 log 標記為 non-acceptance evidence。
- [x] 2.4.4 建立從相同 encoded bytes 初始化 observer fixture 的 constructor。
- [x] 2.4.5 建立單一 hide frame fixture。
- [x] 2.4.6 建立單一 component repair frame fixture。
- [x] 2.4.7 建立單一 entity replace frame fixture。
- [x] 2.4.8 建立單一 rebase manifest/chunk fixture。
- [x] 2.4.9 執行一次最低限度 protocol codegen compile check，將 log 標記為 non-acceptance evidence。

## 3. Deterministic Specs projection pipeline

### 3.1 建立 stable `Outcome`/`ObservableFact` buffers

**目的：** 在 gameplay 計算期間同步產生 projection fact，且不受 thread completion order 影響。
**輸入：** 1.1 policy matrix、現有 `Outcome`/runtime event flow。
**產出：** Stable key types、sharded/thread-local buffers、deterministic reducer。
**依賴：** 1.1、2.1。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-OBSERVABLE-FACT-CONTRACT`。
**完成門檻：** 每筆 outcome/fact 有 stable key，commit 不依賴 insertion/arrival order，沒有 post-hoc full-world effect scan。

- [x] 3.1.1 在 shared runtime 定義 `ObservableFact` enum shell。
- [x] 3.1.2 實作 Specs-safe sharded/thread-local output buffers。
- [x] 3.1.3 實作 sharded fact buffer 的 deterministic merge。
- [x] 3.1.4 從既有 runtime event bridge 移除 unordered direct emit path。
- [x] 3.1.5 為每個 `ObservableFact` variant 定義 safe metadata payload。
- [x] 3.1.6 定義 fact ordering key 的 `tick` 欄位。
- [x] 3.1.7 定義 fact ordering key 的 `phase` 欄位。
- [x] 3.1.8 定義 fact ordering key 的 `canonical_source_order` 欄位。
- [x] 3.1.9 定義 fact ordering key 的 `local_ordinal` 欄位。
- [x] 3.1.10 定義 fact ordering key 的 `fact_kind` 欄位。
- [x] 3.1.11 實作 merged fact buffer 的 stable sort。
- [x] 3.1.12 實作 exact duplicate fact 的 deterministic dedupe。
- [x] 3.1.13 實作 malformed ordering key 的 fail-closed rejection。
- [x] 3.1.14 將 ordered `Outcome` 輸入既有 runtime event bridge。
- [x] 3.1.15 將 ordered `ObservableFact` 輸入新的 team projection bridge。

### 3.2 遷移 gameplay 與 script projection policy

**目的：** 讓所有現有 gameplay action 具有完整四象限 visibility behavior。
**輸入：** 1.1 migration list、3.1 buffer contract。
**產出：** 更新的 omb/omoba-core gameplay systems、script outcome bridge、policy registry。
**依賴：** 3.1。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-PROJECTION-POLICY-COMPLETE`。
**完成門檻：** Inventory 中每個 action 都有 code-owned policy；缺少 policy 時 secure startup/content validation fail closed。

- [x] 3.2.1 在 movement tick producer 產生 movement `ObservableFact`。
- [x] 3.2.2 在 direct damage producer 產生 combat projection fact。
- [x] 3.2.3 在 hero action producer 註冊 projection policy ID。
- [x] 3.2.4 在 `scripts/script-abi` 定義 abi-stable projection policy ID 型別。
- [x] 3.2.5 實作 projection policy registry 的 missing-ID detection。
- [x] 3.2.6 在 entity creation producer 產生 spawn `ObservableFact`。
- [x] 3.2.7 在 death producer 產生 death `ObservableFact`。
- [x] 3.2.8 在 ownership mutation producer 產生 ownership `ObservableFact`。
- [x] 3.2.9 在 projectile spawn producer 產生 projectile creation fact。
- [x] 3.2.10 在 projectile movement producer 產生 projectile movement fact。
- [x] 3.2.11 在 projectile removal producer 產生 projectile removal fact。
- [x] 3.2.12 在 AOE resolution producer 產生 AOE projection fact。
- [x] 3.2.13 在 buff apply producer 產生 buff projection fact。
- [x] 3.2.14 在 buff removal producer 產生 buff removal fact。
- [x] 3.2.15 在 debuff apply producer 產生 debuff projection fact。
- [x] 3.2.16 在 tower place producer 註冊 projection policy ID。
- [x] 3.2.17 在 tower sell producer 註冊 projection policy ID。
- [x] 3.2.18 在 tower upgrade producer 註冊 projection policy ID。
- [x] 3.2.19 在 item use producer 註冊 projection policy ID。
- [x] 3.2.20 在 ability activation producer 註冊 projection policy ID。
- [x] 3.2.21 將 retained HUD event 轉成明確 audience 的 projection fact。
- [x] 3.2.22 將 retained terminal event 轉成明確 audience 的 projection fact。
- [x] 3.2.23 在 script host adapter 接收 abi-stable projection policy ID。
- [x] 3.2.24 在 script host adapter 將 script outcome 轉成 host `ObservableFact`。
- [x] 3.2.25 確認 `scripts/script-abi` 未新增 `specs` dependency。
- [x] 3.2.26 確認 `scripts/script-abi` 未新增 runtime-heavy serialization dependency。
- [x] 3.2.27 為 missing policy error 加入 action ID。
- [x] 3.2.28 為 missing policy error 加入 source module/path。
- [x] 3.2.29 在 secure match startup 執行 policy registry completeness check。
- [x] 3.2.30 在 completeness check 失敗時阻止 secure match 啟動。

### 3.3 實作 Wave A commit 與 Wave B visibility

**目的：** 在同一 Specs tick pipeline 中保留平行計算與 post-step visibility correctness。
**輸入：** 3.1/3.2、既有 dispatcher/process_outcomes。
**產出：** Wave A reduce/commit barrier、fixed-point visibility systems、per-team Wave B jobs。
**依賴：** 3.1、3.2。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-TWO-WAVE-PIPELINE`。
**完成門檻：** `State[T+1]` 只在 deterministic commit 後可見；team jobs 讀 committed state 並可平行執行。

- [x] 3.3.1 將 stable outcome reduce 插入 authoritative tick commit input。
- [x] 3.3.2 在 `omoba-core/src/runtime/native/comp/` 定義 fixed-point `ReplicationScope` component。
- [x] 3.3.3 實作 team-shared geometry visibility resolve。
- [x] 3.3.4 實作 reveal candidate creation。
- [x] 3.3.5 實作 `TeamVisibilityIndex` current-view storage。
- [x] 3.3.6 在 `omoba-core/src/runtime/native/comp/` 定義 fixed-point `VisionSource` component。
- [x] 3.3.7 在 `omoba-core/src/runtime/native/comp/` 定義 fixed-point `StealthProfile` component。
- [x] 3.3.8 在 `omoba-core/src/runtime/native/comp/` 定義 `VisibilityOverride` component。
- [x] 3.3.9 在 `omoba-core/src/runtime/native/comp/` 定義 `RememberPolicy` component。
- [x] 3.3.10 實作 team vision source aggregation。
- [x] 3.3.11 實作 stealth 與 detection level comparison。
- [x] 3.3.12 實作 `ServerOnly` deny precedence。
- [x] 3.3.13 實作 force-hide precedence。
- [x] 3.3.14 實作 `Public` 與 force-show precedence。
- [x] 3.3.15 實作 `OwnerTeam` precedence。
- [x] 3.3.16 實作同 priority override 的 stable rule ID tie-break。
- [x] 3.3.17 實作 hide candidate creation。
- [x] 3.3.18 實作 candidate commitment tick calculation。
- [x] 3.3.19 實作 reveal candidate cancellation。
- [x] 3.3.20 實作 hide candidate cancellation。
- [x] 3.3.21 在 reveal effective tick 擷取 fresh authoritative baseline。
- [x] 3.3.22 在 hide effective tick 產生 remembered-policy disposition。
- [x] 3.3.23 實作 per-team visibility history ring。
- [x] 3.3.24 在 commit barrier 後建立 Wave B read-only state view。
- [x] 3.3.25 在 Specs dispatcher 註冊 per-team Wave B jobs。
- [x] 3.3.26 將不同 team 的 Wave B job 設為可平行排程。
- [x] 3.3.27 將 stable fact reduce 插入 authoritative tick commit input。
- [x] 3.3.28 在 outcome/fact reduce 完成後設置 deterministic commit barrier。

### 3.4 實作 `TeamViewProjector` 與 frame builder

**目的：** 將 committed state/facts 轉成 canonical、安全且可重播的 team frame。
**輸入：** 2.x shared protocol/runtime、3.3 visibility。
**產出：** Per-team projector、external effect sanitizer、frame encoder、padding buckets。
**依賴：** 2.3、3.3。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-TEAM-PROJECTION`、`G-NONINTERFERENCE-READY`。
**完成門檻：** Projector 不輸出 forbidden state；同 logical payload canonical bytes 穩定；hidden dependency 轉成 sanitized outcome。

- [x] 3.4.1 在 `TeamViewProjector` 實作 fact audience filter。
- [x] 3.4.2 實作 hidden-attacker damage sanitizer。
- [x] 3.4.3 實作 `PreStep` canonical frame assembly。
- [x] 3.4.4 實作 fixed-cadence empty frame assembly。
- [x] 3.4.5 實作 authoritative expected team hash projection。
- [x] 3.4.6 在 `TeamViewProjector` 實作 component field redaction。
- [x] 3.4.7 實作 disclosed dependency closure traversal。
- [x] 3.4.8 實作 dependency closure 的 `ServerOnly` fail-closed guard。
- [x] 3.4.9 實作 hidden-source buff sanitizer。
- [x] 3.4.10 實作 hidden-source debuff sanitizer。
- [x] 3.4.11 實作 hidden projectile external effect sanitizer。
- [x] 3.4.12 實作 hidden AOE external effect sanitizer。
- [x] 3.4.13 實作 `Step` canonical frame assembly。
- [x] 3.4.14 實作 `PostStep` canonical frame assembly。
- [x] 3.4.15 實作 phase 內 event-kind ordering。
- [x] 3.4.16 實作 phase 內 replica-ID ordering。
- [x] 3.4.17 實作 phase 內 stable sub-index ordering。
- [x] 3.4.18 實作 steady-state size bucket selection。
- [x] 3.4.19 實作 frame padding bytes generation。
- [x] 3.4.20 實作 mass reveal chunk policy。
- [x] 3.4.21 實作 rebase chunk rate-limit policy。
- [x] 3.4.22 實作不含 hidden source 的 mismatch metadata。

### 3.5 Phase 3 最低限度 integration build

**目的：** 確認 pipeline 可供 transport 整合，不提前執行完整 test matrix。
**輸入：** 3.1–3.4。
**產出：** Phase 3 compile/focused smoke log。
**依賴：** 3.4。
**Owner／Wave：** Primary integrator／Wave 3。
**Gate／Evidence：** `G-PHASE3-BUILDABLE`，非 acceptance gate。
**完成門檻：** Authoritative tick 可產生 synthetic team frames，相關 workspaces 可編譯。

- [x] 3.5.1 執行一次 `omoba-core` 最低限度 compile check，保存 non-acceptance log。
- [x] 3.5.2 以單一 synthetic tick focused smoke 確認 Wave A output 抵達 commit buffer。
- [x] 3.5.3 執行一次 omb 最低限度 compile check，保存 non-acceptance log。
- [x] 3.5.4 在同一 synthetic smoke 確認 commit 後才建立 Wave B read view。
- [x] 3.5.5 在同一 synthetic smoke 確認 Wave B 產生一個 encoded team frame。

## 4. Server team stream、recovery 與 observer sidecar

### 4.1 實作 team session routing 與 replay ring

**目的：** 將 V2 frame 只送給綁定 team，並提供 sequence recovery。
**輸入：** 2.1 negotiation、3.4 encoded frames、現有 KCP session map。
**產出：** Team-bound sessions、direct outbound enqueue、per-team encoded replay ring。
**依賴：** 2.1、3.4。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-TEAM-ROUTING`。
**完成門檻：** Secure session 只收自身 team frame；encoded frame 立即 enqueue；gap 可由 ring/rebase 路由恢復。

- [x] 4.1.1 在 join state 綁定 negotiated protocol version。
- [x] 4.1.2 將 encoded `TeamTickFrame` 直接 enqueue 給同 team sessions。
- [x] 4.1.3 實作 bounded per-team encoded-frame replay ring insert。
- [x] 4.1.4 實作 ring expiry 後 filtered rebase routing。
- [x] 4.1.5 在 join handler 拒絕 V1 client 加入 secure match。
- [x] 4.1.6 在 join state 綁定 authenticated team ID。
- [x] 4.1.7 在 session state 綁定 current view epoch。
- [x] 4.1.8 在 session state 綁定 secure-match capability。
- [x] 4.1.9 在 `omb/src/transport/kcp_transport.rs` 將 team frame 路由到相同 team session set。
- [x] 4.1.10 從 secure path 移除 global `TickBatch` fan-out call。
- [x] 4.1.11 從 secure path 移除 global `StateHash` fan-out call。
- [x] 4.1.12 實作 replay ring sequence lookup。
- [x] 4.1.13 實作 replay ring exact encoded-byte resend。
- [x] 4.1.14 實作 duplicate replay request 的 idempotent response。
- [x] 4.1.15 實作 filtered rebase 完成後的 catch-up frame routing。
- [x] 4.1.16 在 active secure match control path 拒絕 runtime downgrade。

### 4.2 實作非阻塞 observer validation worker

**目的：** 在同 process 另一條 thread 模擬每隊 observer，不阻塞 outbound。
**輸入：** 4.1 encoded stream、2.3 runtime。
**產出：** Validation tap/channel/worker、per-team observer lifecycle、audit metrics。
**依賴：** 2.3、4.1。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-OBSERVER-SIDECAR`、`G-OUTBOUND-NONBLOCKING`。
**完成門檻：** Observer 只消費 filtered bootstrap/actual bytes；validator backpressure 不影響 outbound；每隊 lifecycle 隔離。

- [x] 4.2.1 建立 bounded validation channel。
- [x] 4.2.2 建立獨立 validation worker thread lifecycle。
- [x] 4.2.3 讓 observer 經 V2 filtered bootstrap path 初始化。
- [x] 4.2.4 實作 observer current tick tracking。
- [x] 4.2.5 在 validation channel overflow 時建立 coverage-gap record。
- [x] 4.2.6 建立 active-team 到 observer replica 的 map。
- [x] 4.2.7 在 team 啟用時建立 observer replica。
- [x] 4.2.8 在 team 結束時釋放 observer replica。
- [x] 4.2.9 讓 observer decode outbound 使用的實際 encoded frame bytes。
- [x] 4.2.10 阻止 observer 讀取 authoritative Specs world handle。
- [x] 4.2.11 阻止 observer 讀取 canonical ID mapping。
- [x] 4.2.12 實作 observer team hash 計算。
- [x] 4.2.13 實作 validation audit lag metric。
- [x] 4.2.14 實作 validation queue depth metric。
- [x] 4.2.15 實作 per-team verified sequence coverage tracking。
- [x] 4.2.16 在 coverage gap 後 discard stale observer。
- [x] 4.2.17 以 filtered snapshot rebootstrap discarded observer。
- [x] 4.2.18 從 retained frame 恢復 rebootstrap observer 的 sequence。
- [x] 4.2.19 將 outbound 使用的 encoded `Arc<[u8]>` clone 到 validation channel。

### 4.3 實作 mismatch control 與 authority recovery

**目的：** 將 client/observer divergence 轉成後續 server-authoritative correction。
**輸入：** 2.3 repair/rebase、3.4 team hash、4.2 mismatch signal。
**產出：** `AuthorityRepairCoordinator`、safe diagnostic bundle、client gap/rejoin handlers。
**依賴：** 3.4、4.2。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-AUTHORITY-RECOVERY`。
**完成門檻：** First divergence 可定位；repair/rebase 在 later frame 發出；無法恢復時 secure match fail closed。

- [x] 4.3.1 定義 observer mismatch control message。
- [x] 4.3.2 實作 component-level repair selection。
- [x] 4.3.3 實作 entity replace selection。
- [x] 4.3.4 實作 player rejoin 的 filtered snapshot routing。
- [x] 4.3.5 實作持續 recovery failure 的 secure match safe termination。
- [x] 4.3.6 定義 client hash mismatch control message。
- [x] 4.3.7 定義 safe first-divergence record schema。
- [x] 4.3.8 在 first-divergence record 寫入 team 與 frame sequence。
- [x] 4.3.9 在 first-divergence record 寫入 safe component path。
- [x] 4.3.10 實作 monotonic authority revision allocation。
- [x] 4.3.11 將 `ComponentRepair` 排入後續 `PostStep` frame。
- [x] 4.3.12 將 `EntityReplace` 排入後續 `PostStep` frame。
- [x] 4.3.13 實作 full filtered `TeamViewRebase` selection threshold。
- [x] 4.3.14 將 selected `TeamViewRebase` 排入後續 authority stream。
- [x] 4.3.15 實作 observer coverage-gap rebootstrap request handler。
- [x] 4.3.16 實作 interrupted rebase 的 staging discard。
- [x] 4.3.17 實作 rebase manifest 驗證失敗的 retry disposition。
- [x] 4.3.18 在 retry 上限耗盡時進入 safe termination path。
- [x] 4.3.19 在 safe termination diagnostic 中套用 team redaction。
- [x] 4.3.20 在 safe termination path 禁止 global protocol fallback。

### 4.4 實作 anti-probing、redaction 與 observability

**目的：** 封閉 player/session 與 diagnostic side channel。
**輸入：** 1.1 classification、3.4 projector、4.1/4.3 transport/recovery。
**產出：** Input visibility validation、generalized rejection、padding/redaction、metrics/traces。
**依賴：** 3.4、4.1、4.3。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-ANTI-PROBING-READY`、`G-REDACTION-READY`。
**完成門檻：** Player-visible outputs 不含 forbidden fields；invalid target 不形成 existence oracle；admin diagnostic boundary 明確隔離。

- [x] 4.4.1 將 target input wire field 改為 team-scoped replica ID。
- [x] 4.4.2 實作 invalid target 的 generalized rejection class。
- [x] 4.4.3 在 player log sink 套用 team redaction。
- [x] 4.4.4 實作 visibility transition count metric。
- [x] 4.4.5 實作 steady-state padding byte metric。
- [x] 4.4.6 在 target input 加入 view epoch field。
- [x] 4.4.7 在 target input 加入 disclosure epoch field。
- [x] 4.4.8 依 session team binding 驗證 target input。
- [x] 4.4.9 依 input tick visibility history 驗證 target input。
- [x] 4.4.10 依 ownership rule 驗證 target input。
- [x] 4.4.11 實作 invalid target 的 uniform timing bucket。
- [x] 4.4.12 實作 invalid replica reference rate limit。
- [x] 4.4.13 在 player replay sink 套用 team redaction。
- [x] 4.4.14 在 player crash bundle 套用 team redaction。
- [x] 4.4.15 在 player trace sink 套用 team redaction。
- [x] 4.4.16 建立 server-admin diagnostic capability check。
- [x] 4.4.17 將 full diagnostic transport 與 player session transport 分離。
- [x] 4.4.18 實作 encoded frame byte metric。
- [x] 4.4.19 實作 outbound queue depth metric。
- [x] 4.4.20 實作 observer audit lag metric export。
- [x] 4.4.21 實作 coverage gap metric export。
- [x] 4.4.22 實作 authority repair count metric。
- [x] 4.4.23 實作 authority rebase count metric。
- [x] 4.4.24 實作 redaction violation counter。
- [x] 4.4.25 實作 reveal burst byte accounting。
- [x] 4.4.26 實作 rebase burst byte accounting。

### 4.5 Phase 4 最低限度 server build

**目的：** 確認 server integration 可供 omfx 對接，不提前執行完整 fault/security/stress suite。
**輸入：** 4.1–4.4。
**產出：** Server compile/focused connection smoke log。
**依賴：** 4.4。
**Owner／Wave：** Primary integrator／Wave 4。
**Gate／Evidence：** `G-PHASE4-BUILDABLE`，非 acceptance gate。
**完成門檻：** omb V2 path 可編譯，synthetic client 可完成一次 filtered join 與一個 frame receive。

- [x] 4.5.1 執行一次 omb 最低限度 compile check，保存 non-acceptance log。
- [x] 4.5.2 執行一次 synthetic filtered join focused smoke。
- [x] 4.5.3 在相同 synthetic session 接收一個 team frame。
- [x] 4.5.4 確認 4.5.2–4.5.3 evidence 未標記為 recovery/security acceptance。

## 5. omfx selective replica 與 cutover preparation

### 5.1 將 omfx sim runner 遷移到 `SelectiveReplicaRuntime`

**目的：** 讓 native frontend 只持有 team disclosed deterministic world。
**輸入：** 2.3 shared runtime、4.1 V2 client stream。
**產出：** omfx V2 lockstep client/sim runner、team bootstrap、barrier buffer。
**依賴：** 2.3、4.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-OMFX-SELECTIVE-RUNTIME`。
**完成門檻：** omfx 不建立 global world、不讀 `master_seed`/raw ECS ID，並以 negotiated barrier 推進。

- [x] 5.1.1 在 `omfx/game/src/lockstep_client.rs` decode `TeamGameStart`。
- [x] 5.1.2 在 `omfx/game/src/sim_runner.rs` 新增 `SelectiveReplicaRuntime` owner field。
- [x] 5.1.3 在 omfx client 建立 negotiated barrier buffer。
- [x] 5.1.4 將 accepted input 導入 shared runtime。
- [x] 5.1.5 在 replica stall 時保留最後一份 disclosed render snapshot。
- [x] 5.1.6 在 `omfx/game/src/lockstep_client.rs` decode `TeamTickFrame`。
- [x] 5.1.7 在 omfx client decode replay response control。
- [x] 5.1.8 在 omfx client decode rebase manifest control。
- [x] 5.1.9 在 omfx client decode rebase chunk control。
- [x] 5.1.10 將 `TeamGameStart` filtered snapshot 傳入 `SelectiveReplicaRuntime` bootstrap。
- [x] 5.1.11 從 secure client path 移除 global world bootstrap call。
- [x] 5.1.12 將 barrier buffer default 設為 negotiated 12 ticks。
- [x] 5.1.13 在 barrier buffer 追蹤 expected team sequence。
- [x] 5.1.14 在 expected frame 未到時回報 deterministic stall state。
- [x] 5.1.15 將 public event 導入 shared runtime。
- [x] 5.1.16 將 sanitized external effect 導入 shared runtime。
- [x] 5.1.17 將 visibility transition 導入 shared runtime。
- [x] 5.1.18 將 authority correction 導入 shared runtime。
- [x] 5.1.19 在 replica stall 時維持 network receive loop 運作。
- [x] 5.1.20 在 replica stall 時維持 input collection loop 運作。
- [x] 5.1.21 在 replica stall 時維持 UI loop 運作。

### 5.2 實作 filtered rendering 與 remembered cache

**目的：** Render disclosed state 並以獨立 cache 呈現允許的 last-known ghost。
**輸入：** 5.1 runtime snapshots、`RememberPolicy` transition。
**產出：** omfx render bridge、remembered cache、transition presentation/cache cleanup。
**依賴：** 5.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-FILTERED-RENDERING`。
**完成門檻：** Hidden entity 不在 deterministic snapshot/render cache；remembered data 不可 target/hash；reveal 可關聯 prior remembered ID。

- [x] 5.2.1 將 render bridge identity 改為 team-scoped replica ID。
- [x] 5.2.2 將 hide/forget transition 路由到 render scene cleanup handler。
- [x] 5.2.3 實作 `LastKnown` remembered presentation cache insert。
- [x] 5.2.4 阻止 remembered cache 進入 target lookup。
- [x] 5.2.5 定義 re-reveal 對 remembered presentation 的 association key。
- [x] 5.2.6 在 hide transition 從 deterministic scene 移除 entity node。
- [x] 5.2.7 在 forget transition retire deterministic scene identity。
- [x] 5.2.8 實作 custom remembered presentation registry lookup。
- [x] 5.2.9 實作 remembered presentation cache expiry rule。
- [x] 5.2.10 實作 remembered presentation cache explicit removal。
- [x] 5.2.11 阻止 remembered cache 進入 collision query。
- [x] 5.2.12 阻止 remembered cache 進入 simulation resource。
- [x] 5.2.13 阻止 remembered cache 進入 team hash input。
- [x] 5.2.14 在 re-reveal 時查找可沿用的 remembered replica ID。
- [x] 5.2.15 在 re-reveal baseline 套用後替換 remembered presentation。

### 5.3 實作 client recovery、authority 與 diagnostics

**目的：** 讓 omfx 在 gap/mismatch/rebase 後按 server 指示收斂。
**輸入：** 4.3 server recovery、5.1 runtime。
**產出：** Replay request、repair/rebase handling、redacted client diagnostics。
**依賴：** 4.3、5.1。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-OMFX-RECOVERY`。
**完成門檻：** Duplicate/gap/late/correction/rebase 有 deterministic terminal path；client 不嘗試 global snapshot fallback。

- [x] 5.3.1 實作 duplicate team sequence detection。
- [x] 5.3.2 在 client barrier 套用 `ComponentRepair`。
- [x] 5.3.3 實作 rebase chunk hash verification。
- [x] 5.3.4 實作 client team hash report。
- [x] 5.3.5 在 secure client recovery dispatcher 加入 global-fallback denial guard。
- [x] 5.3.6 實作 missing team sequence gap detection。
- [x] 5.3.7 在 gap detection 後送出 replay request。
- [x] 5.3.8 對 duplicate frame 執行 idempotent ignore。
- [x] 5.3.9 在 client barrier 套用 `EntityReplace`。
- [x] 5.3.10 將完整 `TeamViewRebase` staging swap 排在指定 barrier。
- [x] 5.3.11 實作 rebase manifest hash verification。
- [x] 5.3.12 實作 rebase chunk completeness check。
- [x] 5.3.13 在 rebase 中斷時 discard staging snapshot。
- [x] 5.3.14 實作 barrier stall metric。
- [x] 5.3.15 實作 client gap metric。
- [x] 5.3.16 實作 client rebase metric。
- [x] 5.3.17 實作 team-redacted client diagnostic bundle。
- [x] 5.3.18 從 secure client path 移除 global snapshot fallback。
- [x] 5.3.19 從 secure client path 移除 global state-hash fallback。
- [x] 5.3.20 從 secure client path 移除 master-seed fallback。

### 5.4 準備 match negotiation、shadow 與 cleanup

**目的：** 準備可逆的 V2 shadow/dogfood 與最終 cleanup，不在 final verification 前 irreversible cutover。
**輸入：** 4.x server、5.1–5.3 client。
**產出：** Match-level config、shadow/dogfood switches、cleanup patch set/manifest。
**依賴：** 4.5、5.3。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-CUTOVER-PREPARED`。
**完成門檻：** V2 可 opt-in；legacy 僅限 non-secure match；global path cleanup 已準備但未在 final evidence 前啟用。

- [x] 5.4.1 在 match config 定義 `secure_v2` mode 欄位。
- [x] 5.4.2 實作 server shadow generation switch。
- [x] 5.4.3 準備移除 player global `TickBatch` fan-out 的 patch。
- [x] 5.4.4 準備移除 dead `client_visibility` field 的 patch。
- [x] 5.4.5 建立 cutover/rollback manifest，明確禁止 active secure match downgrade。
- [x] 5.4.6 在 match config 加入 secure V2 opt-in mode。
- [x] 5.4.7 在 match negotiation 拒絕 secure match 混用 V1/V2 client。
- [x] 5.4.8 實作 internal dogfood enable switch。
- [x] 5.4.9 準備移除 player global `StateHash` fan-out 的 patch。
- [x] 5.4.10 準備移除 player global `WorldSnapshot` bootstrap 的 patch。
- [x] 5.4.11 準備移除 player `master_seed` delivery 的 patch。
- [x] 5.4.12 準備移除 player raw ECS ID serialization 的 patch。
- [x] 5.4.13 準備移除 dead `last_visibility_tick` field 的 patch。
- [x] 5.4.14 準備 quarantine legacy viewport authority 的 patch。
- [x] 5.4.15 準備 quarantine nondeterministic vision authority path 的 patch。
- [x] 5.4.16 在 rollback manifest 限定 rollback 只能發生於 match 建立前。

### 5.5 End-to-end 最低限度 build/smoke

**目的：** 在完整 Phase 6 前只確認整合可啟動，不宣稱 acceptance。
**輸入：** Phase 2–5 implementation。
**產出：** Workspace build logs、單一 V2 join/tick/render focused smoke log。
**依賴：** 5.4。
**Owner／Wave：** Primary integrator／Wave 5。
**Gate／Evidence：** `G-E2E-BUILDABLE`，非 acceptance gate。
**完成門檻：** Required workspaces build，單一 secure V2 session 可 join/step/render；所有完整驗證仍未標 pass。

- [x] 5.5.1 執行 scripts workspace 最低限度 build，保存 non-acceptance log。
- [x] 5.5.2 執行一次單一 V2 filtered join focused smoke。
- [x] 5.5.3 Freeze Phase 6 runtime config hash。
- [x] 5.5.4 執行 omb workspace 最低限度 build，保存 non-acceptance log。
- [x] 5.5.5 執行 omfx workspace 最低限度 build，保存 non-acceptance log。
- [x] 5.5.6 在 5.5.2 session 執行 one-frame replica step。
- [x] 5.5.7 從 5.5.6 replica snapshot 執行一次 render handoff。
- [x] 5.5.8 Freeze Phase 6 binary hashes。
- [x] 5.5.9 Freeze Phase 6 content hashes。
- [x] 5.5.10 Freeze Phase 6 evidence manifest hash。
- [x] 5.5.11 在 adjustment contract 標明 B/C correction 會使 frozen evidence stale。

## 6. 集中式 Final Verification、cutover 與 cleanup

### 6.1 執行完整 unit/property 與 schema suite

**目的：** 一次驗證所有低階 contract，不在先前 phase 重複。
**輸入：** 5.5 frozen build/config、完整 test harness。
**產出：** `evidence/final/unit-property/` raw logs、JUnit/summary、hash index。
**依賴：** 5.5。
**Owner／Wave：** Primary integrator／Wave 6A。
**Gate／Evidence：** `G-FINAL-UNIT`、`G-FINAL-SCHEMA`。
**完成門檻：** 所有 required unit/property/schema tests exit 0，無 skip blocking scenario。

- [x] 6.1.1 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` 並保存完整 log。
- [x] 6.1.2 執行 `cargo test --manifest-path scripts/Cargo.toml -p omb-script-abi` 並保存完整 log。
- [x] 6.1.3 執行 omoba-sim determinism property suite。
- [x] 6.1.4 執行 protocol encode suite。
- [x] 6.1.5 執行 projection-policy completeness suite。
- [x] 6.1.6 執行 `cargo test --manifest-path scripts/Cargo.toml -p base_content` 並保存完整 log。
- [x] 6.1.7 執行 omoba-core determinism property suite。
- [x] 6.1.8 執行 visibility resolution property suite。
- [x] 6.1.9 執行 team-scoped identity property suite。
- [x] 6.1.10 執行 scheduled transition property suite。
- [x] 6.1.11 執行 authority repair property suite。
- [x] 6.1.12 執行 bounded random-tape property suite。
- [x] 6.1.13 執行 protocol decode suite。
- [x] 6.1.14 執行 schema version compatibility suite。
- [x] 6.1.15 執行 canonical ordering suite。
- [x] 6.1.16 執行 malformed transition rejection suite。
- [x] 6.1.17 執行 malformed rebase rejection suite。
- [x] 6.1.18 執行 remembered-state simulation exclusion suite。
- [x] 6.1.19 執行 remembered-state hash exclusion suite。
- [x] 6.1.20 彙整 6.1 required suite 的 exit status，確認沒有 skip blocking scenario。

### 6.2 執行 differential、cross-platform 與 non-interference suite

**目的：** 驗證 authoritative projection、server observer 與 omfx replica 的 deterministic parity 與 hidden-state isolation。
**輸入：** 6.1 passing build、paired-world fixtures。
**產出：** `evidence/final/differential/` hash traces、platform reports、paired frame bytes。
**依賴：** 6.1。
**Owner／Wave：** Primary integrator／Wave 6B。
**Gate／Evidence：** `G-FINAL-PARITY`、`G-FINAL-NONINTERFERENCE`。
**完成門檻：** 三方 hash parity 全通過；Windows/Linux pins 相同；所有 hidden-only paired frames byte-identical 到 public causal effect。

- [x] 6.2.1 執行 authoritative projector 對 server observer 的 checkpoint hash differential suite。
- [x] 6.2.2 在 Windows Rust 1.95.0 執行 pinned determinism suite。
- [x] 6.2.3 在 Linux Rust 1.95.0 執行相同 pinned determinism suite。
- [x] 6.2.4 執行 hidden movement paired-world non-interference case。
- [x] 6.2.5 執行 parallel completion-order permutation suite，確認 canonical encoded bytes 不變。
- [x] 6.2.6 執行 authoritative projector 對 synthetic replica 的 checkpoint hash differential suite。
- [x] 6.2.7 執行 authoritative projector 對 omfx replica 的 checkpoint hash differential suite。
- [x] 6.2.8 保存 6.2.2 的 exact Windows toolchain 與 platform metadata。
- [x] 6.2.9 保存 6.2.3 的 exact Linux toolchain 與 platform metadata。
- [x] 6.2.10 比較 Windows 與 Linux checkpoint hashes。
- [x] 6.2.11 執行 hidden component state paired-world non-interference case。
- [x] 6.2.12 執行 hidden RNG paired-world non-interference case。
- [x] 6.2.13 執行 hidden death paired-world non-interference case。
- [x] 6.2.14 為每個 paired-world case 保存 encoded frame byte hash。
- [x] 6.2.15 確認 non-interference case 在 public causal effect 前 byte-identical。

### 6.3 執行 visibility boundary 與 gameplay integration matrix

**目的：** 驗證所有已核准的 visibility state machine 與跨界 projection。
**輸入：** 6.2 passing parity。
**產出：** `evidence/final/boundary/` scenario logs、state/hash timelines。
**依賴：** 6.2。
**Owner／Wave：** Primary integrator／Wave 6C。
**Gate／Evidence：** `G-FINAL-BOUNDARY`。
**完成門檻：** 每個 projection matrix scenario 有 passed evidence，無未分類/未執行 blocking case。

- [x] 6.3.1 執行 team-shared vision scenario。
- [x] 6.3.2 執行 hidden attacker damage cross-boundary scenario。
- [x] 6.3.3 執行 remembered ghost presentation scenario。
- [x] 6.3.4 執行 owner-only resource audience scenario。
- [x] 6.3.5 執行 input-tick visibility-history scenario。
- [x] 6.3.6 執行 force-hide 對 force-show precedence scenario。
- [x] 6.3.7 執行同 priority stable rule ID tie-break scenario。
- [x] 6.3.8 執行 override expiry boundary scenario。
- [x] 6.3.9 執行 reveal candidate cancellation scenario。
- [x] 6.3.10 執行 hide candidate cancellation scenario。
- [x] 6.3.11 執行 scheduled reveal fresh-baseline scenario。
- [x] 6.3.12 執行 scheduled hide scenario。
- [x] 6.3.13 執行 re-reveal identity association scenario。
- [x] 6.3.14 執行 authoritative forget ID retirement scenario。
- [x] 6.3.15 執行 hidden-source buff scenario。
- [x] 6.3.16 執行 hidden-source debuff scenario。
- [x] 6.3.17 執行 projectile enter-visibility scenario。
- [x] 6.3.18 執行 projectile leave-visibility scenario。
- [x] 6.3.19 執行 AOE cross-boundary scenario。
- [x] 6.3.20 執行 remembered entity fog-death non-disclosure scenario。
- [x] 6.3.21 執行 custom remember policy scenario。
- [x] 6.3.22 執行 remembered record target rejection scenario。
- [x] 6.3.23 執行 team-private resource audience scenario。
- [x] 6.3.24 執行 public resource audience scenario。
- [x] 6.3.25 執行 server-only resource non-disclosure scenario。
- [x] 6.3.26 執行 retained event audience scenario。
- [x] 6.3.27 執行 stale disclosure epoch input scenario。
- [x] 6.3.28 執行 ownership-invalid input scenario。
- [x] 6.3.29 執行 accepted disclosed-target action scenario。
- [x] 6.3.30 執行 rejected hidden-target action scenario。

### 6.4 執行 network fault、recovery 與 validator suite

**目的：** 驗證 gap、replay、rebase、observer sidecar 與 non-blocking guarantee。
**輸入：** 6.3 passing integration。
**產出：** `evidence/final/fault-recovery/` injected fault manifest、queue/latency traces、recovery results。
**依賴：** 6.3。
**Owner／Wave：** Primary integrator／Wave 6D。
**Gate／Evidence：** `G-FINAL-RECOVERY`、`G-FINAL-NONBLOCKING`。
**完成門檻：** 每種 fault 有 deterministic terminal disposition；validator slowdown/overflow 不阻塞 outbound；coverage gap 不被誤標 pass。

- [x] 6.4.1 執行 duplicate frame scenario。
- [x] 6.4.2 執行 replay-ring hit scenario。
- [x] 6.4.3 執行 component repair recovery scenario。
- [x] 6.4.4 故意放慢 validation worker 並擷取 outbound latency trace。
- [x] 6.4.5 觸發 validation queue overflow 並確認建立 coverage-gap record。
- [x] 6.4.6 執行 reordered frame scenario。
- [x] 6.4.7 執行 late frame barrier-stall scenario。
- [x] 6.4.8 執行 missing frame gap-detection scenario。
- [x] 6.4.9 執行 corrupt frame rejection scenario。
- [x] 6.4.10 執行 oversized frame rejection scenario。
- [x] 6.4.11 執行 replay-ring expiry scenario。
- [x] 6.4.12 執行 filtered rebase bootstrap scenario。
- [x] 6.4.13 執行 interrupted rebase discard scenario。
- [x] 6.4.14 執行 player rejoin filtered-bootstrap scenario。
- [x] 6.4.15 執行 entity replace recovery scenario。
- [x] 6.4.16 執行 team-view rebase recovery scenario。
- [x] 6.4.17 執行 persistent mismatch safe-termination scenario。
- [x] 6.4.18 由 6.4.4 trace 判定 outbound enqueue 未等待 observer step。
- [x] 6.4.19 由 6.4.4 trace 判定 player stream latency 未受 validator slowdown 阻塞。
- [x] 6.4.20 在 queue overflow 後確認 outbound frame sequence 持續前進。
- [x] 6.4.21 在 queue overflow 後確認 stale observer 被 discard。
- [x] 6.4.22 在 queue overflow 後確認 observer 使用 filtered snapshot rebootstrap。
- [x] 6.4.23 在 queue overflow evidence 將 gap range 標記為 unverified。
- [x] 6.4.24 確認 coverage-gap range 未被 validation summary 計為 pass。

### 6.5 執行 security、anti-probing 與 side-channel inspection

**目的：** 證明 player packet、memory 與 diagnostics 不含 hidden/global data。
**輸入：** 6.4 passing recovery。
**產出：** `evidence/final/security/` packet captures、memory/export scans、fuzz reports、redaction review。
**依賴：** 6.4。
**Owner／Wave：** Primary integrator／Wave 6E。
**Gate／Evidence：** `G-FINAL-HIDDEN-DATA`、`G-FINAL-ANTI-PROBING`、`G-FINAL-REDACTION`。
**完成門檻：** 零 canonical/global/hidden data finding；invalid target 無 existence oracle；padding/cadence 滿足 spec；無 unresolved P0/P1 security finding。

- [x] 6.5.1 對 secure match packet capture 掃描 canonical ECS ID pattern。
- [x] 6.5.2 檢查 omfx deterministic replica memory export 是否含 hidden entity state。
- [x] 6.5.3 執行 hidden-existing replica ID probing timing case。
- [x] 6.5.4 對 protocol transition decoder 執行 fuzzing。
- [x] 6.5.5 分析 hidden-only activity 的 frame cadence。
- [x] 6.5.6 驗證 player session 無法取得 admin diagnostic capability。
- [x] 6.5.7 對 secure match packet capture 掃描 global seed pattern。
- [x] 6.5.8 對 secure match packet capture 掃描 other-team visibility mask pattern。
- [x] 6.5.9 對 secure match packet capture 掃描已知 hidden component sentinel value。
- [x] 6.5.10 檢查 remembered cache export 是否只含已去敏感化 presentation。
- [x] 6.5.11 檢查 player-visible log redaction。
- [x] 6.5.12 檢查 player replay redaction。
- [x] 6.5.13 檢查 player crash bundle redaction。
- [x] 6.5.14 檢查 player trace redaction。
- [x] 6.5.15 執行 nonexistent replica ID probing timing case。
- [x] 6.5.16 執行 stale replica ID probing timing case。
- [x] 6.5.17 比較 hidden-existing 與 nonexistent probing response class。
- [x] 6.5.18 比較 hidden-existing 與 nonexistent probing timing bucket。
- [x] 6.5.19 執行 invalid replica reference rate-limit scenario。
- [x] 6.5.20 對 filtered snapshot decoder 執行 fuzzing。
- [x] 6.5.21 對 rebase manifest/chunk decoder 執行 fuzzing。
- [x] 6.5.22 執行 replayed authority revision attack case。
- [x] 6.5.23 執行 malformed disclosure epoch attack case。
- [x] 6.5.24 分析 hidden-only activity 的 padding bucket。
- [x] 6.5.25 分析 mass reveal chunk size distribution。
- [x] 6.5.26 分析 rebase chunk rate-limit behavior。
- [x] 6.5.27 驗證 admin diagnostic 使用獨立 transport boundary。
- [x] 6.5.28 審查 security findings，確認無 unresolved P0/P1。

### 6.6 執行 10,000 entity performance、bandwidth 與 soak gates

**目的：** 在完整架構啟用時驗證 real-time、memory、bandwidth 與長時間穩定性。
**輸入：** 6.5 security pass、frozen production config、baseline。
**產出：** `evidence/final/performance/` raw traces、30-minute soak logs、summary/gate verdict。
**依賴：** 6.5。
**Owner／Wave：** Primary integrator／Wave 6F。
**Gate／Evidence：** `G-FINAL-TICK-BUDGET`、`G-FINAL-BANDWIDTH`、`G-FINAL-SOAK`。
**完成門檻：** p99 tick+commit ≤ 80% period、steady-state <5 KB/s/player、零 authoritative deadline miss、零 unintended rebase、memory stable、observer coverage 完整。

- [x] 6.6.1 將 frozen stress config 的 entity count 設為 10,000。
- [x] 6.6.2 擷取 authoritative tick p50/p95/p99 raw trace。
- [x] 6.6.3 擷取 per-player steady-state bandwidth distribution。
- [x] 6.6.4 執行 repeated mass reveal/hide memory case。
- [x] 6.6.5 執行 30 分鐘 production-config soak。
- [x] 6.6.6 將 final authoritative tick 指標與 Phase 1 baseline 比較。
- [x] 6.6.7 在 6.6.1 run 啟用 2 teams。
- [x] 6.6.8 在 6.6.1 run 啟用每隊 1 個 observer replica。
- [x] 6.6.9 在 6.6.1 run 啟用 deterministic visibility churn workload。
- [x] 6.6.10 擷取 deterministic commit p50/p95/p99 raw trace。
- [x] 6.6.11 擷取 Wave B visibility/project p50/p95/p99 raw trace。
- [x] 6.6.12 擷取 team-frame encode p50/p95/p99 raw trace。
- [x] 6.6.13 擷取 outbound enqueue p50/p95/p99 raw trace。
- [x] 6.6.14 擷取 observer step p50/p95/p99 raw trace。
- [x] 6.6.15 判定 authoritative tick+commit p99 是否小於 tick period 80%。
- [x] 6.6.16 擷取 reveal burst bandwidth distribution。
- [x] 6.6.17 擷取 rebase burst bandwidth distribution。
- [x] 6.6.18 判定 steady-state bandwidth 是否低於 5 KB/s/player。
- [x] 6.6.19 執行 projectile boundary-churn memory case。
- [x] 6.6.20 執行 AOE boundary-churn memory case。
- [x] 6.6.21 執行 observer rebootstrap memory case。
- [x] 6.6.22 從 soak log 判定 authoritative deadline miss count。
- [x] 6.6.23 從 soak log 判定 unintended rebase count。
- [x] 6.6.24 從 soak log 判定 disconnect count。
- [x] 6.6.25 從 soak log 判定 validation coverage-gap count。
- [x] 6.6.26 從 soak trace 計算 process memory slope。
- [x] 6.6.27 產生含 raw evidence hash 的 immutable performance verdict。
- [x] 6.6.28 以 6.6.1、6.6.7–6.6.9 的 frozen config 啟動 production-cadence stress run。

### 6.7 收斂 failure、更新 evidence lineage 與重跑受影響 group

**目的：** 修正 final verification failure，而不重跑無關完整 suite或隱藏降低 gate。
**輸入：** 6.1–6.6 results。
**產出：** Adjustment records、fixed artifacts、stale/replacement evidence links、all-green final index。
**依賴：** 6.1–6.6。
**Owner／Wave：** Primary integrator／Wave 6G。
**Gate／Evidence：** 所有 final gates。
**完成門檻：** 每個 failure 有 A/B/C disposition；affected evidence 已 stale/replaced；所有 blocking gate 最終 passed，無 failed/blocked/stale terminal record。

6.7 的 conditional leaf 每次只處理一個 pending item；額外 item 必須以 A-level refinement 增加同型 leaf，沒有對應 item 時以 evidence-backed `not-applicable` 結案。

- [x] 6.7.1 將 Phase 6 failed gate 彙整成 failure index；沒有 failure 時建立 immutable no-failure record。
- [x] 6.7.2 對第一個 pending A-level item 更新 task mechanics。
- [x] 6.7.3 對第一個 pending B-level item 暫停 affected branch。
- [x] 6.7.4 對第一個 pending C-level item 停止 affected work。
- [x] 6.7.5 為每個 affected final-verification group 建立 rerun manifest。
- [x] 6.7.6 確認每個 blocking gate 的 terminal status 為 passed。
- [x] 6.7.7 為第一個尚未分類的 failure 指派 A、B 或 C disposition；其他 failure 以 A-level refinement 增加 leaf。
- [x] 6.7.8 對 A-level item 將受影響 evidence 標記 stale。
- [x] 6.7.9 對 B-level item 更新 authoritative design section。
- [x] 6.7.10 對 B-level item 更新受影響 delta requirement/scenario。
- [x] 6.7.11 對 B-level item 更新受影響 task leaf。
- [x] 6.7.12 對 B-level item 將 dependent evidence 標記 stale。
- [x] 6.7.13 為 C-level item 建立使用者核准請求與 affected scope。
- [x] 6.7.14 在 C-level 使用者核准前保持 affected work 未執行。
- [x] 6.7.15 執行 rerun manifest 中的一個 affected verification group；每個額外 group 以 A-level refinement 增加相同格式 leaf。
- [x] 6.7.16 為 rerun result 建立 replacement evidence link。
- [x] 6.7.17 確認所有 stale blocking evidence 都有 passed replacement。
- [x] 6.7.18 比對 final gate threshold 與核准 design，確認沒有降低。
- [x] 6.7.19 比對 required evidence set 與核准 design，確認沒有刪減。

### 6.8 Shadow、dogfood、secure cutover 與 legacy cleanup

**目的：** 以已通過的完整 evidence 啟用 secure V2，並移除 player global disclosure path。
**輸入：** 6.7 all-green evidence、5.4 cutover manifest。
**產出：** Shadow/dogfood acceptance、secure-default config、legacy cleanup commits、rollback record。
**依賴：** 6.7。
**Owner／Wave：** Primary integrator／Wave 6H。
**Gate／Evidence：** `G-RELEASE-SHADOW`、`G-RELEASE-DOGFOOD`、`G-SECURE-DEFAULT`。
**完成門檻：** Shadow/dogfood 無 blocker；secure default 啟用；player global snapshot/hash/seed/raw-ID path 移除；rollback 只允許 pre-match non-secure mode。

- [x] 6.8.1 以 frozen binaries/config 執行 server shadow run。
- [x] 6.8.2 執行一場標準 2-team internal dogfood secure match。
- [x] 6.8.3 將 secure V2 設為新 secure match 的 default mode。
- [x] 6.8.4 套用移除 player global `TickBatch` fan-out 的 cleanup patch。
- [x] 6.8.5 移除 dead viewport/`VisSet` gameplay authority path。
- [x] 6.8.6 執行 active secure match runtime-downgrade rejection case。
- [x] 6.8.7 從 shadow run 產生 parity evidence。
- [x] 6.8.8 從 shadow run 產生 latency evidence。
- [x] 6.8.9 從 shadow run 產生 observer coverage evidence。
- [x] 6.8.10 從 dogfood run 產生 player-visible acceptance record。
- [x] 6.8.11 從 dogfood run 產生 redacted diagnostic acceptance record。
- [x] 6.8.12 保留明確 non-secure legacy pre-match selection。
- [x] 6.8.13 套用移除 player global `StateHash` fan-out 的 cleanup patch。
- [x] 6.8.14 套用移除 player global `WorldSnapshot` bootstrap 的 cleanup patch。
- [x] 6.8.15 套用移除 player `master_seed` delivery 的 cleanup patch。
- [x] 6.8.16 套用移除 player raw ECS ID serialization 的 cleanup patch。
- [x] 6.8.17 移除 dead `client_visibility` storage path。
- [x] 6.8.18 移除 dead `last_visibility_tick` storage path。
- [x] 6.8.19 Quarantine superseded nondeterministic vision authority path。
- [x] 6.8.20 產生只允許 pre-match legacy selection 的 final rollback manifest。
- [x] 6.8.21 執行一場含 player reconnect 的 internal dogfood secure match。
- [x] 6.8.22 執行一場含高 visibility churn 的 internal dogfood secure match。

### 6.9 最終 traceability 與 release review

**目的：** 證明 proposal → design → requirement/scenario → task/gate/evidence 全鏈路完整。
**輸入：** 所有 artifacts、6.8 release evidence。
**產出：** `evidence/final/traceability.md`、release verdict、archive-ready status。
**依賴：** 6.8。
**Owner／Wave：** Primary integrator／Wave 6I。
**Gate／Evidence：** `G-FINAL-TRACEABILITY`、`G-RELEASE-READY`。
**完成門檻：** 每個 blocking requirement/scenario 有 task/evidence；無 placeholder、contradiction、unresolved P0/P1 或 incomplete apply-required artifact。

- [ ] 6.9.1 建立 proposal commitment 到 design decision 的 traceability column。
- [ ] 6.9.2 掃描 artifacts 的未解決 placeholder。
- [ ] 6.9.3 審查每個 L2 是否具備完整 metadata 欄位。
- [ ] 6.9.4 確認 conditional leaf 只有有效 terminal status。
- [ ] 6.9.5 建立 final release verdict shell。
- [ ] 6.9.6 建立 design decision 到 requirement/scenario 的 traceability column。
- [ ] 6.9.7 建立 requirement/scenario 到 permanent task ID 的 traceability column。
- [ ] 6.9.8 建立 task ID 到 evidence record 的 traceability column。
- [ ] 6.9.9 掃描 code 的 forbidden global disclosure reference。
- [ ] 6.9.10 掃描 evidence 的 contradiction marker 與 stale terminal record。
- [ ] 6.9.11 審查 Phase 1 L3 是否符合 Luna 原子化契約。
- [ ] 6.9.12 確認 superseded leaf 都有 replacement task/evidence link。
- [ ] 6.9.13 將 exact binary hashes 寫入 final release verdict。
- [ ] 6.9.14 將 exact config hash 寫入 final release verdict。
- [ ] 6.9.15 將 exact content hashes 寫入 final release verdict。
- [ ] 6.9.16 將 blocking gate summary 寫入 final release verdict。
- [ ] 6.9.17 將 rollback boundary 寫入 final release verdict。
- [ ] 6.9.18 確認 final release verdict 無 unresolved P0/P1。
- [ ] 6.9.19 審查 Phase 2 L3 是否符合 Luna 原子化契約。
- [ ] 6.9.20 審查 Phase 3 L3 是否符合 Luna 原子化契約。
- [ ] 6.9.21 審查 Phase 4 L3 是否符合 Luna 原子化契約。
- [ ] 6.9.22 審查 Phase 5 L3 是否符合 Luna 原子化契約。
- [ ] 6.9.23 審查 Phase 6 L3 是否符合 Luna 原子化契約。
