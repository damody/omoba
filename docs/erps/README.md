# ERPS 操作手冊

ERPS 是獨立 process；matching authority 不嵌入 `omb`。Rust toolchain 固定為根目錄 `rust-toolchain.toml` 的 1.95.0，ECS 使用 `D:/code/omoba/specs`。

## Server

```powershell
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-server -- erps.toml
```

`ERPS_LISTEN` 預設 `127.0.0.1:50051`。Production 必須同時設定 `tls_certificate_path` 與 `tls_private_key_path`。只有 loopback 且 `allow_development_plaintext=true` 時能使用明文。queue/event/control capacities、Elo expansion、15 秒 ready timeout、credit penalties、heartbeat、placement timeout 與 server classes 都能由 TOML 設定。

## omb game server

以 `--features erps-game-server` 建置 `omb`，建立 `ErpsGameServerClient` 與實作 `GameInstanceAdapter`。Adapter 會 register generation、回報 region/capacity/mode cost/max instances、維持 heartbeat/control stream、重連 reconcile，並將 `LaunchMatch` 轉成遊戲 instance。`max_instances` 必須在 1～100；每台 server 的 capacity 與 mode cost 可不同。未開 feature 時既有 KCP gameplay 路徑不變。

## Rust client

只依賴 `erps-client`，使用 `Client::connect`、party commands、`enqueue`、`events`、`accept_match`。斷線後呼叫 `reconnect`；SDK 先以 `GetState` 對帳。完整範例在 `erps/client/examples/party_to_match.rs`。

## C client

公開檔為 `erps/client-ffi/include/erps_client.h`。所有 handle 都是不透明指標；`ErpsEvent` 只能由 `erps_event_release` 釋放，字串只在 event 存活期間有效。單一 consumer 呼叫 `erps_client_poll`；`ERPS_NO_EVENT` 不是錯誤。Windows x64 產物為 `erps_client_ffi.dll`/`.dll.lib`，Linux x86_64 為 `liberps_client_ffi.so`。

## Load test

```powershell
cargo run --manifest-path erps/Cargo.toml -p erps --bin erps-load-test --release -- --players 100000 --seed 1163022419 --workers 1
```

任何 roster、party、capacity 或 committed-player invariant 失敗都以非零結束，不會輸出 PASS。報告含 seed、workers、OS/arch、throughput、match/unmatched 與 logical digest。固定驗證結果見 `load-test.md`。
