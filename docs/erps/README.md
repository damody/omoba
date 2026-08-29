# ERPS 操作手冊

ERPS 是獨立 process；matching authority 不嵌入 `omb`。Rust toolchain 固定為根目錄 `rust-toolchain.toml` 的 1.95.0，ECS 使用 `D:/code/omoba/specs`。

## Server

Matchmaking、GameServer 與 Admin `Snapshot` 的 request envelope 都必須帶 `ApiVersion`；不相容 major 會在讀寫 authority state 前被拒絕。
Admin snapshot 同時公開 queue/server ledger、proposal/reservation 數、ready/credit/launch/reconnect/invariant counters、三種 bounded queue high-watermark，以及 queue wait、candidate、commit、ready check、placement、launch 的 bounded p50/p95/p99 latency。

```powershell
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-server -- erps/config/development.toml
```

`ERPS_LISTEN` 預設 `127.0.0.1:50051`。Production 必須同時設定 `tls_certificate_path`、`tls_private_key_path`、`ERPS_AUTH_TOKEN_MAP`（token → player UUID JSON）與 `ERPS_SERVER_AUTH_TOKEN_MAP`（token → server UUID JSON）；缺少任一 trusted validator 時 TLS 啟動會拒絕。Production game server 還必須回報已在 TOML `server_classes` 定義的 class，容量、1～100 instance 上限與 mode costs 皆以 trusted policy 為準，不能由 server 自行提高。只有 loopback 且 `allow_development_plaintext=true` 時能使用明文。

可部署的無秘密範本位於 `erps/config/production.example.toml`。將它複製到受保護的部署目錄、替換 certificate/key 路徑後啟動：

```powershell
$env:ERPS_LISTEN = "0.0.0.0:50051"
$env:ERPS_AUTH_TOKEN_MAP = Get-Content -Raw -LiteralPath "D:\secure\erps\player-tokens.json"
$env:ERPS_SERVER_AUTH_TOKEN_MAP = Get-Content -Raw -LiteralPath "D:\secure\erps\server-tokens.json"
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-server --release -- D:\secure\erps\production.toml
```

兩個環境變數的值都是 JSON object 內容（不是檔案路徑），key 為不可猜 bearer token，value 分別為 canonical player/server UUID，例如 `{ "replace-with-random-token": "00000000-0000-0000-0000-000000000001" }`。上面的 PowerShell 命令從受 ACL 保護的檔案讀入內容。範例 token 不能用於正式環境；token map、private key 與實際憑證不得放入 repository。正式設定範本預先提供 small／standard／high-capacity 三種 trusted ceiling，可依部署硬體調低或建立新的受信 class。

`erps-server --help` 會列出部署參數。Binary 最多接受一個 config path，並將兩個 auth token map 視為原子設定；只設定其中一個或傳入多餘參數會在綁定 socket 前以非零狀態拒絕，避免看似啟動卻使用錯誤 identity policy。

Matching 設定另包含 `deterministic_seed`、`party_size_rating_adjustment`、`party_spread_rating_adjustment`、`max_party_rating_spread` 與 `credit_suspension_base_seconds`。Domain ID 與排序使用 seed／logical command time；session、invite 與 connection token 仍使用不可猜的安全隨機值。低信用停權時間依近期違規次數遞增，期限結束後可重新參賽並依完成場次逐步恢復信用。

## omb game server

相同 generation 的 Register 是冪等重連：ERPS 保留 authoritative reservation ledger，再以 `ReconcileInstances` 對帳。`Ready`／`Running` 的 `InstanceReport` 必須保留 endpoint 與 connection token，讓斷線期間遺失的 ready event 可以重播。Match completion 使用 bounded pending outbox 並等待 ERPS `match_result_ack`；未 ACK 的結果會跨 stream 重連重送，server 以 bounded idempotency cache 避免重複套用 Elo 或信用恢復。

以 `--features erps-game-server` 建置 `omb`，建立 `ErpsGameServerClient` 與實作 `GameInstanceAdapter`。正式環境的 `ErpsServerConfig` 使用 `https://` endpoint 並設定 `tls_domain`；系統信任根預設啟用，私有 PKI 可再傳入 `tls_ca_pem`。loopback development 才將 `tls_domain` 設為 `None`。Adapter 會 register generation、回報 region/capacity/mode cost/max instances、維持 authenticated heartbeat/control stream、重連 reconcile，並將 `LaunchMatch` 轉成遊戲 instance。遊戲結束後由 `poll_completed_result` 回報完整名次，ERPS 會更新 Elo、信用與 reservation。`max_instances` 必須在 1～100；每台 server 的 capacity 與 mode cost 可不同。未開 feature 時既有 KCP gameplay 路徑不變。

## Rust client

