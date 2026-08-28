# ERPS 實作任務

## 執行規約（給 apply agent）

本清單以 `docs/superpowers/specs/2026-08-28-erps-ecs-matchmaking-design.md` 為產品決策來源，以本 change 的 `design.md` 與 `specs/*/spec.md` 為驗收契約。遇到歧義時依序採用：delta spec → design → 已核准產品設計；不得自行擴張第一版範圍。

執行每個 checkbox 時 MUST：

1. 只處理該 checkbox 與它明列的直接前置項，不順手實作後續階段。
2. 先讀該階段列出的主要檔案；若實際路徑尚不存在，依「預定路徑」建立，不另創同義 module。
3. 先新增或更新能失敗的聚焦測試，再完成最小實作；測試名稱需指出對應 invariant／scenario。
4. 執行該階段「完成門檻」中的最小相關指令。只有編譯與測試通過後才能勾選。
5. 不修改 `scripts/script-abi`、gameplay deterministic hash、既有 transport 預設值或 root `.bat`，除非 checkbox 明確要求。
6. 不提交 `target/`、DLL、EXE、PDB、log、trace、token、憑證或 load-test run 產物。
7. 如果前置 API 尚未存在，停止勾選並回到缺少的前置 task；不得用 placeholder、`todo!()`、永遠成功 stub 或跳過測試宣告完成。

預定 workspace 路徑：

- `erps/Cargo.toml`：workspace root；`erps/core/` 是 package `erps`，另含 server／load-test binaries。
- `erps/proto/`：package `erps-proto` 與 `.proto` source。
- `erps/client/`：package `erps-client`。
- `erps/client-ffi/`：package `erps-client-ffi`、公開 header 與 C smoke tests。
- `omb/src/erps/`：唯一允許的 `omb` integration adapter；不得把 matching core 放入 `omb`。

通用驗證指令以實際建立的 manifest 為準，預定為：

```powershell
cargo fmt --manifest-path erps/Cargo.toml --all -- --check
cargo test --manifest-path erps/Cargo.toml --workspace
cargo clippy --manifest-path erps/Cargo.toml --workspace --all-targets -- -D warnings
openspec validate add-erps-ecs-matchmaking --type change --strict
```

## 1. Workspace 與契約骨架

**目標：** 建立唯一 workspace、package 邊界與 protobuf 契約，使後續 task 不必猜 crate 或檔案位置。

**主要檔案：** `erps/Cargo.toml`、`erps/core/Cargo.toml`、`erps/proto/Cargo.toml`、`erps/proto/proto/erps.proto`、`erps/core/src/config.rs`。

**前置依賴：** 無。

**完成門檻：** `cargo metadata --manifest-path erps/Cargo.toml --no-deps` 成功；最小 workspace test 成功；proto 可產生 Rust types；設定非法值 tests 失敗於明確 validation error。

- [x] 1.1 建立 `erps`、`erps-proto`、`erps-client`、`erps-client-ffi` crates，並全部使用 workspace 固定 Rust 1.95.0 與本地 `../specs` path dependency
- [x] 1.2 在 `erps` 定義 library、`erps-server` 與 `erps-load-test` targets，確認最小 workspace 可建置
- [x] 1.3 建立 `erps-proto` protobuf build pipeline，固定 API major／minor envelope 與不可重用 tag 規則
- [x] 1.4 定義 `MatchmakingService`、`GameServerService`、唯讀 admin service 與 gRPC health service 的 proto 骨架
- [x] 1.5 建立 server、matching、party、profile、placement、transport、metrics、SDK 的聚焦 module 邊界
- [x] 1.6 新增 ERPS 設定型別與檔案載入，涵蓋 queue budget、batch window、Elo、ready、credit、heartbeat、placement、TLS 與 server policy
- [x] 1.7 新增設定 validation tests，拒絕零容量、非法 timeout、`max_instances` 超出 1～100、負 queue budget 與不合法模式成本

## 2. Stable ID、ECS components 與 resources

**目標：** 建立後續所有 system 共用的 domain types 與一個可測試的 Specs `World` constructor。

**主要檔案：** `erps/core/src/id.rs`、`erps/core/src/components/`、`erps/core/src/resources/`、`erps/core/src/world.rs`。

**前置依賴：** 1.1～1.7。

