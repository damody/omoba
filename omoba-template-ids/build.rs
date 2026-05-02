//! Build-time codegen for template ids.
//!
//! Reads `omb/Story/templates.json` and emits `template_ids_gen.rs` into OUT_DIR.
//! Id allocation: sequential u16 in JSON declaration order, starting from 1.
//! Id 0 is reserved as UNSPECIFIED. Tombstone entries (`"tombstone": true`)
//! consume an id but emit no const / lookup entry — append-only safety.

use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct Manifest {
    #[serde(default)] towers: Vec<TowerEntry>,
    #[serde(default)] heroes: Vec<HeroEntry>,
    #[serde(default)] abilities: Vec<AbilityEntry>,
    #[serde(default)] buffs: Vec<Entry>,
    #[serde(default)] summons: Vec<SummonEntry>,
    #[serde(default)] creeps: Vec<CreepEntry>,
    #[serde(default)] projectile_kinds: Vec<ProjKind>,
}

#[derive(Deserialize, Clone)]
struct Entry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
}

/// Tower entry — id + display_name + 全部 stat 欄位都從 templates.json 讀。
/// 對應 omoba_template_ids::TowerStats（src/lib.rs）— 增欄位時兩邊都要改。
#[derive(Deserialize, Clone)]
struct TowerEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] atk: f32,
    #[serde(default)] asd_interval: f32,
    #[serde(default)] range: f32,
    #[serde(default)] bullet_speed: f32,
    #[serde(default)] splash_radius: f32,
    #[serde(default)] hit_radius: f32,
    #[serde(default)] slow_factor: f32,
    #[serde(default)] slow_duration: f32,
    #[serde(default)] cost: i32,
    #[serde(default)] footprint: f32,
    #[serde(default)] hp: f32,
    #[serde(default)] turn_speed_deg: f32,
}

#[derive(Deserialize)]
struct HeroEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] title: String,
    #[serde(default)] background: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] abilities: Vec<String>,
    // intrinsic stats (Phase B)
    #[serde(default)] strength: i32,
    #[serde(default)] agility: i32,
    #[serde(default)] intelligence: i32,
    #[serde(default)] primary_attribute: String,
    #[serde(default)] attack_range: f32,
    #[serde(default)] base_damage: i32,
    #[serde(default)] base_armor: f32,
    #[serde(default)] base_hp: i32,
    #[serde(default)] base_mana: i32,
    #[serde(default)] move_speed: f32,
    #[serde(default)] turn_speed: f32,
    #[serde(default)] level_growth: HeroLevelGrowthEntry,
}

#[derive(Deserialize, Default, Clone)]
struct HeroLevelGrowthEntry {
    #[serde(default)] strength_per_level: f32,
    #[serde(default)] agility_per_level: f32,
    #[serde(default)] intelligence_per_level: f32,
    #[serde(default)] damage_per_level: f32,
    #[serde(default)] hp_per_level: f32,
    #[serde(default)] mana_per_level: f32,
}

/// Creep / Enemy entry — 對應 templates.json creeps[]。
#[derive(Deserialize, Clone)]
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

/// Summon entry — 對應 templates.json summons[]。
#[derive(Deserialize, Clone)]
struct SummonEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] hp: f32,
    #[serde(default)] damage: f32,
    #[serde(default)] duration: f32,
    #[serde(default)] move_speed: f32,
}

/// Ability entry — 對應 templates.json abilities[]。
/// `levels`：核心數值（cooldown/mana_cost/cast_time/range）每級一筆。
/// `extras`：per-level 浮點 extras，key→[v_lvl1, v_lvl2, ...]；長度必須 = max_level。
#[derive(Deserialize, Clone)]
struct AbilityEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] tombstone: bool,
    #[serde(default)] description: String,
    #[serde(default)] ability_type: String,   // "active" | "toggle" | "ultimate" | "passive"
    #[serde(default)] cast_type: String,      // "instant" | "channeled"
    #[serde(default)] target_type: String,    // "none" | "point" | "unit"
    #[serde(default = "default_max_level")] max_level: u8,
    #[serde(default)] levels: Vec<AbilityLevelEntry>,
    #[serde(default)] extras: std::collections::BTreeMap<String, Vec<f32>>,
}

fn default_max_level() -> u8 { 4 }

