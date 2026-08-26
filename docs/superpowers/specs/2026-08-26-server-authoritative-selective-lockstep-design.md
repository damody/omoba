# 伺服器權威選擇性 Lockstep 設計

## 摘要

`omb` 將以伺服器權威的選擇性 lockstep 架構，取代目前向所有玩家廣播完整世界的 lockstep stream。伺服器仍是唯一持有完整模擬的執行個體；每個隊伍只接收並模擬該隊有權得知的 deterministic state 子集。視野由全隊共享，支援 deterministic 自動視野與 gameplay 明確 override，且不依賴 client viewport。

Client-side simulation 從屬於伺服器。Client 與 server 狀態衝突時，一律以 server 結果為準，並透過權威 component repair、entity replacement 或 filtered team-view rebase 收斂。

伺服器 process 內也會為每個 active team 維護非阻塞 observer replica。獨立 validation worker 消費與真實 client 相同的 encoded team stream，並執行相同的 selective replica runtime。驗算不會延遲 outbound traffic。

## 目標

- 不讓隱藏單位、狀態、輸入、RNG state、identity 或事件進入未獲授權玩家的封包或記憶體。
- 對已揭露的 team world 保留 deterministic local step。
- 同一隊伍的所有玩家共享視野。
- 同時支援自動距離／障礙／偵測視野與 script-driven override。
- 以預先排程的 tick reveal entity，不使用 rollback。
- 支援 per-entity `Forget`、`LastKnown` 與自訂 remembered presentation policy。
- 由 server 權威決定輸入、transition、outcome、repair 與 rebase。
- 以 server-local observer replica 非同步驗算每個 team stream。
- 保留既有 steady-state lockstep 頻寬目標：每位玩家低於 5 KB/s。
- 在 authoritative world 與所有 active team observer replica 同時啟用時通過 10,000 entity stress scenario。

## 非目標

- 對 gameplay 已刻意公開給隊伍的資訊提供密碼學保護。
- Secure fog match 發生問題時降級回舊的 global snapshot 或 global `TickBatch` protocol 繼續遊戲。
- 繼續把 camera viewport AOI 當成 gameplay vision 的權威來源。
- 將完整世界送到 client 後只在 renderer 隱藏 entity。
- 把 global master RNG seed，或 entity 隱藏後仍可使用的 PRNG state 交給 client。
- 讓 remembered ghost 參與 simulation、targeting、collision 或 state hash。
- 為每位玩家常駐一份 server-side replica；replica 以 team 為單位。

## 現況與缺口

目前 lockstep path 會向每個 `lockstep_joined` session 廣播同一份 `TickBatch` 與 global `StateHash`。`SnapshotResp` 包含 global `WorldSnapshot`，`GameStart` 也會公開 global `master_seed`。因此只在 renderer 隱藏 entity 無法提供資訊安全。

Legacy transport 雖有 viewport/AOI filtering 與未使用的 `VisSet` state，但這些程式碼以 camera 為中心、使用 player name 與 raw ECS identity，而且 lockstep frame 會完全繞過它，不適合作為權限邊界。

既有 `vision_ecs` 也不能直接使用。它依賴 wall-clock timestamp 與 floating-point visibility geometry，以 player name 建立 cache，且未接入 deterministic runtime dispatcher。只有在轉為 fixed-point deterministic geometry 並明確定義 tick 語義後，才能重用其幾何演算法。

目前 snapshot 與 hash 只涵蓋部分 global ECS 欄位，不足以 bootstrap 完整 replica，也不適合提供給只應取得部分資訊的 client。

## 名詞

- **Authoritative World**：只存在於 server 的完整 gameplay world。
- **Team View**：某個 team 目前有權得知的 state 與 event。
- **Disclosed Entity**：存在於 team deterministic replica 的 entity。
- **Remembered Record**：存放在 replica simulation 之外、已去敏感化的 last-known render data。
- **Canonical Entity**：只存在於 server 的 ECS entity identity。
- **Replica Entity ID**：用於 team wire 與 team replica 的 team-scoped opaque identity。
- **Observable Fact**：包含足夠 metadata、可供後續 team projection 的 deterministic gameplay 結果。
- **External Effect**：由不能安全揭露的 dependency 所造成、經去敏感化的權威結果。
- **Team Observer Replica**：只消費單一 team encoded stream 的 server-local replica。
- **D**：visibility commitment delay。
- **L**：client replica 落後 authoritative server 的 buffer lead。