**完成門檻：** `id`、`components`、`world` 對應的聚焦 tests 全部通過；RPC-facing types 不含 Specs `Entity`。

- [x] 2.1 定義 `PlayerId`、`PartyId`、`TicketId`、`ProposalId`、`MatchId`、`ServerId` 與 `ServerGeneration` stable opaque ID
- [x] 2.2 為 stable ID 建立 deterministic ordering、serialization、parse validation 與 round-trip tests
- [x] 2.3 實作 player components：每模式 `EloRating`、`CreditScore`、`ConnectionId`、`AllowedRegions`、`PlayerState`
- [x] 2.4 實作 party components：`PartyName`、members、leader、revision 與 state
- [x] 2.5 實作 ticket components：mode、enqueue logical time、search range、bucket owner 與 state
- [x] 2.6 實作 proposal／match components：roster、accept deadline、per-player accept state、assignment、reservation 與 lifecycle
- [x] 2.7 實作 game server components：endpoint、region、modes、capacity、cost、instance count、heartbeat、health 與 failure score
- [x] 2.8 建立 bounded command／event／control resources、logical clock、ID allocator 與 deterministic seed resources
- [x] 2.9 建立 `ErpsWorld` constructor，註冊所有 components／resources 並以測試確認預設 invariant

## 3. Profile、Elo 與信用資料

**目標：** 將可持久化玩家資料與 ECS lifecycle 解耦，完成三種模式所需的純函式 rating／credit 規則。

**主要檔案：** `erps/core/src/profile.rs`、`erps/core/src/rating.rs`、`erps/core/src/credit.rs`、`erps/core/tests/rating_properties.rs`。

**前置依賴：** 2.1～2.9、1.6 的 Elo／credit 設定。

**完成門檻：** golden tests 固定 1v1／5v5／八人輸出；property tests 證明 rating／credit 有界且無 NaN；memory provider 相同輸入可重現。

- [x] 3.1 定義 async／可替換的 `PlayerProfileProvider` 邊界與錯誤分類
- [x] 3.2 實作 deterministic memory profile provider，保存每模式 rating、場次與信用分
- [x] 3.3 實作傳統 Elo expected score、可設定 K-factor 與單場 delta clamp
- [x] 3.4 實作 1v1 與 5v5 rating 更新並加入 golden tests
- [x] 3.5 實作八人名次 pairwise 勝／平／負彙總與原 party 無關的 rating 更新
- [x] 3.6 實作信用分 0～100 clamp、拒絕／逾時 penalty、完成場次恢復與遞增停權 policy
- [x] 3.7 新增 property tests，驗證 rating／credit 永不 overflow、NaN 或超過設定邊界

## 4. Party、命令套用與名稱驗證

**目標：** 完成進入 matching core 前的 authenticated party／queue command state machine。

**主要檔案：** `erps/core/src/command.rs`、`erps/core/src/party.rs`、`erps/core/src/session.rs`、`erps/core/tests/party_lifecycle.rs`。

**前置依賴：** 2.1～2.9、3.1～3.7。

**完成門檻：** leader／revision／freeze／mode-size／region／credit 規則皆有 negative tests；相同 `request_id` 不重複 mutation；30 秒 grace 使用可注入 logical clock 測試。

- [ ] 4.1 實作 session-authenticated player command envelope、`request_id` cache 與 deterministic apply order
- [x] 4.2 實作 Unicode NFC party name normalization 與只允許 1～24 個 Unicode letter／number 的 validator
- [x] 4.3 新增 CJK 合法、空白／標點／emoji／控制／零寬／combining-only 非法的名稱測試
- [x] 4.4 實作短效不可猜 invite token、到期與使用次數 policy
- [x] 4.5 實作 create／invite／join／leave／kick／rename party commands 與 leader authorization
- [x] 4.6 實作 party revision optimistic concurrency 與 stale revision tests
- [x] 4.7 實作排隊／proposal／match 期間 roster 與 rename freeze
- [ ] 4.8 實作各模式 party size、全員在線、credit eligibility、region intersection 與 single-active-state enqueue validation
- [ ] 4.9 實作 cancel 與 client 斷線 30 秒 grace period lifecycle

## 5. Queue 分桶與 deterministic 平行管線

