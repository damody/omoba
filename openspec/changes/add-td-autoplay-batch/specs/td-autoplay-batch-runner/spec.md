## ADDED Requirements

### Requirement: Lua 入口可從任意工作目錄執行
系統 SHALL 提供 `scripts/test_td_1_to_100.lua`，並以腳本自身位置解析 repository 根目錄，不依賴呼叫端目前目錄，且固定由 repository Lua 5.4 runtime 執行。

#### Scenario: 從 repository 外部啟動
- **WHEN** 使用者從其他工作目錄以固定 Lua runtime 呼叫 `scripts/test_td_1_to_100.lua`
- **THEN** 所有 Cargo manifest 路徑皆相對正確的 repository 根目錄解析

### Requirement: 測試前建置 release script DLL
Lua 入口 MUST 先執行 `cargo build --manifest-path scripts/Cargo.toml -p base_content --release`；只有建置成功時才能執行 autoplay integration test。

#### Scenario: Script DLL 建置成功
- **WHEN** release `base_content` 建置回傳 exit code 0
- **THEN** Lua 入口執行單次 `layered_td_coarse_autoplay_completes_rounds_1_to_100` 測試並保留即時輸出

#### Scenario: Script DLL 建置失敗
- **WHEN** release `base_content` 建置回傳非零 exit code
- **THEN** Lua 入口不執行 autoplay 測試並以非零狀態結束

### Requirement: 準確傳遞測試結果
Lua 入口 SHALL 將 autoplay 測試的 exit code 傳回呼叫端，且 MUST NOT 使用互動等待阻塞非互動執行。

#### Scenario: 完整測試通過
- **WHEN** 第 1 至 100 關 autoplay integration test 成功
- **THEN** Lua 入口回傳 exit code 0

#### Scenario: 完整測試失敗
- **WHEN** autoplay integration test 失敗
- **THEN** Lua 入口保留 Cargo 錯誤輸出與既有 failure report，並回傳相同的非零 exit code

### Requirement: Lua 入口符合 repository runtime 與產物規範
Lua 入口 MUST 由 `D:\code\omoba\tools\lua\lua.exe` 執行，不得使用其他 script runtime fallback，且 SHALL 僅在已忽略的建置或測試目錄產生 DLL、`target/`、log 或 failure report。

#### Scenario: 檢查版本控制內容
- **WHEN** Lua 入口執行完成並檢查 Git 狀態
- **THEN** 不會出現需要提交的編譯或測試暫存檔