## 權威與安全 invariant

1. 所有 accepted input、visibility、RNG result、spawn、death、damage、buff、repair 與 snapshot 都以 server 為最終權威。
2. Secure match 期間，一個 player session 只能綁定一個 team view。
3. Team frame 不得包含 canonical ECS identity、其他隊伍的 visibility mask、global RNG seed 或 server-only component data。
4. 若兩個 authoritative world 只有 hidden state 不同，則在差異造成刻意公開的 effect 之前，兩者對同一 team 產生的 frame 必須 byte-identical。
5. Remembered record 只供 renderer 使用，不參與 input validation、simulation、collision、targeting 或 hash。
6. Secure match 啟動後不得降級到 global-world protocol。
7. Observer validation 不得阻塞 outbound；缺席或落後必須記為 coverage gap，不得視為驗算成功。
8. Repair 與 rebase 必須通過與一般 frame 相同的 team projection 與 redaction boundary。

## 高階架構

伺服器包含下列 logical component：

1. `AuthoritativeSimulation`：持有完整 Specs world 並執行 fixed tick。
2. `VisibilityResolver`：從 committed state 計算 deterministic team visibility。
3. `VisibilityTransitionScheduler`：把 raw visibility 轉成 scheduled reveal/hide/forget transition。
4. `TeamViewProjector`：把 observable fact 與 authoritative state 轉成特定 team 可安全取得的 representation。
5. `TeamFrameBuilder`：建立並 encode canonical ordered team-specific frame。
6. `TeamStreamRouter`：立即把 encoded frame enqueue 給該 team 的 sessions。
7. `ObserverValidationWorker`：tap 同一份 encoded frame，decode 後推進 team observer replica，並透過 control channel 回報 divergence。
8. `AuthorityRepairCoordinator`：收到 divergence 後，在後續 frame 產生 repair 或 filtered rebase。

共用 `omoba-core` 負責 wire type、canonical team-view serialization、transition application 與 `SelectiveReplicaRuntime`。omfx 與 server validation worker 都使用同一套 runtime。這些型別不得放入 `scripts/script-abi`。

## Deterministic Specs tick pipeline

原先分開描述的「step world」、「收集公開 effect」與「計算 visibility」，改成同一個 Rust Specs tick pipeline，而不是三次 serial full-world scan。

### Tick 開始

Tick `T` 開始時，`State[T]` 與 committed visibility view `V[T]` 已固定。輸入必須依 ownership、session team、輸入攜帶的 view epoch，以及 input tick 的 visibility history 驗證。

### Wave A：平行 gameplay evaluation

Gameplay system 依 Specs storage conflict 與明確 dependency 在既有 dispatcher 平行執行。每個 gameplay system 在計算 authoritative work 時，同步產生兩種 deterministic side output：

- `Outcome`：在 commit barrier 套用的權威 mutation。
- `ObservableFact`：描述發生內容，但尚未決定哪些 team 可以接收的 projection-ready fact。

Movement、combat、skill、script、spawn 與 death 都在原本計算期間產生 projection fact；server 不再於事後重新掃描整個 world 推測 effect。

Parallel writer 不得依賴 shared `Vec` 的 arrival order。每筆 output 必須帶 stable ordering key：

```text
(tick, phase, canonical_source_order, local_ordinal, fact_kind)
```

Thread-local 或 sharded buffer 會在 barrier 合併、排序、去重並驗證。Script 繼續遵守 Outcome contract，不得直接修改不屬於自身邊界的 ECS state。

### Deterministic commit barrier

Server 套用排序後的 outcome，執行 `World::maintain`，並建立 committed `State[T+1]`。此 barrier 不可省略，因為 `T+1` visibility 必須讀到最終 post-step position、stealth、death、ownership 與 vision-source change。

