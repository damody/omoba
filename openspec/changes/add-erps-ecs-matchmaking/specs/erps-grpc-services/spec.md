## ADDED Requirements

### Requirement: ERPS 只以獨立 gRPC process 提供服務
ERPS SHALL 以獨立 process 執行，MUST NOT 嵌入 `omb`。Client SHALL 直接使用 `MatchmakingService`；`omb` SHALL 以 game server 身分使用 `GameServerService`；唯讀營運查詢 SHALL 使用獨立 admin service。

#### Scenario: omb 透過服務註冊
- **WHEN** `omb` 啟動 ERPS integration
- **THEN** 它透過 `GameServerService` 註冊與 control stream 通訊，而不是連結或共享 ERPS `World`

### Requirement: MatchmakingService 涵蓋完整 client lifecycle
`MatchmakingService` SHALL 以 `OpenSession` 提供 session 建立，並提供 party、invite、enqueue、cancel、accept、reject、`GetState` 與 `WatchEvents` 操作。SDK 對外仍可命名為 `connect()`。每個 mutation MUST 帶 `request_id` 並在 session 範圍冪等。

#### Scenario: 重送 enqueue 不建立重複 ticket
- **WHEN** SDK 因網路重試以相同 `request_id` 重送成功的 enqueue
- **THEN** ERPS 回傳相同 logical result 且只存在一張 ticket

### Requirement: GameServerService 使用雙向控制 stream
`GameServerService` SHALL 支援註冊、雙向 `ControlStream` 與 `ReconcileInstances`。Control stream SHALL 承載 heartbeat、launch acknowledgement、instance ready／finished 與 server-directed `LaunchMatch`。

#### Scenario: 重連後 reconcile instance
- **WHEN** game server control stream 中斷後以相同新 generation 重連
- **THEN** server 回報目前 instances，ERPS 在接受新 placement 前完成 capacity ledger 對帳

### Requirement: Protocol 版本可協商且 protobuf tag 不重用
所有 service envelope SHALL 帶 API major／minor。ERPS MUST 拒絕不相容 major，並 SHALL 以 capabilities 處理 minor 差異。已發布 protobuf tag MUST NOT 重用為不同語意。

#### Scenario: 不相容 major 被拒絕
- **WHEN** client 以 ERPS 不支援的 API major 連線
- **THEN** ERPS 回傳明確 version error 且不建立 session 或修改 ECS state

### Requirement: 認證與 transport policy 可部署於正式環境
ERPS SHALL 支援 TLS 與可注入 token validator。Production mode MUST 使用驗證後的 player identity，MUST NOT 信任 client 自稱 player ID；plaintext MUST 只允許 loopback 或明確 development policy。

#### Scenario: Production 拒絕未驗證身分
- **WHEN** 非 loopback client 在 production mode 以無效 token 呼叫 `OpenSession`
- **THEN** ERPS 拒絕 session，且不建立 player ECS state

### Requirement: 所有通訊 queue 有界且關鍵事件可靠
Command、client event 與 game server control queue SHALL 有固定容量。非關鍵狀態 MAY 合併為最新值；proposal、match、launch、cancel 與信用處分 MUST NOT 靜默丟棄。慢 client 長期滿載時 SHALL 中止 stream，並可用 `GetState` 對帳。

#### Scenario: 慢 client 不造成無界記憶體
- **WHEN** client 不消費 `WatchEvents` 且 event queue 持續滿載
- **THEN** ERPS 在固定 budget 內終止 stream、保留 authoritative state，且 client 可重連後用 `GetState` 恢復

### Requirement: ERPS 支援健康檢查與安全關閉
ERPS SHALL 提供 gRPC health check。Graceful shutdown SHALL 停止新 enqueue，將正在 commit 的 proposal／reservation推進到安全邊界後於期限內關閉 streams，MUST NOT 發布半完成 match。

#### Scenario: 關閉期間不接受新 ticket
- **WHEN** ERPS 已進入 graceful shutdown
- **THEN** 新 enqueue 得到明確 unavailable 結果，既有 commit 只會完整成功或完整回滾
