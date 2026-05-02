# Templates.json Unification Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 把 omoba monorepo 所有單位數值資料（hero/creep/summon stats、hero→abilities 關聯、ability cooldown/mana 等、tower 升級樹）統一到 `omb/Story/templates.json`，讓 `omoba_template_ids` 透過 build-time codegen 產生 const + lookup function 變成所有 crate 唯一資料查詢入口。

**Architecture:** templates.json = 唯一資料來源；`omoba-template-ids/build.rs` 編譯期讀 JSON 生成 typed const + lookup function；scripts (base_content) 只寫邏輯（execute / on_tick），數值從生成 const 取；host (omb) / omoba-core / omfx 全部禁止本地寫 match 表（例：`match hero_type { "saika_magoichi" => ... }` 是 anti-pattern，必須改 `hero_abilities(HERO_SAIKA_MAGOICHI)` lookup）。

**Tech Stack:** Rust 1.91.0、abi_stable FFI、serde_json、build.rs codegen。

**執行順序：A → B → E → C → D**（低風險先做）。每 Phase 結尾 commit。

---

## Pre-flight

### Task 0: 確認起始狀態

**目的：** 確認目前 master 沒未提交改動，build/test 都綠。

**Step 1: 檢查 git 狀態**

```bash
cd /d/omoba && git status
cd /d/omoba/omb && git status
cd /d/omoba/omfx && git status
```
Expected: working tree clean (除了 `m omb` submodule pointer)。

**Step 2: 編譯 + 測試現狀作 baseline**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: 編譯 clean、`test result: ok. 129 passed; 0 failed`。

---

## Phase A — Hero → abilities lookup（消除 match 表）

### Task A1: 加 abilities 欄位到 templates.json heroes[]

**Files:**
- Modify: `D:/omoba/omb/Story/templates.json`

**Step 1: 編輯 heroes[]**

每個 hero 加 `abilities` 字串陣列：

```json
"heroes": [
  {
    "id": "saika_magoichi",
    "display_name": "雜賀孫市",
    "title": "千里狙擊手",
    "abilities": ["sniper_mode", "saika_reinforcements", "rain_iron_cannon", "three_stage_technique"]
  },
  {
    "id": "date_masamune",
    "display_name": "伊達政宗",
    "title": "獨眼龍",
    "abilities": ["flame_blade", "fire_dash", "flame_assault", "matchlock_gun"]
  }
]
```

**Step 2: JSON 驗證**

```bash
py -3 -c "import json; json.load(open('D:/omoba/omb/Story/templates.json'))"
```
Expected: 無 output（JSON valid）。

### Task A2: 擴 HeroEntry deserialize 加 abilities

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs:31-37` (HeroEntry struct)

**Step 1: 修改 HeroEntry**

把 build.rs 裡 HeroEntry 加上 `abilities` 欄位：

```rust
#[derive(Deserialize)]
struct HeroEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] title: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] abilities: Vec<String>,
}
```

**Step 2: 確認 build.rs 編譯**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: `Finished` clean。

### Task A3: build.rs 加 ability id 查找表 + emit hero abilities

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs` (function `emit_hero_namespace`)

**Step 1: 在 main() emit_tower_namespace 之前 build ability_id_map**

把 main() 裡擴成在處理 heroes 之前先 build `HashMap<&str, u16>` 給 ability id 查找：

```rust
// 在 emit_tower_namespace 之後、emit_hero_namespace 之前：
let mut ability_id_map: std::collections::HashMap<&str, u16> = std::collections::HashMap::new();
let mut next_aid: u16 = 1;
for e in &m.abilities {
    if !e.tombstone {
        ability_id_map.insert(&e.id, next_aid);
    }
    next_aid += 1;
}
emit_hero_abilities(&mut out, &m.heroes, &ability_id_map);
```

**Step 2: 加 emit_hero_abilities 函式**

```rust
fn emit_hero_abilities(
    out: &mut String,
    entries: &[HeroEntry],
    ability_ids: &std::collections::HashMap<&str, u16>,
) {
    // Per-hero const: HERO_<NAME>_ABILITIES
    for e in entries {
        if e.tombstone { continue; }
        let cname = const_name("hero", &e.id);
        out.push_str(&format!(
            "pub const {}_ABILITIES: &[AbilityId] = &[\n",
            cname,
        ));
        for ab_id in &e.abilities {
            let raw = ability_ids.get(ab_id.as_str()).unwrap_or_else(||
                panic!("hero '{}' references unknown ability '{}'", e.id, ab_id));
            out.push_str(&format!("\tAbilityId({}),\n", raw));
        }
        out.push_str("];\n");
    }
    out.push('\n');

    // Lookup hero_abilities(HeroId) -> &'static [AbilityId]
    out.push_str("pub fn hero_abilities(id: HeroId) -> &'static [AbilityId] {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("hero", &e.id);
            out.push_str(&format!("\t\t{} => {}_ABILITIES,\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => &[],\n\t}\n}\n\n");
}
```

**Step 3: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean。

**Step 4: 確認生成內容正確**

```bash
grep -A8 "HERO_SAIKA_MAGOICHI_ABILITIES" /d/omoba/omoba-template-ids/target/debug/build/omoba-template-ids-*/out/template_ids_gen.rs | head -10
```
Expected: 看到 4 個 AbilityId 對應 sniper_mode / saika_reinforcements / rain_iron_cannon / three_stage_technique 的 raw u16。

### Task A4: 加 unit test 驗證 hero_abilities 正確性

**Files:**
- Modify: `D:/omoba/omoba-template-ids/tests/` (建檔)

**Step 1: 寫測試**

新增 `D:/omoba/omoba-template-ids/tests/hero_abilities.rs`：

```rust
use omoba_template_ids::*;

#[test]
fn saika_magoichi_has_4_abilities() {
    let abs = hero_abilities(HERO_SAIKA_MAGOICHI);
    assert_eq!(abs.len(), 4);
    assert_eq!(abs[0], ABILITY_SNIPER_MODE);
    assert_eq!(abs[1], ABILITY_SAIKA_REINFORCEMENTS);
    assert_eq!(abs[2], ABILITY_RAIN_IRON_CANNON);
    assert_eq!(abs[3], ABILITY_THREE_STAGE_TECHNIQUE);
}

#[test]
fn date_masamune_has_4_abilities() {
    let abs = hero_abilities(HERO_DATE_MASAMUNE);
    assert_eq!(abs.len(), 4);
    assert_eq!(abs[0], ABILITY_FLAME_BLADE);
    assert_eq!(abs[1], ABILITY_FIRE_DASH);
    assert_eq!(abs[2], ABILITY_FLAME_ASSAULT);
    assert_eq!(abs[3], ABILITY_MATCHLOCK_GUN);
}

#[test]
fn unknown_hero_has_no_abilities() {
    let abs = hero_abilities(HeroId::UNSPECIFIED);
    assert_eq!(abs.len(), 0);
}
```

**Step 2: 執行**

```bash
cargo test --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: 3 tests pass。

### Task A5: 改 omoba-core entities.rs init_hero_abilities

**Files:**
- Modify: `D:/omoba/omoba-core/src/state/entities.rs:101-116`

**Step 1: 替換 match 表**

```rust
fn init_hero_abilities(hero_type: &str) -> Vec<AbilityState> {
    use omoba_template_ids::{hero_abilities, hero_by_name};
    let id = hero_by_name(hero_type).unwrap_or_default();
    hero_abilities(id)
        .iter()
        .map(|aid| AbilityState {
            ability_id: aid.as_str().to_string(),
            level: 1,
            cooldown_remaining: 0.0,
            is_available: true,
            last_used: None,
        })
        .collect()
}
```

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```
Expected: clean。

### Task A6: 改 omoba-core simulator.rs get_hero_abilities

**Files:**
- Modify: `D:/omoba/omoba-core/src/input/simulator.rs:140-157`

**Step 1: 替換 match 表**

