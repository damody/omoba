# ERPS ECS 配對服務設計

## 1. 目標

在 `D:\code\omoba` 建立新版 Elo Ranking Pairing System（ERPS）。ERPS 是獨立 process，使用 monorepo 內的 `specs` fork 建立資料導向、可平行化的配對核心，並透過 gRPC 服務 client 與 `omb` game server。

第一版必須支援：

- 1v1：兩名單人玩家。
- 5v5：接受 1～5 人 party；party 不可拆，組成兩支各 5 人隊伍。
- 八人自由混戰：接受 1～4 人 party；party 一起配入同一場，但進場後八名玩家各自是獨立 team，原 party 成員彼此也是對手。
- 初始 Elo 範圍限制隨等待時間逐步放寬。
- 異質 game server；每台 server 可執行 1～100 個 instance，不同模式消耗不同容量權重。
- 配對後全員 ready check；未在期限內同意者停止配對並扣信用分，其餘符合條件者保留等待優先權重新配對。
- Rust client library 與採 poll 模式的 C ABI client library。
- 100,000 名玩家的正確性、吞吐量與延遲測試。

第一版為單一 ERPS authority、記憶體 queue，不支援 ERPS active-active 或 queue 重啟恢復。

## 2. 參考專案與重做範圍

舊 `damody/Elo-Ranking-Pairing-System` 將 Elo、room、queue、MQTT、MySQL 與 game lifecycle 集中在單體服務，使用 `Rc<RefCell<_>>`、集合掃描及 Rayon。新版只保留傳統 Elo 預期勝率與依等待時間擴大搜尋範圍的概念；資料模型、併發邊界、party 組合、ready check、server placement、RPC 與測試全部重做。

## 3. Workspace 與產物

新增以下 crate：

- `erps`：Specs components、resources、systems、Elo、匹配、ready check、信用分、placement 與 runtime。
- `erps-proto`：唯一 protobuf schema 與產生的 gRPC types。
- `erps-client`：Rust async gRPC client library。
- `erps-client-ffi`：C ABI library，產生 Windows x64 DLL/import library/header 與 Linux x86_64 shared object/header。

提供以下 binary：

- `erps-server`：唯一支援的 ERPS 部署形式。
- `erps-load-test`：in-process 核心負載測試與可選完整 gRPC 負載測試。

`omb` 不連結或內嵌 ERPS。Client 直接呼叫 `MatchmakingService`；`omb` 只以 game server 身分呼叫 `GameServerService`。

## 4. 架構與一致性邊界

gRPC handler 不直接修改 Specs `World`。Handler 驗證 envelope 與大小限制後，把命令送入 bounded queue。ERPS runtime 被命令喚醒，在短 batch window 內收集命令，於單一 tick 邊界依穩定順序套用。

昂貴的候選產生與評分依 region、mode、Elo bucket 分 shard 平行執行。相鄰 bucket 以只讀 halo 提供候選；每張 ticket 只有一個 owner shard。候選結果回到 deterministic commit 階段後，必須原子確認 ticket claim、party revision、player state、proposal state 與容量 reservation。任何條件失敗都放棄整個候選，不產生半完成 match。

命令套用、claim 與 commit 是單一 authority；只讀候選產生與評分使用 Specs dispatcher／Rayon worker threads。結果不得依賴 hash iteration order 或 worker 完成順序。相同設定、seed 和命令序列必須得到相同 proposal、roster 與 placement。

## 5. ECS 資料模型

網路與 SDK 只看到穩定 opaque ID，不暴露 Specs `Entity`。

主要 player components：

- `PlayerId`
- 每模式 `EloRating`
- `CreditScore`
- `ConnectionId`
- `AllowedRegions`
- `PlayerState`

主要 party components：

- `PartyId`
- `PartyName`
- `PartyMembers`
- `PartyLeader`
- `PartyRevision`
- `PartyState`

主要 ticket components：

- `TicketId`
- `QueueMode`
- `EnqueuedAt`
- `SearchRange`
- `BucketKey`
- `TicketState`

主要 game server components：

- `ServerId`
- `ServerGeneration`
- `Endpoint`
- `Region`
- `SupportedModes`
- `CapacityTotal`／`CapacityUsed`
- `ModeCosts`
- `MaxInstances`／`RunningInstances`
- `LastHeartbeat`
- `ServerHealth`
- `RecentLaunchFailures`

主要 proposal／match components：

