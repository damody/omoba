# Build-Time Unit & Script API Catalog — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 新增 `gen-docs` binary，產出一份 self-contained HTML，列出目前 base_content.dll 提供的所有單位、屬性與 script-abi 的完整 API + 覆蓋矩陣。

**Architecture:** 獨立 binary `omb/src/bin/gen_docs.rs`；混合三個資料源（DLL runtime load、`Story/<STORY>/entity.json`、`syn` AST parse of script-abi + base_content）；用 `maud` compile-time DSL 輸出單檔 HTML。

**Tech Stack:** Rust + `maud 0.26` (HTML DSL) + `syn 2` (AST) + `clap 4` (CLI) + `abi_stable` + `serde_json`.

**Design doc:** `docs/plans/2026-04-23-build-time-catalog-design.md`

---

## Task 1 — Cargo 設定與空 binary

**Files:**
- Modify: `omb/Cargo.toml`
- Create: `omb/src/bin/gen_docs.rs`

**Step 1 — 修改 `omb/Cargo.toml`**

在 `[dependencies]` 末尾加入（保留其他 deps 不動）：

```toml
maud = { version = "0.26", optional = true }
syn = { version = "2", features = ["full", "visit", "parsing"], optional = true }
clap = { version = "4", features = ["derive"], optional = true }
anyhow = { version = "1", optional = true }
```

把 `[features]` section 末尾加入：

```toml
gen-docs = ["maud", "syn", "clap", "anyhow"]
```

`[[bin]]` section 後面（第 13 行之後）加入：

```toml
[[bin]]
name = "gen-docs"
path = "src/bin/gen_docs.rs"
required-features = ["gen-docs"]
```

**Step 2 — 建立 `omb/src/bin/gen_docs.rs`**（先放 placeholder）：

```rust
//! gen-docs — produce a self-contained HTML catalog of units, abilities,
//! and script API coverage. Design: docs/plans/2026-04-23-build-time-catalog-design.md

fn main() -> anyhow::Result<()> {
    println!("gen-docs placeholder");
    Ok(())
}
```

**Step 3 — 驗證 binary 能編能跑**

Run:
```
cargo build -p omobab --bin gen-docs --features gen-docs
cargo run -p omobab --bin gen-docs --features gen-docs
```

Expected: binary 成功編出；執行印出 `gen-docs placeholder`；主 binary `omobab` 未加 `--features gen-docs` 仍能 build（`cargo build -p omobab --bin omobab` 應該不拉 maud/syn/clap）。

**Step 4 — Commit**

```bash
git add omb/Cargo.toml omb/src/bin/gen_docs.rs
git commit -m "feat(gen-docs): add binary skeleton & feature-gated deps"
```

---

## Task 2 — 共用資料型別 `model.rs`

**Files:**
- Create: `omb/src/bin/gen_docs_lib/mod.rs`
- Create: `omb/src/bin/gen_docs_lib/model.rs`
- Modify: `omb/src/bin/gen_docs.rs`

> **Note:** bin 要分多 module 一般用 `src/bin/<name>/main.rs + submodules`，但本 repo 已定 `path = "src/bin/gen_docs.rs"`。改用 `mod` 指向 sibling 資料夾 `gen_docs_lib/`，在 `gen_docs.rs` 頂部 `#[path = "gen_docs_lib/mod.rs"] mod lib;` 即可。

**Step 1 — 建立 `omb/src/bin/gen_docs_lib/mod.rs`**

```rust
pub mod model;
```

**Step 2 — 建立 `omb/src/bin/gen_docs_lib/model.rs`**

```rust
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnitKind { Tower, Hero, Creep, Unknown }

#[derive(Debug, Clone, Default)]
pub struct TowerStats {
    pub atk: f32,
    pub asd_interval: f32,
    pub range: f32,
    pub bullet_speed: f32,
    pub splash_radius: f32,
    pub hit_radius: f32,
    pub slow_factor: f32,
    pub slow_duration: f32,
    pub cost: i32,
    pub footprint: f32,
    pub hp: f32,
    pub turn_speed_deg: f32,
    pub label: String,
}

#[derive(Debug, Clone, Default)]
pub struct HeroInfo {
    pub name: String,
    pub title: String,
    pub background: String,
    pub strength: f32,
    pub agility: f32,
    pub intelligence: f32,
    pub primary_attribute: String,
    pub attack_range: f32,
    pub base_damage: f32,
    pub base_armor: f32,
    pub base_hp: f32,
    pub base_mana: f32,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub abilities: Vec<String>,
    pub level_growth: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct CreepInfo {
    pub name: String,
    pub enemy_type: String,
    pub hp: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub ai_type: String,
    pub abilities: Vec<String>,
    pub exp_reward: i32,
    pub gold_reward: i32,
}

#[derive(Debug, Clone)]
pub struct UnitEntry {
    pub id: String,
    pub kind: UnitKind,
    pub label: Option<String>,
    pub tower: Option<TowerStats>,
    pub hero: Option<HeroInfo>,
    pub creep: Option<CreepInfo>,
    pub abilities: Vec<String>,
    pub overrides: Vec<String>,
    pub world_calls: BTreeSet<String>,
    pub source_file: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AbilityEntry {
    pub id: String,
    pub def_json: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiGroup {
    UnitHook,
    AbilityHook,
    WorldQuery,
    WorldMutate,
    WorldTower,
    WorldStats,
    WorldRng,
    WorldLog,
    WorldVfx,
    StatKey(StatSection),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatSection {
    All,          // Section 1: 全單位通用
    NonBuilding,  // Section 2
    Visual,       // Section 3
}

#[derive(Debug, Clone)]
pub struct ApiMethod {
    pub name: String,
    pub signature: String,
    pub doc: String,
    pub group: ApiGroup,
    pub sub_group: Option<String>, // e.g. "攻速 / BAT" sub-section header
}

#[derive(Debug, Clone)]
pub struct StatKey {
    pub const_name: String,
    pub string_value: String,
    pub doc: String,
    pub section: StatSection,
    pub sub_group: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ApiSpec {
    pub unit_hooks: Vec<ApiMethod>,
    pub ability_hooks: Vec<ApiMethod>,
    pub world_methods: Vec<ApiMethod>,
    pub stat_keys: Vec<StatKey>,
}

#[derive(Debug, Clone)]
pub struct Warning {
    pub source: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct BuildMeta {
    pub timestamp: String,
    pub git_sha: String,
    pub story: String,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Catalog {
    pub units: Vec<UnitEntry>,
    pub abilities: Vec<AbilityEntry>,
    pub api: ApiSpec,
    pub warnings: Vec<Warning>,
    pub meta: BuildMeta,
}
```

**Step 3 — 修改 `omb/src/bin/gen_docs.rs`** 引入 module：

```rust
//! gen-docs — produce a self-contained HTML catalog of units, abilities,
//! and script API coverage. Design: docs/plans/2026-04-23-build-time-catalog-design.md

#[path = "gen_docs_lib/mod.rs"]
mod lib;

use lib::model;

fn main() -> anyhow::Result<()> {
    // Touch the types so unused warnings don't trip us
    let _ = model::ApiSpec::default();
    println!("gen-docs placeholder (model wired)");
    Ok(())
}
```

**Step 4 — 驗證 build**

Run: `cargo build -p omobab --bin gen-docs --features gen-docs`
Expected: PASS

**Step 5 — Commit**

```bash
git add omb/src/bin/
git commit -m "feat(gen-docs): add shared model types (UnitEntry/ApiSpec/Catalog)"
```

---

## Task 3 — `entity.rs` 讀 Story JSON（TDD）

**Files:**
- Create: `omb/src/bin/gen_docs_lib/entity.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`
- Test: inline `#[cfg(test)] mod tests` 在 entity.rs

**Step 1 — 先在 `entity.rs` 寫 failing test**

