## Context

`omfx/game` 目前在 native target 直接依賴 `omobab = { path = "../../omb" }`，並在 `sim_runner.rs` 內呼叫 `omobab::state::initialization::create_world_for_scene`、`omobab::state::system_dispatcher::build_phase3_dispatcher`、`omobab::comp::GameProcessor::*`、`omobab::scripting::run_script_dispatch` 與多個 `omobab::comp` / `omobab::ability_runtime` types。`native.rs` 還會在前端初始化時尋找 `omb/game.toml`、啟動 `omb/target/debug/omobab.exe`，並在缺少 exe 時 fallback 到 `cargo run`。

這些 coupling 讓 `omfx` 不只是 protocol client，而是把 backend crate 當成 frontend runtime dependency。實際需要共享的是 deterministic simulation replica、script DLL loading、lockstep input application 與 render-facing snapshot projection；這些是遊戲 runtime contract，不應該掛在 backend app crate `omobab` 底下。

我們不新增 `omoba-runtime` crate。現有 crate 已經偏多，且 `omoba-core` 本來就是 shared protocol / transport client / frontend schema crate，因此這次將 shared runtime 收斂到 `omoba-core::runtime`。`runtime` 是 `omoba-core` 的強制核心模組，不是 optional capability；如有 target 不支援 runtime execution，應用 `cfg` 隔離 target-specific code，而不是把 runtime contract 變成可選項。

## Goals / Non-Goals

**Goals:**

- `omfx` native build 不再依賴 `omobab` crate，且 `omfx/game/src` 不再 import `omobab::*`。
- 前後端共用 lockstep simulation 所需 runtime，避免 omfx 複製 backend 邏輯，也避免透過 backend crate 取得 ECS components、systems 或 snapshot projection。
- 不新增 shared runtime crate；改由 mandatory `omoba-core::runtime` 承接 deterministic ECS runtime、snapshot extraction 與 lockstep replica boundary。
- `omfx` executable 本身不再 hard-code `omb/` repo layout、`target/debug/omobab.exe` 或 `cargo run`；dev launchers 負責需要的 backend process orchestration。
- 保留既有 lockstep behavior、snapshot data shape、TD UI、VFX、script DLL loading 與 smoke/stress launcher 的開發體驗。

**Non-Goals:**

- 不重新設計 KCP protocol、lockstep cadence 或 gameplay rules。
- 不把 `omoba-sim` 併入 `omoba-core`；`omoba-sim` 保持 deterministic primitives 小核心。
- 不把所有 backend transport/server code 移到 `omoba-core`；transport listener、session management、server lifecycle 仍屬於 `omb`。
- 不要求 wasm executor 支援本地 deterministic simulation replica；本次 focus 是 native `omfx` 與 `omb` 的依賴邊界。

## Decisions

1. 擴充 mandatory `omoba-core::runtime`，不新增 `omoba-runtime` crate。

   Rationale: 目前 crate 數量已多，新增 runtime crate 會再增加一個需要維護的邊界。`omoba-core` 已承接 shared protocol、transport client 與 frontend shared schema，讓 runtime boundary 也放在這裡，可以讓 `omb` 與 native `omfx` 透過同一個 shared crate 溝通，同時避免 `omfx -> omobab`。因為 runtime 是前後端一致性的核心 contract，規格上不應是 optional feature。

   Alternative considered: 新增 `omoba-runtime`。拒絕原因是它技術上乾淨，但會增加 crate 數量；本次重構目標包含收斂相近 library。Alternative considered: 把 runtime 放進 `omoba-sim`。拒絕原因是 `omoba-sim` 應保持 fixed-point / RNG / hash primitives，不應拉入 `specs`、script host、snapshot DTO 或 protocol dependencies。

2. `omoba-core::runtime` 是強制 contract，但 target-specific execution 可用 `cfg` 隔離。

   `runtime` module SHALL 是 `omoba-core` 的一部分，並提供固定的 shared runtime API。`specs`、`omoba-sim`、`omb-script-abi`、`omoba-template-ids` 與 deterministic ECS runtime 所需 dependencies SHALL 是 `omoba-core` runtime contract 的正常依賴，而不是以 optional feature 表示能力是否存在。若某些 targets（例如 wasm）暫時不能執行 native runtime，相關 implementation MAY 以 `#[cfg(...)]` 隔離，但 `omoba-core` 的 public runtime boundary 不應因此變成可選 contract。`omoba-sim` 繼續是底層 deterministic primitives crate。

   Alternative considered: 用 `runtime` optional feature 保持輕量。拒絕原因是這會讓前後端 shared simulation contract 看起來可有可無，且容易重新出現 `omfx` 與 `omb` 對 runtime availability 的分歧。

3. `omoba-core::runtime` 擁有 deterministic ECS runtime 與 snapshot projection；`omb` 改成 runtime consumer。

   `omoba-core::runtime` SHALL expose world initialization、phase3 dispatcher build、input queue application、outcome processing、script dispatch wrapper、render-only cue queues、`SimWorldSnapshot` 與 `extract_snapshot`。`omb` 的 `state` / `comp` / `ability_runtime` 中屬於 deterministic gameplay 的部分搬到或 re-export 自 `omoba-core::runtime`；backend-only transport、logging、server setting 與 executable lifecycle 留在 `omb`。

   Alternative considered: 只把少數 DTO re-export 到 `omoba-core`。拒絕原因是 omfx 需要完整 local replica，不只是 DTO；piecemeal re-export 會留下大量 cross-crate private coupling。

