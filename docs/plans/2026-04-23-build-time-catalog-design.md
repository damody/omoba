# Build-Time Unit & Script API Catalog — Design

**Date:** 2026-04-23
**Status:** Design approved, ready for implementation plan
**Scope:** 新增一個 dev tool，每次編譯後產生一份單檔 HTML，列出目前 base_content.dll 提供的所有單位、屬性、以及 script-abi 暴露的 API。

## Goals

開發者改完 `base_content.dll` 或 `script-abi` 後，能立刻看到：

1. 當前所有單位種類（塔 / 英雄 / 敵人）＋ 完整屬性
2. Script API 完整 reference（UnitScript / AbilityScript / GameWorld / stat_keys）
3. 覆蓋矩陣：每個單位 override 了哪些 hook、呼叫了哪些 GameWorld API

Non-goals：PDF 輸出、多語系、互動式 diff、watch mode、進 git commit。

## Architecture

### 觸發方式

獨立 binary，不進 `build.rs`：

```bash
cargo run -p omb --bin gen-docs --release -- \
    --out target/docs/index.html \
    --story TD_1 \
    --dll target/release/deps/base_content.dll
```

所有 flag 有預設：`--out` → `target/docs/index.html`；`--story` 讀 `omb/game.toml` 的 `[server].STORY`；`--dll` 依 `--release` 或 debug 推算路徑。

理由：`build.rs` 無法 `dlopen` 正在 build 的 cdylib；獨立 binary 可載入真實 DLL 拿到 `tower_metadata()` 運行時數值。

### 資料源

| 資料 | 來源 | 抽取方式 |
|---|---|---|
| 塔 metadata + ability defs | `base_content.dll` | `abi_stable::library::RootModule::load_from_file()` → `units()` / `abilities()` |
| Hero / Creep base stats | `omb/Story/<STORY>/entity.json` | `serde_json` |
| UnitScript / AbilityScript / GameWorld API | `omb/script-abi/src/{script,ability,world}.rs` | `syn` AST parse |
| Stat keys | `omb/script-abi/src/stat_keys.rs` | `syn` AST（ItemConst + doc attrs）|
| 覆蓋矩陣 | `omb/scripts/base_content/src/*.rs` | `syn` AST，visit `ItemImpl where trait_ == UnitScript`，body walk `ExprMethodCall` |

不改 ABI、不搬現有資料。

### 輸出

單檔 self-contained HTML，CSS / JS 全 inline，目標 ~200KB 內。輸出到 `target/docs/index.html`（不進 git）。

## Components

### Crate 依賴新增（gated on `gen-docs` bin）

- `maud ≈ 0.26` — HTML DSL，compile-time
- `syn ≈ 2`（full）— 解析 trait / impl block
- `clap ≈ 4` — CLI
- `serde_json`、`abi_stable` — already present

### 檔案結構

```
omb/src/bin/gen_docs.rs         ← main, CLI, 協調
omb/src/bin/gen_docs/
    model.rs       共用型別（UnitEntry, ApiSpec, ApiMethod, StatKey, ...）
    dll.rs         DLL loading → UnitInfo / AbilityInfo
    entity.rs      entity.json → HeroInfo / CreepInfo
    api_scan.rs    script-abi syn parse → ApiSpec
    coverage.rs    base_content syn parse → CoverageMatrix
    render.rs      maud 拼 HTML
```

### 核心型別（model.rs）

```rust
pub struct UnitEntry {
    id: String,
    kind: UnitKind,                 // Tower / Hero / Creep / Unknown
    label: Option<String>,
    tower: Option<TowerMetadata>,   // from DLL
    hero:  Option<HeroInfo>,        // from entity.json
    creep: Option<CreepInfo>,
    abilities: Vec<String>,         // ability id refs
    overrides:   Vec<String>,       // overridden UnitScript hook names
    world_calls: BTreeSet<String>,  // GameWorld methods this unit calls
    source_file: Option<String>,    // relative path in base_content
}

pub struct ApiSpec {
    unit_hooks:    Vec<ApiMethod>,
    ability_hooks: Vec<ApiMethod>,
    world_methods: Vec<ApiMethod>,  // 含 group tag
    stat_keys:     Vec<StatKey>,
}

pub struct ApiMethod {
    name: String,
    signature: String,   // rendered from syn::Signature
    doc: String,         // 合併所有 #[doc] attrs
    group: ApiGroup,     // Query / Mutate / Tower / Stats / RNG / Log / VFX
}
```

### api_scan 要點

- `GameWorld` 方法按源碼中的 `// ---- Query ----` 類 section header 分組（visitor 用 line-number 歸組）
- `stat_keys.rs` 同樣方法分三大 SECTION + 子 section
- doc 提取：`attrs.iter().filter_map(|a| a.meta.require_name_value("doc"))` 合併

### coverage 要點