```rust
//! Load hero / creep base stats from omb/Story/<STORY>/entity.json.

use crate::lib::model::{HeroInfo, CreepInfo};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub struct EntityData {
    pub heroes: BTreeMap<String, HeroInfo>,
    pub creeps: BTreeMap<String, CreepInfo>,
}

pub fn load(story_dir: &Path) -> Result<EntityData> {
    let path = story_dir.join("entity.json");
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("reading {}", path.display()))?;
    // 容錯：entity.json 有 // 註解，serde_json 不吃，先 strip
    let cleaned = strip_line_comments(&raw);
    let v: serde_json::Value = serde_json::from_str(&cleaned)
        .with_context(|| format!("parsing {}", path.display()))?;
    parse(v)
}

fn strip_line_comments(src: &str) -> String {
    src.lines()
        .map(|l| {
            // 只 strip 以 "    //" 或開頭 "//" 起頭的整行 comment，
            // 字串內含 "//" 的行維持不動
            let trimmed = l.trim_start();
            if trimmed.starts_with("//") { "" } else { l }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse(v: serde_json::Value) -> Result<EntityData> {
    let mut heroes = BTreeMap::new();
    let mut creeps = BTreeMap::new();
    if let Some(arr) = v.get("heroes").and_then(|x| x.as_array()) {
        for h in arr {
            let id = h.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if id.is_empty() { continue; }
            heroes.insert(id.clone(), parse_hero(h));
        }
    }
    if let Some(arr) = v.get("enemies").and_then(|x| x.as_array()) {
        for c in arr {
            let id = c.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
            if id.is_empty() { continue; }
            creeps.insert(id.clone(), parse_creep(c));
        }
    }
    Ok(EntityData { heroes, creeps })
}

fn f(v: &serde_json::Value, key: &str) -> f32 {
    v.get(key).and_then(|x| x.as_f64()).unwrap_or(0.0) as f32
}
fn s(v: &serde_json::Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").to_string()
}
fn i(v: &serde_json::Value, key: &str) -> i32 {
    v.get(key).and_then(|x| x.as_i64()).unwrap_or(0) as i32
}
fn arr_str(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key).and_then(|x| x.as_array())
        .map(|a| a.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
        .unwrap_or_default()
}

fn parse_hero(v: &serde_json::Value) -> HeroInfo {
    HeroInfo {
        name: s(v, "name"),
        title: s(v, "title"),
        background: s(v, "background"),
        strength: f(v, "strength"),
        agility: f(v, "agility"),
        intelligence: f(v, "intelligence"),
        primary_attribute: s(v, "primary_attribute"),
        attack_range: f(v, "attack_range"),
        base_damage: f(v, "base_damage"),
        base_armor: f(v, "base_armor"),
        base_hp: f(v, "base_hp"),
        base_mana: f(v, "base_mana"),
        move_speed: f(v, "move_speed"),
        turn_speed: f(v, "turn_speed"),
        abilities: arr_str(v, "abilities"),
        level_growth: v.get("level_growth").cloned().unwrap_or(serde_json::Value::Null),
    }
}

fn parse_creep(v: &serde_json::Value) -> CreepInfo {
    CreepInfo {
        name: s(v, "name"),
        enemy_type: s(v, "enemy_type"),
        hp: f(v, "hp"),
        armor: f(v, "armor"),
        magic_resistance: f(v, "magic_resistance"),
        damage: f(v, "damage"),
        attack_range: f(v, "attack_range"),
        move_speed: f(v, "move_speed"),
        ai_type: s(v, "ai_type"),
        abilities: arr_str(v, "abilities"),
        exp_reward: i(v, "exp_reward"),
        gold_reward: i(v, "gold_reward"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comments_keeps_strings() {
        let src = "{\n  // hello\n  \"a\": 1,\n  \"url\": \"http://x\"\n}";
        let out = strip_line_comments(src);
        assert!(!out.contains("hello"));
        assert!(out.contains("http://x"));
    }

    #[test]
    fn parses_hero_and_creep() {
        let raw = r#"{
            "heroes": [{"id":"h1","name":"Hero","base_hp":500,"abilities":["a"]}],
            "enemies": [{"id":"c1","name":"Creep","hp":300,"damage":20}]
        }"#;
        let d = parse(serde_json::from_str(raw).unwrap()).unwrap();
        let h = d.heroes.get("h1").unwrap();
        assert_eq!(h.name, "Hero");
        assert_eq!(h.base_hp, 500.0);
        assert_eq!(h.abilities, vec!["a".to_string()]);
        let c = d.creeps.get("c1").unwrap();
        assert_eq!(c.hp, 300.0);
        assert_eq!(c.damage, 20.0);
    }
}
```

**Step 2 — `mod.rs` 加 `pub mod entity;`**

```rust
pub mod model;
pub mod entity;
```

**Step 3 — 跑 test（應該 fail 一次再 pass；直接實作所以直接跑應 pass）**

Run: `cargo test -p omobab --bin gen-docs --features gen-docs entity::tests`
Expected: PASS (`test entity::tests::strip_comments_keeps_strings ... ok`, `test entity::tests::parses_hero_and_creep ... ok`)

**Step 4 — 額外 integration check（非 required）**：跑一次真實 `Story/TD_1/entity.json`

在 `gen_docs.rs` main 裡暫時加：
```rust
let d = lib::entity::load(std::path::Path::new("omb/Story/TD_1"))?;
println!("loaded {} heroes, {} creeps", d.heroes.len(), d.creeps.len());
```
Run: `cargo run -p omobab --bin gen-docs --features gen-docs`
Expected: 印出 heroes/creeps 數量 > 0。驗證後把這行刪掉。

**Step 5 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/ omb/src/bin/gen_docs.rs
git commit -m "feat(gen-docs): load hero/creep from Story entity.json"
```

---

## Task 4 — `dll.rs` 載入 base_content 拿 units/abilities

**Files:**
- Create: `omb/src/bin/gen_docs_lib/dll.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`

**Step 1 — 建立 `dll.rs`**

```rust
//! Load base_content.dll via abi_stable, extract unit ids + tower_metadata
//! and ability definitions.

use crate::lib::model::{AbilityEntry, TowerStats, UnitKind};
use abi_stable::library::RootModule;
use anyhow::{Context, Result};
use omb_script_abi::manifest::Manifest_Ref;
use std::path::Path;

pub struct DllData {
    pub units: Vec<DllUnit>,
    pub abilities: Vec<AbilityEntry>,
}

pub struct DllUnit {
    pub id: String,
    pub kind: UnitKind,           // Tower if tower_metadata is Some, else Unknown
    pub tower: Option<TowerStats>,
}

pub fn load(dll_path: &Path) -> Result<DllData> {
    let m = Manifest_Ref::load_from_file(dll_path)
        .with_context(|| format!("loading manifest from {}", dll_path.display()))?;
    let units_fn = m.units();
    let abilities_fn = m.abilities();

    let mut units = Vec::new();
    for def in units_fn() {
        let id = def.unit_id.to_string();
        let tower = def.script.tower_metadata().into_option().map(|tm| TowerStats {
            atk: tm.atk,
            asd_interval: tm.asd_interval,
            range: tm.range,
            bullet_speed: tm.bullet_speed,
            splash_radius: tm.splash_radius,
            hit_radius: tm.hit_radius,
            slow_factor: tm.slow_factor,
            slow_duration: tm.slow_duration,
            cost: tm.cost,
            footprint: tm.footprint,
            hp: tm.hp,
            turn_speed_deg: tm.turn_speed_deg,
            label: tm.label.to_string(),
        });
        let kind = if tower.is_some() { UnitKind::Tower } else { UnitKind::Unknown };
        units.push(DllUnit { id, kind, tower });
    }

    let mut abilities = Vec::new();
    for a in abilities_fn() {
        let json_str = a.def_json.to_string();
        let v: serde_json::Value = serde_json::from_str(&json_str).unwrap_or(serde_json::Value::Null);
        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
        abilities.push(AbilityEntry { id, def_json: v });
    }

    Ok(DllData { units, abilities })
}
```

**Step 2 — `mod.rs` 加入**

```rust
pub mod model;
pub mod entity;
pub mod dll;
```

**Step 3 — 冒煙測試**：在 `gen_docs.rs` main 裡暫時加：

```rust
let dll_path = std::path::Path::new("target/release/base_content.dll");
let d = lib::dll::load(dll_path)?;
println!("dll: {} units, {} abilities, first tower id={:?}",
    d.units.len(), d.abilities.len(),
    d.units.iter().find(|u| matches!(u.kind, model::UnitKind::Tower)).map(|u| &u.id));
```

Run: `cd /d D:\omoba && cargo build --release -p base_content` （先確保 DLL 存在；專案可能已有 run.bat 流程）然後：
`cargo run -p omobab --bin gen-docs --features gen-docs`
Expected: 印出 units 數量 > 0、abilities 數量 > 0、至少一個 tower id。

**Step 4 — 移除冒煙 code，保留 `dll.rs`**

**Step 5 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/
git commit -m "feat(gen-docs): load units/abilities from base_content DLL"
```

---

## Task 5 — `api_scan.rs` 解析 trait：UnitScript / AbilityScript / GameWorld

