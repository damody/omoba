# Template ID Codegen Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 template 字串 id（tower / hero / ability / buff / summon / creep / projectile kind）的 source of truth 集中到 `omb/Story/templates.json`，build-time 產生 per-namespace sequential u16 newtype，scripts 用 const 寫 id（打錯字編譯失敗），proto 高頻事件改 uint32，顯示字串走 client 本地反查。

**Architecture:** 新 crate `omoba-template-ids`（純 const，零 runtime dep）用 build.rs 讀 `templates.json` 產 7 個 newtype (`TowerId`/`HeroId`/`AbilityId`/`BuffId`/`SummonId`/`CreepId`/`ProjectileKindId`) + 對應 `pub const TPL_*` + `*_name()` 反查函式。`omb-script-abi` re-export newtypes 並改 trait signature；proto 破壞性一次換；`omoba-core/src/template_ids.rs` 刪除。

**Tech Stack:** Rust 1.91.0 (rust-toolchain.toml 鎖定), abi_stable (FFI), prost (proto), serde_json (build.rs), tokio-kcp (wire), specs 0.20 (ECS).

設計來源：`docs/plans/2026-04-25-template-id-codegen-design.md`

---

## Phase A：新 crate 基礎（Foundation）

### Task 1: Scaffold `omoba-template-ids` crate

**Files:**
- Create: `omoba-template-ids/Cargo.toml`
- Create: `omoba-template-ids/src/lib.rs`
- Create: `omoba-template-ids/build.rs`
- Modify: `Cargo.toml`（repo root workspace）— `members` 加 `"omoba-template-ids"`

**Step 1: 寫 crate manifest**

```toml
# omoba-template-ids/Cargo.toml
[package]
name = "omoba-template-ids"
version = "0.1.0"
edition = "2021"
rust-version = "1.91"

[build-dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dependencies]
# 執行期零依賴 — generated code 不 import 任何東西

[features]
# no features — keep minimal
```

**Step 2: 寫最小 build.rs（先只做 rerun-if-changed）**

```rust
// omoba-template-ids/build.rs
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json_path = manifest.parent().unwrap().join("omb/Story/templates.json");
    println!("cargo:rerun-if-changed={}", json_path.display());
    // 先產空的 gen 檔讓 lib.rs include! 不爆
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        format!("{}/template_ids_gen.rs", out_dir),
        "// placeholder; Task 4 replaces this\n",
    ).unwrap();
}
```

**Step 3: 寫最小 lib.rs**

```rust
// omoba-template-ids/src/lib.rs
//! Build-time generated template ids. Source of truth: omb/Story/templates.json
//! See docs/plans/2026-04-25-template-id-codegen-design.md
include!(concat!(env!("OUT_DIR"), "/template_ids_gen.rs"));
```

**Step 4: 加入 workspace**

修改 `D:/omoba/Cargo.toml`（repo root workspace），`members` 陣列加 `"omoba-template-ids"`。

**Step 5: Verify**

Run: `cargo check -p omoba-template-ids`
Expected: PASS（只有 rerun-if-changed + 空 include，無內容）

**Step 6: Commit**

```bash
git add omoba-template-ids/ Cargo.toml
git commit -m "feat(template-ids): scaffold omoba-template-ids crate with empty build.rs"
```

---

### Task 2: 建立 `Story/templates.json` 初版

**Files:**
- Create: `omb/Story/templates.json`

**Step 1: 聚合來源**

掃 5 個 scene 的 `entity.json` 抽 unique heroes / enemies（可 grep `"id":` 拿列表）。掃 `scripts/base_content/src/{towers,heroes,summons}/*.rs` 抽硬編字串（`RStr::from_str(...)`, `kind_tag: RString::from(...)`）。掃 `scripts/script-abi/src/buff_ids.rs` 抽 buff 常數。

**Step 2: 寫 templates.json（id 0 reserved = UNSPECIFIED，實際 id 從 1 開始）**