```rust
pub fn get_hero_abilities(&self) -> Vec<String> {
    use omoba_template_ids::{hero_abilities, hero_by_name};
    let id = hero_by_name(&self.hero_type).unwrap_or_default();
    hero_abilities(id)
        .iter()
        .map(|aid| aid.as_str().to_string())
        .collect()
}
```

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```
Expected: clean。

### Task A7: Verify Phase A integration

**Step 1: 全 workspace build**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

**Step 2: Run host tests**

```bash
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: 全綠。

**Step 3: Smoke test — 跑 run.bat 並按 W**

```bash
cd /d/omoba && cmd /c run.bat
```
等出現 hero，按 `W` 鍵（sniper_mode）；應看到 buff 效果與 log "[sniper_mode] toggled ON"。
按 `:q` 退出 omb stdin。

### Task A8: Commit Phase A

**Step 1: Stage + commit omoba-template-ids**

```bash
cd /d/omoba/omoba-template-ids
git add -A
git commit -m "templates: hero_abilities() lookup from templates.json heroes[].abilities"
```

**Step 2: Stage + commit omoba-core**

```bash
cd /d/omoba/omoba-core
git add src/state/entities.rs src/input/simulator.rs
git commit -m "templates: init_hero_abilities / get_hero_abilities 走 hero_abilities() lookup，消除 match 表"
```

**Step 3: Stage + commit omb's templates.json**

```bash
cd /d/omoba/omb
git add Story/templates.json
git commit -m "templates: heroes[] 加 abilities 欄位（單一來源）"
```

**Step 4: Bump main repo submodules**

```bash
cd /d/omoba
git add omb omoba-core omoba-template-ids 2>/dev/null
git commit -m "chore: bump submodules for Phase A — hero abilities lookup"
```

---

## Phase B — Hero / Creep / Summon stats 全搬 templates.json

### Task B1: 加 LevelGrowth + HeroStats + CreepStats + SummonStats struct

**Files:**
- Modify: `D:/omoba/omoba-template-ids/src/lib.rs`

**Step 1: 在 TowerStats 之後加新 struct**

```rust
#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct LevelGrowth {
    pub strength_per_level: f32,
    pub agility_per_level: f32,
    pub intelligence_per_level: f32,
    pub damage_per_level: f32,
    pub hp_per_level: f32,
    pub mana_per_level: f32,
}

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct HeroStats {
    pub strength: i32,
    pub agility: i32,
    pub intelligence: i32,
    pub primary_attribute: u8, // 0=str, 1=agi, 2=int
    pub attack_range: f32,
    pub base_damage: i32,
    pub base_armor: f32,
    pub base_hp: i32,
    pub base_mana: i32,
    pub move_speed: f32,
    pub turn_speed: f32,
    pub level_growth: LevelGrowth,
}

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct CreepStats {
    pub hp: f32,
    pub armor: f32,
    pub magic_resistance: f32,
    pub damage: f32,
    pub attack_range: f32,
    pub move_speed: f32,
    pub enemy_type: u8, // codegen enum: caster=0/melee=1/ranged=2/boss=3
    pub ai_type: u8,    // codegen enum: defensive=0/aggressive=1/patrol=2/guard=3/passive=4/berserker=5
    pub exp_reward: i32,
    pub gold_reward: i32,
}

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct SummonStats {
    pub hp: f32,
    pub damage: f32,
    pub duration: f32,
    pub move_speed: f32,
}
```

**Step 2: 確認 lib.rs 編譯**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean。

### Task B2: 擴 HeroEntry / 新增 CreepEntry / SummonEntry deserialize

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs`

**Step 1: 擴 HeroEntry 加 stat 欄位**

```rust
#[derive(Deserialize)]
struct HeroEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] title: String,
    #[serde(default)] background: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] abilities: Vec<String>,
    #[serde(default)] strength: i32,
    #[serde(default)] agility: i32,
    #[serde(default)] intelligence: i32,
    #[serde(default)] primary_attribute: String, // "strength" | "agility" | "intelligence"
    #[serde(default)] attack_range: f32,
    #[serde(default)] base_damage: i32,
    #[serde(default)] base_armor: f32,
    #[serde(default)] base_hp: i32,
    #[serde(default)] base_mana: i32,
    #[serde(default)] move_speed: f32,
    #[serde(default)] turn_speed: f32,
    #[serde(default)] level_growth: HeroLevelGrowthEntry,
}

#[derive(Deserialize, Default)]
struct HeroLevelGrowthEntry {
    #[serde(default)] strength_per_level: f32,
    #[serde(default)] agility_per_level: f32,
    #[serde(default)] intelligence_per_level: f32,
    #[serde(default)] damage_per_level: f32,
    #[serde(default)] hp_per_level: f32,
    #[serde(default)] mana_per_level: f32,
}
```

**Step 2: 把 Manifest.creeps Entry 改成 CreepEntry**

```rust
#[derive(Deserialize)]
struct CreepEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] hp: f32,
    #[serde(default)] armor: f32,
    #[serde(default)] magic_resistance: f32,
    #[serde(default)] damage: f32,
    #[serde(default)] attack_range: f32,
    #[serde(default)] move_speed: f32,
    #[serde(default)] enemy_type: String,
    #[serde(default)] ai_type: String,
    #[serde(default)] exp_reward: i32,
    #[serde(default)] gold_reward: i32,
}
```

**Step 3: 把 Manifest.summons Entry 改成 SummonEntry**

```rust
#[derive(Deserialize)]
struct SummonEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] hp: f32,
    #[serde(default)] damage: f32,
    #[serde(default)] duration: f32,
    #[serde(default)] move_speed: f32,
}
```

**Step 4: Manifest 改用新 type**

```rust
struct Manifest {
    #[serde(default)] towers: Vec<TowerEntry>,
    #[serde(default)] heroes: Vec<HeroEntry>,
    #[serde(default)] abilities: Vec<Entry>,
    #[serde(default)] buffs: Vec<Entry>,
    #[serde(default)] summons: Vec<SummonEntry>,
    #[serde(default)] creeps: Vec<CreepEntry>,
    #[serde(default)] projectile_kinds: Vec<ProjKind>,
}
```

**Step 5: 編譯確認（templates.json 還沒填數值，會用 default 0/空字串，但結構應該過 build）**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean，跑老的 emit_namespace（給 creeps/summons）沒拿新欄位的時候還是會 emit ID const。

### Task B3: build.rs 加 emit_hero_stats / emit_creep_stats / emit_summon_stats

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs`

**Step 1: 加 enum 字串對應 helper**

```rust
fn primary_attribute_to_u8(s: &str) -> u8 {
    match s {
        "" | "strength" => 0,
        "agility" => 1,
        "intelligence" => 2,
        other => panic!("unknown primary_attribute '{}'", other),
    }
}

fn enemy_type_to_u8(s: &str) -> u8 {
    match s {
        "" | "caster" => 0,
        "melee" => 1,
        "ranged" => 2,
        "boss" => 3,
        other => panic!("unknown enemy_type '{}'", other),
    }
}

fn ai_type_to_u8(s: &str) -> u8 {
    match s {
        "" | "defensive" => 0,
        "aggressive" => 1,
        "patrol" => 2,
        "guard" => 3,
        "passive" => 4,
        "berserker" => 5,
        other => panic!("unknown ai_type '{}'", other),
    }
}
```

**Step 2: 加 emit_hero_stats（在 emit_hero_abilities 後）**

