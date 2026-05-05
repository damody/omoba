## 1. Audit Current Launcher Inputs

- [x] 1.1 確認 `run.bat` 使用的 debug output paths：`scripts/target/debug/base_content.dll`、`omb/scripts/base_content.dll`、`omb/target/debug/omobab.exe` 與 `omfx/target/debug/executor.exe`。
- [x] 1.2 從 Cargo manifests、build scripts、shared path dependencies、`proto/game.proto`、`omb/Story/templates.json` 與 `rust-toolchain.toml` 建立 script DLL、backend 與 frontend artifact groups 的 conservative input path lists。

## 2. Add Freshness Helper

- [x] 2.1 新增 PowerShell helper，使用 `LastWriteTimeUtc` 做 recursive artifact freshness checks、missing-output detection，並安全處理 missing 或 unreadable paths。
- [x] 2.2 新增 helper operation，只有當 destination missing 或 older than source DLL 時才 stage `base_content.dll`。
- [x] 2.3 確保 helper exit codes 對 batch usage 保持簡單：fresh、stale 與 error states，讓 `run.bat` 能 fail-safe 處理。

## 3. Update `run.bat`

- [x] 3.1 以 freshness check 取代 unconditional `base_content` Cargo build，只有 stale 時才 build。
- [x] 3.2 以 conditional staging 取代 unconditional `copy /y` `base_content.dll`。
- [x] 3.3 以 freshness check 取代 unconditional backend Cargo build，只有 stale 時才 build。
- [x] 3.4 將 frontend `cargo run` path 改為 freshness-check/build-when-stale，接著從 repo root 直接 launch `omfx/target/debug/executor.exe`。
- [x] 3.5 保留 `run.bat` 既有 process cleanup、error handling、user-visible progress output 與 CRLF line endings。

## 4. Verify Behavior

- [x] 4.1 驗證 missing artifacts 被視為 stale，並會觸發 matching Cargo build step。
- [x] 4.2 成功跑一次 `run.bat`，接著在沒有 source changes 的情況下再跑一次，確認所有 build artifact groups 都回報 up-to-date。
- [x] 4.3 確認第二次 no-change run 在 launch 前不會修改 `omb/scripts/base_content.dll` 的 `LastWriteTime`。
- [x] 4.4 驗證 representative source timestamp newer than artifact 時，affected artifact group 會在 launch 前 rebuild。

## 5. Extend Freshness Helper To All Launch Profiles

- [x] 5.1 為 debug 與 release script DLL、backend、frontend artifacts 加入 profile-aware freshness checks。
- [x] 5.2 讓 DLL staging 同時比較 content 與 timestamps，確保 debug/release launchers 切換時會 stage selected artifact。
- [x] 5.3 為 `run_stress.bat` 加入 conditional release backend spawn staging，從 `omb/target/release/omobab.exe` 到 `omb/target/debug/omobab.exe`。

## 6. Update Remaining `run*.bat` Launchers

- [x] 6.1 更新 `run_smoke.bat`，reuse debug freshness checks、conditional DLL staging、backend/frontend build-if-stale，以及 direct debug executor launch。
- [x] 6.2 更新 `run_smoke_long.bat`，reuse debug freshness checks，同時保留其 60-second smoke runtime。
- [x] 6.3 更新 `run_stress.bat`，reuse release freshness checks、conditional release DLL staging、conditional backend spawn copy、direct release executor launch，以及既有 `game.toml` restore behavior。
- [x] 6.4 保留所有 edited `.bat` files 的 CRLF line endings。

## 7. Verify All Launchers

- [x] 7.1 驗證加入 profile support 後，debug helper checks 仍能正確回報 fresh/stale。
- [x] 7.2 驗證 release helper checks 正確回報 fresh/stale，並在缺少 release artifacts 時 build。
- [x] 7.3 使用 auto-exit paths 跑 `run_smoke.bat` 與 `run_smoke_long.bat`，確認 fresh debug artifacts 會 skip builds。
- [x] 7.4 使用 auto-exit path 跑 `run_stress.bat`，確認 fresh release artifacts 會 skip builds 且 `omb/game.toml` 會 restore。