### Wave B：平行 team projection

`State[T+1]` 建立後，以 team 為單位平行執行 read-only job：

- resolve raw `V[T+1]`；
- 更新 visibility candidate 與 scheduled transition；
- 將 tick fact 投影給 team；
- 為本 tick 生效的 transition 擷取 baseline；
- 建立、encode 並 enqueue team frame。

Wave A 與 Wave B 之間若沒有 barrier，就只能計算 stale `V[T]`，或重複實作 next-state logic。兩個 wave 可保留平行計算，同時確保 post-step visibility 正確。

## Visibility model

### ECS component 與 resource

- `ReplicationScope`：`ServerOnly`、`Public`、`OwnerTeam` 或 `TeamVision`。
- `VisionSource`：owning team、range、height/detection tag 與 enabled state。
- `StealthProfile`：stealth layer 與 detector requirement。
- `VisibilityOverride`：具有 priority 與 expiration tick 的 force-show 或 force-hide grant。
- `RememberPolicy`：`Forget`、`LastKnown` 或已註冊的 custom renderer policy。
- `TeamVisibilityIndex`：每個 team 的 resolved visible canonical entity、visibility epoch 與 transition state。
- `TeamReplicaIdMap`：只屬於單一 team 的 canonical-to-replica identity mapping。

Gameplay 與 script 必須透過 `GrantVisibility`、`RevokeVisibility`、`SetReplicationScope`、`SetRememberPolicy` 等 explicit outcome 改變上述狀態。

### Team sharing

同一隊伍內任何 player 或 unit 所有的 vision source，都會貢獻到同一份 shared team view。最終授權邊界仍是 session；session 只能繼承其綁定 team 的 stream。

### 自動規則與 override resolution

Resolution order 必須 deterministic：

1. `ServerOnly` 拒絕 disclosure。
2. 未過期的 force-hide override 拒絕 disclosure，除非更高 priority 的規則明確覆蓋。
3. `Public` 或適用的 force-show grant 允許 disclosure。
4. `OwnerTeam` 向 owning team disclosure。
5. `TeamVision` 必須通過自動 geometry 與 detection rule。
6. Tie 以 stable rule ID 決定，不得依賴 insertion order。

### Scheduled transition

Default visibility commitment delay 為 3 ticks。Raw visibility change 先建立 candidate；條件在 candidate 成熟時仍成立，才 commit transition。Server 於 effective tick 從當下 authoritative state 擷取新 baseline，不得使用 candidate 建立時的 stale baseline。

Client replica 預設在 120 Hz 下 buffer 12 ticks，也就是 100 ms。兩個值都是 handshake 宣告的 authoritative match configuration。允許範圍為：

- `visibility_commit_delay_ticks`：2 至 4；
- `replica_buffer_ticks`：不得小於 visibility delay，且範圍為 3 至 24。

修改這些值必須有 protocol-compatible match negotiation 與 performance/latency evidence。

### Visibility state machine

```text
Hidden -> RevealCandidate -> Disclosed
Disclosed -> HideCandidate -> Remembered | Hidden
Remembered -> RevealCandidate -> Disclosed
Remembered -> Forget -> Hidden
```

Candidate cancellation 必須 explicit 且 deterministic。允許 re-reveal 時沿用既有 team-scoped replica ID，讓 renderer 能與 remembered record 關聯；不同 team 的 ID 永不共用。

## Team-scoped identity

Raw `specs::Entity::id()` 與 generation 只存在於 server。每個 team 使用自己的 monotonic、match-local、永不重用 `ReplicaEntityId` namespace。Mapping entry 包含 disclosure epoch，避免 stale transition 或 input 影響後續 incarnation。

在允許 remembered 的期間，replica ID 可保持穩定。Canonical entity 被摧毀且該 death 已成為 team-known，或收到 authoritative forget 後，ID 永久 retired。

## Wire Protocol V2

Secure match 必須在 join 前 negotiate protocol V2，且該 match 的所有 player session 都使用 V2。

