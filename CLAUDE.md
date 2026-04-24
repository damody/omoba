# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 專案概觀

omoba 是 MOBA / TD 雙模的 Rust 遊戲，分 server（`omb`）與前端渲染器（`omfx`）兩個 submodule。場景／英雄／塔／技能行為由獨立的 cdylib（`scripts/base_content.dll`）以 abi_stable FFI 載入 — **一次建置需要跑兩次 cargo**（scripts workspace + omb workspace），所以常用動作都以 `.bat` 腳本包好。

## Toolchain

固定於 **Rust 1.91.0**（`rust-toolchain.toml`）。abi_stable 要求 host 與 script DLL 用同一個 rustc，所以不能隨便升 channel 或在某個 workspace 用不同 toolchain。

## 常用指令

全部從 repo 根目錄（`D:\omoba`）在 Windows cmd 下跑：

| 情境 | 指令 |
|---|---|
| 一般 dev run | `run.bat` — debug 編 base_content.dll + omb + omfx，frontend 自動 spawn backend child |
| Stress 壓測 | `run_stress.bat` — release 編 + 生 stress map + 暫時 swap `omb/game.toml` 到 stress variant，結束自動還原 |
| 生 unit / script API catalog HTML | `gen_docs.bat` → 開 `omb/target/docs/index.html` |
| 單 crate 測試 | `cargo test --manifest-path omb/Cargo.toml -p omobab` 或 `... -p omb-script-abi`；scripts 測試用 `--manifest-path scripts/Cargo.toml -p base_content` |
| 手動單步 build | `cargo build --manifest-path scripts/Cargo.toml -p base_content` → copy `scripts/target/{debug,release}/base_content.dll` 到 `omb/scripts/` → `cargo build --manifest-path omb/Cargo.toml -p omobab` |

**.bat 檔必須是 CRLF 行尾**（Windows cmd 遇 LF-only 會把每行首字吃掉 → `'M' is not recognized`）。用 Write tool 新建 `.bat` 後要用 PowerShell 轉換：
```
$p = 'D:\omoba\xxx.bat'; $c = (Get-Content -Raw $p) -replace "(?<!`r)`n","`r`n"; [System.IO.File]::WriteAllText($p, $c, (New-Object System.Text.UTF8Encoding $false))
```

## 目錄架構

**Submodules（獨立 git repo，commit 要進該 repo 再 bump 指標）**：
- `omb/` — 後端，bin 名 `omobab`（位址：github.com/damody/open_moba_backend）
- `omfx/` — 前端（Fyrox 引擎），bin `executor` 是真正跑起來的 entry，內部會 `spawn target/debug/omobab.exe` 當子行程（**hard-coded debug 路徑**；stress 腳本用 release 編完再 copy 過去繞過）
- `map_editor/`、`specs/`（forked）、`log4rs/`（forked）

**非 submodule 的 repo 目錄**：
- `omoba-core/` — client / server 共用 schema：`tower_meta.rs`（48 升級定義型別）、`ability_meta.rs`、`grpc/` / `kcp/` client 實作、`GameEventData` 含 `payload_bytes` 是 transport 層帶過來的吞吐量欄位
- `eui/` — omfx 用的 immediate-mode GUI
- `scripts/base_content/src/{towers,heroes,summons}/` — 所有塔 / 英雄 / 召喚物 script 實作（這是個獨立 cargo workspace，**不是** omb 的 member）
- `omb-mcp/` — MCP server，用 KCP query-only 連 omb，不訂閱 event 洪水
- `proto/game.proto` — prost / tonic 共用 schema（build.rs 按 feature 編）
- `docs/plans/` — 各功能的 design + impl plan（目前 tower-upgrade-paths、ability-system-integration 等有詳細文件）
- `graphify-out/` — 本 repo 的 knowledge graph（見下）

> **注意**：`omb/` 單獨 clone 無法 build，需搭配完整 monorepo（`omb-script-abi` 經 path 依賴 `../scripts/script-abi`）。

## 核心架構要點

### Traits / 生命週期
- **ECS**：specs 0.20（從 0.19 遷移過；`#[derive(Component)]` 從 `use specs::Component;` 來，不是舊的 `specs_derive`）。`Entity` 的 Serialize/Deserialize 是 fork 裡手加的。
- **Script ABI**：`scripts/script-abi` 是 host + cdylib 的**唯一**共用 crate，只能用 abi_stable 型別 — 不要在這拉 specs、serde_json 等。主要 trait：`UnitScript`（塔 / 英雄 / 怪 tick + attack hook）、`AbilityScript`（Q/W/E/R 施放）、`GameWorld`（script 回呼 host 的 FFI）。`stat_keys` 模組是 Dota 2 modifier property 對齊的 key 字串常數。
- **Ability runtime**：`omb/src/ability_runtime/` — `BuffStore`（entity → buff list；`sum_add` / `product_mult` 聚合；payload 任意 JSON，慣例 `*_bonus` = additive、`*_multiplier` = multiplicative）、`UnitStats` helper、`Dispatcher` 快取。

### 傳輸層
三個 feature 互斥：`mqtt` / `grpc` / `kcp`（**default = kcp**），port 50061。KCP 協定是 `[1B tag][4B len BE][prost payload]`，tag 0x01–0x06 對應 PlayerCommand / GameEvent / CommandAck / Subscribe / StateReq / StateResp。`omb/src/transport/` 抽象 `OutboundMsg` / `InboundMsg` / `TransportHandle`，同步 runtime 是 tokio（不是 async-std）。

### 遊戲模式
`omb/game.toml` 的 `STORY` 欄位切 scene 資料夾（`omb/Story/{MVP_1,TD_1,DEBUG_1,TD_STRESS,...}`）；runtime 由 `GameMode`、`PlayerLives`、`TowerKind` 等 resource 控制 TD 波控 / 漏怪 / Ice 減速行為。

### Hero stats 廣播
`omb` 每 0.3 秒對每個 Hero 廣播一次 `hero.stats` snapshot，payload 已聚合 BuffStore 的 `_bonus` / `_multiplier` 到實際生效屬性 + `buffs` 陣列含 `remaining` 倒數（-1 代表 toggle 型無限期）。前端本地每 frame 遞減 `remaining`、下次 backend push 重設權威值避免漂移。四個廣播 site 共用 `build_hero_stats_payload()` builder（`omb/src/state/resource_management.rs` 末段）。

### 效能注意
stress 場景（1000 塔 × 1000 creep）已驗證兩個熱路徑：(1) omfx 的 collision ring（24 段 × 1000 entity = 24K scene node）用 `COLLISION_RING_ENABLED: bool` 全關；(2) name label 的 `ui.send` 做 diff（位置差 < 1 px + 文字未變就跳過）。新加 per-entity 每幀 UI 動作時務必檢查這些節流模式。

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
