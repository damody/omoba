## Why

專案目前程式註解混用英文與中文，降低繁體中文使用者在跨 crate 追蹤遊戲邏輯、FFI 邊界、transport 與 UI 流程時的閱讀一致性。將註解統一整理為自然繁體中文，可以改善維護效率，同時避免改動任何執行期行為。

## What Changes

- 將第一方程式碼中的既有註解與 doc comments 翻譯或改寫為繁體中文。
- 保留程式識別字、API 名稱、協定欄位、log/error message、測試 fixture 字串、檔名與外部規格原文。
- 不新增功能、不變更 public API、不調整資料格式，也不翻譯會影響測試或執行期行為的字串 literal。
- 排除 generated files、build artifacts、third-party/vendor/forked dependency 原始碼，以及不應由本 change 批次改寫的外部文件。

## Capabilities

### New Capabilities

- `code-comment-localization`: 定義程式註解繁體中文化的範圍、保留規則與驗證方式。

### Modified Capabilities

- None.

## Impact

- 影響第一方 Rust、Lua、JavaScript、TypeScript、TOML、Markdown-adjacent code block 等程式來源檔中的註解內容。
- 預期不影響 ABI、network protocol、script API、serialization schema、gameplay logic、build output 或 runtime behavior。
- 實作時需要特別檢查 `omb/`、`omfx/`、`omoba-core/`、`eui/`、`scripts/`、`omb-mcp/`、`map_editor/` 等第一方區域，並避免改動 `specs/`、`log4rs/` 這類 forked dependency，除非後續明確指定。