**Files:**
- Create: `omb/src/bin/gen_docs_lib/api_scan.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`

**Step 1 — 先寫 failing test**

在 `api_scan.rs` 最下方：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const FAKE: &str = r#"
        pub trait UnitScript: Send + Sync {
            /// Called once when the entity is spawned.
            fn on_spawn(&self, _e: EntityHandle, _w: &mut GameWorldDyn<'_>) {}
            /// Called every tick.
            /// `dt` is the tick delta in seconds.
            fn on_tick(&self, _e: EntityHandle, _dt: f32, _w: &mut GameWorldDyn<'_>) {}
        }
    "#;

    #[test]
    fn extracts_unit_hooks_with_docs() {
        let hooks = scan_trait(FAKE, "UnitScript", crate::lib::model::ApiGroup::UnitHook).unwrap();
        assert_eq!(hooks.len(), 2);
        assert_eq!(hooks[0].name, "on_spawn");
        assert!(hooks[0].doc.contains("spawned"));
        assert_eq!(hooks[1].name, "on_tick");
        assert!(hooks[1].doc.contains("tick delta"));
        assert!(hooks[0].signature.contains("on_spawn"));
    }
}
```

**Step 2 — 實作 `api_scan.rs`**

```rust
//! Parse script-abi source files using syn to extract ApiMethod lists and
//! StatKey tables for the reference section.

use crate::lib::model::{ApiGroup, ApiMethod, ApiSpec, StatKey, StatSection};
use anyhow::{Context, Result};
use std::path::Path;
use syn::{File, Item, ItemTrait, TraitItem, TraitItemFn};

pub fn scan(abi_src_dir: &Path) -> Result<ApiSpec> {
    let script_src = std::fs::read_to_string(abi_src_dir.join("script.rs"))?;
    let ability_src = std::fs::read_to_string(abi_src_dir.join("ability.rs"))?;
    let world_src = std::fs::read_to_string(abi_src_dir.join("world.rs"))?;
    let stat_src = std::fs::read_to_string(abi_src_dir.join("stat_keys.rs"))?;

    let unit_hooks = scan_trait(&script_src, "UnitScript", ApiGroup::UnitHook)?;
    let ability_hooks = scan_trait(&ability_src, "AbilityScript", ApiGroup::AbilityHook)?;
    let world_methods = scan_world(&world_src)?;
    let stat_keys = scan_stat_keys(&stat_src)?;

    Ok(ApiSpec { unit_hooks, ability_hooks, world_methods, stat_keys })
}

pub fn scan_trait(src: &str, trait_name: &str, group: ApiGroup) -> Result<Vec<ApiMethod>> {
    let file: File = syn::parse_str(src).context("parse trait file")?;
    let mut out = Vec::new();
    for item in &file.items {
        if let Item::Trait(t) = item {
            if t.ident == trait_name {
                for ti in &t.items {
                    if let TraitItem::Fn(f) = ti {
                        out.push(method_from_trait_item(f, group, None));
                    }
                }
            }
        }
    }
    Ok(out)
}

fn method_from_trait_item(f: &TraitItemFn, group: ApiGroup, sub: Option<String>) -> ApiMethod {
    ApiMethod {
        name: f.sig.ident.to_string(),
        signature: render_sig(&f.sig),
        doc: extract_doc(&f.attrs),
        group,
        sub_group: sub,
    }
}

fn render_sig(sig: &syn::Signature) -> String {
    use quote::ToTokens;
    let mut s = String::new();
    sig.to_tokens(&mut s.parse::<proc_macro2::TokenStream>().unwrap_or_default().into());
    // 用 prettyplease 或直接 to_string
    sig.to_token_stream().to_string()
}

fn extract_doc(attrs: &[syn::Attribute]) -> String {
    let mut lines = Vec::new();
    for a in attrs {
        if a.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &a.meta {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) = &nv.value {
                    let v = s.value();
                    lines.push(v.trim_start().to_string());
                }
            }
        }
    }
    lines.join("\n")
}

pub fn scan_world(src: &str) -> Result<Vec<ApiMethod>> {
    // GameWorld trait 的 method 數量多，源碼用 `// ---- Query ----` 等 section header 分組。
    // syn parse 看不到 comment（除了 doc comment），所以我們用 line-number 做後處理：
    //   1. 先 syn parse 拿到每個 method 的起始 line
    //   2. 再用正則掃描原文找 `// ---- <name> ----` 的行號
    //   3. 每個 method 歸屬於「在它之前最近的 header」
    use regex::Regex;
    let file: File = syn::parse_str(src).context("parse world.rs")?;
    let headers: Vec<(usize, String)> = {
        let re = Regex::new(r"^\s*//\s*----\s*(.+?)\s*----").unwrap();
        src.lines().enumerate()
            .filter_map(|(i, l)| re.captures(l).map(|c| (i + 1, c[1].trim().to_string())))
            .collect()
    };
    let pick_header = |line: usize| -> Option<String> {
        headers.iter().rev().find(|(h, _)| *h <= line).map(|(_, n)| n.clone())
    };
    let group_of = |hdr: &str| -> ApiGroup {
        let l = hdr.to_ascii_lowercase();
        if l.contains("query") { ApiGroup::WorldQuery }
        else if l.contains("mutate") { ApiGroup::WorldMutate }
        else if l.contains("tower") || l.contains("單位屬性") { ApiGroup::WorldTower }
        else if l.contains("rng") || l.contains("deterministic") { ApiGroup::WorldRng }
        else if l.contains("log") { ApiGroup::WorldLog }
        else if l.contains("vfx") || l.contains("side effect") { ApiGroup::WorldVfx }
        else { ApiGroup::WorldStats }
    };

    let mut out = Vec::new();
    for item in &file.items {
        if let Item::Trait(t) = item {
            if t.ident == "GameWorld" {
                for ti in &t.items {
                    if let TraitItem::Fn(f) = ti {
                        use syn::spanned::Spanned;
                        let line = f.span().start().line;
                        let hdr = pick_header(line);
                        let grp = hdr.as_deref().map(group_of).unwrap_or(ApiGroup::WorldStats);
                        out.push(method_from_trait_item(f, grp, hdr));
                    }
                }
            }
        }
    }
    Ok(out)
}

pub fn scan_stat_keys(src: &str) -> Result<Vec<StatKey>> {
    use regex::Regex;
    let file: File = syn::parse_str(src).context("parse stat_keys.rs")?;

    // 三大 SECTION 邊界（row line）
    let sec_re = Regex::new(r"^//\s*SECTION\s+(\d)").unwrap();
    let sec_lines: Vec<(usize, StatSection)> = src.lines().enumerate()
        .filter_map(|(i, l)| sec_re.captures(l).and_then(|c| match &c[1] {
            "1" => Some((i + 1, StatSection::All)),
            "2" => Some((i + 1, StatSection::NonBuilding)),
            "3" => Some((i + 1, StatSection::Visual)),
            _ => None,
        }))
        .collect();
    // Sub-headers: `// ---- ... ----`
    let sub_re = Regex::new(r"^\s*//\s*----\s*(.+?)\s*----").unwrap();
    let sub_headers: Vec<(usize, String)> = src.lines().enumerate()
        .filter_map(|(i, l)| sub_re.captures(l).map(|c| (i + 1, c[1].trim().to_string())))
        .collect();

    let pick_section = |line: usize| -> StatSection {
        sec_lines.iter().rev().find(|(h, _)| *h <= line).map(|(_, s)| *s).unwrap_or(StatSection::All)
    };
    let pick_sub = |line: usize| -> Option<String> {
        sub_headers.iter().rev().find(|(h, _)| *h <= line).map(|(_, n)| n.clone())
    };

    let mut out = Vec::new();
    for item in &file.items {
        if let Item::Const(c) = item {
            use syn::spanned::Spanned;
            let line = c.span().start().line;
            let name = c.ident.to_string();
            let value = match &*c.expr {
                syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(s), .. }) => s.value(),
                syn::Expr::Path(_) => continue, // alias to another const; skip here
                _ => continue,
            };
            out.push(StatKey {
                const_name: name,
                string_value: value,
                doc: extract_doc(&c.attrs),
                section: pick_section(line),
                sub_group: pick_sub(line),
            });
        }
    }
    Ok(out)
}
```

> **Note:** 需要在 Cargo.toml 的 `gen-docs` feature 加 `regex` 與 `quote`、`proc-macro2`。`regex` repo 已經是 deps，`quote`/`proc-macro2` 會隨 syn 一起進來但要顯式寫 optional。更簡單：`render_sig` 用 `quote::ToTokens::to_token_stream().to_string()`，就一個 `quote` 依賴。把 `quote = { version = "1", optional = true }` 加到 deps，`gen-docs = ["maud", "syn", "clap", "anyhow", "quote"]`（regex 已有）。

**Step 3 — 修 Cargo.toml 補依賴**

```toml
quote = { version = "1", optional = true }
proc-macro2 = { version = "1", features = ["span-locations"], optional = true }
```
```toml
gen-docs = ["maud", "syn", "clap", "anyhow", "quote", "proc-macro2"]
```

> `proc-macro2` 加 `span-locations` feature 才會有 `span().start().line`。syn 依賴了 proc-macro2 但預設沒這 feature。

**Step 4 — mod.rs 加入**

```rust
pub mod model;
pub mod entity;
pub mod dll;
pub mod api_scan;
```

**Step 5 — 跑 unit test**

Run: `cargo test -p omobab --bin gen-docs --features gen-docs api_scan::tests`
Expected: `test api_scan::tests::extracts_unit_hooks_with_docs ... ok`

**Step 6 — 冒煙測試真實源碼**：main 暫加：
```rust
let api = lib::api_scan::scan(std::path::Path::new("omb/script-abi/src"))?;
println!("api: {} unit hooks, {} ability hooks, {} world methods, {} stat keys",
    api.unit_hooks.len(), api.ability_hooks.len(), api.world_methods.len(), api.stat_keys.len());
