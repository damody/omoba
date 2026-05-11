## Why

目前 `run.bat` debug 開發流程仍會把 `scripts/lua_data` 內容經由 `omoba-template-ids` build-time codegen 轉成 Rust data；只改塔數值、波次或其他 Lua content 也會觸發相關 crate 重新編譯，讓內容調參迭代成本過高。

需要一個預設給 debug launcher 啟用的 Lua content loading 路徑，讓 `run.bat` / smoke debug 流程能直接讀取 Lua source 的最新值；同時也要能在 release build 以明確 feature/env opt-in 使用，方便用 optimized binary 驗證 Lua content。一般 release/stress 流程仍必須維持現有純 Rust generated data，不預設在 runtime 讀 Lua，以保留壓測與正式執行的效能與可部署性。

## What Changes

- 新增 explicit runtime Lua content loading 模式；debug launchers 預設編譯/啟用，release build 可用同一套 feature/env 明確 opt-in。
- runtime Lua content 模式下，omb host 與 omfx sim_runner 使用同一套 Lua-loaded content snapshot，來源為 `scripts/lua_data`，避免只改 Lua 值時必須重新編譯 `omoba-template-ids`、`omb` 或 `base_content.dll`。
- default release / stress 模式維持現有 build-time Rust generated data；`run_stress.bat` 不啟用 runtime Lua loading，也不得讓 Lua loader 進入預設 release gameplay hot path。
- 調整 freshness 規則：debug launcher 在 Lua content-only 變更時不因 generated-data inputs stale 而 rebuild Rust artifacts；release/stress 仍把 Lua content 視為 codegen input，必要時 rebuild。
- 保留 script ABI、unit ids、story ids、template lookup API 與 deterministic ordering；runtime Lua loader 必須使用與 codegen 相同的 Lua builder contract 與 validation。
- 不改變 gameplay schema、network protocol、FFI layout、runtime-visible string 或 `run_stress.bat` 的 stress map / game.toml swap 行為。

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `dev-run-incremental-build`: debug launchers 需要預設支援 runtime Lua content mode，Lua content-only 變更不觸發 debug Rust rebuild；release/stress launcher 仍以 generated Rust data 與 release artifacts 為準，release opt-in 則可明確編譯/啟用 Lua loader。
- `unit-template-references`: runtime crates 目前不得讀 Lua；需求需放寬為明確 feature/env 啟用的 local runtime loader 可讀 Lua，debug launchers 預設使用，release build 可 opt-in，release/stress/default production path 仍只使用 generated Rust data。

## Impact

- 影響 `run.bat`、`run_smoke.bat`、`run_smoke_long.bat`、`run_stress.bat` 與 `scripts/dev_run_freshness.ps1` 的 launcher/freshness contract。
- 影響 `omoba-template-ids` 的 Lua builder loading code reuse：需要抽出可供 runtime Lua content mode 使用的 loader 或建立等價 helper，避免 codegen 與 runtime path 行為分裂。
- 影響 `omb` story/template initialization path：runtime Lua content mode 可從 Lua-loaded content 建立 runtime structures；release/default path 維持 generated Rust data。
- 影響 `omfx/game/src/sim_runner.rs` 初始化：runtime Lua content mode 需與 host 使用相同 content source，避免 host 與 local replica 模擬資料不一致。
- 新增或調整測試，涵蓋 debug Lua override、release opt-in Lua loader、release/stress 預設不讀 Lua、freshness skip rebuild 與 missing/invalid Lua fail-fast 行為。
