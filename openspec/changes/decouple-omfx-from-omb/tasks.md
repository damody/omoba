## 1. Establish Runtime Boundary

- [x] 1.1 在 `omoba-core` 新增 mandatory `src/runtime/` module，加入 `specs`、`omoba-sim`、`omb-script-abi`、`omoba-template-ids` 與 script/runtime 所需 dependencies。
- [x] 1.2 在 `omoba-core::runtime` 定義 runtime-owned event/sink abstraction，讓 deterministic tick/script/game processing 不再直接依賴 `omb::transport::OutboundMsg`。
- [x] 1.3 在 `omb` 增加 runtime event 到 `OutboundMsg` / backend broadcast 的 adapter，保持現有玩家可見 events 行為。
- [x] 1.4 將 lockstep `PlayerInput` / `PlayerInputEnum` boundary 收斂到 `omoba-core` shared protocol type，讓 runtime 不依賴 `omb::transport::kcp_transport::game_proto`。
- [x] 1.5 確認 wasm/tools 受影響處；若 target 不能執行 native runtime implementation，使用 target-specific `cfg` 隔離 implementation，而不是把 `runtime` 做成 optional feature。

## 2. Move Leaf Runtime Modules

- [x] 2.1 將 `omb/src/util/geometry.rs` 或等效 low-level geometry helpers 搬到 `omoba-core::runtime`，並更新 `omb` imports。
- [x] 2.2 將 `omb/src/ability_runtime` 搬到 `omoba-core::runtime::ability_runtime`，修正對 `omoba_core::ability_meta` 的 self-import。
- [x] 2.3 將 pure ECS foundation 與 components/resources 搬到 `omoba-core::runtime::comp`，先排除 transport-only modules（例如 `mqtt_handler`）與 backend-only broadcast builders。
- [x] 2.4 將 pure item data / registry runtime access 搬到 `omoba-core::runtime`，把 filesystem/config loading adapter 留在 `omb`。
- [x] 2.5 將 pure scene/campaign data structs 搬到 `omoba-core::runtime`，把 `game.toml`、Lua/generated story path discovery 與 file IO 留在 `omb`。

## 3. Split Runtime Processing

- [x] 3.1 拆分 `GameProcessor`：將 tower place/sell/upgrade、ability cast/upgrade、item use、move drain 與 pure outcome mutation 搬到 `omoba-core::runtime`。
- [x] 3.2 將 `GameProcessor` 中 legacy/typed broadcast payload construction 留在 `omb` adapter，runtime 只 emit runtime events 或 mutate ECS。
- [ ] 3.3 將 tick systems 搬到 `omoba-core::runtime::tick`，並用 runtime event sink 取代 direct `OutboundMsg` channels。
- [ ] 3.4 將 phase3 dispatcher build 搬到 `omoba-core::runtime`，暴露 `build_phase3_dispatcher` 或等效 entrypoint。
- [ ] 3.5 將 native script runtime（`ScriptRegistry`、`ScriptEventQueue`、`ScriptUnitTag`、`run_script_dispatch`、`WorldAdapter`）搬到 `omoba-core::runtime`，DLL path discovery/loading orchestration 留在 `omb`。
- [ ] 3.6 將 pure world initialization 搬到 `omoba-core::runtime`，讓 `omb` 提供已載入的 config/scene/items/scripts，避免 runtime 讀 backend-specific paths。

## 4. Snapshot And Input Boundary

- [ ] 4.1 將 `SimWorldSnapshot`、`EntityRenderData`、`HeroStatsExt`、tower/ability snapshot DTO、render-only FX cues 與 `extract_snapshot` 從 `omfx/game/src/sim_runner.rs` 搬到 `omoba-core::runtime`。
- [ ] 4.2 調整 `extract_snapshot` 使用 `omoba-core::runtime` ECS resources/types，並確認只有 render queues 使用 `std::mem::take` drain。
- [x] 4.3 移除 `omfx` 端 `convert_player_input` prost roundtrip，讓 `TickBatchInput.input` 直接使用 `omoba-core` shared protocol type。
- [ ] 4.4 更新 native `omfx` sim runner，讓 init、tick execution、outcome processing、script dispatch 與 snapshot publish 都呼叫 `omoba-core::runtime` entrypoints。

## 5. Frontend/Backend Decoupling

- [x] 5.1 從 `omfx/game/Cargo.toml` 移除 `omobab` dependency，改依賴 mandatory runtime-capable `omoba-core`，並確認 `omfx/game/src/**/*.rs` 沒有 `omobab::` references。
- [x] 5.2 從 `omfx/game/src/native.rs` 移除 `BackendGuard`、`spawn_backend`、`create_job_and_attach`、hard-coded `omb` path discovery 與 frontend-owned `cargo run` fallback。
- [x] 5.3 更新 `run.bat`、`run_smoke.bat` 與其他 debug launchers，由 launcher 在啟動 `executor.exe` 前啟動 backend，並在 frontend 結束後清理它啟動的 backend process。
- [x] 5.4 更新 `run_stress.bat` 直接啟動 release `omobab.exe`，移除為 frontend hard-coded spawn path staging `omb/target/debug/omobab.exe` 的流程。

## 6. Verification

- [x] 6.1 執行 grep guard：`omfx/game/Cargo.toml` 不含 `omobab =`，且 `omfx/game/src/**/*.rs` 不含 `omobab::`、`spawn_backend`、`target/debug/omobab.exe`。
- [x] 6.2 執行 grep guard：repo 內沒有新增 `omoba-runtime` crate 或 `omoba-runtime` dependency。
- [ ] 6.3 執行 `cargo test --manifest-path D:/omoba/omb/Cargo.toml -p omobab --lib`。
- [ ] 6.4 執行 `cargo test --manifest-path D:/omoba/omoba-sim/Cargo.toml --no-default-features`。
- [ ] 6.5 執行 `cargo build --manifest-path D:/omoba/omfx/Cargo.toml -p executor`，確認 native frontend 不需編譯 `D:/omoba/omb` crate。
- [ ] 6.6 執行 smoke launcher 驗證 launcher-owned backend lifecycle 與 snapshot-driven TD UI 行為。
- [ ] 6.7 修改 code 後執行 `graphify update .` 更新 knowledge graph。