```rust
fn emit_hero_stats(out: &mut String, entries: &[HeroEntry]) {
    for e in entries {
        if e.tombstone { continue; }
        let cname = const_name("hero", &e.id);
        out.push_str(&format!(
            "pub const {}_STATS: HeroStats = HeroStats {{\n\
             \tstrength: {}i32,\n\
             \tagility: {}i32,\n\
             \tintelligence: {}i32,\n\
             \tprimary_attribute: {}u8,\n\
             \tattack_range: {:?}_f32,\n\
             \tbase_damage: {}i32,\n\
             \tbase_armor: {:?}_f32,\n\
             \tbase_hp: {}i32,\n\
             \tbase_mana: {}i32,\n\
             \tmove_speed: {:?}_f32,\n\
             \tturn_speed: {:?}_f32,\n\
             \tlevel_growth: LevelGrowth {{\n\
             \t\tstrength_per_level: {:?}_f32,\n\
             \t\tagility_per_level: {:?}_f32,\n\
             \t\tintelligence_per_level: {:?}_f32,\n\
             \t\tdamage_per_level: {:?}_f32,\n\
             \t\thp_per_level: {:?}_f32,\n\
             \t\tmana_per_level: {:?}_f32,\n\
             \t}},\n\
             }};\n",
            cname,
            e.strength, e.agility, e.intelligence,
            primary_attribute_to_u8(&e.primary_attribute),
            e.attack_range, e.base_damage, e.base_armor,
            e.base_hp, e.base_mana, e.move_speed, e.turn_speed,
            e.level_growth.strength_per_level,
            e.level_growth.agility_per_level,
            e.level_growth.intelligence_per_level,
            e.level_growth.damage_per_level,
            e.level_growth.hp_per_level,
            e.level_growth.mana_per_level,
        ));
    }
    out.push('\n');

    out.push_str("pub fn hero_stats(id: HeroId) -> Option<&'static HeroStats> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("hero", &e.id);
            out.push_str(&format!("\t\t{} => Some(&{}_STATS),\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}
```

**Step 3: 加 emit_creep_stats**（同 pattern，pattern 跟 emit_tower_stats 內部那段一樣，把 12 個欄位換成 CreepStats 的 10 個欄位）

```rust
fn emit_creep_stats(out: &mut String, entries: &[CreepEntry]) {
    for e in entries {
        if e.tombstone { continue; }
        let cname = const_name("creep", &e.id);
        out.push_str(&format!(
            "pub const {}_STATS: CreepStats = CreepStats {{\n\
             \thp: {:?}_f32,\n\
             \tarmor: {:?}_f32,\n\
             \tmagic_resistance: {:?}_f32,\n\
             \tdamage: {:?}_f32,\n\
             \tattack_range: {:?}_f32,\n\
             \tmove_speed: {:?}_f32,\n\
             \tenemy_type: {}u8,\n\
             \tai_type: {}u8,\n\
             \texp_reward: {}i32,\n\
             \tgold_reward: {}i32,\n\
             }};\n",
            cname,
            e.hp, e.armor, e.magic_resistance,
            e.damage, e.attack_range, e.move_speed,
            enemy_type_to_u8(&e.enemy_type), ai_type_to_u8(&e.ai_type),
            e.exp_reward, e.gold_reward,
        ));
    }
    out.push('\n');
    out.push_str("pub fn creep_stats(id: CreepId) -> Option<&'static CreepStats> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("creep", &e.id);
            out.push_str(&format!("\t\t{} => Some(&{}_STATS),\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}
```

**Step 4: 加 emit_summon_stats**（同 pattern，4 個欄位）

```rust
fn emit_summon_stats(out: &mut String, entries: &[SummonEntry]) {
    for e in entries {
        if e.tombstone { continue; }
        let cname = const_name("summon", &e.id);
        out.push_str(&format!(
            "pub const {}_STATS: SummonStats = SummonStats {{\n\
             \thp: {:?}_f32,\n\
             \tdamage: {:?}_f32,\n\
             \tduration: {:?}_f32,\n\
             \tmove_speed: {:?}_f32,\n\
             }};\n",
            cname, e.hp, e.damage, e.duration, e.move_speed,
        ));
    }
    out.push('\n');
    out.push_str("pub fn summon_stats(id: SummonId) -> Option<&'static SummonStats> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("summon", &e.id);
            out.push_str(&format!("\t\t{} => Some(&{}_STATS),\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}
```

**Step 5: 在 main() 呼叫 3 個新 emit**

```rust
emit_hero_stats(&mut out, &m.heroes);
emit_creep_stats(&mut out, &m.creeps);
emit_summon_stats(&mut out, &m.summons);
```
（在現有 emit_namespace creep/summon 後）

**Step 6: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean。templates.json 還沒填值，所以 stats 全 0。

### Task B4: 把 hero / creep / summon 完整 stats 填到 templates.json

**Files:**
- Modify: `D:/omoba/omb/Story/templates.json`

**Step 1: 抓 TD_1/entity.json hero 數值貼到 templates.json**

讀 `D:/omoba/omb/Story/TD_1/entity.json` heroes[0]，把 strength / agility / intelligence / primary_attribute / attack_range / base_damage / base_armor / base_hp / base_mana / move_speed / turn_speed / level_growth 跟 background 全部複製到 `D:/omoba/omb/Story/templates.json` heroes[0]：

```json
{
  "id": "saika_magoichi",
  "display_name": "雜賀孫市",
  "title": "千里狙擊手",
  "background": "雜賀眾的領袖，以精準的遠程射擊聞名於戰國時代",
  "abilities": ["sniper_mode", "saika_reinforcements", "rain_iron_cannon", "three_stage_technique"],
  "strength": 18,
  "agility": 28,
  "intelligence": 16,
  "primary_attribute": "agility",
  "attack_range": 900.0,
  "base_damage": 52,
  "base_armor": 1.5,
  "base_hp": 580,
  "base_mana": 300,
  "move_speed": 320.0,
  "turn_speed": 180.0,
  "level_growth": {
    "strength_per_level": 1.8,
    "agility_per_level": 3.2,
    "intelligence_per_level": 1.6,
    "damage_per_level": 2.8,
    "hp_per_level": 58.0,
    "mana_per_level": 26.0
  }
}
```

對 date_masamune 同樣處理（從 B02_1 或對應 entity.json 取值）。

**Step 2: creeps[] 抓 stats 填**

讀 entity.json 的 creeps[]、enemies[]，把 hp / armor / magic_resistance / damage / attack_range / move_speed / enemy_type / ai_type / exp_reward / gold_reward 全填進 templates.json creeps[] 對應 id。

**Step 3: summons[] 抓 stats 填**

同樣處理 saika_gunner。

**Step 4: JSON 驗證**

```bash
py -3 -c "import json; m = json.load(open('D:/omoba/omb/Story/templates.json')); print(m['heroes'][0])"
```
Expected: 印出完整 stat 欄位（不是空值）。

**Step 5: 編譯 + 驗 codegen**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
grep -A20 "HERO_SAIKA_MAGOICHI_STATS" /d/omoba/omoba-template-ids/target/debug/build/omoba-template-ids-*/out/template_ids_gen.rs | head -25
```
Expected: 看到 strength=18, agility=28 等實際值。

### Task B5: 加 stats lookup unit test

**Files:**
- Modify: `D:/omoba/omoba-template-ids/tests/hero_abilities.rs` (rename → `stats_lookup.rs`)

**Step 1: 補測試**

加 hero/creep/summon stats lookup test：

```rust
#[test]
fn saika_magoichi_stats_match_json() {
    let s = hero_stats(HERO_SAIKA_MAGOICHI).unwrap();
    assert_eq!(s.strength, 18);
    assert_eq!(s.agility, 28);
    assert_eq!(s.base_hp, 580);
    assert_eq!(s.attack_range, 900.0);
    assert_eq!(s.primary_attribute, 1); // agility
}

#[test]
fn saika_gunner_summon_stats_match_json() {
    let s = summon_stats(SUMMON_SAIKA_GUNNER).unwrap();
    assert_eq!(s.hp, 400.0);
    assert_eq!(s.damage, 45.0);
    assert_eq!(s.duration, 60.0);
}
```

**Step 2: 跑測試**

```bash
cargo test --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: 全綠。

### Task B6: 改 omb host Hero::from_campaign_data 讀 hero_stats const

**Files:**
- Modify: `D:/omoba/omb/src/comp/hero.rs:83-121`

**Step 1: 改 from_campaign_data**

讓它先查 `hero_stats(hero_by_name(&hero_data.id))`，從 const 取數值。`HeroJD` 仍可保留 abilities 欄位（讓 entity.json 仍可指定，不過不寫的時候 fallback 到 `hero_abilities()` lookup）。

