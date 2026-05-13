## 1. 盤點共用 runtime 邊界

- [x] 1.1 檢查 `omb/Cargo.toml`、`omfx/game/Cargo.toml` 與相關 target dependency，確認 gameplay dependency direction 維持 `omb -> omoba-core` 與 `omfx -> omoba-core`，且沒有 `omfx -> omb`。
- [x] 1.2 搜尋 `omb/src/**/*.rs` 與 `omfx/game/src/**/*.rs` 的 gameplay tick、input apply、outcome processing、snapshot extraction、script dispatch imports，列出仍未直接使用 `omoba-core` 的路徑。
- [x] 1.3 搜尋 `adapter`、`bridge`、`shim`、`convert_*`、`encoded_len()`、`prost::Message::decode` 等 pattern，分類為必要邊界或可移除轉接層。

## 2. 收斂到 omoba-core API

- [x] 2.1 將可直接替代的 `omb` gameplay call sites 改成使用 `omoba-core::runtime`、`omoba-core::comp` 或 shared protocol 型別。
- [x] 2.2 將可直接替代的 native `omfx` gameplay/sim runner call sites 改成使用 `omoba-core` public API/type，避免 backend-specific wrapper。
- [x] 2.3 移除同 process 內 duplicate gameplay type 的 prost roundtrip 或 identity conversion，改用共用來源型別。
- [x] 2.4 保留並標註仍必要的 transport、render projection、thread transfer、script ABI 或 UI mirror 邊界，不把 gameplay rule 複製到這些邊界。

## 3. 清理與驗證

- [x] 3.1 刪除已無 call site 的 adapter/bridge/shim code 與不再需要的 imports/dependencies。
- [x] 3.2 執行搜尋驗證，確認 `omfx/game/src/**/*.rs` 沒有 `omobab::`，且不存在只為 duplicate gameplay type conversion 的 prost roundtrip。
- [x] 3.3 執行 `cargo check --manifest-path omoba-core/Cargo.toml`。
- [x] 3.4 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib` 或記錄阻塞原因。
- [x] 3.5 執行 `cargo build --manifest-path omfx/Cargo.toml -p executor` 與 sim runner smoke test，確認 native frontend 仍可 build。
- [x] 3.6 更新 `tasks.md` checkbox，記錄完成項目與任何保留邊界的理由。

## 保留邊界

- `omb/src/transport/grpc_transport.rs` 保留本地 `game_proto` include，因為它屬於 `grpc` transport feature 的 wire protocol 邊界，不是 native `omfx`/`omb` 同 process gameplay type bridge。
- `omfx/game/src/lockstep_client.rs` 的 `LockstepTickInput` 與 `sim_runner::TickBatchInput` 保留 edge metadata 欄位，職責是跨 thread diagnostic/latency transfer；其中 gameplay `PlayerInput` 型別已來自 `omoba_core::game_proto`。
- `omfx` snapshot/UI mirror 保留 render projection/cache 職責，不重新實作 gameplay rules。
