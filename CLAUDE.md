## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)

## Unit & Script API catalog (gen-docs)

每次想看「目前 base_content 裡有哪些單位 / ability，script API 長什麼樣，哪個 hook 有被 override」時：

1. 先確保 `base_content.dll` 已 build（`run.bat` 會自動 stage 到 `omb/scripts/base_content.dll`；或自己 `cargo build -p base_content --release` 於 `D:/omoba/scripts/` workspace）。
2. 產出 HTML：
   ```
   cd D:/omoba/omb
   cargo run -p omobab --bin gen-docs --features gen-docs --release
   ```
3. 開啟 `D:/omoba/omb/target/docs/index.html`（~100KB self-contained）。

內容：
- Towers / Heroes / Creeps 完整屬性
- UnitScript / AbilityScript / GameWorld 全部 API reference 含 doc comment
- Stat Keys 三分 section（通用 / 僅非建築物 / 視覺）
- Coverage Matrix：每個 unit × UnitScript hook 是否 override

Flags（通常不用）:
- `--dll <path>`：自訂 DLL 路徑（預設會試 5 個常見位置）
- `--story <name>`：override `game.toml` 的 STORY

Smoke test（跑一次完整 pipeline）：
```
cargo test -p omobab --features gen-docs -- --ignored
```

設計文件：`docs/plans/2026-04-23-build-time-catalog-design.md`
