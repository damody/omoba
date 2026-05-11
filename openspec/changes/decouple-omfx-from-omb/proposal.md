## Why

目前 native `omfx` 透過 Cargo path dependency 直接引用 `omobab`，導致前端必須能編譯並連結後端 crate 才能執行。這讓前端 build、執行與未來發佈都被 backend implementation 綁住，也讓「純前端 renderer 透過 protocol 連到任意 omb server」的邊界不清楚。

## What Changes

- **BREAKING**: `omfx` native build 不再允許依賴 `omobab` crate 或從 `omfx/game` 中 import `omobab::*`。
- 將 lockstep 本地 simulation replica 所需的 deterministic runtime、snapshot projection 與 render-facing data contract 收斂到 `omoba-core::runtime`。
- `omfx` 只依賴 shared crates（主要是 `omoba-core`、`omoba-sim`、`omoba-template-ids`）以及 wire protocol，不依賴 backend binary/lib crate。
- 不新增 `omoba-runtime` crate；`omoba-sim` 保持 deterministic primitives 小核心，`omoba-core` 擴充為 shared protocol + transport client + mandatory runtime boundary。
- 保留既有 KCP lockstep、script DLL、scene data、snapshot rendering 與 TD UI 行為；解耦應改變 dependency boundary，不應改變玩家可見行為。
- 開發 launcher 仍可選擇啟動 backend process，但這是 launcher/runtime orchestration，不是 `omfx` crate 對 `omb` 的 compile-time dependency。

## Capabilities

### New Capabilities
- `frontend-backend-decoupling`: 定義 `omfx` 前端 build/run 時不得依賴 `omobab` crate，並透過 `omoba-core::runtime` 與 shared protocol 取得 lockstep simulation 與 snapshot rendering 所需資料。

### Modified Capabilities
- `sim-snapshot-rendering`: 將 snapshot extraction 與 render-facing data contract 中的 backend-crate-specific 要求改為 `omoba-core::runtime` 要求，同時保留既有 snapshot 欄位與 UI 行為。

## Impact

- Affected code: `omoba-core/Cargo.toml`、`omoba-core/src/runtime/**`、`omfx/game/Cargo.toml`、`omfx/game/src/sim_runner.rs`、`omfx/game/src/native.rs`、`omb/Cargo.toml`、`omb/src/state/*`、`omb/src/comp/*`、`omb/src/ability_runtime/*`。
- APIs: `omoba-core::runtime`、`SimWorldSnapshot`、`EntityRenderData`、lockstep `PlayerInput` handling、simulation init/tick/snapshot extraction boundary。
- Dependencies: 移除 `omfx -> omobab` path dependency；把目前 backend-only ECS components、resources、dispatch helpers 與 snapshot projection 搬到 `omoba-core::runtime`，讓 `omb` 與 native `omfx` 都依賴同一個 shared runtime module。
- Systems: Windows dev launchers、native frontend startup、KCP lockstep client、script DLL loading、graphify/OpenSpec specs。