```rust
pub fn from_campaign_data(hero_data: &HeroJD) -> Option<Self> {
    use omoba_template_ids::{hero_abilities, hero_by_name, hero_display, hero_stats, hero_title};
    let id = hero_by_name(&hero_data.id)?;
    let s = hero_stats(id)?;
    let abilities = if hero_data.abilities.is_empty() {
        hero_abilities(id).iter().map(|a| a.as_str().to_string()).collect()
    } else {
        hero_data.abilities.clone()
    };
    Some(Hero {
        id: hero_data.id.clone(),
        name: hero_display(id).to_string(),
        title: hero_title(id).to_string(),
        strength: s.strength,
        agility: s.agility,
        intelligence: s.intelligence,
        // ... 把所有 stat 從 s.* 抓
        abilities,
        // ...
    })
}
```
（具體欄位對應請看 hero.rs 現有 struct）

**Step 2: 編譯確認**

```bash
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

### Task B7: 改 omb host Enemy::from_campaign_data 讀 creep_stats const

**Files:**
- Modify: `D:/omoba/omb/src/comp/enemy.rs:97-144`

**Step 1: 改 from_campaign_data**（同 hero pattern）

從 `creep_stats(creep_by_name(&data.id))` 拿 stat。

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

### Task B8: 改 omb host summon spawn 讀 summon_stats const

**Files:**
- 找：`grep -rn "spawn_summoned_unit\|SaikaGunner\|saika_gunner" D:/omoba/omb/src/`
- Modify 找到的位置

**Step 1: 把 hardcode summon stat 改 lookup**

從 `summon_stats(summon_by_name(...))` 拿 hp/damage/duration/move_speed。

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

### Task B9: entity.json 各類別 slim 成 [{id}]

**Files:**
- Modify: `D:/omoba/omb/Story/TD_1/entity.json`
- Modify: `D:/omoba/omb/Story/MVP_1/entity.json` (若存在)
- Modify: `D:/omoba/omb/Story/B01_1/entity.json` (若存在)

**Step 1: 把每個 hero/creep/enemy/summon 條目縮成 `{"id": "..."}`**

例如 TD_1/entity.json 變成：

```json
{
  "heroes": [{"id": "saika_magoichi"}],
  "enemies": [
    {"id": "training_mage"},
    {"id": "fire_mage"}
  ],
  "creeps": [
    {"id": "ranged_minion"},
    {"id": "siege_minion"}
  ],
  "summons": [{"id": "saika_gunner"}]
}
```

**Step 2: 對 import_campaign.rs HeroJD/EnemyJD/CreepJD/SummonJD 瘦身**

找 `D:/omoba/omb/src/ue4/import_campaign.rs`，把 4 個 JD struct 拿掉所有 stat 欄位，只剩 `id: String`（其他欄位 `#[serde(default)]` 接受未來 campaign override）。

**Step 3: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

### Task B10: Verify Phase B + smoke test

**Step 1: 全 build**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

**Step 2: Run tests**

```bash
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: 全綠。

**Step 3: Smoke test — run.bat (TD_1)**

跑 run.bat 確認 hero stats 顯示正確（HP/攻擊/護甲），4 技能 W/E/R/T 全可用。

**Step 4: Stress test — run_stress.bat**

確認 1000 creep 能跑（穩定 60+ fps）。

### Task B11: Commit Phase B

```bash
cd /d/omoba/omoba-template-ids && git add -A && git commit -m "templates: HeroStats / CreepStats / SummonStats const + lookup fns"
cd /d/omoba/omb && git add -A && git commit -m "templates: hero/creep/summon stats 從 templates.json const lookup 取，entity.json slim 成 [{id}]"
cd /d/omoba/omoba-core && git add -A && git commit -m "templates: bump for Phase B" 2>/dev/null
cd /d/omoba && git add omb omoba-core omoba-template-ids 2>/dev/null
git commit -m "chore: bump submodules for Phase B — hero/creep/summon stats unification"
```

---

## Phase E — Fixture / default 字串清掃

> Phase E 在 Phase B 之後做（先把架構清出來，剩下散落字串一次清）

### Task E1: scripts/base_content/src/lib.rs 字串 → typed const

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/lib.rs:33,37,41,45,52`

**Step 1: 加 import**

```rust
use omb_script_abi::prelude::{TOWER_DART, TOWER_BOMB, TOWER_TACK, TOWER_ICE, SUMMON_SAIKA_GUNNER};
```

**Step 2: 把 5 個 `unit_id: "..."` literal 改成 `TOWER_*.as_str().into()` / `SUMMON_*.as_str().into()`**

**Step 3: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```
Expected: clean。

### Task E2: saika_reinforcements.rs 字串 → typed const

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/heroes/B01_saika_magoichi/No2_saika_reinforcements.rs:88,153`

**Step 1: 加 import**

```rust
use omb_script_abi::prelude::SUMMON_SAIKA_GUNNER;
```

**Step 2: 替換兩處 `"saika_gunner"`** → `SUMMON_SAIKA_GUNNER.as_str()` / `.into()`。

**Step 3: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```

### Task E3: omoba-core/src/config.rs default

**Files:**
- Modify: `D:/omoba/omoba-core/src/config.rs:59`

**Step 1: 加 import + 用 const**

```rust
use omoba_template_ids::HERO_SAIKA_MAGOICHI;
// ...
hero_type: HERO_SAIKA_MAGOICHI.as_str().to_string(),
```

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```

### Task E4: omoba-core/src/tower_meta.rs test data

**Files:**
- Modify: `D:/omoba/omoba-core/src/tower_meta.rs:61`

**Step 1: 改測試**

```rust
use omoba_template_ids::TOWER_DART;
// ...
tower_kind: TOWER_DART.as_str().into(),
```

**Step 2: 編譯確認**

```bash
cargo test --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```

### Task E5: omb/src/comp/mqtt_handler.rs mock JSON

**Files:**
- Modify: `D:/omoba/omb/src/comp/mqtt_handler.rs:71-126`

**Step 1: 加 import + 動態構造 abilities**

```rust
use omoba_template_ids::{HERO_SAIKA_MAGOICHI, hero_abilities};
// ...
"hero_type": HERO_SAIKA_MAGOICHI.as_str(),
"abilities": hero_abilities(HERO_SAIKA_MAGOICHI)
    .iter()
    .map(|aid| serde_json::json!({
        "ability_id": aid.as_str(),
        "cooldown_remaining": 0.0,
        "is_available": true
    }))
    .collect::<Vec<_>>(),
```

**Step 2: 編譯確認**

```bash
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```

### Task E6: tower_upgrade_registry.rs 字串 → typed const

**Files:**
- Modify: `D:/omoba/omb/src/comp/tower_upgrade_registry.rs`

**Step 1: 加 top-level import**

```rust
use omoba_template_ids::{TOWER_DART, TOWER_BOMB, TOWER_TACK, TOWER_ICE,
                         TOWER_DART_STATS, TOWER_BOMB_STATS, TOWER_TACK_STATS, TOWER_ICE_STATS};
```

**Step 2: 改 `let kind = "tower_dart"` 4 處** → `let kind = TOWER_DART.as_str()` 等。

**Step 3: 改 test 內的字串 literal**（line 482-491） → `TOWER_DART.as_str()` 等。

**Step 4: 改 test costs_match_formula line 505** → 從 `TOWER_DART_STATS.cost` 取 base cost：

```rust
let bases = [
    (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
    (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
    (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
    (TOWER_ICE.as_str(),  TOWER_ICE_STATS.cost),
];
```

**Step 5: 編譯 + 跑測試**

```bash
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab tower_upgrade_registry
```
Expected: 全綠。

### Task E7: Verify Phase E + grep 殘留字串