4. `PlayerInput` 使用單一 shared protocol type。

   native `omfx` 的 lockstep client 已從 `omoba-core` 取得 `proto/game.proto` 產生的 `PlayerInput`，但 sim runner 又從 `omobab::lockstep::PlayerInput` 取得另一份 generated type，導致 `convert_player_input` 需要 prost encode/decode roundtrip。解耦後 runtime input boundary SHALL 使用 `omoba-core` 產生的 shared `PlayerInput` type，讓 client 與 runtime 共用同一個 Rust type。

   Alternative considered: 保留 prost roundtrip，只把 decode target 換成另一個 generated type。拒絕原因是 duplicate generated types 仍會增加 drift 風險與不必要配置。

5. backend process startup 移出 `omfx` executable。

   `omfx` startup SHALL 只讀取 connection/env settings 並啟動 frontend runtime；需要 backend 的 dev flow 由 `run.bat`、`run_smoke*.bat`、`run_stress.bat` 在啟動 frontend 前建立 backend process。這讓直接執行 `executor.exe` 不需要 repo 中存在 `omb/`，也避免 frontend 在使用者機器上呼叫 `cargo run`。

   Alternative considered: 保留 opt-in `OMFX_AUTO_SPAWN_BACKEND=1`。暫不採用，因為使用者明確要求前端不需要引用後端即可執行；若未來要開發便利性，可做成獨立 launcher 或 helper，不放回 `omfx/game`。

6. 以 re-export 降低一次性修改面，但 re-export 方向只能是 `omb -> omoba-core::runtime`。

   migration 中可以讓 `omb` 暫時 `pub use omoba_core::runtime::*` 維持 backend 內部路徑過渡；`omfx` 不得使用 `omb` re-export。最終目標是 `omfx` 只 import `omoba_core::*` / `omoba_core::runtime::*`。

## Risks / Trade-offs

- [Risk] 搬移 ECS components 與 systems 容易造成大量 module path churn → Mitigation: 先在 `omoba-core::runtime` 建立 module 邊界，讓 `omb` 測試通過後再改 `omfx` imports。
- [Risk] `omoba-core` 變重，部分 tools 或 wasm target 可能被 runtime dependencies 影響 → Mitigation: 用 target-specific `cfg` 隔離不能執行的 runtime implementation；若某個 tool 真的只需要 primitive，應依賴 `omoba-sim`，而不是期待 `omoba-core` 是超輕量 crate。
- [Risk] `scripts/script-abi`、`specs` fork 與 `abi_stable` 版本必須一致 → Mitigation: `omoba-core::runtime` 使用與 `omb` 相同 path dependencies，並沿用 repo 的 Rust 1.91.0 toolchain。
- [Risk] launcher 接手 backend process lifecycle 後，frontend exit 不再自動透過 `BackendGuard` kill child → Mitigation: launcher 使用 explicit process handle / PowerShell `Start-Process -PassThru` 或 cmd `start /wait` orchestration，並在 exit/cleanup path 停止 backend；smoke/stress scripts 保留 stale process cleanup。
- [Risk] `run_stress.bat` 目前靠複製 release backend 到 debug path 配合 omfx hard-code spawn → Mitigation: 移除該 staging path，改由 stress launcher 直接啟動 release `omobab.exe`。
- [Risk] snapshot structs 移到 `omoba-core::runtime` 後，render code import 路徑會改變 → Mitigation: 在 `omfx/game/src/sim_runner.rs` 保留 thin wrapper/re-export 一段時間，只要 wrapper 不引用 `omobab`。

## Migration Plan

1. 在 `omoba-core` 新增 mandatory `src/runtime/` module，加入必要 runtime dependencies。
2. 將 deterministic gameplay modules 與 runtime entrypoints 搬到 `omoba-core::runtime`，讓 `omb` 改用此 runtime 並保持 backend tests 通過。
3. 將 `SimWorldSnapshot`、render snapshot DTO、render-only queues 與 `extract_snapshot` 搬到 `omoba-core::runtime`，調整 `sim-snapshot-rendering` 對 shared source 的要求。
4. 將 lockstep `PlayerInput` boundary 收斂到 `omoba-core` shared protocol type，刪除 omfx 端 `convert_player_input` prost roundtrip。
5. 修改 `omfx/game/Cargo.toml` 移除 `omobab` dependency，`sim_runner.rs` 改依賴 mandatory `omoba-core::runtime`。
6. 移除 `native.rs` 的 `BackendGuard` / `spawn_backend` / `create_job_and_attach` 與 hard-coded `omb` path discovery；更新 `run*.bat` 由 launcher 啟動與清理 backend。
7. 執行 `cargo test --manifest-path omb/Cargo.toml -p omobab --lib`、`cargo test --manifest-path omoba-sim/Cargo.toml --no-default-features`、`cargo build --manifest-path omfx/Cargo.toml -p executor` 與 smoke launcher 驗證。

Rollback strategy: 若 runtime 搬移造成 regression，保留 change branch 並回退到 `omb` 仍消費原 modules 的階段；不要把 `omfx -> omobab` dependency 加回來作為長期修復。

## Open Questions

- `gen-docs` 的 script catalog helpers 是否需要進 `omoba-core::runtime`？建議第一版先排除，避免把 docs/tooling scope 混進前後端 runtime 解耦。
