## ADDED Requirements

### Requirement: 所有現行腳本使用固定 Lua runtime

系統 SHALL 以 `D:\code\omoba\tools\lua\lua.exe` 執行所有 repository 自有的現行工作流腳本。除了四個根目錄薄 `.bat`，受版控的現行腳本 SHALL 使用 `.lua`，且 SHALL 不以 PowerShell、Python 或 shell 作為 fallback script runtime。

#### Scenario: 掃描現行腳本
- **WHEN** 最終驗證掃描受版控工作流檔案
- **THEN** 只存在四個允許的根目錄 `.bat` 與 Lua 腳本
- **AND** 現行呼叫端不引用已移除的 `.ps1`、`.py`、`.sh` 或額外 `.bat`

#### Scenario: Lua 不在 PATH
- **WHEN** PATH 不包含 `lua`
- **THEN** 四個 launcher 與文件指令仍使用固定絕對路徑 Lua runtime 正常執行

### Requirement: 共用 Lua 模組提供一致基礎能力

系統 SHALL 在 `tools/lua/lib/` 提供 args、path、process、json、udp、hash、time 與 evidence 的小型模組。模組 SHALL 清楚回報錯誤，且 SHALL 不靜默截斷 JSON、忽略外部命令失敗或覆寫既有 evidence run。

#### Scenario: 外部命令失敗
- **WHEN** Lua workflow 執行的外部命令回傳非零 exit code
- **THEN** workflow 回傳非零狀態
- **AND** 錯誤包含操作用途與目標

#### Scenario: evidence run 已存在
- **WHEN** launcher 指向已存在的 evidence run ID
- **THEN** Lua evidence 模組拒絕覆寫
- **AND** 不修改既有 evidence

### Requirement: 平台原生操作維持安全邊界

系統 SHALL 對背景程序、PID 查詢、程序終止、UDP control 與 SHA-256 提供可驗證的實作。若 Lua 標準庫不足，系統 MUST 使用窄介面 Rust helper，而 SHALL NOT 使用 PowerShell 或 Python fallback。

#### Scenario: 終止 session 程序
- **WHEN** launcher 清理某個已記錄 PID
- **THEN** 系統先驗證 PID 對應預期 executable
- **AND** 不終止其他 session 的同名程序

#### Scenario: 傳送 netem control
- **WHEN** Lua scenario 切換某隊的 custom 20-bin profile
- **THEN** UDP payload 保留既有 schema、team ID、weights 與 authoritative tick
- **AND** proxy 套用後續 datagram 並留下相同語意的 evidence

### Requirement: Lua replacement 保持腳本契約

每支 replacement SHALL 保留舊腳本的有效 CLI、環境變數、成功與失敗 exit code、輸出產物、程序順序及清理責任。無法保留的舊入口 SHALL 在文件中提供明確的新 `.lua` 對應路徑。

#### Scenario: 三程序 fog headless 執行
- **WHEN** 使用 Lua launcher 啟動 authoritative server、Team 1 runtime 與 Team 2 runtime
- **THEN** 三個 Rust process 使用原本隔離路由完成同步
- **AND** evidence comparison 驗證視野外資訊未洩漏

#### Scenario: netem 視覺執行
- **WHEN** 使用 Lua launcher 啟動 proxy、兩個 runtime 與兩個 renderer
- **THEN** 啟動及停止順序與既有流程相同
- **AND** 移動、Reveal、Hide、Forget、rebase 與兩隊視野隔離維持通過

### Requirement: 全部移植後集中驗證

系統 SHALL 在所有 production script 與測試資產完成後，集中執行 Lua syntax、module、launcher、tooling、netem、fog 與受影響 Cargo regression tests。

#### Scenario: 最終完整 gate
- **WHEN** 所有舊腳本已替換且呼叫端已更新
- **THEN** 所有 `.lua` 通過 `loadfile` 與 module tests
- **AND** launcher、三程序、netem、fog、character tooling、Cargo tests 與 `git diff --check` 全部通過