**目標：** 建立只讀平行搜尋、單一原子 commit 的核心管線；此階段不實作特定模式的組隊演算法。

**主要檔案：** `erps/core/src/matching/bucket.rs`、`snapshot.rs`、`dispatcher.rs`、`claim.rs`、`erps/core/tests/deterministic_replay.rs`。

**前置依賴：** 2.1～2.9、4.1～4.9。

**完成門檻：** 使用 1、2 與可用 CPU 數 workers 跑相同 fixture，proposal ordering／IDs 完全相同；halo duplicate 與 cancel race 最多只 commit 一次。

- [x] 5.1 實作 mode／region／Elo bucket key、owner shard 與相鄰 halo 建立邏輯
- [x] 5.2 實作階梯式搜尋範圍擴張與最大上限，使用 logical time 而非 wall-clock iteration timing
- [x] 5.3 實作 party effective rating、party size adjustment、internal spread adjustment 與 `max_party_rating_spread`
- [x] 5.4 建立不可變 candidate snapshot，使平行 worker 不持有 ECS mutable storage
- [ ] 5.5 實作 Specs dispatcher／Rayon shard candidate system 與固定 per-shard search budget
- [x] 5.6 在 `matching/dispatcher.rs` 將每個 worker 的 candidate 收集為局部 `Vec<Candidate>`，以 `(oldest_enqueued_at, quality_key, owner_shard, stable_ticket_ids)` 明確排序後再合併；不得以 `HashMap` iteration 或 channel 到達順序決定 winner，並加入反轉 worker 完成順序仍同結果的 test
- [ ] 5.7 實作單一 claim／commit system，原子重驗 ticket、party revision、player 與 proposal state
- [x] 5.8 新增不同 worker thread 數與相同 seed 產生完全相同結果的 replay tests
- [x] 5.9 新增跨 bucket halo duplicate-claim 與 cancel／commit race integration tests

## 6. 三種配對演算法

**目標：** 在第 5 階段 pipeline 上提供三個無 ECS mutable borrow 的純 candidate builder／scorer。

**主要檔案：** `erps/core/src/matching/one_v_one.rs`、`five_v_five.rs`、`free_for_all.rs`、`score.rs`、`erps/core/tests/mode_properties.rs`。

**前置依賴：** 3.3～3.5、5.1～5.9。

**完成門檻：** table tests 覆蓋合法與不可行 party 組合；property tests 證明成員唯一、party 不拆、1v1 為 2 人、5v5 為 5+5、自由混戰為八個單人 team。

- [x] 6.1 實作 1v1 最近 Elo、等待優先與 stable tie-break candidate builder
- [x] 6.2 實作 5v5 party sum-to-five bounded bin-packing primitive
- [x] 6.3 實作 5v5 兩隊 5+5 組合與 party 不拆 invariant checker
- [x] 6.4 實作 5v5 等待、兩隊 Elo 差、隊內離散與 party 結構軟性懲罰 scorer
- [x] 6.5 實作八人模式 1～4 人 party sum-to-eight bounded search
- [x] 6.6 實作八人全場 Elo range／離散／等待 scorer 與八個單人 team roster builder
- [x] 6.7 新增三種模式的 table-driven tests，涵蓋不可行組合、邊界 Elo、長等待與 tie-break
- [x] 6.8 新增隨機 party 組合 property tests，驗證成功 roster 人數、team 數、成員唯一與 party 不拆

## 7. Ready check、重新排隊與信用處分

**目標：** 把 candidate 轉成需全員確認的 proposal，並以可稽核原因分類決定重排與信用處分。

**主要檔案：** `erps/core/src/proposal.rs`、`ready_check.rs`、`requeue.rs`、`erps/core/tests/ready_check.rs`。

**前置依賴：** 3.6、4.1、5.7、6.1～6.8。

**完成門檻：** deadline boundary、duplicate／late response、mixed party failure、infra failure fixtures 全部通過；`AwaitingAccept` 時 reservation ledger 必須為零變化。

