//! Build-time codegen for template ids.
//!
//! Reads Lua builders under `scripts/lua_data` and emits `template_ids_gen.rs` into OUT_DIR.
//! Id allocation: sequential u16 in Lua declaration order, starting from 1.
//! Id 0 is reserved as UNSPECIFIED. Tombstone entries (`"tombstone": true`)
//! consume an id but emit no const / lookup entry — append-only safety.

use mlua::{Lua, LuaSerdeExt, Value as LuaValue};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

/// Format an f32 value from Lua content as a `Fixed64::from_raw(N)` literal,
/// where N = round(v * 1024). Mirrors `omoba_sim::fixed::SCALE = 1024`. Lua
/// numbers are the authoring format; the cast is the lockstep boundary — every
/// f32 we generate goes through here so precision loss is bounded and
/// deterministic across hosts.
fn fixed64_lit(v: f32) -> String {
    let raw = (v * 1024.0).round() as i32;
    format!("Fixed64::from_raw({})", raw)
}

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

/// Tower entry — id + display_name + 全部 stat 欄位都從 templates.lua 讀。
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
    /// Phase D: 3 路線 × 4 級 upgrade tree。長度必須 = 3 (paths)，每路 4 entries。
    /// 空陣列等於 tower 沒有 upgrade tree（測試 / 未來新塔可暫缺）。
    #[serde(default)] upgrades: Vec<Vec<UpgradeEntry>>,
}

/// 單一 upgrade 條目（per path × per level）。
#[derive(Deserialize, Clone)]
struct UpgradeEntry {
    #[serde(default)] name: String,
    #[serde(default)] description: String,
    #[serde(default)] cost: i32,
    #[serde(default)] effects: Vec<UpgradeEffectEntry>,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
enum UpgradeEffectEntry {
    StatMod {
        key: String,
        value: f32,
        #[serde(default = "default_stat_op")] op: String, // "add" or "mul"
    },
    BehaviorFlag { flag: String },
}

fn default_stat_op() -> String { "add".into() }

#[derive(Deserialize)]
struct HeroEntry {
    id: String,
    #[serde(default)] display_name: String,
    #[serde(default)] title: String,
    #[serde(default)] #[allow(dead_code)] background: String,
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

/// Creep / Enemy entry — 對應 templates.lua creeps[]。
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

/// Summon entry — 對應 templates.lua summons[]。
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

/// Ability entry — 對應 templates.lua abilities[]。
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

#[derive(Clone)]
struct LuaContentLoader {
    root: PathBuf,
    root_canonical: PathBuf,
    state: Rc<RefCell<LuaLoaderState>>,
}

#[derive(Default)]
struct LuaLoaderState {
    read_files: BTreeSet<PathBuf>,
    include_stack: Vec<PathBuf>,
}

impl LuaContentLoader {
    fn new(root: PathBuf) -> Self {
        let root_canonical = root
            .canonicalize()
            .unwrap_or_else(|e| panic!("canonicalize content root {}: {}", root.display(), e));
        Self {
            root,
            root_canonical,
            state: Rc::new(RefCell::new(LuaLoaderState::default())),
        }
    }

    fn load<T>(&self, lua: &Lua, rel_path: &str) -> T
    where
        T: DeserializeOwned,
    {
        let value = self
            .load_value(lua, rel_path)
            .unwrap_or_else(|e| panic!("load Lua builder {}: {}", rel_path, e));
        lua.from_value(value)
            .unwrap_or_else(|e| panic!("convert Lua builder {} output: {}", rel_path, e))
    }

    fn load_json_value(&self, lua: &Lua, rel_path: &str) -> serde_json::Value {
        self.load(lua, rel_path)
    }