```json
{
  "_comment": "Single source of truth for all template ids. Append only — changing order shifts ids.",
  "_design_doc": "docs/plans/2026-04-25-template-id-codegen-design.md",

  "towers": [
    { "id": "tower_dart",  "display_name": "Dart Monkey" },
    { "id": "tower_tack",  "display_name": "Tack Shooter" },
    { "id": "tower_bomb",  "display_name": "Bomb Shooter" },
    { "id": "tower_ice",   "display_name": "Ice Tower" }
  ],

  "heroes": [
    { "id": "saika_magoichi", "display_name": "雜賀孫市", "title": "千里狙擊手" },
    { "id": "date_masamune",  "display_name": "伊達政宗", "title": "獨眼龍" }
  ],

  "abilities": [
    { "id": "sniper_mode",            "display_name": "狙擊模式" },
    { "id": "saika_reinforcements",   "display_name": "雜賀援軍" },
    { "id": "rain_iron_cannon",       "display_name": "雨鐵砲" },
    { "id": "three_stage_technique",  "display_name": "三段擊" },
    { "id": "flame_blade",            "display_name": "炎刃" },
    { "id": "fire_dash",              "display_name": "火焰衝擊" },
    { "id": "flame_assault",          "display_name": "炎襲" },
    { "id": "matchlock_gun",          "display_name": "火繩銃" }
  ],

  "buffs": [
    { "id": "stun",           "display_name": "暈眩" },
    { "id": "slow",           "display_name": "減速" },
    { "id": "burn",           "display_name": "燃燒" },
    { "id": "sniper_mode",    "display_name": "狙擊姿態" },
    { "id": "three_stage",    "display_name": "三段擊" }
  ],

  "summons": [
    { "id": "saika_gunner", "display_name": "雜賀鐵炮兵" }
  ],

  "creeps": [
    { "id": "training_mage",  "display_name": "訓練法師" },
    { "id": "fire_mage",      "display_name": "火焰法師" },
    { "id": "great_mage",     "display_name": "大法師" },
    { "id": "practice_dummy", "display_name": "練習假人" },
    { "id": "moving_target",  "display_name": "移動靶" },
    { "id": "training_creep", "display_name": "訓練小兵" },
    { "id": "armored_dummy",  "display_name": "裝甲假人" },
    { "id": "forest_ghost",   "display_name": "森林幽靈" },
    { "id": "wolf",           "display_name": "野狼" },
    { "id": "melee_minion",   "display_name": "近戰兵" },
    { "id": "ranged_minion",  "display_name": "遠程兵" },
    { "id": "siege_minion",   "display_name": "攻城兵" },
    { "id": "saika_gunner",   "display_name": "雜賀鐵炮兵" },
    { "id": "sharpshooter",   "display_name": "神射手" }
  ],

  "projectile_kinds": [
    { "id": "dart" },
    { "id": "spike_opult" },
    { "id": "tack" },
    { "id": "tack_blade" },
    { "id": "bomb" },
    { "id": "bomb_frag" },
    { "id": "saika_shot" },
    { "id": "ice" },
    { "id": "icicle" }
  ]
}
```

**Step 3: Commit**

```bash
git add omb/Story/templates.json
git commit -m "feat(templates): seed Story/templates.json with aggregated template ids"
```

---

### Task 3: Build.rs 讀 JSON 分配 id、驗證

**Files:**
- Modify: `omoba-template-ids/build.rs`

**Step 1: 寫測試（build.rs 本身難測，改寫驗證 logic 在 lib module 內）**

暫略 — build.rs 的正確性透過下游 crate compile + Task 5 unit test 驗證。

**Step 2: 實作完整 build.rs**