```
Expected 數量約：unit ~20、ability ~2、world ~50+、stat_keys ~100+。驗證後刪掉。

**Step 7 — Commit**

```bash
git add omb/Cargo.toml omb/src/bin/gen_docs_lib/
git commit -m "feat(gen-docs): parse script-abi traits + stat_keys via syn"
```

---

## Task 6 — `coverage.rs` 解析 base_content impls

**Files:**
- Create: `omb/src/bin/gen_docs_lib/coverage.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`

**Step 1 — 寫 failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    const FAKE: &str = r#"
        struct DartTower;
        impl UnitScript for DartTower {
            fn unit_id(&self) -> RStr<'_> { "dart".into() }
            fn on_spawn(&self, e: EntityHandle, w: &mut GameWorldDyn<'_>) {
                w.set_tower_atk(e, 10.0);
            }
            fn on_tick(&self, e: EntityHandle, _dt: f32, w: &mut GameWorldDyn<'_>) {
                let enemies = w.query_enemies_in_range(v2, 100.0, e);
                for t in enemies {
                    w.deal_damage(t, 5.0, DamageKind::Physical, RSome(e));
                }
            }
        }
    "#;

    #[test]
    fn detects_impl_and_world_calls() {
        let world_methods: HashSet<String> = ["set_tower_atk","query_enemies_in_range","deal_damage"]
            .iter().map(|s| s.to_string()).collect();
        let result = scan_source(FAKE, "fake.rs", &world_methods).unwrap();
        assert_eq!(result.len(), 1);
        let e = &result[0];
        assert_eq!(e.self_ty, "DartTower");
        assert!(e.trait_name == "UnitScript");
        assert!(e.overrides.contains(&"on_spawn".to_string()));
        assert!(e.overrides.contains(&"on_tick".to_string()));
        assert!(e.world_calls.contains("set_tower_atk"));
        assert!(e.world_calls.contains("query_enemies_in_range"));
        assert!(e.world_calls.contains("deal_damage"));
    }
}
```

**Step 2 — 實作 `coverage.rs`**

```rust
//! Walk base_content source files to detect `impl UnitScript for X` /
//! `impl AbilityScript for X` blocks and collect overridden methods + the
//! GameWorld method names called inside each impl.

use anyhow::{Context, Result};
use std::collections::{BTreeSet, HashSet};
use std::path::Path;
use syn::visit::Visit;
use syn::{File, Item, ItemImpl, ImplItem, ImplItemFn};

#[derive(Debug, Clone)]
pub struct ImplEntry {
    pub self_ty: String,
    pub trait_name: String,    // "UnitScript" or "AbilityScript"
    pub overrides: Vec<String>,
    pub world_calls: BTreeSet<String>,
    pub unit_id: Option<String>,  // from fn unit_id(&self) -> "foo"
    pub source_file: String,
}

pub fn scan_dir(dir: &Path, world_methods: &HashSet<String>) -> Result<Vec<ImplEntry>> {
    let mut out = Vec::new();
    for entry in walkdir(dir)? {
        let src = std::fs::read_to_string(&entry)
            .with_context(|| format!("reading {}", entry.display()))?;
        let rel = entry.strip_prefix(dir).unwrap_or(&entry).display().to_string();
        let more = scan_source(&src, &rel, world_methods)?;
        out.extend(more);
    }
    Ok(out)
}

fn walkdir(dir: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut out = Vec::new();
    fn inner(p: &Path, out: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
        for e in std::fs::read_dir(p)? {
            let e = e?;
            let path = e.path();
            if path.is_dir() { inner(&path, out)?; }
            else if path.extension().and_then(|s| s.to_str()) == Some("rs") { out.push(path); }
        }
        Ok(())
    }
    inner(dir, &mut out)?;
    Ok(out)
}

pub fn scan_source(src: &str, rel: &str, world_methods: &HashSet<String>) -> Result<Vec<ImplEntry>> {
    let file: File = syn::parse_str(src).context("parse source")?;
    let mut out = Vec::new();
    for item in &file.items {
        if let Item::Impl(imp) = item {
            if let Some((_, path, _)) = &imp.trait_ {
                let last = match path.segments.last() {
                    Some(s) => s.ident.to_string(),
                    None => continue,
                };
                if last != "UnitScript" && last != "AbilityScript" {
                    continue;
                }
                let self_ty = quote_ty(&imp.self_ty);
                let mut entry = ImplEntry {
                    self_ty,
                    trait_name: last,
                    overrides: Vec::new(),
                    world_calls: BTreeSet::new(),
                    unit_id: None,
                    source_file: rel.to_string(),
                };
                for it in &imp.items {
                    if let ImplItem::Fn(f) = it {
                        let name = f.sig.ident.to_string();
                        if name == "unit_id" {
                            entry.unit_id = extract_string_return(f);
                        } else {
                            entry.overrides.push(name);
                        }
                        let mut v = CallVisitor {
                            receivers: &["world", "w", "_w"],
                            methods: world_methods,
                            found: &mut entry.world_calls,
                        };
                        v.visit_impl_item_fn(f);
                    }
                }
                out.push(entry);
            }
        }
    }
    Ok(out)
}

fn quote_ty(ty: &syn::Type) -> String {
    use quote::ToTokens;
    ty.to_token_stream().to_string().replace(' ', "")
}

fn extract_string_return(f: &ImplItemFn) -> Option<String> {
    // 找 body 裡第一個 Lit::Str 或 .into() 前面的 literal；夠 heuristic。
    struct Find(Option<String>);
    impl<'ast> Visit<'ast> for Find {
        fn visit_lit_str(&mut self, l: &'ast syn::LitStr) {
            if self.0.is_none() { self.0 = Some(l.value()); }
        }
    }
    let mut f2 = Find(None);
    f2.visit_block(&f.block);
    f2.0
}

struct CallVisitor<'a> {
    receivers: &'a [&'a str],
    methods: &'a HashSet<String>,
    found: &'a mut BTreeSet<String>,
}

impl<'a, 'ast> Visit<'ast> for CallVisitor<'a> {
    fn visit_expr_method_call(&mut self, m: &'ast syn::ExprMethodCall) {
        // receiver 可能是 Path(ident) — e.g. `world.xxx()`
        if let syn::Expr::Path(p) = &*m.receiver {
            if let Some(seg) = p.path.segments.last() {
                let name = seg.ident.to_string();
                if self.receivers.contains(&name.as_str()) {
                    let method = m.method.to_string();
                    if self.methods.contains(&method) {
                        self.found.insert(method);
                    }
                }
            }
        }
        syn::visit::visit_expr_method_call(self, m);
    }
}
```

**Step 3 — mod.rs 加**

```rust
pub mod coverage;
```

**Step 4 — 跑 test**

Run: `cargo test -p omobab --bin gen-docs --features gen-docs coverage::tests`
Expected: PASS

