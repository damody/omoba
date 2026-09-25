## Purpose

定義 Lua、Rust 與 Unreal 生成內容之間的穩定契約，使新增英雄、技能、物品與呈現資產時不必手寫角色專屬 Unreal 程式。

## ADDED Requirements

### Requirement: 單一內容來源
系統 SHALL 從經驗證的內容定義產生 Rust 與 Unreal 所需的穩定 ID、資料與註冊資訊，並拒絕重複或失效的引用。

#### Scenario: 無效技能引用
- **WHEN** 英雄引用不存在的技能 ID
- **THEN** 生成失敗並指出英雄與技能 ID

### Requirement: 英雄模板生成
系統 SHALL 依英雄內容定義產生所需的 Unreal C++ 類別與資產配方，且重跑相同輸入不得產生差異。

#### Scenario: 新增英雄
- **WHEN** 開發者提供完整 Lua 內容、必要的 Rust 行為與美術資產
- **THEN** 生成器產生可編譯的英雄註冊與呈現資料，無須手寫角色 C++ 或 Blueprint graph

### Requirement: 內容版本一致
系統 SHALL 驗證腳本 DLL、生成資料與 Unreal 內容的版本及內容雜湊，遇到必要內容錯配時拒絕開局。

#### Scenario: 舊版腳本 DLL
- **WHEN** 載入的腳本 DLL 與本次生成內容不一致
- **THEN** 系統顯示明確錯配原因並停止啟動該對局