`GetState` 不依賴 Party 存在：回傳玩家自己的三模式 Elo、信用分與仍有效的信用停權截止時間。若玩家正等待配對或 ready check，也會恢復 queue mode、允許 regions、proposal ID 與 authoritative deadline，讓重連後 UI 能正確重建排隊與倒數。`Event::ProposalCancelled` 明確區分 `rejected`、`timed_out`、`party_member_failed`、`other_player_failed` 與 `infrastructure_failure`，並攜帶更新後信用與 eligibility。MatchResult commit 後會主動發布 `Idle` Party 更新及新 Elo/信用，不要求 UI 輪詢猜測結算是否完成。

只依賴 `erps-client`，使用 `Client::connect`、party commands、`enqueue`、`events`、`accept_match`。正式憑證使用 `ConnectOptions::tls`（系統信任根），私有 CA 使用 `ConnectOptions::tls_with_ca`；明文 constructor 僅供 loopback development。`Event::Proposal` 提供 authoritative Unix-millisecond deadline，`Party.members` 保留相容的玩家 ID 列表，`Party.player_details` 提供每位玩家的 ID、顯示 Elo 與信用分；ready timeout／reject 後 server 會發布更新後的 party state 與信用，讓 UI 停止倒數並顯示 `Queued`／`NotReady`。`Event::Matched` 包含完整 teams／roster、endpoint 與 connection token。event stream 使用 bounded buffering，斷線時以 100ms～5s exponential backoff 自動重連並送出一個 `Event::State` 對帳結果；也可顯式呼叫 `reconnect`。完整範例在 `erps/client/examples/party_to_match.rs`：先啟動 `erps-test-fixture`（預設 `127.0.0.1:50059`），再執行 `cargo run --manifest-path erps/Cargo.toml -p erps-client --example party_to_match`；範例會自行建立兩名 1v1 client、雙方接受並驗證相同 match，且不輸出 connection token。可用 `ERPS_ENDPOINT` 覆寫位址。

## C client

C 對應使用 `ERPS_EVENT_PROPOSAL_CANCELLED`、`erps_event_reason`、`erps_event_player_credit`、`erps_event_player_eligible`、`erps_event_credit_suspended_until_ms` 與 `erps_event_player_rating_for_mode`。STATE event 即使玩家尚未建立 Party也包含自己的 profile；`erps_event_queue_mode`、`erps_event_allowed_region_count`、`erps_event_allowed_region` 與 `erps_event_deadline_ms` 可在重連後還原等待中的配對畫面。

公開檔為 `erps/client-ffi/include/erps_client.h`；Windows/Linux 的完整 build、link、runtime loader 與發行檔案命令在 `erps/client-ffi/README.md`。`erps_client_create_tls` 使用系統信任根，`erps_client_create_tls_with_ca` 額外接受 PEM 私有 CA；其餘 API 包含 shutdown、party create/invite/join/leave/kick/rename、enqueue/cancel、accept/reject、`get_state` reconciliation 與 event poll。以公開 `ERPS_EVENT_*` 常數判斷事件，不應硬編數字；proposal 倒數使用 `erps_event_deadline_ms`，party/state UI 可讀 name、leader、state 與各成員 ID/Elo/credit。match roster 以 `erps_event_team_count`、`erps_event_team_player_count`、`erps_event_team_player_id` 讀取。所有 handle 都是不透明指標；`ErpsEvent` 只能由 `erps_event_release` 釋放，字串與 roster pointers 只在 event 存活期間有效。第一個呼叫 `erps_client_poll` 的 thread 成為唯一 poll consumer，其他 thread 會收到 `ERPS_THREAD_MISUSE`；命令可由多執行緒呼叫並由 SDK 安全序列化。`ERPS_NO_EVENT` 不是錯誤；若 bounded local queue 滿載或 stream 中止，排空後 poll 會回報 `ERPS_RUNTIME_ERROR`，失敗細節可由 `erps_client_last_error` 讀取，client 應呼叫 `erps_client_get_state` 對帳。

## Load test

```powershell
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-load-test --release -- --players 100000 --seed 1163022419 --workers 1
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-load-test --release -- --grpc --players 1000 --seed 1163022419 --workers 8
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-load-test --release -- --players 1000 --seed 42 --workers 8 --output erps/target/baseline.json
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-load-test --release -- --players 1000 --seed 42 --workers 8 --baseline erps/target/baseline.json
```

任何 roster、party、capacity 或 committed-player invariant 失敗都以非零結束，不會輸出 PASS。報告含 CPU、logical CPUs、seed、workers、完整 scenario settings、throughput、latency、match/unmatched 與 logical digest。`--baseline` 只有在環境與設定一致時標記 comparable；`--grpc` 會真的以相同 seed 建立三種模式、完整 party roster 與每位 client，而不是只附加 2 人 smoke。