**Step 5 — 冒煙測試真實源碼**：main 暫加（用 Task 5 拿到的 world_methods 作 filter）：
```rust
let world_names: HashSet<String> = api.world_methods.iter().map(|m| m.name.clone()).collect();
let cov = lib::coverage::scan_dir(std::path::Path::new("omb/scripts/base_content/src"), &world_names)?;
println!("coverage: {} impl blocks", cov.len());
for e in &cov { println!("  {} {} @{} overrides={} calls={}",
    e.trait_name, e.self_ty, e.source_file, e.overrides.len(), e.world_calls.len()); }
```
Run: `cargo run -p omobab --bin gen-docs --features gen-docs`
Expected: 8 個 hero impl + 4 個 tower impl（根據 memory `ability_script_ffi.md`）加上其他 AbilityScript impl。驗證後刪掉冒煙 code。

**Step 6 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/
git commit -m "feat(gen-docs): detect UnitScript/AbilityScript impls + world calls"
```

---

## Task 7 — `merge.rs` 合併四個資料源成 `Catalog`

**Files:**
- Create: `omb/src/bin/gen_docs_lib/merge.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`

**Step 1 — 實作**

```rust
//! Merge DllData + EntityData + ApiSpec + ImplEntry list into final Catalog.

use crate::lib::coverage::ImplEntry;
use crate::lib::dll::DllData;
use crate::lib::entity::EntityData;
use crate::lib::model::{
    AbilityEntry, ApiSpec, BuildMeta, Catalog, UnitEntry, UnitKind, Warning,
};
use std::collections::HashMap;

pub fn merge(
    dll: DllData,
    entity: EntityData,
    api: ApiSpec,
    impls: Vec<ImplEntry>,
    warnings: Vec<Warning>,
    meta: BuildMeta,
) -> Catalog {
    // 以 unit_id 為 key 建 impl lookup
    let mut by_id: HashMap<String, ImplEntry> = HashMap::new();
    for i in impls {
        // unit_id 沒抽到就用 self_ty 的 snake_case 當猜測
        let key = i.unit_id.clone().unwrap_or_else(|| snake(&i.self_ty));
        by_id.insert(key, i);
    }

    let mut units: Vec<UnitEntry> = Vec::new();

    // 1. DLL 提供的 units（塔 + Unknown）
    for u in dll.units {
        let imp = by_id.remove(&u.id);
        let (overrides, world_calls, src) = match imp {
            Some(i) => (i.overrides, i.world_calls, Some(i.source_file)),
            None => (Vec::new(), Default::default(), None),
        };
        let label = u.tower.as_ref().map(|t| t.label.clone());
        units.push(UnitEntry {
            id: u.id,
            kind: u.kind,
            label,
            tower: u.tower,
            hero: None,
            creep: None,
            abilities: Vec::new(),
            overrides,
            world_calls,
            source_file: src,
        });
    }

    // 2. entity.json 的 heroes（合併 DLL impl 資訊）
    for (id, h) in entity.heroes {
        let imp = by_id.remove(&id);
        let (overrides, world_calls, src) = match imp {
            Some(i) => (i.overrides, i.world_calls, Some(i.source_file)),
            None => (Vec::new(), Default::default(), None),
        };
        units.push(UnitEntry {
            id: id.clone(),
            kind: UnitKind::Hero,
            label: Some(h.name.clone()),
            tower: None,
            abilities: h.abilities.clone(),
            hero: Some(h),
            creep: None,
            overrides,
            world_calls,
            source_file: src,
        });
    }

    // 3. entity.json 的 creeps
    for (id, c) in entity.creeps {
        let imp = by_id.remove(&id);
        let (overrides, world_calls, src) = match imp {
            Some(i) => (i.overrides, i.world_calls, Some(i.source_file)),
            None => (Vec::new(), Default::default(), None),
        };
        units.push(UnitEntry {
            id: id.clone(),
            kind: UnitKind::Creep,
            label: Some(c.name.clone()),
            tower: None,
            abilities: c.abilities.clone(),
            hero: None,
            creep: Some(c),
            overrides,
            world_calls,
            source_file: src,
        });
    }

    // 4. 剩下的 impl（可能是 AbilityScript 或 orphan UnitScript）當 warning
    let mut warnings = warnings;
    for (k, i) in by_id {
        if i.trait_name == "UnitScript" {
            warnings.push(Warning {
                source: i.source_file.clone(),
                message: format!("orphan UnitScript impl for {} (unit_id={}) not referenced by DLL manifest or entity.json",
                    i.self_ty, k),
            });
        }
    }

    Catalog {
        units,
        abilities: dll.abilities,
        api,
        warnings,
        meta,
    }
}

