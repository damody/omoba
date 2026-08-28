## ADDED Requirements

### Requirement: Character pipeline orchestration 使用 Lua

Repository 自有的 character pipeline bootstrap、診斷與 package helper SHALL 由固定 Lua runtime 執行。Lua orchestration MAY 明確啟動 user-local venv Python、Blender、ComfyUI 或其他第三方工具，但 SHALL NOT 以 repository 自有 Python script 作為入口或 fallback。

#### Scenario: 準備 Linux toolchain
- **WHEN** 使用者執行 Lua toolchain prepare
- **THEN** Lua workflow 在 user-local cache 建立或驗證 Linux venv
- **AND** 不執行或修改 shared AI root 的 Windows portable Python

#### Scenario: 執行第三方 Python 驗證
- **WHEN** Lua workflow 驗證 PyTorch CUDA 或 ComfyUI
- **THEN** 它明確執行選定 venv 的 Python executable
- **AND** 診斷 evidence 記錄 executable path、版本、exit code 與驗證結果

#### Scenario: 產生並驗證 smoke package
- **WHEN** Lua character pipeline 建立非 `hero_example` smoke package
- **THEN** package 維持既有 Lua schema 與 ASCII snake_case hero ID
- **AND** Lua validator 回報 package valid，且不產生大型 PNG 或 GLB
