## 1. 共用 Lua Content Loader

- [ ] 1.1 從 `omoba-template-ids/build.rs` 抽出 Lua builder loading、path validation、include-cycle detection、read tracking、template manifest/story loading 與 validation 到可被 build-time codegen 與 runtime Lua content mode 共用的模組或 crate API
- [ ] 1.2 保留 `omoba-template-ids` 現有 generated Rust public API 與 deterministic id/order 行為，並確認 build-time `cargo:rerun-if-changed` 仍涵蓋所有實際讀取的 Lua/helper/config files
- [ ] 1.3 新增或調整 `omoba-template-ids` tests，覆蓋共用 loader 的 include ordering、unsafe path rejection、include cycle error、story discovery sorting 與 creep template reference validation

## 2. Runtime Content Source

- [ ] 2.1 設計並實作 runtime Lua content feature/env switch，例如 Cargo feature `runtime-lua-content` 搭配 `OMB_LUA_CONTENT=1` 與 `OMB_LUA_CONTENT_ROOT`；未啟用時所有 runtime 初始化維持 generated Rust data path
- [ ] 2.2 在 `omb` 新增 feature/env-gated Lua-loaded content snapshot 到 `CampaignData`/template lookup 初始化路徑，讓 story、template stats、render metadata、ability metadata 與 creep reference validation 可從 Lua-loaded snapshot 取得
- [ ] 2.3 更新 `omb` server initialization，使 runtime Lua env 啟用時從 Lua-loaded content 初始化 configured `STORY`，production/default path 仍使用 `CampaignData::load_generated`
- [ ] 2.4 更新 `omfx/game/src/sim_runner.rs` 與呼叫端，讓 runtime Lua env 啟用時使用同一個 content root 與 story id 初始化 replica world，未啟用時維持 generated path
- [ ] 2.5 確認 release build 可用 runtime Lua content feature 編譯並由 env opt-in 啟用；default release/stress build 不需要該 loader，且任何 `mlua` dependency outside `omoba-template-ids` 都由 optional feature gate 保護

## 3. Launcher 與 Freshness

- [ ] 3.1 更新 `run.bat`、`run_smoke.bat` 與 `run_smoke_long.bat` 以 runtime Lua content feature 建置 debug artifacts 並設定 runtime Lua content env，保留既有 DLL staging、backend spawn 與 smoke auto-start/auto-exit 行為
- [ ] 3.2 確認 `run_stress.bat` 不啟用 runtime Lua content feature/env，仍 regenerate stress map、swap/restore `omb/game.toml`，並使用 release artifacts/generated Rust data
- [ ] 3.3 更新 `scripts/dev_run_freshness.ps1`，使 debug profile 排除 `scripts/lua_data` content-only timestamp 作為 Rust artifact stale input，但 release profile 仍把 Lua content 視為 generated-data input
- [ ] 3.4 保留 debug profile 對 `omoba-template-ids` Rust sources、Cargo manifests、build scripts、shared path dependencies、protocol files 與 `scripts/base_content/src` 的 stale detection

## 4. 驗證

- [ ] 4.1 新增測試或 smoke 檢查，確認 debug Lua template-only 變更不觸發 debug Rust rebuild，且 `omb` 與 `omfx` `sim_runner` 使用最新 Lua-loaded value
- [ ] 4.2 新增測試或 smoke 檢查，確認 debug story-only 變更不觸發 debug Rust rebuild，且 configured `STORY` 從最新 Lua-loaded story 初始化
- [ ] 4.3 新增測試，確認 invalid/missing/rejected debug Lua content 會 fail-fast，錯誤訊息包含 rejected path、missing story、missing template reference 或 validation failure
- [ ] 4.4 新增測試或檢查，確認 release build 可用 runtime Lua content feature/env opt-in 啟用 loader，且 `run_stress.bat`/default release path 不啟用 runtime Lua loader；Lua content 變更仍會讓 affected release generated-data artifacts stale 並重建 generated Rust data
- [ ] 4.5 執行 `cargo test --manifest-path omoba-template-ids/Cargo.toml`、`cargo check --manifest-path omb/Cargo.toml -p omobab`、`cargo check --manifest-path omb/Cargo.toml -p omobab --release --features runtime-lua-content`、`cargo check --manifest-path omfx/Cargo.toml`，並記錄任何既有或新增失敗
- [ ] 4.6 修改 code 後執行 `graphify update .`，保持 knowledge graph 同步