    fn load_value(&self, lua: &Lua, rel_path: &str) -> mlua::Result<LuaValue> {
        let full_path = self.resolve_existing(rel_path)?;
        {
            let state = self.state.borrow();
            if let Some(first) = state.include_stack.iter().position(|p| p == &full_path) {
                let mut cycle: Vec<String> = state.include_stack[first..]
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect();
                cycle.push(full_path.display().to_string());
                return Err(mlua::Error::external(format!(
                    "Lua include cycle: {}",
                    cycle.join(" -> ")
                )));
            }
        }

        {
            let mut state = self.state.borrow_mut();
            state.read_files.insert(full_path.clone());
            state.include_stack.push(full_path.clone());
        }

        let result = (|| {
            let source = fs::read_to_string(&full_path).map_err(|e| {
                mlua::Error::external(format!("read {}: {}", full_path.display(), e))
            })?;
            let builder: mlua::Function = lua.load(&source).set_name(rel_path).eval()?;
            let ctx = self.create_context(lua)?;
            builder.call(ctx)
        })();

        self.state.borrow_mut().include_stack.pop();
        result
    }

    fn create_context(&self, lua: &Lua) -> mlua::Result<mlua::Table> {
        let ctx = lua.create_table()?;

        let include_loader = self.clone();
        ctx.set(
            "include",
            lua.create_function(move |lua, rel_path: String| include_loader.load_value(lua, &rel_path))?,
        )?;

        let read_text_loader = self.clone();
        ctx.set(
            "read_text",
            lua.create_function(move |_lua, rel_path: String| read_text_loader.read_text(&rel_path))?,
        )?;

        let read_toml_loader = self.clone();
        ctx.set(
            "read_toml",
            lua.create_function(move |lua, rel_path: String| {
                let text = read_toml_loader.read_text(&rel_path)?;
                let parsed: toml::Value = toml::from_str(&text).map_err(|e| {
                    mlua::Error::external(format!("parse TOML {}: {}", rel_path, e))
                })?;
                lua.to_value(&parsed)
            })?,
        )?;

        Ok(ctx)
    }

    fn read_text(&self, rel_path: &str) -> mlua::Result<String> {
        let full_path = self.resolve_existing(rel_path)?;
        self.state.borrow_mut().read_files.insert(full_path.clone());
        fs::read_to_string(&full_path).map_err(|e| {
            mlua::Error::external(format!("read {}: {}", full_path.display(), e))
        })
    }

    fn resolve_existing(&self, rel_path: &str) -> mlua::Result<PathBuf> {
        let rel = Path::new(rel_path);
        if rel.as_os_str().is_empty() || rel.is_absolute() {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': must be relative to scripts/lua_data",
                rel_path
            )));
        }
        for component in rel.components() {
            match component {
                Component::Normal(_) | Component::CurDir => {}
                Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                    return Err(mlua::Error::external(format!(
                        "rejected content path '{}': parent/absolute paths are not allowed",
                        rel_path
                    )));
                }
            }
        }
        let full_path = self.root.join(rel);
        let canonical = full_path.canonicalize().map_err(|e| {
            mlua::Error::external(format!("resolve {}: {}", full_path.display(), e))
        })?;
        if !canonical.starts_with(&self.root_canonical) {
            return Err(mlua::Error::external(format!(
                "rejected content path '{}': resolved outside scripts/lua_data",
                rel_path
            )));
        }
        Ok(canonical)
    }

    fn read_files(&self) -> Vec<PathBuf> {
        self.state.borrow().read_files.iter().cloned().collect()
    }
}

struct StoryBundle {
    id: String,
    entity: serde_json::Value,
    ability: serde_json::Value,
    mission: serde_json::Value,
    map: serde_json::Value,
}

fn main() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let content_root = manifest_dir.parent().unwrap().join("scripts/lua_data");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", content_root.display());

    let lua = Lua::new();
    let loader = LuaContentLoader::new(content_root.clone());
    let m: Manifest = loader.load(&lua, "templates.lua");
    let stories = load_stories(&loader, &lua, &content_root, &m);

    for path in loader.read_files() {
        println!("cargo:rerun-if-changed={}", path.display());
    }

    let mut out = String::new();
    out.push_str("// AUTO-GENERATED by omoba-template-ids/build.rs — DO NOT EDIT.\n");
    out.push_str("// Source: scripts/lua_data Lua builders\n\n");

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

    // Phase D: tower upgrade tree const + lookup
    emit_tower_upgrades(&mut out, &m.towers);

    // Generated story data for pure-Rust runtime loading.
    emit_story_data(&mut out, &stories);

    let out_dir = std::env::var("OUT_DIR").unwrap();
    let out_path = format!("{}/template_ids_gen.rs", out_dir);
    fs::write(&out_path, out).unwrap();
}