#[derive(Deserialize, Clone, Default)]
struct AbilityLevelEntry {
    #[serde(default)] cooldown: f32,
    #[serde(default)] mana_cost: f32,
    #[serde(default)] cast_time: f32,
    #[serde(default)] range: f32,
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
    println!("cargo:rerun-if-changed=build.rs");

    let raw = fs::read_to_string(&json_path)
        .unwrap_or_else(|e| panic!("read {}: {}", json_path.display(), e));
    let m: Manifest = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("parse templates.json: {}", e));

    let mut out = String::new();
    out.push_str("// AUTO-GENERATED by omoba-template-ids/build.rs — DO NOT EDIT.\n");
    out.push_str("// Source: omb/Story/templates.json\n\n");

    emit_tower_namespace(&mut out, &m.towers);
    emit_hero_namespace(&mut out, &m.heroes);
    let ability_ids_for_ns: Vec<Entry> = m.abilities.iter().map(|a| Entry {
        id: a.id.clone(), display_name: a.display_name.clone(), tombstone: a.tombstone,
    }).collect();
    emit_namespace(&mut out, "Ability",        "ability",    &ability_ids_for_ns, true);
    emit_namespace(&mut out, "Buff",           "buff",       &m.buffs,     true);
    let summon_ids: Vec<Entry> = m.summons.iter().map(|s| Entry {
        id: s.id.clone(), display_name: s.display_name.clone(), tombstone: s.tombstone,
    }).collect();
    emit_namespace(&mut out, "Summon",         "summon",     &summon_ids,  true);
    let creep_ids: Vec<Entry> = m.creeps.iter().map(|c| Entry {
        id: c.id.clone(), display_name: c.display_name.clone(), tombstone: c.tombstone,
    }).collect();
    emit_namespace(&mut out, "Creep",          "creep",      &creep_ids,   true);
    emit_projectile_kinds(&mut out, &m.projectile_kinds);

    // Hero → abilities lookup（必須在 abilities namespace emit 後做，因為 AbilityId 才存在）。
    // build.rs 自己 build 一個 ability id map，把字串 abilities 翻成 raw u16。
    let mut ability_id_map: std::collections::HashMap<&str, u16> =
        std::collections::HashMap::new();
    let mut next_aid: u16 = 1;
    for a in &m.abilities {
        if !a.tombstone {
            ability_id_map.insert(&a.id, next_aid);
        }
        next_aid += 1;
    }
    emit_hero_abilities(&mut out, &m.heroes, &ability_id_map);

    // Phase B: hero / creep / summon stats const + lookup
    emit_hero_stats(&mut out, &m.heroes);
    emit_creep_stats(&mut out, &m.creeps);
    emit_summon_stats(&mut out, &m.summons);

    // Phase C: ability const + lookup
    emit_ability_const(&mut out, &m.abilities);
    emit_ability_description(&mut out, &m.abilities);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/template_ids_gen.rs", out_dir);
    fs::write(&out_path, out).unwrap();
}

fn escape_str_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Strip a leading namespace prefix (e.g. "tower_tack" → "tack" when ns="tower").
/// Falls back to the full id when no prefix match.
fn strip_prefix<'a>(id: &'a str, ns_lower: &str) -> &'a str {
    let pfx = format!("{}_", ns_lower);
    id.strip_prefix(&pfx).unwrap_or(id)
}

/// Build the exported const identifier. For namespace "tower" and id "tower_tack",
/// returns "TOWER_TACK". If id already has the namespace prefix, it is removed.
fn const_name(ns_lower: &str, id: &str) -> String {
    let ns_upper = ns_lower.to_uppercase();
    let stripped = strip_prefix(id, ns_lower);
    // Guard: after stripping, "tack".to_uppercase() = "TACK" → ns_upper + "_" + "TACK"
    format!("{}_{}", ns_upper, stripped.to_uppercase())
}

