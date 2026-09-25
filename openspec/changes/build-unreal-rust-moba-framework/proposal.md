## Why

目前 Unreal 前端已有 bridge 與 Lua 到 C++ 生成器，但其遊戲模擬路徑仍與新完成的 `omoba-client-runtime` 重複，英雄生成器也含角色專屬邏輯。需要把現有元件收斂成可重複使用的 MOBA 框架，讓日常內容製作只需 Lua、Rust 與美術資產。

## What Changes

- 建立三路 MOBA 的權威對局規則、內容、Bot 與單機／區網執行流程。
- Unreal 透過 `omoba-client-runtime` 的 localhost IPC 接收安全呈現資料並送出輸入，正式 MOBA 模式不再持有第二份遊戲世界。
- 整合 Lua 內容模型、Rust 規則資料與 `base_content.dll`，生成通用 Unreal C++ 類別、註冊與資產配方。
- 使用 `BpGeneratorUltimate` MCP 與 Unreal Editor 溝通，完成資產匯入、必要的 Blueprint/UMG 建立、編譯驗證與 PIE 測試。
- 建立可重複的建置、版本檢查、headless 測試及端到端驗收流程。

## Capabilities

### New Capabilities

- `moba-match-runtime`：權威 MOBA 對局、三路地圖、Bot、經濟、勝負與單機／區網一致性。
- `moba-content-generation`：Lua/Rust 內容契約、英雄生成、Unreal C++ 與資產配方。
- `unreal-moba-presentation`：Unreal 經由 client runtime IPC 呈現對局、送出輸入、處理視野與重連。
- `unreal-editor-automation`：使用 BpGeneratorUltimate MCP 建立與驗證 Editor 資產及 PIE。

### Modified Capabilities

無。

## Impact

涉及 `omb`、`omoba-core`、`omoba-client-runtime`、`omoba-template-ids`、`scripts`、`proto/game.proto`、`omfue/codegen`、`omfue/bridge`、`omfue/Plugins/OmRuntime`、`omfue/Plugins/BpGeneratorUltimate` 與啟動工具。既有 TD 模式須持續可執行；單機 MOBA 與 LAN MOBA 共用同一權威規則。設計決定與階段驗收詳見 `docs/superpowers/specs/2026-09-25-unreal-rust-moba-framework-design.md`。