fn escape_str_literal(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn load_stories(
    loader: &LuaContentLoader,
    lua: &Lua,
    content_root: &Path,
    manifest: &Manifest,
) -> Vec<StoryBundle> {
    let mut story_ids: Vec<String> = fs::read_dir(content_root)
        .unwrap_or_else(|e| panic!("read content root {}: {}", content_root.display(), e))
        .filter_map(|entry| {
            let entry = entry.unwrap_or_else(|e| panic!("read content root entry: {}", e));
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let has_story_files = ["entity.lua", "ability.lua", "mission.lua", "map.lua"]
                .iter()
                .all(|name| path.join(name).is_file());
            has_story_files.then_some(id)
        })
        .collect();
    story_ids.sort();

    let active_creeps: HashSet<&str> = manifest
        .creeps
        .iter()
        .filter(|creep| !creep.tombstone)
        .map(|creep| creep.id.as_str())
        .collect();

    story_ids
        .into_iter()
        .map(|id| {
            let entity = loader.load_json_value(lua, &format!("{}/entity.lua", id));
            let ability = loader.load_json_value(lua, &format!("{}/ability.lua", id));
            let mission = loader.load_json_value(lua, &format!("{}/mission.lua", id));
            let map = loader.load_json_value(lua, &format!("{}/map.lua", id));
            validate_map_creep_references(&id, &map, &active_creeps);
            StoryBundle { id, entity, ability, mission, map }
        })
        .collect()
}

fn validate_map_creep_references(story_id: &str, map: &serde_json::Value, active_creeps: &HashSet<&str>) {
    let forbidden = [
        "Label",
        "HP",
        "DefendPhysic",
        "DefendMagic",
        "MoveSpeed",
        "damage",
        "attack_range",
        "enemy_type",
        "ai_type",
        "exp_reward",
        "gold_reward",
        "coins",
    ];
    let Some(creeps) = map.get("Creep").and_then(serde_json::Value::as_array) else {
        return;
    };
    for creep in creeps {
        let name = creep
            .get("Name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("story {} map Creep[] entry missing Name", story_id));
        for field in forbidden {
            if creep.get(field).is_some() {
                panic!(
                    "story {} map Creep '{}' has forbidden map-local unit field '{}'",
                    story_id, name, field
                );
            }
        }
        if !active_creeps.contains(name) {
            panic!(
                "story {} map Creep '{}' does not resolve to a generated creep template",
                story_id, name
            );
        }
    }
}

fn emit_story_data(out: &mut String, stories: &[StoryBundle]) {
    out.push_str("\n// ===== Generated shipped story data =====\n");
    for story in stories {
        let const_name = story_const_name(&story.id);
        out.push_str(&format!("pub static {}: GeneratedStory = GeneratedStory {{\n", const_name));
        out.push_str(&format!("    id: \"{}\",\n", escape_str_literal(&story.id)));
        out.push_str(&format!("    entity: {},\n", emit_story_value(&story.entity, 1)));
        out.push_str(&format!("    ability: {},\n", emit_story_value(&story.ability, 1)));
        out.push_str(&format!("    mission: {},\n", emit_story_value(&story.mission, 1)));
        out.push_str(&format!("    map: {},\n", emit_story_value(&story.map, 1)));
        out.push_str("};\n\n");
    }

    out.push_str("pub fn story_ids() -> &'static [&'static str] {\n    &[\n");
    for story in stories {
        out.push_str(&format!("        \"{}\",\n", escape_str_literal(&story.id)));
    }
    out.push_str("    ]\n}\n\n");

    out.push_str("pub fn story_by_name(name: &str) -> Option<&'static GeneratedStory> {\n    match name {\n");
    for story in stories {
        out.push_str(&format!(
            "        \"{}\" => Some(&{}),\n",
            escape_str_literal(&story.id),
            story_const_name(&story.id)
        ));
    }
    out.push_str("        _ => None,\n    }\n}\n");
}

fn story_const_name(id: &str) -> String {
    format!("STORY_{}", sanitize_const_ident(id))
}

fn sanitize_const_ident(id: &str) -> String {
    let mut out = String::new();
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_uppercase());
        } else {
            out.push('_');
        }
    }
    out
}