```rust
// omoba-template-ids/build.rs
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)] towers: Vec<Entry>,
    #[serde(default)] heroes: Vec<HeroEntry>,
    #[serde(default)] abilities: Vec<Entry>,
    #[serde(default)] buffs: Vec<Entry>,
    #[serde(default)] summons: Vec<Entry>,
    #[serde(default)] creeps: Vec<Entry>,
    #[serde(default)] projectile_kinds: Vec<ProjKind>,
}

#[derive(Deserialize)]
struct Entry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
}

#[derive(Deserialize)]
struct HeroEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] title: String,
    #[serde(default)] tombstone: bool,
}

#[derive(Deserialize)]
struct ProjKind {
    id: String,
    #[serde(default)] tombstone: bool,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json_path = manifest_dir.parent().unwrap().join("omb/Story/templates.json");
    println!("cargo:rerun-if-changed={}", json_path.display());

    let raw = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("read {}: {}", json_path.display(), e));
    let m: Manifest = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse templates.json: {}", e));

    let mut out = String::new();
    out.push_str("// AUTO-GENERATED by omoba-template-ids/build.rs — DO NOT EDIT\n\n");

    emit_namespace(&mut out, "Tower",      "TOWER",      &m.towers, true);
    emit_hero_namespace(&mut out, &m.heroes);
    emit_namespace(&mut out, "Ability",    "ABILITY",    &m.abilities, true);
    emit_namespace(&mut out, "Buff",       "BUFF",       &m.buffs, true);
    emit_namespace(&mut out, "Summon",     "SUMMON",     &m.summons, true);
    emit_namespace(&mut out, "Creep",      "CREEP",      &m.creeps, true);
    emit_projectile_kinds(&mut out, &m.projectile_kinds);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/template_ids_gen.rs", out_dir);
    fs::write(&out_path, out).unwrap();
}

fn emit_namespace(out: &mut String, ty: &str, prefix: &str, entries: &[Entry], has_display: bool) {
    // newtype
    out.push_str(&format!(
        "#[repr(transparent)]\n\
         #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]\n\
         pub struct {}Id(pub u16);\n\
         impl {}Id {{\n\
         \tpub const UNSPECIFIED: Self = Self(0);\n\
         \tpub const fn raw(self) -> u16 {{ self.0 }}\n\
         }}\n\n", ty, ty,
    ));

    // consts
    let mut seen_ids: HashSet<&str> = HashSet::new();
    let mut next: u16 = 1;
    for e in entries {
        if e.tombstone { next += 1; continue; }
        if !seen_ids.insert(&e.id) {
            panic!("duplicate {} id: {}", ty, e.id);
        }
        out.push_str(&format!(
            "pub const TPL_{}_{}: {}Id = {}Id({});\n",
            prefix, e.id.to_uppercase(), ty, ty, next,
        ));
        next += 1;
    }
    out.push('\n');

    // forward lookup: id_string → Option<NewtypeId>
    out.push_str(&format!(
        "pub fn {}_by_name(s: &str) -> Option<{}Id> {{\n\
         \tmatch s {{\n",
        prefix.to_lowercase(), ty,
    ));
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            out.push_str(&format!("\t\t\"{}\" => Some({}Id({})),\n", e.id, ty, next));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");

    // reverse lookup: id → &'static str (original id string, for logs/compat)
    out.push_str(&format!(
        "pub fn {}_id_str(id: {}Id) -> &'static str {{\n\
         \tmatch id.0 {{\n\
         \t\t0 => \"\",\n",
        prefix.to_lowercase(), ty,
    ));
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            out.push_str(&format!("\t\t{} => \"{}\",\n", next, e.id));
        }
        next += 1;
    }
    out.push_str("\t\t_ => {\n\
                 \t\t\tdebug_assert!(false, \"unknown id: {}\", id.0);\n\
                 \t\t\t\"?\"\n\
                 \t\t}\n\
                 \t}\n}\n\n");

    // display_name lookup
    if has_display {
        out.push_str(&format!(
            "pub fn {}_display(id: {}Id) -> &'static str {{\n\
             \tmatch id.0 {{\n\
             \t\t0 => \"\",\n",
            prefix.to_lowercase(), ty,
        ));
        let mut next: u16 = 1;
        for e in entries {
            if !e.tombstone {
                let display = if e.display_name.is_empty() { &e.id } else { &e.display_name };
                out.push_str(&format!("\t\t{} => \"{}\",\n", next, display.replace('"', "\\\"")));
            }
            next += 1;
        }
        out.push_str("\t\t_ => {\n\
                     \t\t\tdebug_assert!(false, \"unknown id: {}\", id.0);\n\
                     \t\t\t\"?\"\n\
                     \t\t}\n\
                     \t}\n}\n\n");
    }
}

fn emit_hero_namespace(out: &mut String, entries: &[HeroEntry]) {
    // newtype + consts + id_str + display — same as emit_namespace but with title
    let converted: Vec<Entry> = entries.iter().map(|h| Entry {
        id: h.id.clone(),
        display_name: h.display_name.clone(),
        tombstone: h.tombstone,
    }).collect();
    emit_namespace(out, "Hero", "HERO", &converted, true);

    // extra: hero_title lookup
    out.push_str("pub fn hero_title(id: HeroId) -> &'static str {\n\tmatch id.0 {\n\t\t0 => \"\",\n");
    let mut next: u16 = 1;
    for h in entries {
        if !h.tombstone {
            out.push_str(&format!("\t\t{} => \"{}\",\n", next, h.title.replace('"', "\\\"")));
        }
        next += 1;
    }
    out.push_str("\t\t_ => \"\",\n\t}\n}\n\n");
}

fn emit_projectile_kinds(out: &mut String, entries: &[ProjKind]) {
    // projectile_kind 沒有 display_name（視覺 kind）
    let converted: Vec<Entry> = entries.iter().map(|p| Entry {
        id: p.id.clone(),
        display_name: String::new(),
        tombstone: p.tombstone,
    }).collect();
    emit_namespace(out, "ProjectileKind", "PROJECTILE", &converted, false);
}
```

**Step 3: 跑 `cargo build -p omoba-template-ids` 驗證**

Run: `cd D:/omoba && cargo build -p omoba-template-ids`
Expected: PASS, `target/debug/build/omoba-template-ids-*/out/template_ids_gen.rs` 內容非空

**Step 4: Commit**

```bash
git add omoba-template-ids/build.rs
git commit -m "feat(template-ids): build.rs reads templates.json and generates newtype const tables"
```

---

### Task 4: Unit tests for `omoba-template-ids`

**Files:**
- Create: `omoba-template-ids/tests/generated.rs`

**Step 1: 寫測試**

```rust
// omoba-template-ids/tests/generated.rs
use omoba_template_ids::*;

#[test]
fn tower_consts_exist() {
    assert_eq!(TPL_TOWER_TOWER_TACK.0, 2);
    assert_eq!(TPL_TOWER_TOWER_DART.0, 1);
}

#[test]
fn forward_lookup_by_name() {
    assert_eq!(tower_by_name("tower_tack"), Some(TPL_TOWER_TOWER_TACK));
    assert_eq!(tower_by_name("nonexistent"), None);
    assert_eq!(hero_by_name("saika_magoichi"), Some(TPL_HERO_SAIKA_MAGOICHI));
}

#[test]
fn reverse_id_str() {
    assert_eq!(tower_id_str(TPL_TOWER_TOWER_TACK), "tower_tack");
    assert_eq!(tower_id_str(TowerId(0)), "");
}

#[test]
fn display_name_lookup() {
    assert_eq!(creep_display(TPL_CREEP_TRAINING_MAGE), "訓練法師");
    assert_eq!(hero_display(TPL_HERO_SAIKA_MAGOICHI), "雜賀孫市");
    assert_eq!(hero_title(TPL_HERO_SAIKA_MAGOICHI), "千里狙擊手");
}

#[test]
fn unspecified_is_zero() {
    assert_eq!(TowerId::UNSPECIFIED.0, 0);
    assert_eq!(tower_display(TowerId::UNSPECIFIED), "");
}

#[test]
fn projectile_kinds_sequential() {
    assert_eq!(projectile_by_name("dart"), Some(ProjectileKindId(1)));
    assert_eq!(projectile_by_name("tack"), Some(ProjectileKindId(3)));
    assert_eq!(projectile_id_str(ProjectileKindId(3)), "tack");
}
```