- `ProposalId`
- `ProposalRoster`
- `AcceptDeadline`
- `PlayerAcceptStates`
- `MatchId`
- `MatchRoster`
- `AssignedServer`
- `ReservationCost`
- `MatchState`

## 6. Party 規則

Party 使用短效 invite token 加入。Party name 只是顯示資料，不是驗證憑證，也不要求唯一。UI 應同時顯示 leader 或 party 短 ID 區分重名。

Party name 在 Unicode NFC normalization 後必須為 1～24 個 Unicode 字元，只允許 Unicode letter／number 類別，包含 ASCII 字母、數字與 CJK。空白、標點、emoji、控制字元、零寬字元與 combining-only 字元一律拒絕。只有 leader 能更名，且排隊、ready check 或 match 期間不可更名。

所有 party 變更使用 revision 做 optimistic concurrency check。只有 leader 能 enqueue／cancel。所有成員必須在線、符合信用資格且未參與其他 ticket、proposal 或 match。

模式 party 大小限制：

- 1v1：1。
- 5v5：1～5。
- 八人自由混戰：1～4。

## 7. Elo 與搜尋範圍

每種模式有獨立 rating，預設初始值 1000。第一版使用傳統 Elo，K-factor 可依模式與玩家已完成場次設定。ERPS 提供賽後結果回報 API；rating storage 透過可注入 `PlayerProfileProvider` 隔離。測試預設 memory provider，正式環境可接外部 profile service。

Party effective rating 使用成員平均值，加上可設定的 party size adjustment 與 internal spread adjustment。Ranked queue 設定 `max_party_rating_spread`；超過時拒絕 enqueue，避免以平均值掩蓋極端差距。

搜尋範圍採可設定的階梯式擴張。建議預設為初始 100 Elo、每等待 5 秒增加 50、最大 600。具體數值不是 RPC contract。候選必須滿足 effective rating 範圍、party spread policy 與所有成員允許 region 的交集。

1v1 使用標準雙方 Elo 更新。5v5 以對方 team effective rating 計算每名玩家結果。八人自由混戰把每名玩家視為與其他七人進行成對比較：較高名次為勝、同名次為平、較低為負；彙總後套用每場最大變動限制。原 party 不共享自由混戰結果。

## 8. 匹配演算法

### 8.1 1v1

在相容 region 與互相重疊的搜尋範圍中，選擇 Elo 差最小的兩張單人 ticket。相同品質時優先最久等待，再以穩定 ID tie-break。

### 8.2 5v5

使用 bounded party bin-packing 找出兩邊人數恰好為 5 的組合，party 不可拆。候選評分依序考慮：

1. 最久等待 party。
2. 兩隊 effective Elo 差。
3. 隊內 Elo 離散度。
4. Party 結構差異。
5. 穩定 ID tie-break。

Party 結構平衡是軟性懲罰，等待時間增加後逐步放寬，不要求全域最佳解。

### 8.3 八人自由混戰

以 1～4 人 party 為不可拆的配對票，bounded search 湊滿恰好 8 人。候選評分考慮全場最高／最低 Elo 差、整體離散度、最久等待與穩定 ID。進場 roster 將八名玩家分成八個單人 team。

## 9. Ready check 與信用分

匹配候選不會直接成為 match。ERPS 建立 proposal，要求每位玩家個別在設定期限內同意；預設 15 秒。Server 回傳 authoritative deadline，SDK 的本機倒數只供顯示。

狀態機：

```text
Queued
  -> Proposed
  -> AwaitingAccept
       | all accepted -> AwaitingPlacement -> Launching -> Ready -> Matched
       | reject       -> proposal cancelled
       ` timeout      -> proposal cancelled