fn emit_story_value(value: &serde_json::Value, indent: usize) -> String {
    let space = "    ".repeat(indent);
    let child = "    ".repeat(indent + 1);
    match value {
        serde_json::Value::Null => "StoryValue::Null".to_string(),
        serde_json::Value::Bool(v) => format!("StoryValue::Bool({})", v),
        serde_json::Value::Number(v) => {
            let n = v.as_f64().unwrap_or_else(|| panic!("story number is not f64: {}", v));
            format!("StoryValue::Number({:?})", n)
        }
        serde_json::Value::String(v) => {
            format!("StoryValue::String(\"{}\")", escape_str_literal(v))
        }
        serde_json::Value::Array(items) => {
            if items.is_empty() {
                return "StoryValue::Array(&[])".to_string();
            }
            let mut s = String::from("StoryValue::Array(&[\n");
            for item in items {
                s.push_str(&child);
                s.push_str(&emit_story_value(item, indent + 1));
                s.push_str(",\n");
            }
            s.push_str(&space);
            s.push_str("])");
            s
        }
        serde_json::Value::Object(items) => {
            if items.is_empty() {
                return "StoryValue::Object(&[])".to_string();
            }
            let mut s = String::from("StoryValue::Object(&[\n");
            for (key, item) in items {
                s.push_str(&child);
                s.push_str(&format!(
                    "(\"{}\", {}),\n",
                    escape_str_literal(key),
                    emit_story_value(item, indent + 1)
                ));
            }
            s.push_str(&space);
            s.push_str("])");
            s
        }
    }
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
             \tatk: {},\n\
             \tasd_interval: {},\n\
             \trange: {},\n\
             \tbullet_speed: {},\n\
             \tsplash_radius: {},\n\
             \thit_radius: {},\n\
             \tslow_factor: {},\n\
             \tslow_duration: {},\n\
             \tcost: {}i32,\n\
             \tfootprint: {},\n\
             \thp: {},\n\
             \tturn_speed_deg: {},\n\
             }};\n",
            cname,
            fixed64_lit(e.atk), fixed64_lit(e.asd_interval), fixed64_lit(e.range), fixed64_lit(e.bullet_speed),
            fixed64_lit(e.splash_radius), fixed64_lit(e.hit_radius), fixed64_lit(e.slow_factor), fixed64_lit(e.slow_duration),
            e.cost, fixed64_lit(e.footprint), fixed64_lit(e.hp), fixed64_lit(e.turn_speed_deg),
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