**Step 2: Run tests**

Run: `cargo test -p omoba-template-ids`
Expected: All 6 tests pass

**Step 3: Commit**

```bash
git add omoba-template-ids/tests/generated.rs
git commit -m "test(template-ids): roundtrip + display + unspecified-zero tests"
```

---

## Phase B：Wire up dependencies

### Task 5: 讓 `omb-script-abi` depend on `omoba-template-ids`

**Files:**
- Modify: `scripts/script-abi/Cargo.toml`
- Modify: `scripts/script-abi/src/lib.rs`

**Step 1: 加 dependency**

在 `scripts/script-abi/Cargo.toml` 的 `[dependencies]` 加：
```toml
omoba-template-ids = { path = "../../omoba-template-ids" }
```

**Step 2: Re-export newtypes**

在 `scripts/script-abi/src/lib.rs` `prelude` module 加：
```rust
pub use omoba_template_ids::{
    TowerId, HeroId, AbilityId, BuffId, SummonId, CreepId, ProjectileKindId,
};
```

**Step 3: Verify scripts workspace builds**

Run: `cd D:/omoba/scripts && cargo check -p omb-script-abi`
Expected: PASS

**Step 4: Commit**

```bash
git add scripts/script-abi/
git commit -m "feat(script-abi): depend on omoba-template-ids, re-export newtypes"
```

---

### Task 6: 讓 `omoba-core` depend on `omoba-template-ids`

**Files:**
- Modify: `omoba-core/Cargo.toml`

**Step 1: 加 dependency**

在 `omoba-core/Cargo.toml` 的 `[dependencies]` 加：
```toml
omoba-template-ids = { path = "../omoba-template-ids" }
```

**Step 2: 暫不改 `omoba-core/src/lib.rs`**（Task 20 才刪 `template_ids.rs`）

**Step 3: Verify**

Run: `cargo check -p omoba-core`
Expected: PASS

**Step 4: Commit**

```bash
git add omoba-core/Cargo.toml
git commit -m "feat(omoba-core): depend on omoba-template-ids"
```

---

### Task 7: 讓 `omb` depend on `omoba-template-ids`

**Files:**
- Modify: `omb/Cargo.toml`

**Step 1: 加 dependency**

在 `omb/Cargo.toml` 的 `[dependencies]` 加：
```toml
omoba-template-ids = { path = "../omoba-template-ids" }
```

（注意：omb 是獨立 workspace 在 `omb/`，所以 path 是 `../omoba-template-ids`。但 repo root workspace 的 `Cargo.toml` 不含 omb。）

**Step 2: Verify**

Run: `cd D:/omoba/omb && cargo check -p omobab`
Expected: PASS

**Step 3: Commit**

```bash
git add omb/Cargo.toml
git commit -m "feat(omb): depend on omoba-template-ids"
```

---

## Phase C：FFI ABI 換型（Breaking change, scripts/abi）

### Task 8: 加 `UnitTemplateId` + `UnitTemplateKind` 到 script-abi

**Files:**
- Modify: `scripts/script-abi/src/types.rs`

**Step 1: 加定義**

在 `types.rs` 末尾加：
```rust
#[repr(u8)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq)]
pub enum UnitTemplateKind {
    Tower,
    Hero,
    Creep,
    Summon,
}

/// 跨 tower/hero/creep/summon 的統一 unit template id，u16 空間依 kind 分 namespace。
/// 由 `UnitScript::unit_id()` 回傳；FFI 層用 `#[repr(C)]` 打包 kind + 純 u16。
#[repr(C)]
#[derive(StableAbi, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct UnitTemplateId {
    pub kind: UnitTemplateKind,
    pub id: u16,
}