```

`AcceptMatch`／`RejectMatch` 必須攜帶 `proposal_id` 與 `request_id`。重複命令冪等；延遲的舊 proposal 回覆不能影響新 proposal。

取消 proposal 時：

- 主動拒絕者停止配對，預設扣 2 信用分。
- 到期未回應者停止配對，預設扣 5 信用分。
- ERPS 或 game server 基礎設施失敗不扣任何玩家信用分。
- 已同意的單人 ticket 保留原始 `enqueued_at` 自動回 queue。
- 未受失敗成員影響且 roster 完整的 party 保留等待時間自動回 queue。
- 含拒絕／未回應成員的 party 保留 roster 但進入 `NotReady`，不得自動踢人或改 roster。Leader 必須移除失敗成員，或等待其恢復資格後再次 enqueue。

信用分為 0～100，預設 100。低於 60 暫停排隊，停權時間依近期違規次數遞增。正常完成若干場比賽後恢復 1 分，上限 100。信用分透過 `PlayerProfileProvider` 保存；若使用 memory provider，ERPS 重啟後重設。

## 10. Game server 註冊與容量

Game server 透過 gRPC 主動註冊，提供 server ID、generation、endpoint、region、支援模式、總容量、各模式成本、1～100 的 instance 上限及目前 instances。ERPS 設定檔依 server 或 server class 定義可信上限；server 回報不得超過上限。

Game server 使用雙向 control stream 定期 heartbeat，建議預設每 2 秒一次。連續 3 次未收到後停止新 placement；約 10 秒後視為失聯，釋放尚未確認的 reservation。Generation 防止舊連線或延遲 heartbeat 覆蓋新程序狀態。

不同模式消耗設定的容量權重；placement 必須同時滿足 region、supported mode、容量單位與 max instances。通過硬條件後，評分綜合容量碎片、負載與近期 launch failure。優先在相容 region 內配置；只有所有 party 成員明確允許的 region 才可使用。

Ready check 期間只做 soft capacity feasibility check，不扣容量。全員同意後才建立原子 reservation。若暫時沒有容量，proposal 進入有期限的 placement waiting；逾時後所有玩家無懲罰回 queue並保留原等待時間。

配置狀態：

```text
Reserved -> Accepted -> Ready -> Running -> Finished
    |           |
    ` reject    ` ready timeout
          -> Released / retry another server
