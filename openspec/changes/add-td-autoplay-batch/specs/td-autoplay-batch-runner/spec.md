## ADDED Requirements

### Requirement: 批次檔可從任意工作目錄執行
系統 SHALL 提供 `scripts/test_td_1_to_100.lua`，並以腳本自身位置解析 repository 根目錄，不依賴呼叫端目前目錄，且固定由 repository Lua 5.4 runtime 執行。

#### Scenario: 從 repository 外部啟動
- **WHEN** 使用者以完整或相對路徑從其他工作目錄呼叫批次檔
- **THEN** 所有 Cargo manifest 路徑皆相對正確的 repository 根目錄解析

### Requirement: 測試前建置 release script DLL
批次檔 MUST 先執行 `cargo build --manifest-path scripts\Cargo.toml -p base_content --release`；只有建置成功時才能執行 autoplay integration test。

#### Scenario: Script DLL 建置成功
- **WHEN** release `base_content` 建置回傳 exit code 0
- **THEN** 批次檔執行單次 `layered_td_coarse_autoplay_completes_rounds_1_to_100` 測試並保留即時輸出

#### Scenario: Script DLL 建置失敗
- **WHEN** release `base_content` 建置回傳非零 exit code
- **THEN** 批次檔不執行 autoplay 測試並以非零狀態結束

### Requirement: 準確傳遞測試結果
批次檔 SHALL 將 autoplay 測試的 exit code 傳回呼叫端，且 MUST NOT 使用 `pause` 阻塞非互動執行。

#### Scenario: 完整測試通過
- **WHEN** 第 1 至 100 關 autoplay integration test 成功
- **THEN** 批次檔回傳 exit code 0

#### Scenario: 完整測試失敗
- **WHEN** autoplay integration test 失敗
- **THEN** 批次檔保留 Cargo 錯誤輸出與既有 failure report，並回傳相同的非零 exit code

### Requirement: 批次檔符合 repository 產物規範
批次檔 MUST 使用 CRLF 行尾，且 SHALL 僅在已忽略的建置或測試目錄產生 DLL、`target/`、log 或 failure report。

#### Scenario: 檢查版本控制內容
- **WHEN** 批次檔執行完成並檢查 Git 狀態
- **THEN** 不會出現需要提交的編譯或測試暫存檔