### `TeamGameStart`

`TeamGameStart` 包含 protocol 與 snapshot schema version、player/team ID、authoritative server tick、replica start tick、tick rate、visibility delay、replica buffer、verified filtered team snapshot，以及 public/team-private deterministic metadata。不得包含 global master seed。

### `TeamTickFrame`

`TeamTickFrame` 包含：

- `server_tick`、`replica_tick`、monotonic `team_sequence` 與 `view_epoch`；
- `PreStep` transition；
- `Step` accepted input、public server event、random tape entry 與 external effect；
- `PostStep` authoritative repair 與 optional hash checkpoint；
- content/schema compatibility metadata。

Frame 依 phase、event kind、replica entity ID 與 stable sub-index canonical ordering。

### Transition

- `RevealEntity`：replica ID、disclosure epoch、effective tick、kind、完整 safe baseline 與已揭露 dependency record。
- `HideEntity`：replica ID、disclosure epoch、effective tick，以及 policy 允許的 sanitized remembered presentation。
- `ForgetEntity`：replica ID 與 effective tick。
- `ReplaceEntity`：完整替換一個 disclosed entity 的 authority record。

### Repair 與 rebase

1. `ComponentRepair` 在 server 指定的 barrier 覆寫 disclosed field。
2. `EntityReplace` 原子替換一個 disclosed entity。
3. `TeamViewRebase` 以 filtered snapshot 取代整個 deterministic team replica，並從指定 frame sequence 繼續。

所有 correction 都攜帶較新的 authority revision。Revision conflict 永遠以 server 為準。

## Randomness

Global master seed 與可重用 global PRNG state 只存在於 server。Disclosed local simulation 只能取得已決定的 authoritative random outcome，或綁定 disclosure epoch 與有限 tick window 的短期 random tape。Random tape 不得推導 window 之外或後續 hidden period 的數值；依賴 hidden state 的 random behavior 必須投影成 authoritative external effect。

## 跨 visibility dependency

> 只有在 deterministic evaluation 所需的全部 dependency 都能安全 disclosure 時，client 才能 local simulate 該 action；否則 server 必須將結果投影成 sanitized external effect。

範例：

- Hidden attacker 傷害 visible hero：只 disclosure target、amount、damage class、tick 與 rule 允許的 attribution，不公開 attacker ID 或 position。
- Hidden projectile 進入 vision：reveal 當前 baseline；owner 仍 hidden 時使用 anonymous public source surrogate。
- Visible projectile 的 target 進入 fog：移除 private target reference，依 policy 改用 disclosed trajectory、hide projectile，或稍後送 authoritative impact outcome。
- AOE 跨越 boundary：只 disclosure disclosed target 的 effect，不公開 hidden target count。
- Hidden caster 對 visible unit 施加 buff：公開 buff effect，但不公開 caster identity。
- Remembered enemy 在 fog 中死亡：last-known record 保留到 death 成為 team-known，或其他 policy 主動 forget。

每個 gameplay system 與 script-visible action 都必須聲明 visible-visible、hidden-visible、visible-hidden 與 hidden-hidden projection policy。缺少 policy 是 blocking integration error。

## Client 與 observer replica runtime

`SelectiveReplicaRuntime` 在 replica tick `T` 依序執行：

1. 等待 expected sequence 與 tick frame；
2. 原子套用 `PreStep` transition；
3. 注入 accepted input、event、random tape 與 external effect；
4. 對 disclosed entity/resource 執行一個 fixed deterministic tick；
5. 套用 `PostStep` authority revision；
6. 需要時計算 canonical team-view hash；
7. 產生 render snapshot。

Remembered record 存在獨立 render cache。Frame missing 或 late 時，replica 停在 barrier，不猜測 authoritative gap 期間的資料。

## 非阻塞 server observer validation

Encoded team frame 立即 enqueue 給該 team 的 network session；validation 不在 send critical path。

相同的 encoded `Arc<[u8]>` 會 tap 到 server process 內獨立 validation worker thread 的 bounded channel。Worker 為每個 active team 維護 isolated observer replica，並且：