/// Hero → abilities[] codegen — 從 templates.lua heroes[i].abilities 字串陣列翻成
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
             \tattack_range: {},\n\
             \tbase_damage: {}i32,\n\
             \tbase_armor: {},\n\
             \tbase_hp: {}i32,\n\
             \tbase_mana: {}i32,\n\
             \tmove_speed: {},\n\
             \tturn_speed: {},\n\
             \tlevel_growth: LevelGrowth {{\n\
             \t\tstrength_per_level: {},\n\
             \t\tagility_per_level: {},\n\
             \t\tintelligence_per_level: {},\n\
             \t\tdamage_per_level: {},\n\
             \t\thp_per_level: {},\n\
             \t\tmana_per_level: {},\n\
             \t}},\n\
             }};\n",
            cname,
            e.strength, e.agility, e.intelligence,
            primary_attribute_to_u8(&e.primary_attribute),
            fixed64_lit(e.attack_range), e.base_damage, fixed64_lit(e.base_armor),
            e.base_hp, e.base_mana, fixed64_lit(e.move_speed), fixed64_lit(e.turn_speed),
            fixed64_lit(e.level_growth.strength_per_level),
            fixed64_lit(e.level_growth.agility_per_level),
            fixed64_lit(e.level_growth.intelligence_per_level),
            fixed64_lit(e.level_growth.damage_per_level),
            fixed64_lit(e.level_growth.hp_per_level),
            fixed64_lit(e.level_growth.mana_per_level),
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
             \thp: {},\n\
             \tarmor: {},\n\
             \tmagic_resistance: {},\n\
             \tdamage: {},\n\
             \tattack_range: {},\n\
             \tmove_speed: {},\n\
             \tenemy_type: {}u8,\n\
             \tai_type: {}u8,\n\
             \texp_reward: {}i32,\n\
             \tgold_reward: {}i32,\n\
             }};\n",
            cname,
            fixed64_lit(e.hp), fixed64_lit(e.armor), fixed64_lit(e.magic_resistance),
            fixed64_lit(e.damage), fixed64_lit(e.attack_range), fixed64_lit(e.move_speed),
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
             \thp: {},\n\
             \tdamage: {},\n\
             \tduration: {},\n\
             \tmove_speed: {},\n\
             }};\n",
            cname,
            fixed64_lit(e.hp), fixed64_lit(e.damage),
            fixed64_lit(e.duration), fixed64_lit(e.move_speed),
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
                "\tAbilityLevelDataConst {{ cooldown: {}, mana_cost: {}, cast_time: {}, range: {} }},\n",
                fixed64_lit(ld.cooldown), fixed64_lit(ld.mana_cost),
                fixed64_lit(ld.cast_time), fixed64_lit(ld.range),
            ));
        }
        out.push_str("];\n");

        // Emit each extra's per-level slice as a named static so the &[(key, &[..])]
        // can reference it (avoids `&[Fixed64; N]` literal lifetime issues).
        for (k, v) in &e.extras {
            let extra_const = format!("{}_EXTRA_{}", cname, sanitize_ident(k).to_uppercase());
            out.push_str(&format!("pub const {}: &[Fixed64] = &[", extra_const));
            for (i, x) in v.iter().enumerate() {
                if i > 0 { out.push_str(", "); }
                out.push_str(&fixed64_lit(*x));
            }
            out.push_str("];\n");
        }
        out.push_str(&format!("pub const {}_EXTRAS: &[(&str, &[Fixed64])] = &[\n", cname));
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

fn stat_op_to_variant(s: &str) -> &'static str {
    match s {
        "" | "add" => "Add",
        "mul" => "Mul",
        other => panic!("unknown stat_op '{}', expected add|mul", other),
    }
}