- [x] 7.1 實作 `Queued -> Proposed -> AwaitingAccept` proposal 建立與 authoritative deadline
- [x] 7.2 實作每人 `AcceptMatch`／`RejectMatch`、proposal binding、request 冪等與 stale proposal error
- [x] 7.3 實作全員接受才進 `AwaitingPlacement`，並驗證 ready check 期間不建立正式 reservation
- [x] 7.4 實作主動拒絕與 deadline timeout 的個人信用處分及停止配對
- [x] 7.5 實作 infra failure 不處分任何玩家的 failure classification
- [x] 7.6 實作已同意單人與未受影響完整 party 保留原 `enqueued_at` 自動回 queue
- [x] 7.7 實作含失敗成員 party 保留 roster 並進 `NotReady`，不得自動踢人
- [x] 7.8 新增 mixed-party proposal 取消、late accept、duplicate response 與 deadline boundary integration tests

## 8. Game server registry 與 placement

**目標：** 在所有玩家接受後，安全地把 proposal 配置到異質 game server，且任何錯誤都完整釋放 reservation。

**主要檔案：** `erps/core/src/server_registry.rs`、`placement.rs`、`reservation.rs`、`launch.rs`、`erps/core/tests/placement_properties.rs`。

**前置依賴：** 2.7、7.1～7.8、1.6 的 server policy。

**完成門檻：** 隨機 fleet property tests 證明 capacity 與 instance count 永不超限；reject／timeout／stale generation 後 ledger 可對帳；client 只在 `Ready` 後可取得 endpoint／token。

- [x] 8.1 實作 server register validation、server-class trusted limits 與 generation replacement
- [x] 8.2 實作 heartbeat health thresholds、stale generation ignore 與 server-lost lifecycle
- [x] 8.3 實作 instance reconcile 與 authoritative capacity／reservation ledger 對帳
- [x] 8.4 實作 region、supported mode、capacity units 與 `max_instances` 雙重硬限制
- [x] 8.5 實作 capacity fragmentation、負載與 recent launch failure 的 deterministic placement scorer
- [x] 8.6 實作 ready check soft feasibility 與全員接受後的 atomic reservation
- [x] 8.7 實作 `Reserved -> Accepted -> Ready -> Running -> Finished` lifecycle 與 timeout
- [x] 8.8 實作 game server reject／ready timeout 的完整 reservation release 與其他 server retry
- [x] 8.9 實作 placement waiting timeout，讓玩家無信用處分並保留等待時間回 queue
- [x] 8.10 實作 running server lost event，明確不遷移執行中 match
- [x] 8.11 新增隨機 fleet／mode cost property tests，證明 capacity 與 instance 永不超配

## 9. gRPC server、認證與背壓

**目標：** 將已測試的 core state machine 暴露為有版本、可認證、有界且可恢復狀態的 gRPC services。

**主要檔案：** `erps/proto/proto/erps.proto`、`erps/core/src/grpc/`、`erps/core/src/bin/erps-server.rs`、`erps/core/tests/grpc_e2e.rs`。

**前置依賴：** 1.3～1.7、4.1～4.9、7.1～7.8、8.1～8.11。

**完成門檻：** 真實 loopback gRPC test 完成 connect → party → enqueue → accept → launch → ready → match；慢 consumer 記憶體保持在 budget；production identity／TLS negative tests 通過。

- [x] 9.1 完成 `MatchmakingService.OpenSession`、party、queue、ready、`GetState` 與 `WatchEvents` handlers；SDK 對外可保留 `connect()`，proto RPC 不得命名為 `Connect`
- [x] 9.2 完成 `GameServerService` register、雙向 `ControlStream` 與 `ReconcileInstances` handlers
- [x] 9.3 完成唯讀 admin queue／capacity／match／reservation／health handlers
- [x] 9.4 實作 API major reject、minor capability negotiation 與 protocol compatibility tests
- [x] 9.5 實作可注入 token validator，確保 production identity 不信任 client-provided player ID
- [x] 9.6 實作 TLS 設定與 plaintext 僅 loopback／explicit development guard
- [x] 9.7 實作 bounded command、client event、control stream queue 及 critical／latest-state 分流
- [x] 9.8 實作慢 consumer stream termination 與 `GetState` reconciliation tests
- [x] 9.9 實作 gRPC health check、drain mode 與 bounded graceful shutdown
- [x] 9.10 新增真實 gRPC end-to-end test：client 配對、全員接受、server launch／ready、client 收到 endpoint／token

## 10. `omb` game server adapter