- visit `ItemImpl`，判斷 `trait_.path.segments.last().ident == "UnitScript"`（or `AbilityScript`），`self_ty` 當單位 key
- body 用 `syn::visit::Visit::visit_expr_method_call`，receiver 匹配 `world` / `w` / `_w` → 收集 method name
- 用 `ApiSpec.world_methods` 的 name set filter，排除 `.into()`、`.clone()` 等無關 call

## HTML Layout

```
┌─ header: 專案名 · build timestamp · git sha · story name ─────────────┐
├─ sidebar (sticky 260px)                main (flex, scrollable)────────┤
│  § Units / Abilities / UnitScript /    § 各 section card 渲染         │
│    AbilityScript / GameWorld API /                                    │
│    Stat Keys / Coverage Matrix                                        │
└─ top bar: 🔍 search | □ show only used | □ dark mode ─────────────────┘
```

### 各區塊渲染

| 區塊 | 內容 |
|---|---|
| Tower | card：label + id + 屬性表（atk/range/asd/cost/...）+ abilities tag + source file + collapsible overrides / world calls |
| Hero | name/title/主屬性/三圍/level_growth + abilities |
| Ability | name + cooldown/mana/range + level data table + declaring file |
| Hook | `fn on_tick(...)` + doc + 「implemented by: X units」連結 |
| GameWorld method | 同 Hook；group 用 `<details>` |
| Stat key | table：const name / string value / doc / tag |
| Coverage matrix | 大 table：row=unit, col=hook；cell ✓；sticky 首 row/col |

### Inline JS（≤50 行）

- search：substring match 過濾 card `data-id` / `data-name`
- "show only used"：淡化沒人 impl 的 hook
- dark mode：toggle `<body class="dark">`

### Footer

build 時間、git rev、STORY 名稱、資料源檔列表（方便 PR review 時看差異）。

## Error Handling

| 類別 | 例子 | 行為 |
|---|---|---|
| **Fatal** | DLL 找不到；Manifest load 失敗；`script-abi` 源碼 syn 解析失敗 | `anyhow::bail!`，exit 1 |
| **Soft** | `entity.json` 缺欄位；某 `.rs` 檔解析失敗；塔 `tower_metadata()` 回 `RNone` | 收到 `Vec<Warning>`，render 到 HTML 頂部紅底警告區塊，exit 0 |

理由：dev tool 不該因一個 typo 就產不出文件；但警告放顯眼位置確保被注意到。

## Main Flow

```
1. parse CLI → Config { dll_path, story, out_path }
2. let dll = load_manifest(&dll_path)?                    // fatal
3. let api = api_scan::scan("omb/script-abi/src")?        // fatal
4. let coverage = coverage::scan(
       "omb/scripts/base_content/src", &api
   ).unwrap_or_else(|e| { warnings.push(e); empty });
5. let entity = entity::load(&story_path)
       .unwrap_or_else(|e| { warnings.push(e); empty });
6. let model = merge(dll, api, coverage, entity, warnings, git_meta());
7. let html = render::page(&model);
8. fs::create_dir_all(out.parent())?; fs::write(out, html)?;
9. println!("generated {out} ({U} units, {A} abilities, {W} warnings)");
```

## Testing

- `api_scan.rs` unit test：餵 fake `UnitScript` trait 源碼，驗 `ApiMethod` list、method count、doc 非空、group tag
- `coverage.rs` unit test：餵 fake `impl UnitScript for Foo { fn on_tick ... world.deal_damage ... }`，驗 `overrides` 含 `on_tick`、`world_calls` 含 `deal_damage`
- `render.rs` 不寫測試，目視驗證
- 整合冒煙測試：`cargo run --bin gen-docs`，assert exit 0 + output file 存在 + 內含 `"saika_magoichi"`

## Open Questions / Future Work

- 是否把 output 推到 GitHub Pages？目前先不做，等 dev tool 穩定再說
- 是否加 diff 模式（build 前後比對）？等真有 PR review 痛點再說

## Decision Log

| Q | Decision | Rationale |
|---|---|---|
| 觸發時機 | 獨立 binary `cargo run --bin gen-docs` | `build.rs` 無法 dlopen 自己 build 的 cdylib |
| 內容範圍 | A (單位) + B (API ref) + C (覆蓋矩陣) | 全部都要；驗證報告 (D) 另開 lint 做 |
| 輸出格式 | 單檔 self-contained HTML，maud | 好 diff、無外部 tooling、雙擊即開 |
| 覆蓋矩陣抽法 | `syn` AST parse | 比 grep 精準、比 rustdoc JSON 穩定 |
| Hero/creep 屬性源 | `Story/<STORY>/entity.json` + DLL | 不改 ABI，用現有資料 |
| Binary 位置 / 輸出路徑 | `omb/src/bin/gen_docs.rs` → `target/docs/index.html` | 不另開 xtask crate；不進 git |