impl UnitTemplateId {
    pub const UNSPECIFIED: Self = Self { kind: UnitTemplateKind::Tower, id: 0 };
    pub const fn tower(t: crate::TowerId) -> Self { Self { kind: UnitTemplateKind::Tower,  id: t.0 } }
    pub const fn hero(h: crate::HeroId)   -> Self { Self { kind: UnitTemplateKind::Hero,   id: h.0 } }
    pub const fn creep(c: crate::CreepId) -> Self { Self { kind: UnitTemplateKind::Creep,  id: c.0 } }
    pub const fn summon(s: crate::SummonId) -> Self { Self { kind: UnitTemplateKind::Summon, id: s.0 } }
}
```

注意：`crate::TowerId` 需要 re-export 到 crate root（不只 prelude）。修 `lib.rs`：
```rust
pub use omoba_template_ids::{
    TowerId, HeroId, AbilityId, BuffId, SummonId, CreepId, ProjectileKindId,
};
```

**Step 2: Verify**

Run: `cd D:/omoba/scripts && cargo check -p omb-script-abi`
Expected: PASS

**Step 3: Commit**

```bash
git add scripts/script-abi/src/
git commit -m "feat(script-abi): add UnitTemplateId + UnitTemplateKind types"
```

---

### Task 9: 改 `UnitScript::unit_id() -> UnitTemplateId`

**Files:**
- Modify: `scripts/script-abi/src/script.rs` (line 20)

**Step 1: 改 trait signature**

把 `script.rs` 第 20 行：
```rust
fn unit_id(&self) -> RStr<'_>;
```
改成：
```rust
fn unit_id(&self) -> UnitTemplateId;
```

同步改 `types` import 含 `UnitTemplateId`。

**Step 2: 同步改 `skill_id`, `state_id`, `ability_id`, `modifier_id`, `order_kind` 等 `RStr` 欄位？**

分兩類：
- **已有 newtype 對應**（ability_id → `AbilityId`，modifier_id → `BuffId`）：改
- **目前沒有 newtype**（state_id 狀態字串、order_kind）：**留 RStr 這輪不動**（非 wire payload，非 high-frequency）

本 task 只改 `unit_id`。其他 id 欄位在後續 tasks 再處理。

**Step 3: Verify（會有一堆 compile error — scripts/base_content 暫時壞，正常）**

Run: `cd D:/omoba/scripts && cargo check -p omb-script-abi`
Expected: PASS（script-abi 本身）

Run: `cd D:/omoba/scripts && cargo check -p base_content`
Expected: FAIL（base_content 所有 `impl UnitScript` 的 `unit_id` 還回 RStr）— 記下 error 數量

**Step 4: Commit**

```bash
git add scripts/script-abi/src/script.rs
git commit -m "refactor(script-abi): UnitScript::unit_id returns UnitTemplateId (wip — base_content fix in Task 11)"
```

---

### Task 10: 改 `AbilityScript::ability_id() -> AbilityId`

**Files:**
- Modify: `scripts/script-abi/src/ability.rs` (line 30)

**Step 1: 改 trait signature**

把 `ability.rs` 第 30 行：
```rust
fn ability_id(&self) -> RStr<'_>;
```
改成：
```rust
fn ability_id(&self) -> crate::AbilityId;
```

**Step 2: Verify**

Run: `cd D:/omoba/scripts && cargo check -p omb-script-abi`
Expected: PASS（script-abi 本身）

**Step 3: Commit**

```bash
git add scripts/script-abi/src/ability.rs
git commit -m "refactor(script-abi): AbilityScript::ability_id returns AbilityId"
```

---

### Task 11: 改 `GameWorld` 的 buff API 從 `RStr` 改 `BuffId`

**Files:**
- Modify: `scripts/script-abi/src/world.rs` (lines 57–71)

**Step 1: 改 method signatures**

```rust
fn add_buff(&mut self, target: EntityHandle, buff_id: crate::BuffId, duration: f32);
fn remove_buff(&mut self, target: EntityHandle, buff_id: crate::BuffId);
fn has_buff(&self, target: EntityHandle, buff_id: crate::BuffId) -> bool;
fn add_stat_buff(
    &mut self,
    target: EntityHandle,
    buff_id: crate::BuffId,
    duration: f32,
    modifiers_json: RStr<'_>,
);
```

**Step 2: 改 `spawn_projectile` 的 `kind_tag: RString` 成 `ProjectileKindId`**

grep `ProjectileSpec` struct 定義：
```rust
pub struct ProjectileSpec {
    // ...
    pub kind_tag: RString,    // ← 改成
    pub kind: crate::ProjectileKindId,
}
```

**Step 3: Verify**

Run: `cd D:/omoba/scripts && cargo check -p omb-script-abi`
Expected: PASS

**Step 4: Commit**

```bash
git add scripts/script-abi/src/world.rs scripts/script-abi/src/types.rs
git commit -m "refactor(script-abi): GameWorld buff API + ProjectileSpec use newtype ids"
```

---

## Phase D：Scripts/base_content 遷移

### Task 12: 修復 base_content towers（4 個塔）

**Files:**
- Modify: `scripts/base_content/src/towers/dart.rs`
- Modify: `scripts/base_content/src/towers/tack.rs`
- Modify: `scripts/base_content/src/towers/bomb.rs`
- Modify: `scripts/base_content/src/towers/ice.rs`

**Step 1: 每個塔檔套路一致**

```rust
// 舊
fn unit_id(&self) -> RStr<'_> {
    RStr::from_str("tower_tack")
}

// 新
fn unit_id(&self) -> UnitTemplateId {
    UnitTemplateId::tower(TPL_TOWER_TOWER_TACK)
}
```

`spawn_projectile` 呼叫處：
```rust
// 舊
kind_tag: RString::from("tack"),

// 新
kind: TPL_PROJECTILE_TACK,
```

`add_buff` / `has_buff` 呼叫處：
```rust
// 舊
w.add_buff(target, RStr::from_str("slow"), 2.0);