fn snake(s: &str) -> String {
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 { out.push('_'); }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
```

**Step 2 — mod.rs 加**

```rust
pub mod merge;
```

**Step 3 — 驗證 build**

Run: `cargo build -p omobab --bin gen-docs --features gen-docs`
Expected: PASS

**Step 4 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/
git commit -m "feat(gen-docs): merge data sources into Catalog"
```

---

## Task 8 — `render.rs` HTML skeleton + CSS/JS + header/footer

**Files:**
- Create: `omb/src/bin/gen_docs_lib/render.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`

**Step 1 — 實作 skeleton（先只有 header + sidebar + 空 main）**

```rust
//! Render Catalog into a single self-contained HTML string using maud.

use crate::lib::model::{Catalog, UnitKind};
use maud::{html, Markup, DOCTYPE, PreEscaped};

const CSS: &str = include_str!("render_style.css");
const JS:  &str = include_str!("render_script.js");

pub fn page(c: &Catalog) -> String {
    let tower_count = c.units.iter().filter(|u| u.kind == UnitKind::Tower).count();
    let hero_count = c.units.iter().filter(|u| u.kind == UnitKind::Hero).count();
    let creep_count = c.units.iter().filter(|u| u.kind == UnitKind::Creep).count();

    let page: Markup = html! {
        (DOCTYPE)
        html lang="zh-Hant" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "omoba · Unit & Script API Catalog" }
                style { (PreEscaped(CSS)) }
            }
            body {
                header.topbar {
                    div.title { "omoba catalog" }
                    div.meta {
                        span { "story: " (c.meta.story) }
                        span { "git: " (c.meta.git_sha) }
                        span { "built: " (c.meta.timestamp) }
                    }
                    div.controls {
                        input #q type="search" placeholder="🔍 filter units / methods";
                        label { input #only-used type="checkbox"; " show only used" }
                        label { input #dark type="checkbox"; " dark" }
                    }
                }

                @if !c.warnings.is_empty() {
                    section.warnings {
                        h2 { "Warnings (" (c.warnings.len()) ")" }
                        ul {
                            @for w in &c.warnings {
                                li { strong { (w.source) } ": " (w.message) }
                            }
                        }
                    }
                }

                div.layout {
                    nav.sidebar {
                        h3 { "Units" }
                        ul {
                            li { a href="#towers" { "Towers (" (tower_count) ")" } }
                            li { a href="#heroes" { "Heroes (" (hero_count) ")" } }
                            li { a href="#creeps" { "Creeps (" (creep_count) ")" } }
                        }
                        h3 { "API" }
                        ul {
                            li { a href="#abilities" { "Abilities (" (c.abilities.len()) ")" } }
                            li { a href="#unit-hooks" { "UnitScript Hooks (" (c.api.unit_hooks.len()) ")" } }
                            li { a href="#ability-hooks" { "AbilityScript (" (c.api.ability_hooks.len()) ")" } }
                            li { a href="#world" { "GameWorld (" (c.api.world_methods.len()) ")" } }
                            li { a href="#stat-keys" { "Stat Keys (" (c.api.stat_keys.len()) ")" } }
                        }
                        h3 { "Report" }
                        ul {
                            li { a href="#coverage" { "Coverage Matrix" } }
                        }
                    }
                    main.content {
                        (section_units(c))
                        (section_abilities(c))
                        (section_api(c))
                        (section_stat_keys(c))
                        (section_coverage(c))
                    }
                }

                footer.footer {
                    "sources: " (c.meta.sources.join(" · "))
                }
                script { (PreEscaped(JS)) }
            }
        }
    };
    page.into_string()
}

fn section_units(_c: &Catalog) -> Markup { html! { section#units { h2 { "Units" } p { "(coming in next task)" } } } }
fn section_abilities(_c: &Catalog) -> Markup { html! { section#abilities { h2 { "Abilities" } p { "(coming)" } } } }
fn section_api(_c: &Catalog) -> Markup { html! { section#api { h2 { "Script API" } p { "(coming)" } } } }
fn section_stat_keys(_c: &Catalog) -> Markup { html! { section#stat-keys { h2 { "Stat Keys" } p { "(coming)" } } } }
fn section_coverage(_c: &Catalog) -> Markup { html! { section#coverage { h2 { "Coverage Matrix" } p { "(coming)" } } } }
```

**Step 2 — 建立 CSS**：`omb/src/bin/gen_docs_lib/render_style.css`

```css
:root{
  --bg:#fafafa; --fg:#222; --panel:#fff; --border:#ddd; --muted:#666;
  --accent:#2563eb; --warn:#d4351c; --used:#0a7a2f; --unused:#aaa;
}
body.dark{
  --bg:#15171c; --fg:#e6e6e6; --panel:#1d2027; --border:#2a2e37;
  --muted:#9aa0aa; --accent:#68a4ff; --unused:#555;
}
*{box-sizing:border-box}
body{margin:0;font-family:system-ui,-apple-system,Segoe UI,sans-serif;
  background:var(--bg);color:var(--fg);line-height:1.5}
code,pre,.mono{font-family:ui-monospace,SFMono-Regular,Consolas,monospace}
.topbar{display:flex;align-items:center;gap:1rem;padding:.6rem 1rem;
  background:var(--panel);border-bottom:1px solid var(--border);position:sticky;top:0;z-index:9}
.topbar .title{font-weight:600;font-size:1.05rem}
.topbar .meta{display:flex;gap:.8rem;color:var(--muted);font-size:.85rem}
.topbar .controls{margin-left:auto;display:flex;gap:.8rem;align-items:center}
.topbar input[type=search]{padding:.3rem .5rem;border:1px solid var(--border);
  border-radius:4px;background:var(--panel);color:var(--fg);min-width:280px}
.warnings{background:#fef1ef;border-left:4px solid var(--warn);padding:.6rem 1rem;margin:0}
body.dark .warnings{background:#3a1f1c}
.warnings h2{margin:.2rem 0 .4rem;font-size:1rem;color:var(--warn)}
.warnings ul{margin:0;padding-left:1.2rem}
.layout{display:flex;min-height:calc(100vh - 50px)}
.sidebar{width:260px;flex:0 0 260px;border-right:1px solid var(--border);
  background:var(--panel);padding:1rem;position:sticky;top:50px;align-self:flex-start;
  height:calc(100vh - 50px);overflow-y:auto}
.sidebar h3{margin:.8rem 0 .3rem;font-size:.75rem;color:var(--muted);
  text-transform:uppercase;letter-spacing:.08em}
.sidebar ul{list-style:none;padding:0;margin:0}
.sidebar li a{display:block;padding:.25rem .4rem;color:var(--fg);text-decoration:none;
  border-radius:3px;font-size:.9rem}
.sidebar li a:hover{background:var(--bg);color:var(--accent)}
.content{flex:1;padding:1rem 1.5rem;overflow-x:hidden}
section{margin-bottom:2.5rem}
section>h2{border-bottom:2px solid var(--border);padding-bottom:.3rem;margin-top:0}
.card{background:var(--panel);border:1px solid var(--border);border-radius:6px;
  padding:.8rem 1rem;margin-bottom:.6rem}
.card h3{margin:0 0 .3rem;font-size:1.05rem}
.card .sub{color:var(--muted);font-size:.85rem;margin-bottom:.4rem}
.card .tags{display:flex;flex-wrap:wrap;gap:.3rem;margin:.3rem 0}
.tag{padding:.1rem .5rem;background:var(--bg);border:1px solid var(--border);
  border-radius:10px;font-size:.8rem}
.tag.ability{background:#e8f1ff;border-color:#bcd3ff;color:#1247a8}
body.dark .tag.ability{background:#1d2b45;border-color:#2b4373;color:#8cb4ff}
.kv{display:grid;grid-template-columns:max-content 1fr;gap:.2rem .8rem;
  font-size:.9rem;margin:.3rem 0}
.kv dt{color:var(--muted)}
.api-method{padding:.6rem 0;border-bottom:1px dashed var(--border)}
.api-method .sig{font-size:.85rem;color:var(--accent);word-break:break-all}
.api-method .doc{color:var(--muted);margin-top:.2rem;white-space:pre-wrap;font-size:.9rem}
.api-method.unused{opacity:.45}
table.matrix{border-collapse:collapse;font-size:.8rem}
table.matrix th,table.matrix td{border:1px solid var(--border);padding:.2rem .4rem;text-align:center}
table.matrix th.sticky-col,table.matrix td.sticky-col{
  position:sticky;left:0;background:var(--panel);text-align:left;z-index:2}
table.matrix thead th{position:sticky;top:50px;background:var(--panel);z-index:3}
.hidden{display:none !important}
.footer{border-top:1px solid var(--border);padding:.6rem 1rem;color:var(--muted);font-size:.8rem}
```

**Step 3 — 建立 JS**：`omb/src/bin/gen_docs_lib/render_script.js`

```js
(function(){
  const q = document.getElementById('q');
  const onlyUsed = document.getElementById('only-used');
  const dark = document.getElementById('dark');

  function filter(){
    const needle = (q.value || '').toLowerCase();
    document.querySelectorAll('[data-search]').forEach(el=>{
      const hay = el.dataset.search.toLowerCase();
      el.classList.toggle('hidden', needle && !hay.includes(needle));
    });
  }
  function applyOnlyUsed(){
    document.querySelectorAll('.api-method').forEach(el=>{
      const used = el.dataset.used === '1';
      el.classList.toggle('unused', !used);
      if (onlyUsed.checked){ el.classList.toggle('hidden', !used); }
    });
  }
  function applyDark(){ document.body.classList.toggle('dark', dark.checked); }

  if (q) q.addEventListener('input', filter);
  if (onlyUsed) onlyUsed.addEventListener('change', applyOnlyUsed);
  if (dark) dark.addEventListener('change', applyDark);
  applyOnlyUsed();
})();
```

**Step 4 — mod.rs 加 `pub mod render;`**

**Step 5 — 驗證 build**

Run: `cargo build -p omobab --bin gen-docs --features gen-docs`
Expected: PASS

**Step 6 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/
git commit -m "feat(gen-docs): HTML skeleton with sidebar/CSS/JS (sections TBD)"
```

---

## Task 9 — `render.rs` 單位 / abilities section 實作

**Files:**
- Modify: `omb/src/bin/gen_docs_lib/render.rs`

**Step 1 — 替換 `section_units` 與 `section_abilities`**

```rust
fn section_units(c: &Catalog) -> Markup {
    let towers: Vec<&UnitEntry> = c.units.iter().filter(|u| u.kind == UnitKind::Tower).collect();
    let heroes: Vec<&UnitEntry> = c.units.iter().filter(|u| u.kind == UnitKind::Hero).collect();
    let creeps: Vec<&UnitEntry> = c.units.iter().filter(|u| u.kind == UnitKind::Creep).collect();
    html! {
        section#towers {
            h2 { "Towers (" (towers.len()) ")" }
            @for u in towers { (tower_card(u)) }
        }
        section#heroes {
            h2 { "Heroes (" (heroes.len()) ")" }
            @for u in heroes { (hero_card(u)) }
        }
        section#creeps {
            h2 { "Creeps (" (creeps.len()) ")" }
            @for u in creeps { (creep_card(u)) }
        }
    }
}

fn tower_card(u: &UnitEntry) -> Markup {
    let t = u.tower.as_ref().cloned().unwrap_or_default();
    let search = format!("{} {} tower", u.id, t.label);
    html! {
        div.card data-search=(search) {
            h3 { (t.label) " " span.sub { "(" (u.id) ")" } }
            dl.kv {
                dt { "atk" } dd { (t.atk) }
                dt { "range" } dd { (t.range) }
                dt { "asd" } dd { (t.asd_interval) "s" }
                dt { "bullet speed" } dd { (t.bullet_speed) }
                dt { "splash / hit r" } dd { (t.splash_radius) " / " (t.hit_radius) }
                dt { "slow" } dd { "×" (t.slow_factor) " · " (t.slow_duration) "s" }
                dt { "cost" } dd { (t.cost) }
                dt { "hp / footprint" } dd { (t.hp) " / " (t.footprint) }
            }
            (impl_block(u))
        }
    }
}