/// Emit `TOWER_<NAME>_UPGRADES: &[&[UpgradeDefConst]] = &[Path0, Path1, Path2]`
/// + `tower_upgrades(id) -> &'static [&'static [UpgradeDefConst]]` lookup。
/// 跳過 tombstone / 沒有 upgrades 的 tower。
fn emit_tower_upgrades(out: &mut String, entries: &[TowerEntry]) {
    use omoba_core_dummy::upgrade_cost; // local helper, see bottom

    for e in entries {
        if e.tombstone || e.upgrades.is_empty() {
            continue;
        }
        if e.upgrades.len() != 3 {
            panic!("tower '{}' upgrades must have 3 paths, got {}", e.id, e.upgrades.len());
        }
        for (path_idx, path) in e.upgrades.iter().enumerate() {
            if path.len() != 4 {
                panic!(
                    "tower '{}' path {} must have 4 levels, got {}",
                    e.id, path_idx, path.len()
                );
            }
            for (lvl_idx, u) in path.iter().enumerate() {
                let lvl = (lvl_idx + 1) as u8;
                let expected = upgrade_cost(e.cost, lvl);
                if u.cost != expected {
                    panic!(
                        "tower '{}' path {} L{} cost mismatch: declared {} but base {} × multiplier = {}",
                        e.id, path_idx, lvl, u.cost, e.cost, expected
                    );
                }
            }
        }

        let cname = const_name("tower", &e.id);

        // Per-upgrade effects array — emit a named static so the slice address is stable.
        for (path_idx, path) in e.upgrades.iter().enumerate() {
            for (lvl_idx, u) in path.iter().enumerate() {
                let effects_const = format!("{}_UPGRADE_P{}_L{}_EFFECTS", cname, path_idx, lvl_idx + 1);
                out.push_str(&format!(
                    "pub const {}: &[UpgradeEffectConst] = &[\n",
                    effects_const
                ));
                for ef in &u.effects {
                    match ef {
                        UpgradeEffectEntry::StatMod { key, value, op } => {
                            out.push_str(&format!(
                                "\tUpgradeEffectConst {{ kind: UpgradeEffectKindC::StatMod, key: \"{}\", value: {}, op: StatOpC::{} }},\n",
                                escape_str_literal(key), fixed64_lit(*value), stat_op_to_variant(op)
                            ));
                        }
                        UpgradeEffectEntry::BehaviorFlag { flag } => {
                            out.push_str(&format!(
                                "\tUpgradeEffectConst {{ kind: UpgradeEffectKindC::BehaviorFlag, key: \"{}\", value: Fixed64::from_raw(0), op: StatOpC::Add }},\n",
                                escape_str_literal(flag)
                            ));
                        }
                    }
                }
                out.push_str("];\n");
            }
        }

        // Per-path slice
        for (path_idx, path) in e.upgrades.iter().enumerate() {
            let path_const = format!("{}_UPGRADES_P{}", cname, path_idx);
            out.push_str(&format!(
                "pub const {}: &[UpgradeDefConst] = &[\n",
                path_const
            ));
            for (lvl_idx, u) in path.iter().enumerate() {
                let effects_const = format!("{}_UPGRADE_P{}_L{}_EFFECTS", cname, path_idx, lvl_idx + 1);
                out.push_str(&format!(
                    "\tUpgradeDefConst {{ name: \"{}\", description: \"{}\", cost: {}i32, effects: {} }},\n",
                    escape_str_literal(&u.name),
                    escape_str_literal(&u.description),
                    u.cost,
                    effects_const,
                ));
            }
            out.push_str("];\n");
        }

        // Top-level &[&[UpgradeDefConst]]
        out.push_str(&format!(
            "pub const {}_UPGRADES: &[&[UpgradeDefConst]] = &[\n",
            cname
        ));
        for path_idx in 0..3 {
            let path_const = format!("{}_UPGRADES_P{}", cname, path_idx);
            out.push_str(&format!("\t{},\n", path_const));
        }
        out.push_str("];\n");
    }
    out.push('\n');

    // tower_upgrades(id) lookup
    out.push_str("pub fn tower_upgrades(id: TowerId) -> Option<&'static [&'static [UpgradeDefConst]]> {\n\tmatch id.0 {\n");
    let mut next: u16 = 1;
    for e in entries {
        if !e.tombstone {
            if e.upgrades.is_empty() {
                // skip
            } else {
                let cname = const_name("tower", &e.id);
                out.push_str(&format!("\t\t{} => Some({}_UPGRADES),\n", next, cname));
            }
        }
        next += 1;
    }
    out.push_str("\t\t_ => None,\n\t}\n}\n\n");
}

mod omoba_core_dummy {
    /// Mirror of `omoba_core::tower_meta::upgrade_cost` — duplicate locally to
    /// avoid build-time dependency on omoba-core.
    pub fn upgrade_cost(base_cost: i32, level: u8) -> i32 {
        let mul: f32 = match level {
            1 => 0.25,
            2 => 0.50,
            3 => 1.00,
            4 => 2.50,
            _ => return 0,
        };
        (base_cost as f32 * mul) as i32
    }
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