**目標：** 讓 `omb` 成為 ERPS game server client，不把 ERPS matching state 或 Specs `World` 嵌入 `omb`。

**主要檔案：** `omb/src/erps/mod.rs`、`omb/src/erps/client.rs`、`omb/src/erps/instance_adapter.rs`、`omb/Cargo.toml` 與對應 integration tests。

**前置依賴：** 9.2、9.4～9.7；不得先於 GameServerService contract 穩定化。

**完成門檻：** feature 關閉時既有 `omb` build／test 路徑不變；feature 開啟時 adapter 可 register、heartbeat、launch、ready、finish、reconnect／reconcile。

- [x] 10.1 在 `omb` 新增 feature／設定隔離的 ERPS game server client，不改既有預設 gameplay 啟動
- [x] 10.2 實作 `omb` register、generation、heartbeat 與 capability／mode cost 回報
- [x] 10.3 實作 `LaunchMatch` 接收、instance 建立 acknowledgement、`Ready` endpoint／token 與 finished 回報
- [x] 10.4 實作 control stream 重連與 `ReconcileInstances`
- [x] 10.5 新增 `omb` adapter integration test，驗證 ERPS 未啟用時既有流程不變，啟用時完整註冊／launch

## 11. Rust client SDK

**目標：** 提供不暴露 tonic generated internals 的型別安全 public API。

**主要檔案：** `erps/client/src/lib.rs`、`connection.rs`、`party.rs`、`queue.rs`、`events.rs`、`examples/`。

**前置依賴：** 9.1、9.4～9.8。

**完成門檻：** public example 只匯入 `erps-client` 即完成 party-to-match；重連後以 `GetState` 對帳，不重複 enqueue／accept。

- [x] 11.1 實作 `erps-client` connection、TLS、token、API negotiation 與 session lifecycle
- [x] 11.2 實作 party／invite／queue／accept／reject 的 typed async commands 與 automatic request IDs
- [x] 11.3 實作 typed event stream、bounded local handling 與 critical event preservation
- [x] 11.4 實作斷線重連、grace period awareness 與 `GetState` reconciliation
- [x] 11.5 建立 Rust SDK 範例與端到端 tests，涵蓋 party 建立至 match ready

## 12. C ABI poll SDK

**目標：** 以 Rust SDK 為唯一網路實作，提供 layout／ownership／threading 明確的 C poll facade。

**主要檔案：** `erps/client-ffi/src/lib.rs`、`include/erps_client.h`、`tests/c_smoke/`、platform build scripts。

**前置依賴：** 11.1～11.5；C SDK 不得自行重寫 gRPC client。

**完成門檻：** 真實 C compiler 可只靠公開 header 與發行 library 完成端到端 smoke；ownership tests 無 double-free／leak；所有 panic 留在 Rust ABI 邊界內。

- [x] 12.1 定義 versioned C ABI header、opaque client／event handles、error codes 與 accessor ownership contract
- [x] 12.2 實作 C client create／connect／shutdown 與內部 Rust async runtime／network thread
- [x] 12.3 實作 thread-safe party／queue／accept／reject command functions
- [x] 12.4 實作 bounded event bridge 與單 consumer `erps_client_poll()`
- [x] 12.5 實作 event accessors、唯一 release API 與 Rust/C allocator isolation
- [x] 12.6 對所有 exported functions 加入 panic containment 與 invalid handle／thread misuse diagnostics
- [x] 12.7 建立 Windows x64 DLL/import library/header 與 Linux x86_64 shared object/header build targets
- [x] 12.8 建立真實 C compiler smoke program，驗證 create、party、enqueue、poll、accept、match、release、shutdown
- [x] 12.9 加入 ABI symbol／header compatibility check，避免非刻意的 breaking export 變更

## 13. 可觀測性與營運安全

**目標：** 讓 queue、matching、ready 與 placement 可量測，但 observability 不成為 authoritative state 或敏感資料外洩來源。

**主要檔案：** `erps/core/src/metrics.rs`、`telemetry.rs`、`logging.rs` 與 redaction tests。

**前置依賴：** 5～9 階段已有穩定 lifecycle event；可分批隨各階段加入，最後於本節統一驗證。

**完成門檻：** 關閉／開啟 tracing 產生相同 logical result；token redaction tests 通過；bounded queue high-watermark 可由 admin／metrics 觀察。