// 新
w.add_buff(target, TPL_BUFF_SLOW, 2.0);
```

**Step 2: Verify**

Run: `cd D:/omoba/scripts && cargo check -p base_content`
Expected: tower-related errors 降至 0（heroes / summons / abilities 仍有 error）

**Step 3: Commit**

```bash
git add scripts/base_content/src/towers/
git commit -m "refactor(base_content): towers use template id consts instead of strings"
```

---

### Task 13: 修復 base_content heroes + abilities

**Files:**
- Modify: `scripts/base_content/src/heroes/B01_saika_magoichi/mod.rs` + 4 個 ability 檔
- Modify: `scripts/base_content/src/heroes/B02_date_masamune/mod.rs` + 4 個 ability 檔

**Step 1: 套路：`unit_id` 改 `UnitTemplateId::hero(TPL_HERO_SAIKA_MAGOICHI)`，`ability_id` 改 `TPL_ABILITY_SNIPER_MODE` 等**

**Step 2: Verify**

Run: `cd D:/omoba/scripts && cargo check -p base_content`
Expected: hero-related errors 降至 0

**Step 3: Commit**

```bash
git add scripts/base_content/src/heroes/
git commit -m "refactor(base_content): heroes + abilities use template id consts"
```

---

### Task 14: 修復 base_content summons

**Files:**
- Modify: `scripts/base_content/src/summons/saika_gunner.rs`

**Step 1: 改 unit_id / buff / projectile**

套路同上：`TPL_SUMMON_SAIKA_GUNNER` + 其他 id。

**Step 2: Verify whole scripts workspace**

Run: `cd D:/omoba/scripts && cargo build --release`
Expected: PASS; `scripts/target/release/base_content.dll` 產生

Copy to host：
```bash
cp scripts/target/release/base_content.dll omb/scripts/base_content.dll
```

**Step 3: Commit**

```bash
git add scripts/base_content/src/summons/
git commit -m "refactor(base_content): summons use template id consts; all scripts compile"
```

---

## Phase E：omb host 適配新 ABI

### Task 15: 修 omb host 對 script-abi buff / unit_id / projectile API 呼叫點

**Files:**
- 由 `cargo check -p omobab` 的 error 指引逐個改

**Step 1: grep + 修**

Run: `cd D:/omoba/omb && cargo check -p omobab 2>&1 | head -60`
Expected: 一批 signature mismatch（`unit_id_of` 回 `RStr` 但 scripts 新 signature 有 UnitTemplateId；`add_buff` 接 RStr 但 host 給 `&str` 等）

需要改 host 端：
- `GameWorld::add_buff` 的 adapter impl：把 `BuffId` 轉回 `&str`（透過 `omoba_template_ids::buff_id_str(id)`）餵給舊的 BuffStore（其 key 仍是 `String`）
- 或更激進：把 BuffStore 的 key 一次性改成 `BuffId`（較大改動；建議這輪留 key=String，adapter 做 conversion）

**Step 2: `UnitScript::unit_id` 回 `UnitTemplateId`，host 要怎麼用？**

host 原本用 `RStr` 做 HashMap key 註冊 script instance。改成：
- Registry key：`UnitTemplateId`（直接比 `(kind, id)` tuple，快且無 string alloc）
- 或 flatten 成兩個 maps：`tower_scripts: HashMap<TowerId, Box<dyn UnitScript>>` 等

**推薦**：flatten，避免跨 kind 混淆。`scripts/base_content/src/lib.rs` 的 registry 同步改。

**Step 3: Verify**

Run: `cd D:/omoba/omb && cargo check -p omobab`
Expected: PASS

**Step 4: Commit**

```bash
git add omb/src/ scripts/base_content/src/lib.rs
git commit -m "refactor(omb): host adapters + script registry use UnitTemplateId / BuffId"
```

---

### Task 16: omb 載入 `Story/templates.json` 並建立 per-id catalog

**Files:**
- Create: `omb/src/template_catalog.rs`
- Modify: `omb/src/lib.rs` / `omb/src/main.rs`（add mod）

**Step 1: catalog 結構**

```rust
// omb/src/template_catalog.rs
use omoba_template_ids::{CreepId, creep_by_name};

pub struct CreepTemplate {
    pub id: CreepId,
    pub base_hp: f32,
    pub armor: f32,
    // ... 從 templates.json 讀「如果有 base 欄位」或從現有 entity.json `enemies[]` 搬
}

pub struct TemplateCatalog {
    pub creeps: std::collections::HashMap<CreepId, CreepTemplate>,
    // heroes, towers, abilities ...
}

impl TemplateCatalog {
    pub fn load() -> Self {
        // 讀 omb/Story/templates.json 的 creeps[] / heroes[] / abilities[]
        // 對每個 entry 查 *_by_name() 得 newtype id，填 HashMap
        todo!()
    }
}
```

**Step 2: 把舊 entity.json 的 creep/hero/ability 屬性資料搬進 templates.json**

修改 `Story/templates.json`：
- `creeps[]` 內每個 entry 加 `base: { hp, armor, move_speed, damage, ... }`
- `heroes[]` 內每個 entry 加完整 stat block
- `abilities[]` 同理

**Step 3: 改 scene `entity.json` 退化成引用**

```json
// Story/TD_1/entity.json (new)
{
  "heroes_used":  ["saika_magoichi"],
  "creeps_used":  ["training_mage", "fire_mage", "great_mage"],
  "hero_spawns":  [ { "id": "saika_magoichi", "x": 100.0, "y": 200.0 } ],
  "waves":        [ /* 原 wave 定義 */ ]
}
```

**Step 4: 修 omb loader 讀新 schema + catalog cross-reference**

Find existing loader in `omb/src/...`（可能 `configs.rs` / `story.rs` / `startup`）。改成先 `TemplateCatalog::load()`，scene JSON 只讀 `*_used`。

**Step 5: Verify**

Run: `cd D:/omoba/omb && cargo build -p omobab`
Expected: PASS（可能有 runtime 風險但 compile 過）

**Step 6: Commit**

```bash
git add omb/src/template_catalog.rs omb/Story/ omb/src/
git commit -m "feat(omb): load Story/templates.json as catalog; scene entity.json refs by id"
```

---

## Phase F：Proto 破壞性遷移

### Task 17: 改 proto 欄位（TowerCreate / HeroStatic / BuffAdd / BuffRemove / BuffSnapshot）

**Files:**
- Modify: `proto/game.proto`

**Step 1: 改 schema**

```proto
message TowerCreate {
  uint64 id = 1;
  Position16 pos = 2;
  Fixed16 hp = 3;
  Fixed16 max_hp = 4;
  reserved 5, 6;          // was string kind, string name
  uint32 tower_id = 7;    // TowerId.0
}

