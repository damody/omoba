## MODIFIED Requirements

### Requirement: launcher-specific runtime behavior 保持不變

incremental freshness checks SHALL NOT 改變 launcher-specific runtime setup。`run_smoke.bat` SHALL 保留 2-second auto-start 與 10-second auto-exit。`run_smoke_long.bat` SHALL 保留 2-second auto-start 與 60-second auto-exit。`run_stress.bat` SHALL 持續 regenerate stress map、launch 前把 `omb/game.toml` swap 到 stress variant，並在完成或失敗後 restore 原本的 `omb/game.toml`。

`run.bat --autoplay-100` SHALL 沿用 debug freshness build、runtime Lua content 與 DLL staging，設定 `OMFX_AUTOPLAY_100=1` 及無英雄 TD 環境後啟動 omfx visual autoplay。未提供 `--autoplay-100` 時，`run.bat` SHALL 保持既有一般啟動行為。

#### Scenario: smoke launchers 保留 auto-exit settings
- **WHEN** `run_smoke.bat` 或 `run_smoke_long.bat` 以 fresh artifacts 執行
- **THEN** launcher 視情況 skip fresh builds
- **AND** 設定與以往相同的 `OMFX_AUTO_START_AFTER_SEC` 與 `OMFX_AUTO_EXIT_AFTER_SEC` values

#### Scenario: stress launcher 在 skipped builds 後仍 restores game.toml
- **WHEN** `run_stress.bat` 以所有 release artifacts fresh 的狀態執行
- **THEN** launch 前仍會把 `omb/game.toml` swap 到 stress variant
- **AND** frontend exit 後 restore 原本的 `omb/game.toml`

#### Scenario: run.bat 啟動可視化 autoplay
- **WHEN** 使用者從 repository 根目錄執行 `run.bat --autoplay-100`
- **THEN** launcher 完成既有 debug freshness checks 與 DLL staging
- **AND** 以 `OMFX_AUTOPLAY_100=1` 及無英雄 TD 設定啟動 omfx executor

#### Scenario: run.bat 一般模式保持不變
- **WHEN** 使用者執行不帶 `--autoplay-100` 的 `run.bat`
- **THEN** launcher 不設定 `OMFX_AUTOPLAY_100`
- **AND** 使用既有一般 frontend session 流程