fn hero_card(u: &UnitEntry) -> Markup {
    let h = match &u.hero { Some(h) => h.clone(), None => return html!{} };
    let search = format!("{} {} hero {}", u.id, h.name, h.title);
    html! {
        div.card data-search=(search) {
            h3 { (h.name) " " span.sub { "— " (h.title) " · " (u.id) } }
            p.sub { (h.background) }
            dl.kv {
                dt { "attrs (S/A/I)" } dd { (h.strength) " / " (h.agility) " / " (h.intelligence)
                    " (" (h.primary_attribute) ")" }
                dt { "hp / mana" } dd { (h.base_hp) " / " (h.base_mana) }
                dt { "dmg / range" } dd { (h.base_damage) " / " (h.attack_range) }
                dt { "armor" } dd { (h.base_armor) }
                dt { "move / turn" } dd { (h.move_speed) " / " (h.turn_speed) }
            }
            @if !u.abilities.is_empty() {
                div.tags {
                    @for a in &u.abilities { span.tag.ability { (a) } }
                }
            }
            @if !h.level_growth.is_null() {
                details { summary { "level growth" }
                    pre.mono { (serde_json::to_string_pretty(&h.level_growth).unwrap_or_default()) }
                }
            }
            (impl_block(u))
        }
    }
}

fn creep_card(u: &UnitEntry) -> Markup {
    let c = match &u.creep { Some(c) => c.clone(), None => return html!{} };
    let search = format!("{} {} creep {}", u.id, c.name, c.enemy_type);
    html! {
        div.card data-search=(search) {
            h3 { (c.name) " " span.sub { "(" (u.id) " · " (c.enemy_type) ")" } }
            dl.kv {
                dt { "hp / armor / mr" } dd { (c.hp) " / " (c.armor) " / " (c.magic_resistance) }
                dt { "dmg / range" } dd { (c.damage) " / " (c.attack_range) }
                dt { "move" } dd { (c.move_speed) }
                dt { "ai" } dd { (c.ai_type) }
                dt { "reward" } dd { (c.exp_reward) " xp · " (c.gold_reward) " g" }
            }
            @if !u.abilities.is_empty() {
                div.tags {
                    @for a in &u.abilities { span.tag.ability { (a) } }
                }
            }
            (impl_block(u))
        }
    }
}

fn impl_block(u: &UnitEntry) -> Markup {
    if u.overrides.is_empty() && u.world_calls.is_empty() && u.source_file.is_none() {
        return html!{};
    }
    html! {
        details.impl-block {
            summary {
                "impl (" (u.overrides.len()) " hooks, "
                (u.world_calls.len()) " world calls)"
                @if let Some(src) = &u.source_file { " · " span.sub { (src) } }
            }
            @if !u.overrides.is_empty() {
                p { strong { "overrides: " }
                    @for (i, h) in u.overrides.iter().enumerate() {
                        @if i > 0 { ", " }
                        code { (h) }
                    }
                }
            }
            @if !u.world_calls.is_empty() {
                p { strong { "world calls: " }
                    @for (i, h) in u.world_calls.iter().enumerate() {
                        @if i > 0 { ", " }
                        code { (h) }
                    }
                }
            }
        }
    }
}

fn section_abilities(c: &Catalog) -> Markup {
    html! {
        section#abilities {
            h2 { "Abilities (" (c.abilities.len()) ")" }
            @for a in &c.abilities {
                (ability_card(a))
            }
        }
    }
}

fn ability_card(a: &crate::lib::model::AbilityEntry) -> Markup {
    let name = a.def_json.get("name").and_then(|v| v.as_str()).unwrap_or(&a.id);
    let desc = a.def_json.get("description").and_then(|v| v.as_str()).unwrap_or("");
    let search = format!("{} {} ability", a.id, name);
    html! {
        div.card data-search=(search) {
            h3 { (name) " " span.sub { "(" (a.id) ")" } }
            @if !desc.is_empty() { p.sub { (desc) } }
            details { summary { "def json" }
                pre.mono { (serde_json::to_string_pretty(&a.def_json).unwrap_or_default()) }
            }
        }
    }
}
```

**Step 2 — Build & smoke**

Run: `cargo build -p omobab --bin gen-docs --features gen-docs`
Expected: PASS

**Step 3 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/render.rs
git commit -m "feat(gen-docs): render tower/hero/creep/ability cards"
```

---

## Task 10 — `render.rs` API reference + stat keys + coverage matrix

**Files:**
- Modify: `omb/src/bin/gen_docs_lib/render.rs`

**Step 1 — 替換 `section_api`、`section_stat_keys`、`section_coverage`**

```rust
fn section_api(c: &Catalog) -> Markup {
    // 計算哪些 method 被用過（for "show only used"）
    let used: std::collections::HashSet<&str> = c.units.iter()
        .flat_map(|u| u.world_calls.iter().map(|s| s.as_str()))
        .collect();
    let used_hooks: std::collections::HashSet<&str> = c.units.iter()
        .flat_map(|u| u.overrides.iter().map(|s| s.as_str()))
        .collect();

    html! {
        section#unit-hooks {
            h2 { "UnitScript Hooks (" (c.api.unit_hooks.len()) ")" }
            @for m in &c.api.unit_hooks { (method_entry(m, used_hooks.contains(m.name.as_str()))) }
        }
        section#ability-hooks {
            h2 { "AbilityScript (" (c.api.ability_hooks.len()) ")" }
            @for m in &c.api.ability_hooks { (method_entry(m, true)) }
        }
        section#world {
            h2 { "GameWorld API (" (c.api.world_methods.len()) ")" }
            @for m in &c.api.world_methods { (method_entry(m, used.contains(m.name.as_str()))) }
        }
    }
}

fn method_entry(m: &crate::lib::model::ApiMethod, used: bool) -> Markup {
    let data_used = if used { "1" } else { "0" };
    let search = format!("{} {}", m.name, m.doc);
    html! {
        div.api-method data-used=(data_used) data-search=(search) {
            div {
                @if let Some(g) = &m.sub_group { span.tag { (g) } " " }
                code.sig { (m.signature) }
            }
            @if !m.doc.is_empty() { div.doc { (m.doc) } }
        }
    }
}

fn section_stat_keys(c: &Catalog) -> Markup {
    let groups = [
        (crate::lib::model::StatSection::All, "Section 1 · 全單位通用"),
        (crate::lib::model::StatSection::NonBuilding, "Section 2 · 僅非建築物"),
        (crate::lib::model::StatSection::Visual, "Section 3 · 視覺 / 前端"),
    ];
    html! {
        section#stat-keys {
            h2 { "Stat Keys (" (c.api.stat_keys.len()) ")" }
            @for (sec, label) in groups.iter() {
                h3 { (label) }
                table.kv.stat-table {
                    thead { tr { th { "const" } th { "string" } th { "group" } th { "doc" } } }
                    tbody {
                        @for s in c.api.stat_keys.iter().filter(|s| &s.section == sec) {
                            tr data-search=(format!("{} {} {}", s.const_name, s.string_value, s.doc)) {
                                td { code { (s.const_name) } }
                                td { code { "\"" (s.string_value) "\"" } }
                                td { @if let Some(g) = &s.sub_group { (g) } }
                                td.mono { (s.doc) }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn section_coverage(c: &Catalog) -> Markup {
    // 只取有 impl 的 unit + UnitScript hooks
    let units: Vec<&UnitEntry> = c.units.iter()
        .filter(|u| !u.overrides.is_empty() || !u.world_calls.is_empty())
        .collect();
    let hook_names: Vec<&str> = c.api.unit_hooks.iter().map(|m| m.name.as_str()).collect();

    html! {
        section#coverage {
            h2 { "Coverage Matrix" }
            p.sub { "每格表示該 unit 有 override 對應 UnitScript hook" }
            div style="overflow-x:auto" {
                table.matrix {
                    thead {
                        tr {
                            th.sticky-col { "unit" }
                            @for h in &hook_names { th { (h) } }
                        }
                    }
                    tbody {
                        @for u in &units {
                            tr {
                                td.sticky-col { code { (u.id) } }
                                @for h in &hook_names {
                                    @let on = u.overrides.iter().any(|o| o == h);
                                    td { @if on { "✓" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

**Step 2 — Build**

Run: `cargo build -p omobab --bin gen-docs --features gen-docs`
Expected: PASS

**Step 3 — Commit**

```bash
git add omb/src/bin/gen_docs_lib/render.rs
git commit -m "feat(gen-docs): render API reference, stat keys, coverage matrix"
```

---

## Task 11 — Main 整合 CLI + 產出完整流程

**Files:**
- Modify: `omb/src/bin/gen_docs.rs`
- Modify: `omb/src/bin/gen_docs_lib/mod.rs`（若需加 `pub mod`）

**Step 1 — 重寫 `gen_docs.rs`**

```rust
//! gen-docs — produce a self-contained HTML catalog of units, abilities,
//! and script API coverage.
//!
//! Design: docs/plans/2026-04-23-build-time-catalog-design.md

