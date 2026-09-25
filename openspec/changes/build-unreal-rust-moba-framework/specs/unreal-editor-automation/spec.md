## Purpose

定義開發工具透過 BpGeneratorUltimate 與 Unreal Editor 溝通的流程，以可重複的資產操作及 PIE 驗證取代角色逐項手動設定。

## ADDED Requirements

### Requirement: 編輯器資產配方
工具 SHALL 根據生成的資產配方查詢、匯入或更新 Unreal 資產，設定必要引用並回報每項結果。

#### Scenario: 重跑相同配方
- **WHEN** 同一資產配方對已完成的 Unreal 專案重跑
- **THEN** 工具回報沒有非預期變更，所有必要資產與引用仍有效

### Requirement: 必要的 Blueprint 驗證
工具 SHALL 在配方需要 Blueprint 或 UMG 時產生、編譯並驗證資產，編譯失敗不得回報成功。

#### Scenario: Blueprint 編譯失敗
- **WHEN** 產生的 Blueprint 無法編譯
- **THEN** 工具回報資產路徑與編譯錯誤，保留待處理狀態

### Requirement: PIE 自動驗收
工具 SHALL 能透過 Editor 通道讀取 PIE 狀態與日誌、執行測試操作並保存可檢查的結果。

#### Scenario: Unreal 啟動檢查
- **WHEN** 建置流程啟動 Unreal Editor 與最小 PIE
- **THEN** 工具確認 Editor 連線、取得 PIE 狀態並產生成功或失敗報告
