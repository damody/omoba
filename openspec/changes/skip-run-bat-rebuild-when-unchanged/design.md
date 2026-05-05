## 背景

`run*.bat` launchers 目前每次啟動都會執行 build/copy preparation。`run.bat`、`run_smoke.bat` 與 `run_smoke_long.bat` 使用 debug artifacts；`run_stress.bat` 使用 release artifacts、stage release DLL，並把 release backend copy 到 `omb/target/debug/omobab.exe`，因為 omfx spawn 的 backend path hard-coded 為 debug。Cargo 內部可以 incremental，但 scripts 仍會每次付出 Cargo startup/check cost，且常常 rewrite staged artifacts。

launcher 是 Windows-only，且必須能從 `cmd` 使用。本 repo 的 batch files 必須維持 CRLF line endings。Rust toolchain 與 ABI boundary 都是固定的，因此任何 freshness optimization 都必須 fail safe：如果 script 不能證明 artifact 是 fresh，就必須 build。

## 目標 / 非目標

**目標：**
- 當對應 output artifact 存在，且所有 relevant inputs 都比該 output 舊時，`run.bat`、`run_smoke.bat` 與 `run_smoke_long.bat` 跳過 debug Cargo build invocations。
- `run_stress.bat` 在相同 conservative freshness rule 下跳過 release Cargo build invocations。
- 當選定 source artifact 的 content/timestamp 已 staged 時，避免 rewrite `omb/scripts/base_content.dll` 或 stress backend spawn copy。
- 保留 first-run、clean-target 與 changed-source 行為。
- 讓每個 launcher 維持簡單可跑：`run.bat`、`run_smoke.bat`、`run_smoke_long.bat` 與 `run_stress.bat` 仍是 entry points。

**非目標：**
- 不改 stress map generation、`game.toml` swap/restore、smoke auto-start/auto-exit settings 或 runtime gameplay semantics。
- 不改 Rust workspace structure、Cargo profiles、script ABI 或 runtime DLL loading。
- 不嘗試取代 Cargo 的完整 dependency graph；launcher 只做 conservative pre-check。

## 決策

1. 使用一個小型 PowerShell freshness helper，由所有 `run*.bat` scripts 呼叫，而不是撰寫複雜的 batch-only timestamp logic。

   理由：recursive timestamp checks、UTC comparisons 與 path quoting 在純 batch 中容易出錯。PowerShell 5.1 已存在於支援的 Windows environment。

   曾考慮的替代方案：依賴 `cargo build` 回報 `Fresh`。這能保 correctness，但仍會 invoke Cargo，且無法阻止 unconditional DLL copy。另一個替代方案是純 `forfiles`/batch logic，但較難維護，也更容易處理 spaces 或 nested paths 失敗。

2. 將 freshness 建模為 debug 與 release artifact groups 的 conservative output-vs-input checks。

   script DLL group 會把 `scripts/target/{debug,release}/base_content.dll` 與 `scripts` workspace manifests、`scripts/base_content/src`、`scripts/script-abi/src`、shared path dependency sources、`omoba-template-ids`、`omoba-sim`、`omb/Story/templates.json`、`Cargo.lock` 與 `rust-toolchain.toml` 比較。

   backend group 會把 `omb/target/{debug,release}/omobab.exe` 與 `omb` manifests/sources、`proto/game.proto`、shared path dependency sources、`scripts/script-abi`、`omoba-template-ids`、`omoba-sim`、`Cargo.lock` 與 `rust-toolchain.toml` 比較。

   frontend group 會把 `omfx/target/{debug,release}/executor.exe` 與 `omfx` workspace manifests/sources、shared client crates、`proto/game.proto`、`Cargo.lock` 與 `rust-toolchain.toml` 比較。

   理由：這能在 no-change path 避免 invoke Cargo，同時在一般 project inputs 更新時仍會 rebuild。任何 missing output 或 helper error 都視為 stale。

   曾考慮的替代方案：successful builds 後更新 stamp files。output timestamps 更簡單，也避免引入另一個可能獨立變 stale 的 state file。

3. 只在需要時 stage copied artifacts；當 debug 與 release 共用同一個 destination 時使用 hash checks。

   selected script DLL fresh 後，helper 會比較 `scripts/target/<profile>/base_content.dll` 與 `omb/scripts/base_content.dll`。只有 destination missing、older，或目前 staged 的 build profile/content 不同時，才更新 staged copy。`run_stress.bat` 同樣只在 release executable 尚未 staged 到 `omb/target/debug/omobab.exe` 時，才 copy `omb/target/release/omobab.exe` 到該位置。

   曾考慮的替代方案：保留 `copy /y`。這很簡單，但會直接造成 artifact churn，讓 no-change launches 無法與 changed DLL launches 區分。

4. frontend executable fresh 時直接 launch。

   如果 `omfx/target/<profile>/executor.exe` fresh，launcher 會從 repo root 直接執行該 executable，而不是 `cargo run`。如果 stale，script 先 build 再 launch executable。這保留相同 working directory，同時在常見 no-change path 避免 Cargo。

   曾考慮的替代方案：永遠使用 `cargo run`。這對 Cargo-managed execution 較安全，但不符合跳過不必要 Cargo work 的目標。

## 風險 / 取捨

- 遺漏 input path 可能留下 stale artifact -> input sets 刻意維持寬鬆，包含 shared path dependencies 與 generated-data sources，並把 helper failures 視為 stale。
- Timestamp granularity 或 clock skew 可能產生錯誤的 freshness decisions -> 使用 `LastWriteTimeUtc`，並在 values missing 或 ambiguous 時 rebuild。
- 直接 launch `executor.exe` 可能在未來與 `cargo run` 不同，若 Cargo 注入 required environment -> 從相同 repo root launch，executable missing 或 stale 時 fallback 到 Cargo build。
- Debug 與 release DLL/backend copies 共用 destinations -> 比較 hashes 而不只 timestamps，讓 debug 與 stress launchers 切換時能正確 stage selected profile。
- freshness helper 增加一個需要維護的 script -> 讓它 data-driven，且範圍限制在 timestamp comparison/copy decisions。

## 遷移計畫

1. 新增 freshness helper，並更新每個 `run*.bat` launcher，讓它對 script DLL、backend、frontend 與 artifact staging decisions 呼叫 helper。
2. 驗證 clean 或 missing-artifact runs 仍會 build 所有 required artifacts。
3. 驗證立即第二次 launch 會回報 build artifacts up-to-date，且不 rewrite staged artifacts。
4. 若任何 launcher skip 了必要 build，rollback 方式是 revert `run*.bat` scripts 與 helper script。

## 未決問題

無。