- 透過與 remote team observer 相同的 filtered snapshot path bootstrap；
- decode 實際 wire bytes；
- 執行共用 `SelectiveReplicaRuntime`；
- 不得存取 authoritative Specs world、canonical ID 或其他 team state；
- 將 checkpoint hash 與相同 team/tick 的 server canonical projected hash 比較。

Mismatch 時記錄 first divergent tick、team、frame sequence、hash、transition epoch 與安全的 component path information，再透過 control channel 回報。Authoritative coordinator 在後續 frame 送 repair/rebase；真實 client 與 local observer 消費相同 correction。

Validation channel 滿時 outbound 仍繼續。Server 記錄 verification coverage gap、丟棄 stale observer，透過相同 bootstrap path 取得 filtered snapshot，並從最新 retained frame 繼續。Coverage gap 必須告警，且不得視為驗算成功。

## Input validation 與 anti-probing

Client command 引用 team-scoped replica ID，並攜帶 command 發出時的 view epoch。Server 驗證 player/session/team binding、ownership、command permission、replica ID mapping、disclosure epoch、command tick visibility、input timing 與 deduplication ID。

Invalid target command 只回傳 generalized rejection class，並維持 uniform processing timing；response 不得透露 undisclosed canonical entity 是否存在。重複 invalid reference 必須 rate limit。

## State hash

Player session 不再使用 global state hash，改用 canonical team-view hash。Hash 依 replica ID 與 schema field order，涵蓋所有 deterministic disclosed component 與 team-visible deterministic resource；排除 remembered render record、server-only data、diagnostic、不影響 simulation 的 queue 與其他 team state。

Authoritative server 計算 expected hash。Client 與 observer hash 只是 evidence，不是 authority；mismatch 先觸發 correction 與 diagnostic，再考慮 disconnect policy。

## Network recovery 與 rejoin

- Duplicate frame 依 team sequence 與 authority revision idempotently ignore。
- Sequence gap 先從 bounded per-team encoded-frame ring replay。
- Requested sequence 已過期時，server 送 verified filtered rebase 與 catch-up frame。
- Rejoin player 只取得所屬 team 的 snapshot/stream。
- Transition late 時，client 停在相關 tick barrier。
- Interrupted rebase 只有在 snapshot ID、chunk hash 與 final manifest 全部通過後才能套用。

## Side-channel control

Frame 維持 fixed tick cadence。Sensitive payload 使用 configured size bucket 與 padding，避免從精確 length 推測 hidden activity。Mass reveal/rebase traffic 獨立 chunk、rate limit，並與 steady-state 分開量測。

Log、replay、crash bundle 與 performance trace 使用相同 team redaction rule。完整 authoritative diagnostic 需要明確 server-admin capability，且不得透過 player session 傳輸。

## Observability

Metric 以 opaque match/team ID 標記，涵蓋 visible entity count、transition count、padding 前後 frame bytes、build/encode/enqueue/replica-step duration、outbound/validation queue depth、observer audit lag、coverage gap、hash mismatch、repair/rebase、client barrier stall、gap replay、rejoin duration、projection-policy failure 與 redaction violation。

Authoritative server 外部輸出的 diagnostic 不得包含 canonical entity identity。

## Performance gate

Blocking stress target 是既有 10,000-entity scenario，並以 production tick rate 同時啟用 authoritative world、兩個 team projection job、兩個 team observer replica、transport encoding 與 visibility churn。

必要 gate：

- p99 authoritative tick 加 required commit work 不超過 tick period 的 80%；
- p99 projection/enqueue 在 client buffer deadline 前完成；
- outbound delivery 永不等待 observer validation；
- 30 分鐘 stress soak 為零 unintended rebase、零 authoritative tick deadline miss；
- observer 無未回報 coverage gap，且 injected gap 後可 catch up；
- steady-state network usage 低於每位玩家 5 KB/s，reveal/rebase burst 獨立量測且有上限；
- repeated visibility churn 與 replica rebootstrap 期間 memory 穩定；
- 沒有 blocking security 或 hidden-data finding。