fn emit_namespace(out: &mut String, ty: &str, ns_lower: &str, entries: &[Entry], has_display: bool) {
    // newtype with #[repr(transparent)] — wire-safe for abi_stable via raw u16
    // newtype with #[repr(transparent)] — wire-safe for abi_stable via raw u16.
    // `.as_str()` returns the original id string ("tower_tack" etc.) for use
    // where FFI still expects RStr<'_> — scripts do `RStr::from_str(TOWER_TACK.as_str())`
    // which is still compile-time-checked via const existence.
    out.push_str(&format!(
        "#[repr(transparent)]\n\
         #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]\n\
         pub struct {}Id(pub u16);\n\
         impl {}Id {{\n\
         \tpub const UNSPECIFIED: Self = Self(0);\n\
         \tpub const fn raw(self) -> u16 {{ self.0 }}\n\
         \tpub fn as_str(self) -> &'static str {{ {}_id_str(self) }}\n\
         }}\n\n",
        ty, ty, ns_lower,
    ));

    // consts — sequential 1..N skipping tombstones (but consuming id number)
    let mut seen: HashSet<&str> = HashSet::new();
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            if !seen.insert(&e.id) {
                panic!("duplicate {} id: {}", ty, e.id);
            }
            let cname = const_name(ns_lower, &e.id);
            out.push_str(&format!(
                "pub const {}: {}Id = {}Id({});\n",
                cname, ty, ty, next,
            ));
        }
        next = next.checked_add(1)
            .unwrap_or_else(|| panic!("{}Id exhausted u16 space", ty));
    }
    out.push('\n');

    // forward lookup: name → Option<Id>
    out.push_str(&format!(
        "pub fn {}_by_name(s: &str) -> Option<{}Id> {{\n\tmatch s {{\n",
        ns_lower, ty,
    ));
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            out.push_str(&format!(
                "\t\t\"{}\" => Some({}Id({})),\n",
                escape_str_literal(&e.id), ty, next,
            ));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");

    // reverse: id → original id string (for logs / backward-compat JSON)
    out.push_str(&format!(
        "pub fn {}_id_str(id: {}Id) -> &'static str {{\n\tmatch id.0 {{\n\t\t0 => \"\",\n",
        ns_lower, ty,
    ));
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            out.push_str(&format!(
                "\t\t{} => \"{}\",\n",
                next, escape_str_literal(&e.id),
            ));
        }
        next += 1;
    }
    out.push_str(
        "\t\t_ => {\n\
         \t\t\tdebug_assert!(false, \"unknown id: {}\", id.0);\n\
         \t\t\t\"?\"\n\
         \t\t}\n\
         \t}\n}\n\n",
    );

    // display lookup if applicable
    if has_display {
        out.push_str(&format!(
            "pub fn {}_display(id: {}Id) -> &'static str {{\n\tmatch id.0 {{\n\t\t0 => \"\",\n",
            ns_lower, ty,
        ));
        let mut next: u16 = 1;
        for e in entries {
            if !e.tombstone {
                let display = if e.display_name.is_empty() { &e.id } else { &e.display_name };
                out.push_str(&format!(
                    "\t\t{} => \"{}\",\n",
                    next, escape_str_literal(display),
                ));
            }
            next += 1;
        }
        out.push_str(
            "\t\t_ => {\n\
             \t\t\tdebug_assert!(false, \"unknown id: {}\", id.0);\n\
             \t\t\t\"?\"\n\
             \t\t}\n\
             \t}\n}\n\n",
        );
    }
}

