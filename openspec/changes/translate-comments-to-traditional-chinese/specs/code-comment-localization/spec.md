## ADDED Requirements

### Requirement: 第一方程式註解繁體中文化

實作 SHALL 將第一方程式碼中的既有註解與 doc comments 翻譯或改寫為自然繁體中文。第一方範圍 SHALL 包含專案自有 source、scripts、tools 與 configuration comments，且 SHALL 排除 generated files、build artifacts、vendored dependency 與 forked dependency 原始碼，除非後續明確指定。

#### Scenario: 第一方註解完成翻譯

- **WHEN** 實作完成後檢查第一方 source files 中的註解
- **THEN** 既有英文語意註解已翻譯或改寫為繁體中文
- **AND** 技術 symbol、API 名稱、protocol 名稱與檔名可保留原文

#### Scenario: 排除非第一方或產物路徑

- **WHEN** 實作掃描 `target/`、generated files、vendored dependency 或 forked dependency 原始碼
- **THEN** 這些路徑不會被批次翻譯
- **AND** 若需要處理 forked dependency 的 local patch 註解，必須在實作紀錄中明確標示

### Requirement: 不改變 runtime-visible 行為

實作 MUST NOT 翻譯或改寫會影響執行期行為、測試判斷、外部介面或資料格式的內容。包含但不限於 string literals、log/error messages、UI text、network payload、serialization field names、test expected strings、fixture data、public API identifiers 與 command-line output。

#### Scenario: Runtime 字串保持不變

- **WHEN** 審查實作 diff
- **THEN** 非註解的 string literals、protocol payload、schema field names 與 test expected strings 保持不變
- **AND** 若有任何非註解文字變更，必須能證明它不會影響 runtime behavior 或測試結果

#### Scenario: Public API 與 ABI 保持不變

- **WHEN** 實作完成後比較 public symbols、FFI types、proto schema 與 script ABI 相關型別
- **THEN** 這些介面沒有因註解翻譯而重新命名、刪除或改變 layout

### Requirement: 保留可追溯的技術語彙

翻譯後的註解 SHALL 保留必要英文技術名詞、程式 symbol、crate/module/type/function 名稱與規格關鍵字，讓讀者可以直接對應到實際 code 或外部文件。中文敘述 SHALL 使用台灣常用詞彙，避免簡體中文用語。

#### Scenario: Symbol 名稱不被意譯

- **WHEN** 註解引用 `GameWorld`、`UnitScript`、`AbilityScript`、`BuffStore`、`KCP`、`Entity`、`snapshot` 或相同類型的 code symbol
- **THEN** 翻譯後註解仍保留該原文 symbol 或足以直接搜尋的原文關鍵字

#### Scenario: 中文用語維持一致

- **WHEN** 同一概念在多個第一方檔案中出現
- **THEN** 翻譯使用一致的繁體中文說法
- **AND** 不引入簡體中文詞彙或大陸慣用語

### Requirement: 實作驗證只允許註解層級變更

實作 SHALL 在完成後提供驗證紀錄，證明主要變更限於註解文字。驗證 SHALL 包含人工 diff review，以及至少一組可負擔的 format、test 或 build smoke；若完整測試不可行，必須記錄未執行原因。

#### Scenario: Diff review 確認無行為改動

- **WHEN** 實作完成後審查 git diff
- **THEN** 除註解翻譯、過時註解移除或格式化造成的註解排列外，沒有邏輯、控制流程、資料結構或設定值改動

#### Scenario: 可負擔驗證通過或記錄限制

- **WHEN** 執行實作 tasks 中指定的驗證指令
- **THEN** 指令通過
- **OR** 若因環境、耗時或既有失敗無法完成，結果摘要會記錄未通過或未執行的原因