- [x] 13.1 加入結構化 logs，且不得記錄 session token、invite token 或 connection token
- [x] 13.2 加入 per-mode／region queue、throughput、latency、Elo quality、ready、credit、capacity 與 reservation metrics
- [x] 13.3 加入 bounded queue depth／high-watermark、launch failure、reconnect 與 invariant failure metrics
- [x] 13.4 加入 optional OpenTelemetry tracing feature，確保未啟用時不影響核心 deterministic state
- [x] 13.5 新增 metrics／logs tests，確認敏感 token 不外洩且 gameplay／matching結果不依賴 observability

## 14. 100000 玩家 load test

**目標：** 用同一 scenario／invariant checker 分別量測 core 與完整 gRPC 路徑；正確性失敗不得產生 PASS 報告。

**主要檔案：** `erps/core/src/bin/erps-load-test.rs`、`erps/core/src/load_test/`、`erps/core/tests/load_smoke.rs`、`docs/erps/load-test.md`。

**前置依賴：** 5～9、11、12 階段；14.1～14.9 完成後才執行 14.10／14.11。

**完成門檻：** 固定 seed 的 100,000 player core run 零 invariant failure；縮小 gRPC run 經 Rust／C clients 與模擬 server 完成；報告包含可比較的環境與設定 metadata。

- [x] 14.1 實作 seedable scenario generator，預設 100,000 玩家與三種模式比例
- [x] 14.2 實作 5v5 1～5 人、八人 1～4 人 party distribution 與合法／非法輸入生成
- [x] 14.3 實作多 region、異質容量、不同 mode cost 與 1～100 `max_instances` fleet generator
- [x] 14.4 實作 enqueue、cancel、accept、reject、timeout、server 上下線與 match completion workload
- [x] 14.5 實作 in-process core runner 與 `--grpc` 完整 RPC runner，共用 seed 與 invariant checker
- [x] 14.6 實作零重複玩家、零拆 party、roster／team 正確、零超配、零 committed loss、零無責任處分 checks
- [x] 14.7 實作 invariant violation 最小必要診斷與非零退出，不得在失敗時宣告效能 PASS
- [x] 14.8 輸出 throughput、p50／p95／p99、Elo quality、ready rate、capacity utilization、retry、unmatched 與 memory high-watermark
- [x] 14.9 記錄硬體、worker threads、seed 與所有 matching 設定，實作同環境 baseline compare
- [x] 14.10 執行固定 seed 100,000 玩家 core run，保存可重現報告並確認所有硬性 invariant
- [x] 14.11 執行縮小但完整的 `--grpc` Rust／C／game-server run，保存傳輸路徑報告

## 15. 最終驗證與交付

**目標：** 只在所有 capability 有可追溯 evidence、所有支援平台驗證完成且 workspace 無產物污染後宣告 implementation complete。

**主要檔案：** `docs/erps/`、本 change 的 `tasks.md`／`specs/`、CI 或本機 evidence 摘要；不得提交原始 load run dump。

**前置依賴：** 1～14 全部完成。

**完成門檻：** ERPS workspace fmt／clippy／tests、`omb` adapter tests、C SDK platform smoke、OpenSpec strict validation 全部成功；六個 capability 的 42 個 scenarios 皆能指向 test 或明確 evidence。

- [x] 15.1 執行 `cargo fmt` 與所有 ERPS crates 的 clippy／unit／property tests
- [x] 15.2 執行 ERPS ECS／gRPC／`omb` adapter integration suites
- [x] 15.3 在 Windows x64 與 Linux x86_64 驗證 C SDK build、symbol、header 與 smoke test
- [x] 15.4 驗證 production TLS／token policy、development plaintext guard 與 graceful shutdown
- [x] 15.5 驗證相同 seed 在不同 worker thread 數產生相同 logical match 結果
- [x] 15.6 撰寫 ERPS server、`omb` 註冊、Rust SDK、C SDK、設定與 load-test 操作文件
- [x] 15.7 檢查 repository 不含 `target/`、DLL、EXE、PDB、log、trace、token 或 load-test 暫存產物
- [x] 15.8 彙整 capability-to-test evidence，確認六份 OpenSpec specs 的所有 scenario 都有自動化測試或明確驗證紀錄