```

`Accepted` 只代表 game server 收到 launch；只有 instance 回報 `Ready` 並提供 endpoint／connection token 後，ERPS 才通知 clients。Launch 失敗時保留玩家等待時間並重試其他 server。Running server 失聯時標記 `ServerLost` 並通知 client；第一版不遷移執行中的遊戲。

## 11. gRPC contract

`MatchmakingService` 提供：

- `OpenSession`（SDK 對外方法可命名為 `connect()`，proto RPC 避免與 tonic constructor 衝突）
- `CreateParty`、`CreateInvite`、`JoinParty`、`LeaveParty`、`KickMember`、`RenameParty`
- `Enqueue`、`CancelQueue`
- `AcceptMatch`、`RejectMatch`
- `GetState`
- `WatchEvents`
- 經授權的賽後結果回報入口或對應 internal service

`GameServerService` 提供：

- `Register`
- 雙向 `ControlStream`
- `ReconcileInstances`

所有 mutation 帶 `request_id` 並在 session 生命週期內去重。Proto envelope 帶 API major／minor；不相容 major 拒絕，minor 透過 capability negotiation。Protobuf tag 只新增，不重用。

Command queue、client event queue、server control queue 全部有界。非關鍵狀態可合併成最新值；proposal、match、launch、cancel、信用處分等關鍵事件不可靜默丟棄。慢 client 的 stream 長期滿載時中止連線；SDK 重連後用 `GetState` 對帳。

Transport 同時支援 TLS 與明文。正式環境預設 TLS；明文只允許 loopback 或明確 development 設定。身分驗證使用可注入 token validator；production 不接受 client 自稱的 player ID。

## 12. Rust 與 C SDK

`erps-client` 提供型別安全的 async gRPC API與事件 stream，涵蓋 session、party、queue、ready check、match、server lost、重連與 state reconciliation。

`erps-client-ffi` 內部擁有 Rust async runtime 與網路背景執行緒。背景執行緒只寫入 bounded event queue，絕不呼叫使用者 callback。遊戲主迴圈以 `erps_client_poll()` 取得事件。

C event 採 library-owned opaque handle，加 accessor 與唯一 release API，避免 allocator 混用。`poll` 只允許一個 consumer thread；命令 API thread-safe。所有 exported functions 捕捉 panic並轉成明確 error code，panic 不得穿越 C ABI。

第一版發行 Windows x64 與 Linux x86_64 動態 library、header、範例與 C smoke test。macOS 與 static library 打包不在第一版範圍。

## 13. 斷線、錯誤與關閉

Client 斷線後保留預設 30 秒 grace period。期間完成的 proposal／match 仍保留為可對帳狀態；超時後未匹配 ticket 才取消。Party leader 斷線不立即解散 party。

ERPS 是單一 process authority。重啟後 session、party、queue、proposal 與 reservation 消失；client 重新連線／排隊，game server 重新註冊／reconcile。Rating 與信用分是否保留取決於注入的 profile provider。

ERPS 提供 gRPC health check 與 graceful shutdown。關閉時停止接受新 enqueue，在限定時間內將正在 commit 的 proposal／reservation推進到安全邊界，之後終止 streams。不得發布半完成 match。

## 14. 可觀測性與管理

核心輸出結構化 logs、metrics 與可選 OpenTelemetry tracing。Metrics 至少涵蓋：

- 每模式／region queue depth。
- Command、proposal、match throughput。
- Queue wait、candidate compute、commit、ready check、placement 與 launch latency 分位數。
- Elo quality、party structure difference。
- Server capacity／instances／reservation／launch failure。
- Credit penalties、timeouts、reconnects 與 bounded queue high-watermark。

第一版提供唯讀 gRPC admin API 查詢 queue、server capacity、match rate、reservation 與健康狀態。不提供管理 UI、人工改 rating 或強制匹配。

## 15. 測試策略

### 15.1 單元與 property tests

- Elo 預期勝率、K-factor、多人名次更新與單場變動上限。
- 階梯式搜尋範圍與 party effective rating。
- 5v5 恰好 5+5、party 不拆與 bounded search tie-break。
- 八人模式恰好 8 人、party 不超過 4、輸出八個單人 team。
- Ready-check deadline、冪等回覆、信用處分與無責任 infra failure。
- 任意 server 能力與 match 序列都不超容量或 instances。

### 15.2 ECS 整合測試

- 相同 seed／命令序列產生相同 proposal、match 與 placement。
- 相鄰 shard halo 不重複 claim。
- Enqueue、cancel、party revision、ready response 與 commit 競爭。
- Reject／timeout 後單人與 party 的不同恢復規則。
- Server reject、ready timeout、heartbeat loss、reservation release 與 retry。
- 無容量時不發送假成功。

### 15.3 gRPC 與 SDK 測試

- Rust client 完整流程與 state reconciliation。
- 真正編譯／連結 C smoke test，驗證 create、party、enqueue、poll、accept、event release、shutdown。
- Request 去重、舊 proposal 回覆、斷線重連與慢 consumer 背壓。
- Game server register、heartbeat、launch accept／ready／reject／finish。
- TLS、development plaintext guard 與 token validator。

### 15.4 `erps-load-test`

預設模擬 100,000 玩家，混合三種模式、合法 party 大小、多 region 與容量／成本／instance 上限不同的 game servers。持續產生 enqueue、cancel、accept、reject、timeout、server 上下線與 match completion。

參數包含 seed、執行時間、worker threads、玩家數、模式比例、party size distribution、ready response distribution 與 server fleet。輸出 command／match throughput、queue wait p50／p95／p99、candidate／commit latency、ready success、Elo quality、容量利用率、reservation retry、未匹配數與記憶體高水位。

預設 in-process 模式測核心上限；`--grpc` 模式測完整 RPC 與 SDK 路徑。硬性 invariant：

- 零重複玩家。
- 零 party 拆分。
- 零 roster 人數／team 數錯誤。
- 零 server 超配。
- 零 committed match 遺失。
- 無責任玩家不被扣信用分。
- 相同 seed 結果一致。

效能結果必須記錄硬體、worker 數與設定，支援與保存的 baseline 比較；第一版不設定與硬體無關的絕對 p99 門檻。

## 16. 非目標

第一版不包含：

- Queue、party、proposal 或 reservation 的資料庫恢復。
- 多 ERPS 節點 active-active／distributed queue。
- Running match migration。
- 管理 Web UI。
- 玩家合作／作弊偵測。
- Glicko、TrueSkill。
- 自動建立或縮減雲端 game server。
- macOS C SDK 與 static library 打包。

## 17. 驗收條件

- 三種模式均依 party 與 team 規則完成匹配。
- 搜尋範圍隨等待時間放寬，且不超設定上限。
- 全員同意後才進 placement；拒絕／逾時、信用分與重新排隊規則正確。
- 異質 game server 不超容量權重或 1～100 instance 上限。
- Client 只在 game instance `Ready` 後收到 endpoint 與 connection token。
- Rust SDK 與 C poll SDK 完成真實 gRPC 流程。
- 100,000 玩家負載測試通過所有硬性 invariants，並產生可重現效能報告。
