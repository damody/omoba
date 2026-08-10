## ADDED Requirements

### Requirement: 指定 dev launcher 啟動無英雄 TD session
`run.bat` 與 `run_10000.bat` SHALL 在啟動 frontend/backend session 前設定 `OMB_NO_HEROES=1`，且共用 campaign 初始化 SHALL 在該值為 `1` 時不建立任何 Hero entity。

#### Scenario: 一般 dev launcher 不生成英雄
- **WHEN** 使用者執行 `run.bat`
- **THEN** frontend local simulation 與 launcher-owned backend 都繼承 `OMB_NO_HEROES=1`
- **AND** campaign 初始化完成後 Hero entity 數量為零

#### Scenario: 10000 金幣 launcher 不生成英雄
- **WHEN** 使用者執行 `run_10000.bat`
- **THEN** launcher 同時設定 `OMB_TD_STARTING_GOLD=10000` 與 `OMB_NO_HEROES=1`
- **AND** campaign 初始化完成後 Hero entity 數量為零

### Requirement: Hero-free 初始化不得留下 hero side effects
當 resolved hero policy 為停用時，campaign 初始化 MUST 在建立 hero components 與 enqueue hero `ScriptEvent::Spawn` 前結束，並 SHALL 輸出一次包含 `OMB_NO_HEROES=1` 的診斷。

#### Scenario: 停用 policy 不建立 hero state
- **WHEN** campaign hero creation 收到停用 policy
- **THEN** 不建立帶有 `Hero`、`PlayerOwner`、`Gold`、`Inventory` 或 hero `ScriptUnitTag` 的 entity
- **AND** 不排入 hero `ScriptEvent::Spawn`

#### Scenario: 其他 TD 資源繼續初始化
- **WHEN** `OMB_NO_HEROES=1` 啟用
- **THEN** creep wave、map-authored tower、player lives 與其他非 hero campaign resource 的初始化流程不因該旗標而跳過

### Requirement: 未 opt-in caller 保持相容
`OMB_NO_HEROES` 未設定或值不為精確字串 `1` 時，runtime SHALL 保留既有 campaign hero creation 行為。

#### Scenario: 未設定旗標
- **WHEN** caller 未設定 `OMB_NO_HEROES`
- **THEN** campaign 依既有 mode 與 hero source 建立預期 Hero entities

#### Scenario: 非啟用值
- **WHEN** `OMB_NO_HEROES` 為空字串、`0` 或其他非 `1` 值
- **THEN** runtime 不得停用 campaign hero creation

### Requirement: Launcher batch 格式可由 cmd.exe 穩定解析
`run.bat` 與 `run_10000.bat` MUST 使用 CRLF 行尾及 UTF-8 without BOM，且 SHALL 可由 Windows `cmd.exe` 解析而不產生 `'M' is not recognized`。

#### Scenario: Byte-level 格式驗證
- **WHEN** 驗證工具讀取兩個 batch 檔的原始 bytes
- **THEN** 每個 LF byte 前一個 byte 都是 CR
- **AND** 檔案開頭不是 UTF-8 BOM

#### Scenario: cmd.exe 啟動解析
- **WHEN** 以受控 smoke 執行 launcher 並觀察到第一個 freshness step
- **THEN** 輸出不得包含 `'M' is not recognized`