不得以關閉 observer validation 的方式降低或繞過 gate。Threshold 變更屬於 material design change，必須取得明確核准。

## 測試策略

完整測試與檢查只執行一次，時點在 server、shared runtime、observer validator、protocol 與 omfx client 全部整合後。Implementation phase 不重複完整 suite。實作期間只允許維持 branch 可用所需的最低限度 compile check 或 focused smoke；這些不算 acceptance evidence，也不能取代 final verification。

### Unit 與 property test

- visibility precedence、team sharing、stealth/detection、override expiry 與 candidate cancellation；
- transition state machine、disclosure epoch、team-scoped ID 與 stale-ID rejection；
- parallel scheduling 下的 canonical ordering；
- projection policy completeness；
- remembered record 不參與 simulation/hash；
- repair revision ordering、idempotence、random tape window 與 epoch isolation；
- non-interference：hidden-only change 在 allowed public effect 前產生 byte-identical team frame。

### Differential 與 integration test

- authoritative projection、server observer 與 omfx replica hash 一致；
- Windows/Linux determinism pin 一致；
- reveal、hide、canceled transition、re-reveal、forget；
- hidden damage、projectile crossing、AOE、buff、fog death、shared team vision；
- visibility boundary 上的 input acceptance/rejection；
- component repair、entity replacement、rebase convergence；
- 故意放慢 validator 時 outbound 仍不受阻塞。

### Fault 與 adversarial test

- late、duplicate、reordered、missing、corrupt、oversized frame；
- reconnect、ring expiry、interrupted rebase、validation channel overflow；
- hidden-target probing、replica-ID enumeration、malformed epoch、replay attack；
- packet capture 掃描 canonical ID、global seed、hidden value 與未 padding 的 sensitive size pattern；
- protocol transition 與 snapshot decoding fuzzing。

### Stress test

- 10,000 entities 與至少兩個 teams；
- rapid vision-source movement、repeated mass reveal/hide；
- mass projectile/ AOE boundary crossing；
- 30 分鐘 soak 與 CPU、memory、bandwidth、queue、audit-lag report。

## 交付計畫

工作拆成數個 implementation phase，最後才執行一次 consolidated final verification。完整 testing、security inspection、stress testing 與 release check 不在每個 phase 重複執行。

### Phase 0：contract、threat model 與 baseline

- inventory deterministic component、resource、input、event、script outcome、snapshot/hash field；
- 分類為 `Public`、`TeamPrivate`、`VisibilityBound` 或 `ServerOnly`；
- 定義既有 gameplay system 的 projection policy coverage；
- 固定 protocol V2、snapshot、transition、repair、canonical hash schema；
- 擷取目前 10,000-entity CPU、memory、bandwidth baseline；
- 建立 non-interference/redaction test harness。

產出：inventory、classification、schema、harness 與 baseline data。

### Phase 1：shared selective replica foundation

- 將 `SelectiveReplicaRuntime` 與 canonical team hash 抽入 `omoba-core`；
- 加入 team-scoped identity、transition application、random tape、repair/rebase primitive；
- 從 `proto/game.proto` 產生 protocol V2 type；
- 建立 filtered snapshot encode/decode 與 compatibility guard；
- 提供消費相同 encoded frame 的 synthetic server-observer/client fixture。

產出：server 與 omfx 可整合、可編譯的 shared runtime/protocol；完整 determinism 與 fault validation 延後到 Phase 5。

### Phase 2：deterministic ECS projection pipeline

- 加入 Outcome/ObservableFact stable buffer contract；
- 遷移 gameplay system 與 script 以產生完整 projection fact；
- 實作 Wave A deterministic reduce/commit barrier；
- 實作 fixed-point team visibility、override、transition scheduling；
- 實作 Wave B parallel per-team projection/frame encoding；
- 拒絕缺少 cross-boundary projection policy 的內容。

產出：authoritative gameplay 可產生完整 projection fact 與 deterministic team frame；完整 non-interference/boundary validation 延後到 Phase 5。