/// Towers 走自己的 emit：除了 id/display 外還要生 `TOWER_<NAME>_STATS: TowerStats`
/// 跟 `tower_stats(id)` lookup。
fn emit_tower_namespace(out: &mut String, entries: &[TowerEntry]) {
    let converted: Vec<Entry> = entries.iter().map(|t| Entry {
        id: t.id.clone(),
        display_name: t.display_name.clone(),
        tombstone: t.tombstone,
    }).collect();
    emit_namespace(out, "Tower", "tower", &converted, /*display*/true);

    // Per-tower const: TOWER_<NAME>_STATS
    for e in entries {
        if e.tombstone {
            continue;
        }
        let cname = const_name("tower", &e.id);
        out.push_str(&format!(
            "pub const {}_STATS: TowerStats = TowerStats {{\n\
             \tatk: {:?}_f32,\n\
             \tasd_interval: {:?}_f32,\n\
             \trange: {:?}_f32,\n\
             \tbullet_speed: {:?}_f32,\n\
             \tsplash_radius: {:?}_f32,\n\
             \thit_radius: {:?}_f32,\n\
             \tslow_factor: {:?}_f32,\n\
             \tslow_duration: {:?}_f32,\n\
             \tcost: {}i32,\n\
             \tfootprint: {:?}_f32,\n\
             \thp: {:?}_f32,\n\
             \tturn_speed_deg: {:?}_f32,\n\
             }};\n",
            cname,
            e.atk, e.asd_interval, e.range, e.bullet_speed,
            e.splash_radius, e.hit_radius, e.slow_factor, e.slow_duration,
            e.cost, e.footprint, e.hp, e.turn_speed_deg,
        ));
    }
    out.push('\n');

    // Lookup tower_stats(TowerId) → Option<&'static TowerStats>
    out.push_str("pub fn tower_stats(id: TowerId) -> Option<&'static TowerStats> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            let cname = const_name("tower", &e.id);
            out.push_str(&format!(
                "\t\t{} => Some(&{}_STATS),\n",
                next, cname,
            ));
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}

fn emit_hero_namespace(out: &mut String, entries: &[HeroEntry]) {
    // Reuse emit_namespace machinery for id/display; add hero_title lookup.
    let converted: Vec<Entry> = entries.iter().map(|h| Entry {
        id: h.id.clone(),
        display_name: h.display_name.clone(),
        tombstone: h.tombstone,
    }).collect();
    emit_namespace(out, "Hero", "hero", &converted, true);

    out.push_str("pub fn hero_title(id: HeroId) -> &'static str {\n\tmatch id.0 {\n\t\t0 => \"\",\n");
    let mut next: u16 = 1;
    for h in entries {
        if !h.tombstone {
            out.push_str(&format!(
                "\t\t{} => \"{}\",\n",
                next, escape_str_literal(&h.title),
            ));
        }
        next += 1;
    }
    out.push_str("\t\t_ => \"\",\n\t}\n}\n\n");
}

/// Hero → abilities[] codegen — 從 templates.json heroes[i].abilities 字串陣列翻成
/// `HERO_<NAME>_ABILITIES: &[AbilityId] = &[AbilityId(N), ...]` const + lookup
/// `hero_abilities(HeroId) -> &'static [AbilityId]`。消除 omoba-core 端寫死的
/// `match hero_type { "saika_..." => [...] }` match 表。
fn emit_hero_abilities(
    out: &mut String,
    entries: &[HeroEntry],
    ability_ids: &std::collections::HashMap<&str, u16>,
) {
    for e in entries {
        if e.tombstone {
            continue;
        }
        let cname = const_name("hero", &e.id);
        out.push_str(&format!(
            "pub const {}_ABILITIES: &[AbilityId] = &[\n",
            cname,
        ));
        for ab_id in &e.abilities {
            let raw = ability_ids.get(ab_id.as_str()).unwrap_or_else(|| {
                panic!(
                    "hero '{}' references unknown ability '{}'",
                    e.id, ab_id
                )
            });
            out.push_str(&format!("\tAbilityId({}),\n", raw));
        }
        out.push_str("];\n");
    }
    out.push('\n');

    out.push_str(
        "pub fn hero_abilities(id: HeroId) -> &'static [AbilityId] {\n\tmatch id.0 {\n",
    );
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

/// emit `HERO_<NAME>_STATS: HeroStats` const + `hero_stats(id)` lookup。
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

/// emit `CREEP_<NAME>_STATS: CreepStats` const + `creep_stats(id)` lookup。
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

/// emit `SUMMON_<NAME>_STATS: SummonStats` const + `summon_stats(id)` lookup。
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

fn ability_type_to_variant(s: &str) -> &'static str {
    match s {
        "" | "active" => "Active",
        "toggle" => "Toggle",
        "ultimate" => "Ultimate",
        "passive" => "Passive",
        other => panic!("unknown ability_type '{}'", other),
    }
}

fn cast_type_to_variant(s: &str) -> &'static str {
    match s {
        "" | "instant" => "Instant",
        "channeled" => "Channeled",
        other => panic!("unknown cast_type '{}'", other),
    }
}

fn target_type_to_variant(s: &str) -> &'static str {
    match s {
        "" | "none" => "None",
        "point" => "Point",
        "unit" => "Unit",
        other => panic!("unknown target_type '{}'", other),
    }
}

