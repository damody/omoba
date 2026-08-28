## Why

專案目前同時依賴 Batch、PowerShell、Python 與 shell 腳本，造成參數、程序生命週期、錯誤處理與跨平台行為分散，維護及重現測試的成本持續增加。統一採用 repository 內固定版本的 Lua 5.4，可讓所有現行工作流共用同一套可檢查的腳本基礎設施。

## What Changes

- 新增共用 Lua 腳本 runtime 模組，涵蓋參數、路徑、程序、JSON、UDP、hash、時間與 evidence 操作。
- 將受版控的 `.ps1`、`.py`、`.sh` 與非必要 `.bat` 工作流移植為 `.lua`。
- **BREAKING**：移除被取代的舊腳本入口；自動化與文件必須改呼叫對應 `.lua`。
- 保留 `run.bat`、`run_10000.bat`、`run_2player.bat`、`run_ue.bat` 四個根目錄相容入口，但縮減為固定呼叫 `D:\code\omoba\tools\lua\lua.exe` 的薄 wrapper。
- 保留既有 CLI、環境變數、exit code、產物、process 安全邊界、netem 與 fog evidence 語意。
- 更新現行 Rust 呼叫端、測試、文件與 OpenSpec 路徑，並在所有移植完成後集中執行完整驗證。

## Capabilities

### New Capabilities

- `lua-script-runtime`: 定義固定 Lua runtime、共用模組、腳本移植範圍、相容性、安全性與最終驗證要求。

### Modified Capabilities

- `dev-run-incremental-build`: 將 launcher 的 freshness、artifact staging 與啟動流程由 Batch／PowerShell 邏輯改為 Lua，同時保留四個根目錄入口。
- `character-pipeline-toolchain-bootstrap`: 將 bootstrap 與相關工具的 orchestration script 改為 Lua，但保持既有外部 Python toolchain 與資產安全邊界。

## Impact

- 影響根目錄四個 launcher、`scripts/`、`tools/selective_lockstep/`、`docs/tools/`、`docs/character_pipeline/tools/` 與所有現行呼叫端。
- 新增 `tools/lua/lib/` 與 Lua 測試 harness；固定依賴 `D:\code\omoba\tools\lua\lua.exe`。
- Lua 標準庫不足的 UDP 或平台功能可由小型 Rust helper 提供，但不得以 PowerShell 或 Python 作為 fallback。
- 不修改 gameplay、server-authoritative lockstep、team replica、戰爭迷霧或網路延遲模型。