**Step 1: 全 workspace build**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```
Expected: clean。

**Step 2: grep 殘留**

```bash
grep -rn '"tower_dart"\|"tower_bomb"\|"tower_tack"\|"tower_ice"' /d/omoba/omb/src /d/omoba/scripts/base_content/src /d/omoba/omoba-core/src /d/omoba/omfx/game/src 2>/dev/null | grep -v "templates.json\|build.rs\|tower_upgrade_registry.*//"
```
Expected: 空 output（所有合理用途已換成 typed const）。

```bash
grep -rn '"saika_magoichi"\|"date_masamune"\|"sniper_mode"\|"saika_reinforcements"\|"rain_iron_cannon"\|"three_stage_technique"\|"saika_gunner"' /d/omoba/omb/src /d/omoba/scripts/base_content/src /d/omoba/omoba-core/src 2>/dev/null
```
Expected: 空 output。

### Task E8: Commit Phase E

```bash
cd /d/omoba/scripts && git add -A && git commit -m "templates: base_content tower/summon unit_id 走 TOWER_*/SUMMON_* typed const"
cd /d/omoba/omb && git add -A && git commit -m "templates: tower_upgrade_registry / mqtt_handler / 各處字串 literal 走 typed const"
cd /d/omoba/omoba-core && git add -A && git commit -m "templates: config default + tower_meta test 走 typed const"
cd /d/omoba && git add omb omoba-core 2>/dev/null
git commit -m "chore: bump submodules for Phase E — string literal 清掃"
```

---

## Phase C — Ability 數值 codegen

> Phase C 比較複雜（const + runtime build helper），下面 task 較長。

### Task C1: 在 omoba-template-ids/src/lib.rs 加 AbilityConst struct

**Files:**
- Modify: `D:/omoba/omoba-template-ids/src/lib.rs`

**Step 1: 加 enum mirror + const struct**

```rust
#[repr(u8)] #[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AbilityTypeC { Active = 0, Toggle = 1, Ultimate = 2, Passive = 3 }

#[repr(u8)] #[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CastTypeC { Instant = 0, Channeled = 1 }

#[repr(u8)] #[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TargetTypeC { None = 0, Point = 1, Unit = 2 }

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct AbilityLevelDataConst {
    pub cooldown: f32,
    pub mana_cost: f32,
    pub cast_time: f32,
    pub range: f32,
}

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct AbilityConst {
    pub ability_type: AbilityTypeC,
    pub cast_type: CastTypeC,
    pub target_type: TargetTypeC,
    pub max_level: u8,
    pub levels: &'static [AbilityLevelDataConst],
    /// per-level extras（HashMap key + per-level f32 array）
    pub extras: &'static [(&'static str, &'static [f32])],
}
```

**Step 2: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```

### Task C2: 改 build.rs 擴 abilities entry + emit AbilityConst

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs`

**Step 1: 把 `abilities: Vec<Entry>` 改成 `abilities: Vec<AbilityEntry>`**

```rust
#[derive(Deserialize)]
struct AbilityEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] ability_type: String, // "active" | "toggle" | "ultimate" | "passive"
    #[serde(default)] cast_type: String,    // "instant" | "channeled"
    #[serde(default)] target_type: String,  // "none" | "point" | "unit"
    #[serde(default)] max_level: u8,
    #[serde(default)] levels: Vec<AbilityLevelEntry>,
    /// extras: { "range_bonus": [100, 200, 300, 400], ... }
    #[serde(default)] extras: std::collections::BTreeMap<String, Vec<f32>>,
}