#[path = "gen_docs_lib/mod.rs"]
mod lib;

use anyhow::{Context, Result};
use clap::Parser;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "gen-docs", about = "Generate omoba unit & script API catalog HTML")]
struct Args {
    /// Output HTML path
    #[arg(long, default_value = "target/docs/index.html")]
    out: PathBuf,
    /// Story folder name under omb/Story/ (overrides game.toml)
    #[arg(long)]
    story: Option<String>,
    /// Path to base_content.dll (auto-detected if omitted)
    #[arg(long)]
    dll: Option<PathBuf>,
    /// script-abi src directory
    #[arg(long, default_value = "omb/script-abi/src")]
    abi_src: PathBuf,
    /// base_content src directory (for coverage scan)
    #[arg(long, default_value = "omb/scripts/base_content/src")]
    content_src: PathBuf,
    /// Story root (default: omb/Story)
    #[arg(long, default_value = "omb/Story")]
    story_root: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let story = args.story.clone().unwrap_or_else(read_story_from_game_toml);
    let dll_path = args.dll.clone().unwrap_or_else(default_dll_path);

    let mut warnings: Vec<lib::model::Warning> = Vec::new();

    // 1. DLL
    let dll = lib::dll::load(&dll_path)
        .with_context(|| format!("loading DLL {}", dll_path.display()))?;

    // 2. API scan (fatal)
    let api = lib::api_scan::scan(&args.abi_src)?;

    // 3. Coverage (soft)
    let world_names: HashSet<String> = api.world_methods.iter().map(|m| m.name.clone()).collect();
    let impls = match lib::coverage::scan_dir(&args.content_src, &world_names) {
        Ok(v) => v,
        Err(e) => {
            warnings.push(lib::model::Warning {
                source: args.content_src.display().to_string(),
                message: format!("coverage scan failed: {e}"),
            });
            Vec::new()
        }
    };

    // 4. entity.json (soft)
    let story_dir = args.story_root.join(&story);
    let entity = match lib::entity::load(&story_dir) {
        Ok(d) => d,
        Err(e) => {
            warnings.push(lib::model::Warning {
                source: story_dir.display().to_string(),
                message: format!("entity.json load failed: {e}"),
            });
            lib::entity::EntityData { heroes: Default::default(), creeps: Default::default() }
        }
    };

    // 5. merge
    let meta = lib::model::BuildMeta {
        timestamp: now_rfc3339(),
        git_sha: git_short_sha().unwrap_or_else(|_| "unknown".into()),
        story: story.clone(),
        sources: vec![
            dll_path.display().to_string(),
            story_dir.display().to_string(),
            args.abi_src.display().to_string(),
            args.content_src.display().to_string(),
        ],
    };
    let catalog = lib::merge::merge(dll, entity, api, impls, warnings, meta);

    // 6. render & write
    let html = lib::render::page(&catalog);
    if let Some(parent) = args.out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&args.out, &html)
        .with_context(|| format!("writing {}", args.out.display()))?;

    println!(
        "gen-docs: {} units, {} abilities, {} warnings → {}",
        catalog.units.len(),
        catalog.abilities.len(),
        catalog.warnings.len(),
        args.out.display(),
    );
    Ok(())
}

fn read_story_from_game_toml() -> String {
    let path = "omb/game.toml";
    let src = std::fs::read_to_string(path).unwrap_or_default();
    // 非常簡單的抓 STORY = "..." 行
    for line in src.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix("STORY") {
            if let Some(q1) = rest.find('"') {
                if let Some(q2) = rest[q1+1..].find('"') {
                    return rest[q1+1..q1+1+q2].to_string();
                }
            }
        }
    }
    "TD_1".to_string()
}

fn default_dll_path() -> PathBuf {
    // 優先 release，再 debug
    let release = PathBuf::from("target/release/base_content.dll");
    if release.exists() { return release; }
    PathBuf::from("target/debug/base_content.dll")
}

fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn git_short_sha() -> Result<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
```

**Step 2 — 編譯 + 跑 smoke test**

先確保 `base_content.dll` 已經 build：
```
cargo build --release -p base_content
```

再跑：
```
cargo run -p omobab --bin gen-docs --features gen-docs --release
```

Expected: 輸出 `gen-docs: N units, M abilities, W warnings → target/docs/index.html`；檔案存在且大於 10KB。

**Step 3 — 瀏覽器開啟確認**

手動打開 `target/docs/index.html`：
- sidebar 可導航各 section
- 至少一個 tower card 有數值
- 至少一個 hero card 顯示
- API reference 有 method list
- Coverage matrix 表格存在
- 搜尋框 / dark toggle 可動作

若有問題回到對應 Task 修。

**Step 4 — Commit**

```bash
git add omb/src/bin/gen_docs.rs
git commit -m "feat(gen-docs): wire CLI + end-to-end pipeline"
```

---

## Task 12 — 冒煙測試 + 文件更新

**Files:**
- Create: `omb/tests/gen_docs_smoke.rs`
- Modify: `omb/Cargo.toml`（`chrono` 已經有，不用動）
- Modify: `D:/omoba/CLAUDE.md`（加一行說明用法）

**Step 1 — 寫 integration smoke test**

```rust
//! Smoke test: run gen-docs and check the output HTML has expected markers.
//! Only runs when base_content.dll exists (dev machine); CI can guard with env.

#![cfg(feature = "gen-docs")]

use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore] // run explicitly: cargo test --features gen-docs -- --ignored
fn produces_html_with_known_content() {
    let dll = PathBuf::from("../target/release/base_content.dll");
    if !dll.exists() {
        eprintln!("base_content.dll missing; skipping");
        return;
    }
    let out = PathBuf::from("../target/docs/smoke.html");
    let status = Command::new(env!("CARGO"))
        .args(["run", "--release", "-p", "omobab", "--bin", "gen-docs",
               "--features", "gen-docs", "--",
               "--out", out.to_str().unwrap()])
        .current_dir("..")
        .status()
        .expect("spawn gen-docs");
    assert!(status.success(), "gen-docs failed");

    let html = std::fs::read_to_string(&out).expect("read output");
    assert!(html.contains("omoba catalog"), "missing title");
    assert!(html.contains("UnitScript Hooks"), "missing API section");
    assert!(html.contains("Coverage Matrix"), "missing coverage section");
}
```

> 加 `#[ignore]` 因為需要預先 build DLL，本地開發執行 `cargo test -p omobab --features gen-docs -- --ignored` 跑。

**Step 2 — 跑 smoke test**

Run: `cargo test -p omobab --features gen-docs -- --ignored produces_html_with_known_content`
Expected: PASS

**Step 3 — 更新 `CLAUDE.md`**

在 `D:/omoba/CLAUDE.md` 加一節（append 最後）：

```markdown
## Unit & Script API catalog (gen-docs)

- `cargo run -p omobab --bin gen-docs --features gen-docs --release` → 產出 `target/docs/index.html`
- 需要先 `cargo build --release -p base_content` 以便載入 DLL
- 內容：所有單位屬性、script-abi 完整 API reference、覆蓋矩陣
- 設計：`docs/plans/2026-04-23-build-time-catalog-design.md`
```

**Step 4 — Commit**

```bash
git add omb/tests/gen_docs_smoke.rs CLAUDE.md
git commit -m "test(gen-docs): smoke test + CLAUDE.md usage note"
```

---

## Completion Checklist

- [ ] `cargo build -p omobab --bin omobab` 不受新 deps 影響（feature-gated）
- [ ] `cargo run -p omobab --bin gen-docs --features gen-docs --release` 產生 `target/docs/index.html`
- [ ] 開啟 HTML 能看到 Units / Abilities / API / Coverage 四大區
- [ ] Search / dark mode / show only used 三個 JS 功能皆可用
- [ ] Warning 區塊存在且被觸發時顯示在頂部
- [ ] `cargo test -p omobab --features gen-docs` 全過
- [ ] CLAUDE.md 記載用法

---

**Plan complete and saved to `docs/plans/2026-04-23-build-time-catalog-plan.md`. Two execution options:**

**1. Subagent-Driven (this session)** — 我每個 task dispatch 一個 fresh subagent，task 之間做 code review，快速迭代
**2. Parallel Session (separate)** — 開新 session 用 executing-plans，批次執行 + checkpoint

**Which approach?**
