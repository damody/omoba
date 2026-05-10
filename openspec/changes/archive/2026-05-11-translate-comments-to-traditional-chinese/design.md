## Context

omoba 是跨多個 workspace 與 submodule 的 Rust 遊戲專案，第一方程式碼分散在 backend、frontend renderer、shared schema、script content、MCP 與 UI helper 等區域。現有註解同時包含英文、中文與少量 protocol/API 專有名詞，閱讀時需要在不同語言脈絡間切換。

這個 change 是純維護性改動：只處理程式註解與 doc comments 的文字，不改變任何程式碼語意、public API、FFI 型別、serialization schema、log/error output 或測試 fixture。

## Goals / Non-Goals

**Goals:**

- 將第一方程式碼中的既有註解整理為自然、台灣用語的繁體中文。
- 保留英文專有名詞與程式識別字，避免翻譯後降低與 code symbol 的可追溯性。
- 將實作拆成可審查的區域批次，每批都能確認沒有非註解行為改動。
- 維持 `.bat` 檔 CRLF 行尾與 Rust formatting 既有規則。

**Non-Goals:**

- 不翻譯 runtime-visible 字串，例如 UI text、log message、error message、network payload、snapshot payload、test expected string 或 fixture data。
- 不改寫 generated files、build output、vendored dependency 或 forked dependency 原始碼。
- 不藉由翻譯註解順手重構、重新命名、改 API 或修正行為。
- 不要求所有註解變得更長；過時或冗餘註解可以在實作時移除，但不得改變程式行為。

## Decisions

1. 以「第一方維護區域」作為預設範圍。

   包含 `omb/`、`omfx/`、`omoba-core/`、`eui/`、`scripts/`、`omb-mcp/`、`map_editor/` 與專案自有工具。排除 `specs/`、`log4rs/` 這類 forked dependency，因為批次翻譯第三方或 forked 原始碼會放大 merge 成本，也可能讓後續 upstream sync 更困難。若某個 forked dependency 內有明確的 omoba-local patch 註解，應在實作時獨立標示並避免混入主批次。

2. 僅處理 lexical comment，不處理 string literal。

   實作時應搜尋語言對應的註解形式，例如 Rust `//`、`///`、`//!`、`/* */`，Lua `--`，JavaScript/TypeScript `//`、`/** */`，TOML `#`。即使字串內容看起來像註解，也不得翻譯，除非它確定只是 documentation-only fixture 且測試不依賴原文。

3. 保留技術名詞、symbol 與規格語彙原文。

   `GameWorld`、`UnitScript`、`AbilityScript`、`BuffStore`、`KCP`、`prost`、`snapshot`、`tick`、`Entity`、`SHALL` 等與程式、協定或既有 spec 直接相連的詞彙應保留原文或採中英混用。這比全面中文化更能避免讀者無法對應到實際 symbol。

4. 以分區 inventory 加人工審查取代一次性全域替換。

   全域 search 可以用來盤點註解分布，但翻譯應按目錄與語言分批執行。每批完成後檢查 diff，確認只有註解行變動，並補跑合適的 format/test。這能降低誤改 runtime string 或產生大規模不可審 diff 的風險。

5. 測試以「行為未變」為主要驗證。

   因為這是註解改動，主要驗證是 `git diff` 確認非註解內容未變，以及跑可負擔的 crate tests/build smoke。若某些完整 build 因環境或耗時不可行，tasks 需記錄未跑項目與原因。

## Risks / Trade-offs

- [Risk] 翻譯時誤改 string literal 或 test fixture，導致 runtime 行為或測試結果改變 → Mitigation: 分語言使用 comment-aware 審查，並用 diff 檢查非註解行是否變動。
- [Risk] 技術註解翻譯後與 code symbol 對應性變差 → Mitigation: 保留核心英文 symbol 與 protocol/API 名詞，只翻譯語意說明。
- [Risk] 大規模註解 diff 造成 review 困難 → Mitigation: 按第一方目錄分批提交實作，必要時將 docs-only 或 script-only 區域拆開。
- [Risk] 改到 forked dependency 後增加未來 upstream sync 成本 → Mitigation: 預設排除 `specs/`、`log4rs/`，除非後續另開 change 處理。

## Migration Plan

1. 盤點第一方程式碼註解分布，列出要處理與排除的路徑。
2. 按目錄與語言分批翻譯註解，保留英文 symbol/API/protocol 名詞。
3. 每批檢查 diff，確認變動限於註解或刪除過時註解。
4. 執行格式化與可負擔測試，最後產出未處理或需人工確認的註解清單。

Rollback 策略是回退本 change 的註解 diff；因不涉及資料 migration 或 runtime state，沒有部署順序需求。

## Open Questions

- 是否要在後續實作中包含 forked dependency 的 omoba-local patch 註解，或完全排除 `specs/`、`log4rs/`？目前設計預設完全排除。
- Markdown 文件中的一般段落是否要另外中文化？目前本 change 只處理程式註解與 code-adjacent comments，不批次翻譯文件正文。
