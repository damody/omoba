# Capability-to-test evidence

| Capability | 自動化 evidence |
|---|---|
| matchmaking core | `mode_properties.rs`, `deterministic_replay.rs`, `claim_races.rs`, rating/placement property tests |
| party/ready/credit | `party_lifecycle.rs`, `ready_check.rs`, party/credit unit tests |
| gRPC services | `grpc_e2e.rs`, gRPC API/token/config unit tests |
| game-server placement | `placement_properties.rs`, server/placement unit tests, `omb/tests/erps_adapter.rs` |
| Rust/C SDK | Rust client unit tests, `grpc_e2e.rs`, C ABI tests, clang compile/link/run smoke |
| load validation | load-test unit tests and fixed 100,000-player report in `load-test.md` |

所有 generated/network 邊界只傳 stable opaque IDs；Specs `Entity` 不會出現在 protobuf 或 SDK。敏感欄位 redaction、queue high-watermark 與 observability independence 由 metrics tests 驗證。

## Scenario traceability

- `erps-party-ready-credit`：CJK／特殊符號名稱、stale revision、全員接受、stale accept、timeout、launch failure、party NotReady、30 秒 grace，分別由 party unit tests、`ready_check.rs`、credit/proposal/session tests、`grpc_e2e.rs` 覆蓋。
- `erps-matchmaking-core`：tick command ordering、halo claim、worker determinism、5v5 party、FFA individual teams、等待擴張、party scoring、FFA tie，由 command/bucket/claim/rating tests、`deterministic_replay.rs`、`mode_properties.rs` 覆蓋。
- `erps-load-validation`：隨機 roster、cancel race、C ready、100k、gRPC、超配 fail-fast、環境 metadata，由 property/claim tests、C `e2e.c`、`--grpc` runner 與固定 load report 覆蓋。
- `erps-grpc-services`：omb register、idempotent enqueue、reconcile、major reject、token identity、slow consumer、drain，由 `omb/tests/erps_adapter.rs`、gRPC unit/E2E tests與 bounded actor/event implementation 覆蓋。
- `erps-game-server-placement`：stale generation、lost health、instance ceiling、ready 前零 reservation、Accepted 不可連、reject release，由 server/placement/proposal unit tests與 `placement_properties.rs` 覆蓋。
- `erps-client-sdks`：Rust full flow、C poll、single release、concurrent bounded commands、真實 C compiler link，由 `grpc_e2e.rs`、FFI unit/ABI tests、Windows clang 與 Linux gcc smoke 覆蓋。