#[derive(Deserialize, Default)]
struct AbilityLevelEntry {
    #[serde(default)] cooldown: f32,
    #[serde(default)] mana_cost: f32,
    #[serde(default)] cast_time: f32,
    #[serde(default)] range: f32,
}
```

**Step 2: 加 enum 字串對應 helper**

```rust
fn ability_type_to_u8(s: &str) -> u8 {
    match s { "" | "active" => 0, "toggle" => 1, "ultimate" => 2, "passive" => 3,
              other => panic!("unknown ability_type '{}'", other) }
}
fn cast_type_to_u8(s: &str) -> u8 {
    match s { "" | "instant" => 0, "channeled" => 1,
              other => panic!("unknown cast_type '{}'", other) }
}
fn target_type_to_u8(s: &str) -> u8 {
    match s { "" | "none" => 0, "point" => 1, "unit" => 2,
              other => panic!("unknown target_type '{}'", other) }
}
```

**Step 3: 加 emit_ability_const 函式**

```rust
fn emit_ability_const(out: &mut String, entries: &[AbilityEntry]) {
    for e in entries {
        if e.tombstone { continue; }
        let cname = const_name("ability", &e.id);

        // levels static slice
        out.push_str(&format!("static {}_LEVELS: &[AbilityLevelDataConst] = &[\n", cname));
        for lv in &e.levels {
            out.push_str(&format!(
                "\tAbilityLevelDataConst {{ cooldown: {:?}_f32, mana_cost: {:?}_f32, cast_time: {:?}_f32, range: {:?}_f32 }},\n",
                lv.cooldown, lv.mana_cost, lv.cast_time, lv.range
            ));
        }
        out.push_str("];\n");

        // 各 extras key 各自 emit static array
        for (key, vals) in &e.extras {
            let arr_name = format!("{}_EXTRA_{}", cname, key.to_uppercase());
            out.push_str(&format!("static {}: &[f32] = &[", arr_name));
            for v in vals {
                out.push_str(&format!("{:?}_f32, ", v));
            }
            out.push_str("];\n");
        }

        // extras tuple slice
        out.push_str(&format!("static {}_EXTRAS: &[(&'static str, &'static [f32])] = &[\n", cname));
        for (key, _) in &e.extras {
            let arr_name = format!("{}_EXTRA_{}", cname, key.to_uppercase());
            out.push_str(&format!("\t(\"{}\", {}),\n", escape_str_literal(key), arr_name));
        }
        out.push_str("];\n");

        // const AbilityConst
        out.push_str(&format!(
            "pub const {}_CONST: AbilityConst = AbilityConst {{\n\
             \tability_type: AbilityTypeC::{},\n\
             \tcast_type: CastTypeC::{},\n\
             \ttarget_type: TargetTypeC::{},\n\
             \tmax_level: {}u8,\n\
             \tlevels: {}_LEVELS,\n\
             \textras: {}_EXTRAS,\n\
             }};\n",
            cname,
            match ability_type_to_u8(&e.ability_type) { 0=>"Active", 1=>"Toggle", 2=>"Ultimate", 3=>"Passive", _=>unreachable!() },
            match cast_type_to_u8(&e.cast_type) { 0=>"Instant", 1=>"Channeled", _=>unreachable!() },
            match target_type_to_u8(&e.target_type) { 0=>"None", 1=>"Point", 2=>"Unit", _=>unreachable!() },
            e.max_level,
            cname, cname,
        ));
    }
    out.push('\n');

    // lookup ability_const
    out.push_str("pub fn ability_const(id: AbilityId) -> Option<&'static AbilityConst> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("ability", &e.id);
            out.push_str(&format!("\t\t{} => Some(&{}_CONST),\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}
```

**Step 4: main() 呼叫**

```rust
emit_ability_const(&mut out, &m.abilities);
```

但注意：原本 abilities 是用 `emit_namespace(out, "Ability", "ability", &m.abilities, true)`，現在 `m.abilities` type 改成 `Vec<AbilityEntry>` 不是 `Vec<Entry>`，要 convert：

```rust
let ab_entries: Vec<Entry> = m.abilities.iter().map(|a| Entry {
    id: a.id.clone(),
    display_name: a.display_name.clone(),
    tombstone: a.tombstone,
}).collect();
emit_namespace(&mut out, "Ability", "ability", &ab_entries, true);
emit_ability_const(&mut out, &m.abilities);
```

**Step 5: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean，gen file 應該已經能 emit `ABILITY_*_CONST`（雖然 abilities 還沒填 stats，但會用 default 0）。

### Task C3: 把所有 ability stats 填到 templates.json

**Files:**
- Modify: `D:/omoba/omb/Story/templates.json`

**Step 1: 抓 base_content/src/heroes/*/No*.rs 的 hardcode 值填 abilities[]**

對每個 ability，從現有 script `*_def()` 函式抓值填到 templates.json 對應 abilities[] entry：

範例 sniper_mode：

```json
{
  "id": "sniper_mode",
  "display_name": "狙擊模式",
  "ability_type": "toggle",
  "cast_type": "instant",
  "target_type": "none",
  "max_level": 4,
  "levels": [
    {"cooldown": 0.0, "mana_cost": 0.0, "cast_time": 0.0, "range": 0.0},
    {"cooldown": 0.0, "mana_cost": 0.0, "cast_time": 0.0, "range": 0.0},
    {"cooldown": 0.0, "mana_cost": 0.0, "cast_time": 0.0, "range": 0.0},
    {"cooldown": 0.0, "mana_cost": 0.0, "cast_time": 0.0, "range": 0.0}
  ],
  "extras": {
    "range_bonus": [100.0, 200.0, 300.0, 400.0],
    "damage_bonus_pct": [0.15, 0.25, 0.35, 0.45],
    "attack_speed_penalty": [-0.30, -0.30, -0.30, -0.30],
    "move_speed_penalty": [-0.50, -0.50, -0.50, -0.50],
    "accuracy_bonus": [0.10, 0.10, 0.10, 0.10]
  }
}
```

`rain_iron_cannon` 的 `TRUE_DMG_PCT` / `AOE_RADIUS` / `ARC_HALF_ANGLE_RAD` 等改放 extras：

```json
{
  "id": "rain_iron_cannon",
  ...
  "extras": {
    "true_damage_pct": [0.15, 0.25, 0.35, 0.45],
    "aoe_radius": [150.0, 150.0, 150.0, 150.0],
    "arc_degrees": [90.0, 90.0, 90.0, 90.0]
  }
}
```

對其他 6 個 abilities 同樣處理。

**Step 2: JSON 驗證 + codegen 確認**

```bash
py -3 -c "import json; m = json.load(open('D:/omoba/omb/Story/templates.json')); print(m['abilities'][0])"
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
grep -B1 -A12 "ABILITY_SNIPER_MODE_CONST" /d/omoba/omoba-template-ids/target/debug/build/omoba-template-ids-*/out/template_ids_gen.rs | head -20
```
Expected: 看到實際數值。

### Task C4: 在 script-abi 加 build_ability_def_from_const helper

**Files:**
- Modify: `D:/omoba/scripts/script-abi/src/lib.rs`

**Step 1: 加 helper 函式**

```rust
pub fn build_ability_def_from_const(
    ability_id: omoba_template_ids::AbilityId,
    c: &'static omoba_template_ids::AbilityConst,
) -> omoba_core::ability_meta::AbilityDef {
    use omoba_core::ability_meta::*;
    use std::collections::HashMap;

    let mut levels = HashMap::new();
    for (lvl_idx, ld) in c.levels.iter().enumerate() {
        let mut extra = HashMap::new();
        for (key, per_lvl) in c.extras {
            if let Some(v) = per_lvl.get(lvl_idx) {
                extra.insert(key.to_string(), serde_json::json!(*v));
            }
        }
        levels.insert((lvl_idx + 1).to_string(), AbilityLevelData {
            cooldown: ld.cooldown,
            mana_cost: ld.mana_cost,
            cast_time: ld.cast_time,
            range: ld.range,
            extra,
        });
    }

    AbilityDef {
        id: ability_id.as_str().to_string(),
        name: omoba_template_ids::ability_display(ability_id).to_string(),
        description: String::new(),
        ability_type: match c.ability_type {
            omoba_template_ids::AbilityTypeC::Active => AbilityType::Active,
            omoba_template_ids::AbilityTypeC::Toggle => AbilityType::Toggle,
            omoba_template_ids::AbilityTypeC::Ultimate => AbilityType::Ultimate,
            omoba_template_ids::AbilityTypeC::Passive => AbilityType::Passive,
        },
        target_type: match c.target_type {
            omoba_template_ids::TargetTypeC::None => TargetType::None,
            omoba_template_ids::TargetTypeC::Point => TargetType::Point,
            omoba_template_ids::TargetTypeC::Unit => TargetType::Unit,
        },
        cast_type: match c.cast_type {
            omoba_template_ids::CastTypeC::Instant => CastType::Instant,
            omoba_template_ids::CastTypeC::Channeled => CastType::Channeled,
        },
        icon: None,
        max_level: c.max_level,
        levels,
        effects_preview: vec![],
        conditions: vec![],
        properties: HashMap::new(),
    }
}
```

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```
Expected: clean。

### Task C5: 改 No1_sniper_mode.rs 使用 helper

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/heroes/B01_saika_magoichi/No1_sniper_mode.rs`

**Step 1: 把 sniper_mode_ffi() / sniper_mode_def() 縮成 helper 呼叫**

```rust
pub fn sniper_mode_ffi() -> AbilityDefFFI {
    use omb_script_abi::prelude::{ABILITY_SNIPER_MODE, ABILITY_SNIPER_MODE_CONST};
    AbilityDefFFI {
        ability_def: omb_script_abi::build_ability_def_from_const(
            ABILITY_SNIPER_MODE, &ABILITY_SNIPER_MODE_CONST
        ),
        handler: AbilityScript_TO::from_value(SniperModeHandler, TD_Opaque),
    }
}
```

刪除舊的 sniper_mode_def() 函式。execute() 邏輯不動。

**Step 2: 編譯確認**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```
Expected: clean。

### Task C6: 改其餘 7 個 ability scripts 同樣處理

**Files:**
- Modify: `D:/omoba/scripts/base_content/src/heroes/B01_saika_magoichi/No2_saika_reinforcements.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B01_saika_magoichi/No3_rain_iron_cannon.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B01_saika_magoichi/No4_three_stage_technique.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B02_date_masamune/No1_flame_blade.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B02_date_masamune/No2_fire_dash.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B02_date_masamune/No3_flame_assault.rs`
- Modify: `D:/omoba/scripts/base_content/src/heroes/B02_date_masamune/No4_matchlock_gun.rs`

**Step 1: 每個都換成 helper 呼叫**（同 C5 pattern）

對應 `ABILITY_*_CONST` const 名稱即可。execute() 內部的 hardcode（如 `AOE_RADIUS = 150.0`）改從 `ABILITY_*_CONST.extras` 取：

```rust
fn get_extra(c: &AbilityConst, key: &str, lvl: u8) -> f32 {
    c.extras.iter()
        .find(|(k, _)| *k == key)
        .and_then(|(_, vals)| vals.get((lvl - 1) as usize).copied())
        .unwrap_or(0.0)
}
```

**Step 2: 編譯**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
```
Expected: clean。

### Task C7: Verify Phase C + smoke test

**Step 1: 全 build**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```

**Step 2: Smoke test**

```bash
cd /d/omoba && cmd /c run.bat
```

按 W → sniper_mode 應正常 toggle，buff 數值（range/damage/movespeed）跟之前 hardcode 版本相同。
按 R (rain_iron_cannon) 應正常 AoE 真實傷害。
其他技能依序測試。

### Task C8: Commit Phase C

```bash
cd /d/omoba/omoba-template-ids && git add -A && git commit -m "templates: AbilityConst const + ability_const() lookup"
cd /d/omoba/scripts && git add -A && git commit -m "templates: ability scripts *_def() 走 build_ability_def_from_const helper"
cd /d/omoba/omb && git add Story/templates.json && git commit -m "templates: abilities[] 加 cooldown/mana/cast_time/range/extras 全數值"
cd /d/omoba && git add omb omoba-template-ids scripts 2>/dev/null
git commit -m "chore: bump submodules for Phase C — ability data unification"
```

---

## Phase D — Tower upgrade tree codegen

> Phase D 是最大的 task — 522 行 hardcode 升級樹搬到 JSON

### Task D1: omoba-template-ids 加 UpgradeDefConst struct

**Files:**
- Modify: `D:/omoba/omoba-template-ids/src/lib.rs`

**Step 1: 加 enum mirror + const struct**

```rust
#[repr(u8)] #[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StatOpC { Add = 0, Mul = 1 }

#[repr(u8)] #[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UpgradeEffectKindC { StatMod = 0, BehaviorFlag = 1 }

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct UpgradeEffectConst {
    pub kind: UpgradeEffectKindC,
    pub key: &'static str,   // StatMod: stat key; BehaviorFlag: flag string
    pub value: f32,           // StatMod only
    pub op: StatOpC,          // StatMod only
}

#[repr(C)] #[derive(Copy, Clone, Debug)]
pub struct UpgradeDefConst {
    pub name: &'static str,
    pub description: &'static str,
    pub cost: i32,
    pub effects: &'static [UpgradeEffectConst],
}
```

**Step 2: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```

### Task D2: 擴 TowerEntry 加 upgrades + emit TOWER_*_UPGRADES

**Files:**
- Modify: `D:/omoba/omoba-template-ids/build.rs`

**Step 1: TowerEntry 加 upgrades**

```rust
struct TowerEntry {
    // ... 既有 12 stat 欄位
    #[serde(default)] upgrades: std::collections::BTreeMap<String, Vec<TowerUpgradeJsonEntry>>,
}

#[derive(Deserialize, Default)]
struct TowerUpgradeJsonEntry {
    name: String,
    #[serde(default)] description: String,
    cost: i32,
    #[serde(default)] effects: Vec<UpgradeEffectJsonEntry>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum UpgradeEffectJsonEntry {
    StatMod { key: String, value: f32, op: String /* "Add" | "Mul" */ },
    BehaviorFlag { flag: String },
}
```

**Step 2: 加 emit_tower_upgrades 函式**

```rust
fn emit_tower_upgrades(out: &mut String, entries: &[TowerEntry]) {
    for e in entries {
        if e.tombstone || e.upgrades.is_empty() { continue; }
        let cname = const_name("tower", &e.id);

        // 為每個 path 各 emit 一個 static slice
        for (path_str, upgrades) in &e.upgrades {
            let path: u8 = path_str.parse().expect("upgrades path key must be u8");
            for (lvl_idx, u) in upgrades.iter().enumerate() {
                let lvl = lvl_idx + 1;
                let arr_name = format!("{}_UPG_P{}_L{}_EFFECTS", cname, path, lvl);
                out.push_str(&format!("static {}: &[UpgradeEffectConst] = &[\n", arr_name));
                for ef in &u.effects {
                    match ef {
                        UpgradeEffectJsonEntry::StatMod { key, value, op } => {
                            let op_v = match op.as_str() { "Add" => "Add", "Mul" => "Mul",
                                other => panic!("unknown StatOp '{}'", other) };
                            out.push_str(&format!(
                                "\tUpgradeEffectConst {{ kind: UpgradeEffectKindC::StatMod, key: \"{}\", value: {:?}_f32, op: StatOpC::{} }},\n",
                                escape_str_literal(key), value, op_v));
                        }
                        UpgradeEffectJsonEntry::BehaviorFlag { flag } => {
                            out.push_str(&format!(
                                "\tUpgradeEffectConst {{ kind: UpgradeEffectKindC::BehaviorFlag, key: \"{}\", value: 0.0_f32, op: StatOpC::Add }},\n",
                                escape_str_literal(flag)));
                        }
                    }
                }
                out.push_str("];\n");
            }
        }

        // 各 path 的 UpgradeDefConst slice
        let mut paths_sorted: Vec<&String> = e.upgrades.keys().collect();
        paths_sorted.sort();
        for path_str in &paths_sorted {
            let path: u8 = path_str.parse().unwrap();
            let upgrades = &e.upgrades[*path_str];
            let path_arr = format!("{}_UPG_P{}", cname, path);
            out.push_str(&format!("static {}: &[UpgradeDefConst] = &[\n", path_arr));
            for (lvl_idx, u) in upgrades.iter().enumerate() {
                let lvl = lvl_idx + 1;
                out.push_str(&format!(
                    "\tUpgradeDefConst {{ name: \"{}\", description: \"{}\", cost: {}i32, effects: {}_UPG_P{}_L{}_EFFECTS }},\n",
                    escape_str_literal(&u.name),
                    escape_str_literal(&u.description),
                    u.cost,
                    cname, path, lvl
                ));
            }
            out.push_str("];\n");
        }

        // 主 const: TOWER_<NAME>_UPGRADES
        out.push_str(&format!("pub const {}_UPGRADES: &[&[UpgradeDefConst]] = &[", cname));
        for path_str in &paths_sorted {
            let path: u8 = path_str.parse().unwrap();
            out.push_str(&format!("{}_UPG_P{}, ", cname, path));
        }
        out.push_str("];\n");
    }
    out.push('\n');

    // lookup
    out.push_str("pub fn tower_upgrades(id: TowerId) -> &'static [&'static [UpgradeDefConst]] {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("tower", &e.id);
            out.push_str(&format!("\t\t{} => {}_UPGRADES,\n", next, cname));
        }
        next += 1;
    }
    out.push_str("\t\t_ => &[],\n\t}\n}\n\n");
}
```

**Step 3: main() 呼叫 emit_tower_upgrades(out, &m.towers)**

放在 emit_tower_namespace 之後。

**Step 4: 編譯確認**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
```
Expected: clean（templates.json 還沒填 upgrades，tower_upgrades 都回 `&[]`）。

### Task D3: 把 tower_upgrade_registry.rs 48 個 UpgradeDef 抓到 templates.json

**Files:**
- Modify: `D:/omoba/omb/Story/templates.json` towers[]

**Step 1: 對每個 tower 的 register 函式逐 path 逐 level 抓**

範例 dart Path 0：

```json
{
  "id": "tower_dart",
  ... 既有 12 stats ...
  "upgrades": {
    "0": [
      {
        "name": "Long Range Darts",
        "description": "射程 350→400",
        "cost": 50,
        "effects": [{"type": "stat_mod", "key": "AttackRangeBonus", "value": 50.0, "op": "Add"}]
      },
      {
        "name": "Enhanced Eyesight",
        "description": "射程 →450, damage 10→15",
        "cost": 100,
        "effects": [
          {"type": "stat_mod", "key": "AttackRangeBonus", "value": 50.0, "op": "Add"},
          {"type": "stat_mod", "key": "BaseDamageOutgoingPercentage", "value": 0.5, "op": "Add"}
        ]
      },
      {
        "name": "Razor Sharp Shots",
        "description": "穿透 +1, damage →20",
        "cost": 200,
        "effects": [
          {"type": "behavior_flag", "flag": "sharp_pierce"},
          {"type": "stat_mod", "key": "BaseDamageOutgoingPercentage", "value": 0.5, "op": "Add"}
        ]
      },
      {
        "name": "Spike-o-pult",
        "description": "改投巨釘: splash 100, damage 40, 彈速減半",
        "cost": 500,
        "effects": [{"type": "behavior_flag", "flag": "spike_o_pult"}]
      }
    ],
    "1": [...],
    "2": [...]
  }
}
```

對 4 個塔 × 3 paths × 4 levels = 48 個 entry 全填。

**Step 2: 驗證**

```bash
py -3 -c "import json; m = json.load(open('D:/omoba/omb/Story/templates.json')); print(len(m['towers'][0]['upgrades']['0']))"
```
Expected: 4（每個 path 4 levels）。

**Step 3: 編譯確認 codegen**

```bash
cargo build --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
grep "TOWER_DART_UPGRADES" /d/omoba/omoba-template-ids/target/debug/build/omoba-template-ids-*/out/template_ids_gen.rs | head -2
```
Expected: 看到 `TOWER_DART_UPGRADES` const。

### Task D4: 改 omb tower_upgrade_registry.rs 走 const lookup

**Files:**
- Modify: `D:/omoba/omb/src/comp/tower_upgrade_registry.rs`

**Step 1: 把 register_dart / register_bomb / register_tack / register_ice 4 個函式刪掉，改成 loop**

```rust
use omoba_template_ids::{
    TOWER_DART, TOWER_BOMB, TOWER_TACK, TOWER_ICE,
    UpgradeDefConst, UpgradeEffectConst, UpgradeEffectKindC, StatOpC,
    tower_upgrades,
};

pub struct TowerUpgradeRegistry {
    defs: HashMap<(String, u8, u8), TowerUpgradeDef>,
}

impl TowerUpgradeRegistry {
    pub fn new() -> Self {
        let mut reg = Self { defs: HashMap::new() };
        for &tid in &[TOWER_DART, TOWER_BOMB, TOWER_TACK, TOWER_ICE] {
            let kind_str = tid.as_str().to_string();
            for (path_idx, path_arr) in tower_upgrades(tid).iter().enumerate() {
                for (lvl_idx, c) in path_arr.iter().enumerate() {
                    let def = TowerUpgradeDef {
                        tower_kind: kind_str.clone(),
                        path: path_idx as u8,
                        level: (lvl_idx + 1) as u8,
                        name: c.name.to_string(),
                        description: c.description.to_string(),
                        cost: c.cost,
                        effects: c.effects.iter().map(upgrade_effect_const_to_runtime).collect(),
                    };
                    reg.defs.insert((kind_str.clone(), path_idx as u8, (lvl_idx + 1) as u8), def);
                }
            }
        }
        reg
    }

    pub fn get(&self, kind: &str, path: u8, level: u8) -> Option<&TowerUpgradeDef> {
        self.defs.get(&(kind.to_string(), path, level))
    }
}

fn upgrade_effect_const_to_runtime(c: &UpgradeEffectConst) -> UpgradeEffect {
    match c.kind {
        UpgradeEffectKindC::StatMod => UpgradeEffect::StatMod {
            key: c.key.to_string(),
            value: c.value,
            op: match c.op { StatOpC::Add => StatOp::Add, StatOpC::Mul => StatOp::Mul },
        },
        UpgradeEffectKindC::BehaviorFlag => UpgradeEffect::BehaviorFlag {
            flag: c.key.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_template_ids::{TOWER_DART_STATS, TOWER_BOMB_STATS, TOWER_TACK_STATS, TOWER_ICE_STATS};

    #[test]
    fn all_four_towers_have_12_upgrades_each() {
        let reg = TowerUpgradeRegistry::new();
        for &tid in &[TOWER_DART, TOWER_BOMB, TOWER_TACK, TOWER_ICE] {
            for path in 0..3u8 {
                for level in 1..=4u8 {
                    assert!(reg.get(tid.as_str(), path, level).is_some(),
                        "{} path {} L{}", tid.as_str(), path, level);
                }
            }
        }
    }

    #[test]
    fn costs_match_formula() {
        use omoba_core::tower_meta::upgrade_cost;
        let reg = TowerUpgradeRegistry::new();
        let bases = [
            (TOWER_DART.as_str(), TOWER_DART_STATS.cost),
            (TOWER_BOMB.as_str(), TOWER_BOMB_STATS.cost),
            (TOWER_TACK.as_str(), TOWER_TACK_STATS.cost),
            (TOWER_ICE.as_str(),  TOWER_ICE_STATS.cost),
        ];
        for (kind, base) in bases {
            for path in 0..3u8 {
                for level in 1..=4u8 {
                    let def = reg.get(kind, path, level).unwrap();
                    assert_eq!(def.cost, upgrade_cost(base, level),
                        "{} path {} L{}", kind, path, level);
                }
            }
        }
    }
}
```

預期檔案行數從 522 → ~80。

**Step 2: 編譯**

```bash
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
```

**Step 3: Run tests**

```bash
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab tower_upgrade_registry
```
Expected: `costs_match_formula` + `all_four_towers_have_12_upgrades_each` 全綠（12 個 upgrades 全部能查到 + cost 跟 formula 對得起來）。

### Task D5: Verify Phase D + smoke test

**Step 1: 全 build + 全 test**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab --lib
```
Expected: clean + 全綠。

**Step 2: Smoke test — run.bat 點塔升級**

```bash
cd /d/omoba && cmd /c run.bat
```

放一個 dart tower → 點選 → 看升級 UI 顯示「Long Range Darts」「Enhanced Eyesight」等正確 name + description + cost。實際升級後塔 stat 變化跟之前一樣（射程 +50 等）。

### Task D6: Commit Phase D

```bash
cd /d/omoba/omoba-template-ids && git add -A && git commit -m "templates: TOWER_*_UPGRADES const + tower_upgrades() lookup"
cd /d/omoba/omb && git add -A && git commit -m "templates: tower_upgrade_registry 522 → ~80 行（loop iterate const fill）"
cd /d/omoba && git add omb omoba-template-ids 2>/dev/null
git commit -m "chore: bump submodules for Phase D — tower upgrade tree unification"
```

---

## Final Verification

### Task F1: Full repo verification

**Step 1: 全 workspace build**

```bash
cargo build --release --manifest-path /d/omoba/scripts/Cargo.toml -p base_content
cp /d/omoba/scripts/target/release/base_content.dll /d/omoba/omb/scripts/base_content.dll
cargo build --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab
cargo build --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```
Expected: 全 clean。

**Step 2: All tests**

```bash
cargo test --release --manifest-path /d/omoba/omb/Cargo.toml -p omobab --lib
cargo test --release --manifest-path /d/omoba/omoba-template-ids/Cargo.toml
cargo test --release --manifest-path /d/omoba/omoba-core/Cargo.toml --lib
```
Expected: 全綠。

**Step 3: gen-docs HTML 完整**

```bash
cd /d/omoba && cmd /c gen_docs.bat
```
打開 `omb/target/docs/index.html` 確認 4 塔 / 2 hero / 8 abilities 全部資料齊全。

**Step 4: run_stress.bat 驗證**

```bash
cd /d/omoba && cmd /c run_stress.bat
```
1000 creep 場景跑得起來，FPS ≥ 60。

**Step 5: grep 殘留**

```bash
grep -rn '"saika_magoichi"\|"date_masamune"\|"tower_dart"\|"tower_bomb"\|"tower_tack"\|"tower_ice"\|"sniper_mode"\|"saika_reinforcements"\|"rain_iron_cannon"\|"three_stage_technique"\|"flame_blade"\|"fire_dash"\|"flame_assault"\|"matchlock_gun"\|"saika_gunner"' /d/omoba/omb/src /d/omoba/omoba-core/src /d/omoba/omfx/game/src /d/omoba/scripts/base_content/src 2>/dev/null
```
Expected: 空 output。所有合理 reference 都走 typed const。

### Task F2: 寫 acceptance commit

```bash
cd /d/omoba
git log --oneline -10
git tag templates-unification-complete
```

---

## Acceptance Criteria

✅ 改 hero `base_damage` 只動 `omb/Story/templates.json` 一處  
✅ 改 tower upgrade `cost` 只動 `omb/Story/templates.json` 一處  
✅ `entity.json` 各類別 `[{"id":"..."}]`，行數縮 80%+  
✅ `tower_upgrade_registry.rs` < 100 行（從 522 縮）  
✅ Ability `*_def()` 各 < 20 行（從 100+ 縮）  
✅ grep `"saika_magoichi"|"tower_dart"|"sniper_mode"` 在 `src/` 下空 output  
✅ 全 cargo test 全綠  
✅ run.bat / run_stress.bat / gen_docs.bat 全部能跑

---

## Files Modified Summary

| 檔案 | Lines changed (估) |
|---|---|
| `omb/Story/templates.json` | +~600 (加 hero/creep/summon stats + abilities + ability data + tower upgrades) |
| `omb/Story/{TD_1,MVP_1,B01_1}/entity.json` | -~300 (entity 條目 slim) |
| `omoba-template-ids/src/lib.rs` | +~100 (struct definitions) |
| `omoba-template-ids/build.rs` | +~400 (4 個 emit fn + entry struct) |
| `omoba-template-ids/tests/*.rs` | +~80 (新測試) |
| `omb/src/comp/hero.rs` | -~30 (從 const 讀) |
| `omb/src/comp/enemy.rs` | -~50 (從 const 讀) |
| `omb/src/comp/tower_upgrade_registry.rs` | -440 (522 → ~80) |
| `omb/src/ue4/import_campaign.rs` | -~80 (JD struct 瘦身) |
| `omb/src/comp/mqtt_handler.rs` | ~20 lines (mock 從 const 構造) |
| `omoba-core/src/state/entities.rs` | -~10 (match → lookup) |
| `omoba-core/src/input/simulator.rs` | -~10 (match → lookup) |
| `omoba-core/src/config.rs` | ~2 lines |
| `omoba-core/src/tower_meta.rs` | ~3 lines (test) |
| `scripts/script-abi/src/lib.rs` | +~50 (build_ability_def_from_const helper) |
| `scripts/base_content/src/lib.rs` | ~5 lines (typed const) |
| `scripts/base_content/src/heroes/**/*.rs` | -~600 (8 ability *_def() 全縮 < 20 行) |

**Net: -~700 行 hardcode 程式碼，+~1100 行 JSON 資料 + codegen**
