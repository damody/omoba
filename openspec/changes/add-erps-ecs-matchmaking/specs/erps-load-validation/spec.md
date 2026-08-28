## ADDED Requirements

### Requirement: 核心規則具備單元與 property 驗證
ERPS SHALL 以單元與 property tests 驗證 Elo、搜尋擴張、party validation、5v5／八人 bounded matching、ready check、信用分及容量 ledger。隨機輸入 MUST 永遠維持 roster、party 與容量 invariant。

#### Scenario: 隨機 party 組合不破壞 roster
- **WHEN** property test 產生任意合法與非法 party size、Elo 與 enqueue sequence
- **THEN** 所有成功 match 都符合模式人數、team 數與 party 不拆規則，非法輸入得到明確拒絕

### Requirement: ECS 整合測試覆蓋併發與失敗路徑
ERPS SHALL 測試跨 shard claim、party revision、cancel／accept 競爭、proposal timeout、server reject、heartbeat loss、reservation release 與 retry。測試 MUST 證明任一失敗只會完整 commit 或完整回滾。

#### Scenario: Cancel 與 candidate commit 競爭
- **WHEN** cancel command 與包含同 ticket 的候選在相鄰 tick 邊界競爭
- **THEN** ticket 只會成功取消或成功進入一個 proposal，不會同時存在兩個 authority state

### Requirement: gRPC 與 SDK 使用真實端到端測試
測試 SHALL 啟動真實 ERPS gRPC server、Rust client、模擬 game server，並 SHALL 使用真實 C compiler 建置／連結 C ABI smoke test。測試 MUST 覆蓋 TLS policy、token validator、冪等、重連、慢 consumer 與 instance ready。

#### Scenario: C client 收到 ready match
- **WHEN** C smoke client 經 gRPC enqueue／accept，而模擬 game server 回報 instance `Ready`
- **THEN** C client 透過 poll 收到 match event並安全釋放所有 event 資源

### Requirement: Load test 預設模擬 100000 玩家
`erps-load-test` SHALL 預設建立 100,000 玩家，混合三種模式、合法 party size、多 region 與異質 game server，並持續產生 enqueue、cancel、accept、reject、timeout、server 上下線與 match completion。

#### Scenario: 預設大規模測試完成
- **WHEN** 使用固定 seed 執行預設 load test
- **THEN** 工具完成 100,000 玩家 scenario，輸出設定、硬體、worker 數、throughput、latency、Elo quality、容量利用率與記憶體高水位

### Requirement: Load test 同時提供核心與完整 gRPC 模式
Load test SHALL 預設提供 in-process core 模式，並 SHALL 以 `--grpc` 提供完整序列化、網路與 SDK 路徑。兩種模式 MUST 使用相同 scenario seed 與 invariant checker 語意。

#### Scenario: gRPC 模式納入傳輸成本
- **WHEN** 操作者以 `--grpc` 執行相同 scenario
- **THEN** 報告清楚標示完整 RPC 路徑，且不將結果與 in-process core latency 混為同一 baseline

### Requirement: 100000 玩家驗證硬性 invariant
所有成功 load run MUST 證明零重複玩家、零 party 拆分、零 roster／team 數錯誤、零 server 超配、零 committed match 遺失、零無責任信用處分，且相同 seed 結果一致。任一 invariant 違反 MUST 使程序非零結束。

#### Scenario: 超配立即使測試失敗
- **WHEN** invariant checker 發現任一 server 的 capacity 或 instance count 超過上限
- **THEN** load test 記錄最小必要診斷、以非零狀態結束且不得宣告效能 PASS

### Requirement: 效能以可重現 baseline 評估
Load report SHALL 記錄硬體、worker threads、seed 與全部影響匹配的設定，並 SHALL 支援與同環境保存 baseline 比較。第一版 MUST NOT 使用未標示硬體條件的絕對 p99 作為跨機器驗收門檻。

#### Scenario: Baseline 比較具有環境資訊
- **WHEN** 操作者要求比較目前 run 與 baseline
- **THEN** 報告同時顯示兩次執行環境與設定差異，避免把不可比結果宣告為 regression