### Phase 3：server team stream 與 observer validator

- Session 綁定 team-specific V2 stream；
- encoded team frame 立即 enqueue 並保留 bounded replay ring；
- 執行 isolated validation worker 與每個 active team 一份 observer replica；
- 加入 mismatch reporting 與後續 repair/rebase coordination；
- 實作 filtered join/rejoin 與 coverage-gap rebootstrap；
- 加入 redacted metric、trace、replay evidence、packet audit。

產出：server team stream、recovery 與 asynchronous observer validation；完整 parity/fault validation 延後到 Phase 5。

### Phase 4：omfx integration 與 cutover preparation

- 以 team `SelectiveReplicaRuntime` 取代 global local replica；
- 加入 barrier buffer、remembered render cache、transition presentation、repair/rebase handling；
- 要求 match-level V2 capability negotiation；
- 準備 shadow/dogfood configuration，但不啟用 secure default；
- 準備移除 global `TickBatch`、`StateHash`、master seed、`WorldSnapshot`、dead viewport/`VisSet` 與 nondeterministic vision code 的 cleanup patch，final verification 前不進行 irreversible cutover。

產出：可接受 consolidated verification 的 end-to-end V2 implementation。

### Phase 5：consolidated final verification 與 cutover

Phase 0 至 4 全部整合後才執行：

- 完整 unit/property suite；
- authoritative/server observer/omfx differential test；
- Windows/Linux determinism check；
- 所有 visibility-boundary integration scenario；
- network fault、reconnect、replay-ring、rebase、validator-overflow scenario；
- protocol fuzzing、hidden-target probing、packet inspection、redaction review、side-channel check；
- 10,000-entity performance suite 與 30 分鐘 stress soak，所有 active team observer 必須啟用；
- 一次檢視所有 blocking performance、bandwidth、security、verification-coverage gate；
- 修正 failure、標記受影響 evidence stale，只重跑受影響的 final-verification group；
- 執行 V2 shadow 與 internal dogfood acceptance；
- 所有 evidence 通過後才將 V2 設為 secure default；
- 移除 player 對 global protocol path 的存取並套用 cleanup。

Phase exit gate：所有 blocking gate 通過，packet/client-memory inspection 找不到 hidden information exposure，observer validation 保持非阻塞，且沒有 unresolved release blocker。

## Rollout 與 rollback

Rollout 依序為 server shadow generation、internal dogfood、opt-in secure match、secure default。Protocol version 在 match 建立時選擇；secure match 不允許 V1/V2 client 共存。

Rollback 只允許在 match 啟動前選擇明確標示為 non-secure 的 legacy mode。Active secure match 若無法 repair/rebase，必須安全結束並保存 diagnostic；不得傳送 global world state 後繼續。

## Compatibility 與 cleanup

Migration 期間，old/new type 可在 match-level protocol selection 下共存。V2 player path 絕不能呼叫 legacy global snapshot/hash serialization。Admin/query tool 必須使用不同 capability 與 transport boundary。

Cutover 後移除 global player lockstep fan-out、global player snapshot、player-visible master seed、player wire 上的 raw ECS ID、dead `client_visibility`/`last_visibility_tick`，以及任何可能暴露資料的 renderer-only fog assumption。

## 已核准決策

- 未授權 client 的封包與記憶體不得包含 hidden information。
- Visibility rule 結合 deterministic automatic vision 與 explicit gameplay override。
- Visibility 以 team 共享。
- Client 模擬 disclosed subworld，而不是只接收 state delta。
- Reveal 使用 scheduled commitment，不使用 rollback。
- Remember behavior 由 policy 決定；預設 forget，特定 unit 可保留 last-known presentation。
- 所有 conflict 永遠以 server 為準。
- 每個 active team 都有一份 server-local observer replica 驗算。
- Observer validation 非同步執行，絕不阻塞 outbound frame。
- Gameplay outcome 與 observable fact 在同一個 Specs gameplay wave 產生。
- Post-step team visibility 與 projection 在 deterministic commit barrier 後的第二個 parallel wave 執行。