message HeroStatic {
  uint64 id = 1;
  reserved 2, 3;          // was string name, string title
  uint32 base_str = 4;
  uint32 base_agi = 5;
  uint32 base_int = 6;
  reserved 7;             // was repeated string ability_ids
  uint32 level = 8;
  uint32 xp = 9;
  uint32 xp_next = 10;
  uint32 skill_points = 11;
  repeated AbilityLevelPair ability_levels = 12;
  uint32 hero_id = 13;    // HeroId.0
  repeated uint32 ability_ids = 14;  // AbilityId.0 per entry
}

message BuffAdd {
  uint64 entity_id = 1;
  reserved 2;             // was string buff_id
  uint32 remaining_ms = 3;
  string payload_json = 4;
  uint32 buff_id = 5;     // BuffId.0
}

message BuffRemove {
  uint64 entity_id = 1;
  reserved 2;             // was string buff_id
  uint32 buff_id = 3;
}

message BuffSnapshot {
  reserved 1;             // was string buff_id
  uint32 remaining_ms = 2;
  string payload_json = 3;
  uint32 buff_id = 4;     // BuffId.0
}
```

**Step 2: Verify proto 語法**

Run: `cd D:/omoba/omb && cargo check --features kcp -p omobab`
Expected: FAIL — `TowerCreate.kind` field 被 omb 引用處會炸

**Step 3: Commit (schema only)**

```bash
git add proto/game.proto
git commit -m "refactor(proto): replace string template ids with uint32 in high-frequency events"
```

---

### Task 18: 修 omb encoder 用新 proto 欄位

**Files:**
- Modify: omb 內所有 build `TowerCreate{...}` / `HeroStatic{...}` / `BuffAdd{...}` 的地方

**Step 1: grep + 改**

grep `TowerCreate {` / `HeroStatic {` / `BuffAdd {` / `BuffRemove {` / `BuffSnapshot {` 找 site，每個 site 把舊 string field 換成 newtype.0。

範例：
```rust
// 舊
TowerCreate {
    id: e.id() as u64,
    kind: kind_str.to_string(),
    name: label.to_string(),
    // ...
}

// 新
TowerCreate {
    id: e.id() as u64,
    tower_id: tower_id.0 as u32,
    // ...
}
```

**Step 2: Verify**

Run: `cd D:/omoba/omb && cargo check --features kcp -p omobab`
Expected: PASS

**Step 3: Commit**

```bash
git add omb/src/
git commit -m "fix(omb): encoder fills new uint32 template-id fields in proto events"
```

---

### Task 19: 修 omoba-core kcp client decoder 用新 proto 欄位

**Files:**
- Modify: `omoba-core/src/kcp/client.rs`

**Step 1: grep `TowerCreate` / `HeroStatic` / `BuffAdd` decode site**

把 parse 出舊 string field 的 code 改成 `TowerId(msg.tower_id as u16)` 等 newtype 包裝。產生 legacy-shape JSON 給 omfx 時，用 `tower_display(TowerId(x))` 反查字串塞回去。

**Step 2: Verify**

Run: `cargo check -p omoba-core --features kcp`
Expected: PASS

**Step 3: Commit**

```bash
git add omoba-core/src/kcp/client.rs
git commit -m "fix(omoba-core): kcp decoder unpacks new uint32 fields, reverse-looks up display strings"
```

---

## Phase G：清理 + Smoke test

### Task 20: 刪除 `omoba-core/src/template_ids.rs`

**Files:**
- Delete: `omoba-core/src/template_ids.rs`
- Modify: `omoba-core/src/lib.rs`（刪 `pub mod template_ids;`）
- Modify: 所有 `use omoba_core::template_ids::*` 的 site 改成 `use omoba_template_ids::*`

**Step 1: grep `template_ids::` 所有 call site**

Run: `grep -r "omoba_core::template_ids\|crate::template_ids" --include="*.rs"`

**Step 2: 逐個改 import + 函式名（`encode_projectile_kind` 已無對應 — 不再 encode，而是 native newtype；`projectile_kind_by_id(id: u32)` → `projectile_id_str(ProjectileKindId(id as u16))`）**

**Step 3: 刪檔 + 改 lib.rs**

```rust
// omoba-core/src/lib.rs
// pub mod template_ids;  ← 刪
// pub use template_ids::...;  ← 刪
```

**Step 4: Verify**

Run: `cargo check --workspace`
Expected: PASS

Run: `cd D:/omoba/omb && cargo check -p omobab`
Expected: PASS

Run: `cd D:/omoba/scripts && cargo check --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git rm omoba-core/src/template_ids.rs
git add omoba-core/ omb/ scripts/
git commit -m "refactor: remove legacy omoba-core/template_ids.rs; all consumers use omoba-template-ids"
```

---

### Task 21: Smoke test — build + run TD_1

**Files:** 無 — 跑 `run.bat`

**Step 1: Full build（debug）**

Run: `D:/omoba/run.bat`
Expected: scripts + omb + omfx 全部 compile；omfx 視窗出現

**Step 2: 跑 TD_1 一波**

手動在遊戲內驗證：
- creep 生成 — 中文名（訓練法師 / 火焰法師）顯示正確
- tower 放置 — name label（Tack Shooter / Dart Monkey）顯示
- projectile 命中有 slow/burn 效果
- 無 console error `unknown CreepId` / `unknown BuffId`

**Step 3: 沒 regression 就 tag commit**

```bash
git commit --allow-empty -m "smoke: TD_1 full gameplay loop ok with template id codegen"
```

---

### Task 22: Smoke test — TD_STRESS wire bandwidth check

**Files:** 無

**Step 1: 跑 stress**

Run: `D:/omoba/run_stress.bat`

**Step 2: 觀察**

- 打開 omb 的 bandwidth stat print（`GameEvent.payload_bytes` 累計）
- 對比設計文件估計：`ProjectileCreate` + `CreepCreate` + `BuffAdd` 合計每秒下降 ~10–12 KB
- FPS 不應下降（廣播改動只影響 wire 非 CPU）

**Step 3: 若超出預期差異（例如沒下降）**

- profile `GameEvent.encoded_len()` 前後對比 — 可能 proto encode 沒真的用到新欄位
- 確認 Task 18 的 encoder 改全

**Step 4: Commit empty marker**

```bash
git commit --allow-empty -m "smoke: TD_STRESS shows expected wire bandwidth drop"
```

---

### Task 23: Final cleanup — grep residual string ids

**Files:** 全 repo grep

**Step 1: grep scripts 殘留字串 id**

Run: `grep -rn 'RStr::from_str("tower_' scripts/base_content/`
Run: `grep -rn 'RStr::from_str("saika\|RStr::from_str("date_' scripts/base_content/`
Run: `grep -rn 'RString::from("tack\|RString::from("bomb\|RString::from("saika_shot' scripts/base_content/`

Expected: 0 matches

**Step 2: grep proto `string` 欄位殘留高頻訊息**

Run: `grep 'string.*=' proto/game.proto | grep -iE 'kind|name|buff|ability|tower'`
Expected: 只剩刻意保留的（`display_name` 這類 client-side 不用的）

**Step 3: grep `KNOWN_PROJECTILE_KINDS` / `KNOWN_CREEP_NAMES`**

Run: `grep -r "KNOWN_PROJECTILE_KINDS\|KNOWN_CREEP_NAMES" --include="*.rs"`
Expected: 0 matches

**Step 4: Final commit**

若有殘留，修；否則：
```bash
git commit --allow-empty -m "feat: template id codegen migration complete (design: 2026-04-25)"
```

---

## Rollback guide

若 Phase F（proto）出大問題：
1. `git revert` Task 17-19 commits（proto + encoder + decoder）
2. 保留 Phase A-E（template_ids crate + scripts 新 signature）
3. Wire 暫回字串，但編譯期安全保留

若 Phase C（ABI）出 abi_stable 相容性問題：
1. 回退 `UnitTemplateId` 改成裸 `u16`（host 自己記得哪個 script 註冊在哪個 kind）
2. 或回退成 `RStr<'_>`，只做 Phase F 的 proto/wire 優化（放棄編譯期安全）

---

## Phase 範圍總結

| Phase | Tasks | 主要風險 |
|-------|-------|---------|
| A. Foundation | 1-4 | build.rs 決定性順序 |
| B. Wire-up deps | 5-7 | path dep 跨 workspace 是否順暢 |
| C. Script-abi ABI 換型 | 8-11 | abi_stable newtype + repr(transparent) 相容性 |
| D. base_content 遷移 | 12-14 | 遍歷所有 script 檔不漏 |
| E. omb host 適配 | 15-16 | BuffStore key 是否同步換 |
| F. Proto 破壞性改 | 17-19 | encoder/decoder 對稱性、舊 field 真的 reserved |
| G. Cleanup + smoke | 20-23 | 回歸測試覆蓋 |

**Total: 23 tasks, ~2-3 個工作天（單人）**

## Checkpoints / 驗證節點

- Task 4 後：`cargo test -p omoba-template-ids` 全綠
- Task 14 後：`cd scripts && cargo build --release` 全綠
- Task 16 後：`cd omb && cargo check -p omobab` 全綠
- Task 19 後：`cargo check --workspace` 全綠
- Task 21 後：TD_1 遊玩無 regression
- Task 22 後：TD_STRESS wire 下降 ~10-12 KB/s

## Skills reference

- @superpowers:executing-plans — task-by-task 執行
- @superpowers:subagent-driven-development — 每 task 用 fresh subagent + review
- @superpowers:test-driven-development — Task 4 / 測試先行
- @superpowers:verification-before-completion — 每 Task 的 "Verify" step 不可跳