/// Emit `ABILITY_<NAME>_CONST: AbilityConst` const + `ability_const(id)` lookup。
/// 每個 ability 的 levels 數目必須等於 max_level；extras[k].len() 也必須 = max_level。
fn emit_ability_const(out: &mut String, entries: &[AbilityEntry]) {
    // emit per-ability levels static + extras static + the const itself
    for e in entries {
        if e.tombstone { continue; }
        if e.levels.len() != e.max_level as usize {
            panic!(
                "ability '{}': levels.len()={} but max_level={}",
                e.id, e.levels.len(), e.max_level
            );
        }
        for (k, v) in &e.extras {
            if v.len() != e.max_level as usize {
                panic!(
                    "ability '{}': extras['{}'].len()={} but max_level={}",
                    e.id, k, v.len(), e.max_level
                );
            }
        }

        let cname = const_name("ability", &e.id);

        // emit per-extra static slice (must be named to take address of in const)
        // but actually `&[...]` is fine inline; we'll just inline the literal.
        out.push_str(&format!("pub const {}_LEVELS: &[AbilityLevelDataConst] = &[\n", cname));
        for ld in &e.levels {
            out.push_str(&format!(
                "\tAbilityLevelDataConst {{ cooldown: {:?}_f32, mana_cost: {:?}_f32, cast_time: {:?}_f32, range: {:?}_f32 }},\n",
                ld.cooldown, ld.mana_cost, ld.cast_time, ld.range,
            ));
        }
        out.push_str("];\n");

        // Emit each extra's per-level slice as a named static so the &[(key, &[..])]
        // can reference it (avoids `&[f32; N]` literal lifetime issues).
        for (k, v) in &e.extras {
            let extra_const = format!("{}_EXTRA_{}", cname, sanitize_ident(k).to_uppercase());
            out.push_str(&format!("pub const {}: &[f32] = &[", extra_const));
            for (i, x) in v.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&format!("{:?}_f32", x));
            }
            out.push_str("];\n");
        }
        out.push_str(&format!("pub const {}_EXTRAS: &[(&str, &[f32])] = &[\n", cname));
        for (k, _v) in &e.extras {
            let extra_const = format!("{}_EXTRA_{}", cname, sanitize_ident(k).to_uppercase());
            out.push_str(&format!("\t(\"{}\", {}),\n", escape_str_literal(k), extra_const));
        }
        out.push_str("];\n");

        out.push_str(&format!(
            "pub const {}_CONST: AbilityConst = AbilityConst {{\n\
             \tability_type: AbilityTypeC::{},\n\
             \tcast_type: CastTypeC::{},\n\
             \ttarget_type: TargetTypeC::{},\n\
             \tmax_level: {}u8,\n\
             \tdescription: \"{}\",\n\
             \tlevels: {}_LEVELS,\n\
             \textras: {}_EXTRAS,\n\
             }};\n",
            cname,
            ability_type_to_variant(&e.ability_type),
            cast_type_to_variant(&e.cast_type),
            target_type_to_variant(&e.target_type),
            e.max_level,
            escape_str_literal(&e.description),
            cname, cname,
        ));
    }
    out.push('\n');

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

/// Emit `ability_description(id)` lookup — runtime helper for tooltips。
fn emit_ability_description(out: &mut String, entries: &[AbilityEntry]) {
    out.push_str("pub fn ability_description(id: AbilityId) -> &'static str {\n\tmatch id.0 {\n\t\t0 => \"\",\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            out.push_str(&format!(
                "\t\t{} => \"{}\",\n",
                next, escape_str_literal(&e.description),
            ));
        }
        next += 1;
    }
    out.push_str("\t\t_ => \"\",\n\t}\n}\n\n");
}

/// 把 extras key（例 "true_damage_pct"）轉成合法 Rust ident 片段（純 ASCII 大寫字母 + 底線）。
fn sanitize_ident(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_uppercase() } else { '_' })
        .collect()
}

fn emit_projectile_kinds(out: &mut String, entries: &[ProjKind]) {
    // No display_name for projectile kinds (they are visual kinds, not labels).
    let converted: Vec<Entry> = entries.iter().map(|p| Entry {
        id: p.id.clone(),
        display_name: String::new(),
        tombstone: p.tombstone,
    }).collect();
    emit_namespace(out, "ProjectileKind", "projectile", &converted, false);
}
