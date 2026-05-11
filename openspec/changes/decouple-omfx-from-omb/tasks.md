## 1. Expand Mandatory `omoba-core::runtime`

- [ ] 1.1 在 `omoba-core` 新增 mandatory `src/runtime/` module，加入 `specs`、`omoba-sim`、`omb-script-abi`、`omoba-template-ids` 與 script/runtime 所需 dependencies。
- [ ] 1.2 建立 `omoba-core/src/runtime/` module，將 `omb` 中 deterministic gameplay 的 `comp`、resources、outcome queues、ability runtime 與 shared helpers 搬入此 module。
- [ ] 1.3 將 world initialization、phase3 dispatcher build、input queue application、`GameProcessor` deterministic drains、`process_outcomes` 與 script dispatch wrapper 暴露為 `omoba-core::runtime` entrypoints。
- [ ] 1.4 更新 `omb` 依賴 mandatory runtime-capable `omoba-core`，並用 `pub use omoba_core::runtime::*` 或直接 imports 讓 backend 既有 tests 與 binary 能編譯。
- [ ] 1.5 檢查 wasm/tools 受影響處；若 target 不能執行 native runtime implementation，使用 target-specific `cfg` 隔離 implementation，而不是把 `runtime` 做成 optional feature。

## 2. Snapshot And Input Boundary

- [ ] 2.1 將 `SimWorldSnapshot`、`EntityRenderData`、`HeroStatsExt`、tower/ability snapshot DTO、render-only FX cues 與 `extract_snapshot` 從 `omfx/game/src/sim_runner.rs` 搬到 `omoba-core::runtime`。
- [ ] 2.2 調整 `extract_snapshot` 使用 `omoba-core::runtime` ECS resources/types，並確認只有 render queues 使用 `std::mem::take` drain。
- [ ] 2.3 將 lockstep `PlayerInput` boundary 收斂到 `omoba-core` shared protocol type，移除 `omfx` 端 `convert_player_input` prost roundtrip。
- [ ] 2.4 更新 native `omfx` sim runner，讓 init、tick execution、outcome processing、script dispatch 與 snapshot publish 都呼叫 `omoba-core::runtime` entrypoints。

## 3. Frontend/Backend Decoupling

- [ ] 3.1 從 `omfx/game/Cargo.toml` 移除 `omobab` dependency，改依賴 mandatory runtime-capable `omoba-core`，並確認 `omfx/game/src/**/*.rs` 沒有 `omobab::` references。
- [ ] 3.2 從 `omfx/game/src/native.rs` 移除 `BackendGuard`、`spawn_backend`、`create_job_and_attach`、hard-coded `omb` path discovery 與 frontend-owned `cargo run` fallback。
- [ ] 3.3 更新 `run.bat`、`run_smoke.bat` 與其他 debug launchers，由 launcher 在啟動 `executor.exe` 前啟動 backend，並在 frontend 結束後清理它啟動的 backend process。
- [ ] 3.4 更新 `run_stress.bat` 直接啟動 release `omobab.exe`，移除為 frontend hard-coded spawn path staging `omb/target/debug/omobab.exe` 的流程。

## 4. Verification

- [ ] 4.1 執行 grep guard：`omfx/game/Cargo.toml` 不含 `omobab =`，且 `omfx/game/src/**/*.rs` 不含 `omobab::`、`spawn_backend`、`target/debug/omobab.exe`。
- [ ] 4.2 執行 grep guard：repo 內沒有新增 `omoba-runtime` crate 或 `omoba-runtime` dependency。
- [ ] 4.3 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`。
- [ ] 4.4 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`。
- [ ] 4.5 執行 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor`，確認 native frontend 不需編譯 `D:/omoba/omb` crate。
- [ ] 4.6 執行 smoke launcher 驗證 launcher-owned backend lifecycle 與 snapshot-driven TD UI 行為。
- [ ] 4.7 修改 code 後執行 `graphify update .` 更新 knowledge graph。
