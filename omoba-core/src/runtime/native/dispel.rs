//! Dispel by class (E214): what the v3 documents' `dispel` says a dispel removes.
//!
//! * The classes come from compiled content: every status in `V3_CATALOG_JSON` (hero abilities
//!   and shelf items) under its `runtime_id`, else the executor's default key (its id, or
//!   `<id>:{caster}` for a per-source status). `{caster}` in a runtime id matches the
//!   applier's `id:gen`, `{rank}` a rank number. A key several statuses share (`stun`) takes the
//!   strictest class among them: the BuffStore keeps one entry per key, so it cannot tell which
//!   status applied it.
//! * `Strength::Basic` removes class `basic`; `Strength::Strong` also `strong`. `none` and
//!   `death_only` stay.
//! * `Purge::Debuffs` (a dispel by the unit or an ally) removes `debuff` statuses, `Purge::Buffs`
//!   (an enemy purge) `buff` ones, whoever applied them. A `hidden` status goes by its applier
//!   (the key's `…:<id>:<gen>` suffix or the payload's `source_entity_id/gen`): an enemy's is a
//!   debuff, the unit's own or an ally's a buff; with no living applier it stays.
//! * Keys the catalog does not list keep the older strong-dispel rules (controls, the legacy slow
//!   and healing reduction, hostile source-tagged buffs), for a strong dispel of debuffs only.
use super::ability_runtime::BuffStore;
use super::comp::Faction;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::sync::OnceLock;
use specs::{Entity, World, WorldExt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Class { Basic, Strong, DeathOnly, None }

impl Class {
    fn parse(text: &str) -> Option<Class> {
        Some(match text { "basic" => Class::Basic, "strong" => Class::Strong, "death_only" => Class::DeathOnly, "none" => Class::None, _ => return None })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strength { Basic, Strong }

/// Which side of the unit's statuses a dispel takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purge { Debuffs, Buffs }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule { pub class: Class, pub side: Option<Purge> }

fn side_of(visibility: &Value) -> Option<Purge> {
    match visibility.as_str() { Some("debuff") => Some(Purge::Debuffs), Some("buff") => Some(Purge::Buffs), _ => None }
}

/// Whether a BuffStore key is an instance of a runtime id (`{caster}`, `{rank}` placeholders);
/// shared with the buff icons' key lookup (E218).
pub use omoba_template_ids::runtime_id_matches;

fn is_pattern(runtime_id: &str) -> bool { runtime_id.contains("{caster}") || runtime_id.contains("{rank}") }

/// Runtime ids to dispel rules: exact keys, then `{caster}`/`{rank}` patterns; and each status's
/// own rule by its v3 id (for the control-source rows below).
#[derive(Debug, Default)]
pub struct Registry { exact: HashMap<String, Rule>, patterns: Vec<(String, Rule)>, by_status: HashMap<String, Rule> }

/// A status that holds a shared host control (`stun`, `root`, `silence`: one BuffStore key for
/// every status) also leaves a row `__ctl:<status id>` with payload `{"states": [...]}` for the
/// same time (ability_kit `apply_status`). A dispel then judges each such status by its own class:
/// the shared key goes when every status holding it was dispelled, and stays while one survives,
/// whatever the strictest class of the shared key is. A shared key with no such row (a hand-written
/// handler's stun) keeps the older rule.
pub const CONTROL_SOURCE_PREFIX: &str = "__ctl:";

/// The payload of a control-source row.
pub fn control_source_payload(states: &[&str]) -> Value {
    serde_json::json!({ "states": states })
}

fn controls_of(payload: &Value) -> Vec<&str> {
    payload.get("states").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str)
        .filter(|state| is_control(state)).collect()
}

impl Registry {
    /// From a V3_CATALOG_JSON document; statuses without a known class are left out.
    pub fn from_catalog(doc: &Value) -> Registry {
        /// The status's BuffStore keys: its `runtime_id`, else the executor's default (m3-ir-design
        /// §7): the status id, with `:{caster}` for a per-source status. The catalog row does not
        /// say per_source, so both are listed; a status that is not per-source never makes the second.
        fn runtime_keys(status: &Value) -> Vec<String> {
            match (status["runtime_id"].as_str(), status["id"].as_str()) {
                (Some(runtime_id), _) => vec![runtime_id.to_owned()],
                (None, Some(id)) => vec![id.to_owned(), format!("{id}:{{caster}}")],
                (None, None) => Vec::new(),
            }
        }
        let mut rules: BTreeMap<String, Rule> = BTreeMap::new();
        let mut by_status: HashMap<String, Rule> = HashMap::new();
        for list in ["abilities", "items"] {
            for entry in doc[list].as_array().into_iter().flatten() {
                for status in entry["v3"]["statuses"].as_array().into_iter().flatten() {
                    let Some(class) = status["dispel"].as_str().and_then(Class::parse) else { continue };
                    let side = side_of(&status["visibility"]);
                    if let Some(id) = status["id"].as_str() {
                        by_status.entry(id.to_owned())
                            .and_modify(|rule| { rule.class = rule.class.max(class); rule.side = rule.side.or(side); })
                            .or_insert(Rule { class, side });
                    }
                    for key in runtime_keys(status) {
                        rules.entry(key)
                            .and_modify(|rule| { rule.class = rule.class.max(class); rule.side = rule.side.or(side); })
                            .or_insert(Rule { class, side });
                    }
                }
            }
        }
        let mut registry = Registry { by_status, ..Registry::default() };
        for (key, rule) in rules {
            if is_pattern(&key) { registry.patterns.push((key, rule)); } else { registry.exact.insert(key, rule); }
        }
        registry
    }

    /// The rule for a BuffStore key.
    pub fn rule(&self, key: &str) -> Option<Rule> {
        self.exact.get(key).copied().or_else(|| self.patterns.iter().find(|(id, _)| runtime_id_matches(id, key)).map(|(_, rule)| *rule))
    }

    /// The rule of a status by its v3 id.
    pub fn status(&self, id: &str) -> Option<Rule> { self.by_status.get(id).copied() }

    /// The rule for a runtime id as the content writes it (`…:{caster}`).
    pub fn declared(&self, runtime_id: &str) -> Option<Rule> {
        self.exact.get(runtime_id).copied().or_else(|| self.patterns.iter().find(|(id, _)| id == runtime_id).map(|(_, rule)| *rule))
    }
}

/// The compiled content's registry, parsed once.
pub fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| serde_json::from_str::<Value>(omoba_template_ids::V3_CATALOG_JSON)
        .map(|doc| Registry::from_catalog(&doc)).unwrap_or_default())
}

/// Whether a buff id ends in an applier `…:<id>:<gen>` suffix.
fn id_names_applier(buff_id: &str) -> bool {
    let mut parts = buff_id.rsplitn(3, ':');
    parts.next().is_some_and(|gen| gen.parse::<i32>().is_ok())
        && parts.next().is_some_and(|id| id.parse::<u32>().is_ok())
        && parts.next().is_some()
}

/// The living caster a source-tagged buff names: `…:<id>:<gen>` ids or a `source_entity_id/gen` payload.
fn applier(world: &World, buff_id: &str, payload: &Value) -> Option<Entity> {
    let entities = world.entities();
    let from = |id: u32, gen: i32| {
        let entity = entities.entity(id);
        (entity.gen().id() == gen && entities.is_alive(entity)).then_some(entity)
    };
    if let (Some(id), Some(gen)) = (payload.get("source_entity_id").and_then(Value::as_u64), payload.get("source_entity_gen").and_then(Value::as_i64)) {
        return u32::try_from(id).ok().zip(i32::try_from(gen).ok()).and_then(|(id, gen)| from(id, gen));
    }
    let mut parts = buff_id.rsplitn(3, ':');
    let gen = parts.next()?.parse::<i32>().ok()?;
    let id = parts.next()?.parse::<u32>().ok()?;
    parts.next()?;
    from(id, gen)
}

fn is_control(buff_id: &str) -> bool {
    use omb_script_abi::buff_ids::BuffId;
    [BuffId::Stun.as_str(), BuffId::Root.as_str(), BuffId::Silence.as_str()].contains(&buff_id)
}

/// Remove the unit's statuses the dispel takes; returns how many went.
pub fn dispel(world: &World, entity: Entity, strength: Strength, purge: Purge) -> usize {
    dispel_with(registry(), world, entity, strength, purge)
}

pub fn dispel_with(registry: &Registry, world: &World, entity: Entity, strength: Strength, purge: Purge) -> usize {
    let Some(own) = world.read_storage::<Faction>().get(entity).cloned() else { return 0 };
    let mut doomed: Vec<String> = {
        let buffs = world.read_resource::<BuffStore>();
        let factions = world.read_storage::<Faction>();
        let hostile = |source: Entity| factions.get(source).is_some_and(|f| own.is_hostile_to(f));
        buffs.iter_for(entity).filter(|(id, entry)| {
            // A control-source row goes by its own status's class; a debuff unless the status says.
            if let Some(status) = id.strip_prefix(CONTROL_SOURCE_PREFIX) {
                let Some(rule) = registry.status(status) else { return false };
                let removable = rule.class == Class::Basic || (rule.class == Class::Strong && strength == Strength::Strong);
                return removable && rule.side.unwrap_or(Purge::Debuffs) == purge;
            }
            match registry.rule(id) {
                Some(rule) => {
                    let removable = rule.class == Class::Basic || (rule.class == Class::Strong && strength == Strength::Strong);
                    let side = rule.side.or_else(|| applier(world, id, &entry.payload)
                        .map(|source| if hostile(source) { Purge::Debuffs } else { Purge::Buffs }));
                    removable && side == Some(purge)
                }
                None => {
                    if strength != Strength::Strong || purge != Purge::Debuffs { return false; }
                    if *id == super::magic_immunity::BUFF || id.starts_with("__") { return false; }
                    if is_control(id) || *id == "slow" || *id == "healing_reduction" { return true; }
                    // A `…:<id>:<gen>` id names its applier as well as a payload does (E211).
                    let tagged = entry.payload.get("source_entity_id").is_some() || id.starts_with("generic_") || id_names_applier(id);
                    if !tagged { return false; }
                    match applier(world, id, &entry.payload) {
                        Some(source) => hostile(source),
                        None => id.starts_with("generic_"),
                    }
                }
            }
        }).map(|(id, _)| id.to_owned()).collect()
    };
    follow_controls(world, entity, &mut doomed);
    doomed.sort();
    let mut buffs = world.write_resource::<BuffStore>();
    for id in &doomed { buffs.remove(entity, id); }
    doomed.len()
}

/// Shared controls follow the statuses holding them (CONTROL_SOURCE_PREFIX): a control goes when
/// every source row holding it goes, and stays while one survives.
fn follow_controls(world: &World, entity: Entity, doomed: &mut Vec<String>) {
    {
        let buffs = world.read_resource::<BuffStore>();
        let mut held: BTreeMap<&str, (usize, usize)> = BTreeMap::new(); // control -> (rows, rows surviving)
        for (id, entry) in buffs.iter_for(entity) {
            if !id.starts_with(CONTROL_SOURCE_PREFIX) { continue; }
            let survives = !doomed.iter().any(|doomed| doomed == id);
            for control in controls_of(&entry.payload) {
                let counts = held.entry(control).or_default();
                counts.0 += 1;
                if survives { counts.1 += 1; }
            }
        }
        for (control, (_, surviving)) in held {
            let pos = doomed.iter().position(|id| id == control);
            match (surviving, pos) {
                (0, None) if buffs.has(entity, control) => doomed.push(control.to_owned()),
                (n, Some(at)) if n > 0 => { doomed.remove(at); }
                _ => {}
            }
        }
    }
}

/// v3 `remove_status` by filters (not one named status): which of the unit's statuses go.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PurgeSpec {
    /// `basic`: class basic; `strong`: basic and strong; `all`: every class.
    pub mode: PurgeMode,
    /// `only` / `all_debuffs` / `all_buffs`: one side; `None` both.
    pub side: Option<Purge>,
    /// `team_of_source`: by the applier's team relative to the unit.
    pub team: SourceTeam,
    /// Status ids that stay (`except`).
    pub except: Vec<String>,
    /// `states`: only statuses holding one of these controls (and the controls themselves).
    pub states: Vec<String>,
    /// `limit`: at most this many statuses (key order); `None` no limit.
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurgeMode { Basic, Strong, All }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceTeam { Any, Enemy, Ally }

impl PurgeSpec {
    pub fn payload(&self) -> Value {
        serde_json::json!({ (PURGE_KEY): {
            "mode": match self.mode { PurgeMode::Basic => "basic", PurgeMode::Strong => "strong", PurgeMode::All => "all" },
            "side": self.side.map(|side| match side { Purge::Debuffs => "debuffs", Purge::Buffs => "buffs" }),
            "team": match self.team { SourceTeam::Any => "any", SourceTeam::Enemy => "enemy", SourceTeam::Ally => "ally" },
            "except": self.except, "states": self.states, "limit": self.limit,
        } })
    }
    pub fn from_payload(buff_id: &str, duration: omoba_sim::Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != PURGE_BUFF || duration != omoba_sim::Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(PURGE_KEY)?;
        let strings = |key: &str| -> Option<Vec<String>> {
            row.get(key)?.as_array()?.iter().map(|v| v.as_str().map(str::to_owned)).collect()
        };
        let spec = PurgeSpec {
            mode: match row.get("mode")?.as_str()? { "basic" => PurgeMode::Basic, "strong" => PurgeMode::Strong, "all" => PurgeMode::All, _ => return None },
            side: match row.get("side")? { Value::Null => None, side => Some(match side.as_str()? { "debuffs" => Purge::Debuffs, "buffs" => Purge::Buffs, _ => return None }) },
            team: match row.get("team")?.as_str()? { "any" => SourceTeam::Any, "enemy" => SourceTeam::Enemy, "ally" => SourceTeam::Ally, _ => return None },
            except: strings("except")?, states: strings("states")?,
            limit: match row.get("limit")? { Value::Null => None, n => Some(u32::try_from(n.as_u64()?).ok()?) },
        };
        (spec.except.len() <= 32 && spec.states.len() <= 8 && spec.states.iter().all(|s| is_control(s))).then_some(spec)
    }
}

/// The one-shot packet a script sends to the unit it purges (never stored).
pub const PURGE_KEY: &str = "__status_purge_v1";
pub const PURGE_BUFF: &str = "__status_purge";

/// Whether a BuffStore key is an instance of a status id (`id`, `id:…`).
fn key_of_status(key: &str, status: &str) -> bool {
    key == status || key.strip_prefix(status).is_some_and(|rest| rest.starts_with(':'))
}

/// Removes the statuses `spec` names from `entity`; returns how many went.
pub fn purge(world: &World, entity: Entity, spec: &PurgeSpec) -> usize {
    purge_with(registry(), world, entity, spec)
}

pub fn purge_with(registry: &Registry, world: &World, entity: Entity, spec: &PurgeSpec) -> usize {
    let Some(own) = world.read_storage::<Faction>().get(entity).cloned() else { return 0 };
    let mut doomed: Vec<String> = {
        let buffs = world.read_resource::<BuffStore>();
        let factions = world.read_storage::<Faction>();
        let hostile = |source: Entity| factions.get(source).is_some_and(|f| own.is_hostile_to(f));
        buffs.iter_for(entity).filter(|(id, entry)| {
            // Controls and their source rows have no applier: a control is an enemy's debuff.
            let control_row = id.strip_prefix(CONTROL_SOURCE_PREFIX);
            let control = is_control(id) || control_row.is_some();
            if id.starts_with("__") && control_row.is_none() { return false; }
            if *id == super::magic_immunity::BUFF { return false; }
            if !spec.states.is_empty() {
                let holds = if is_control(id) { spec.states.iter().any(|s| s == id) }
                    else { controls_of(&entry.payload).iter().any(|c| spec.states.iter().any(|s| s == c)) };
                if !holds { return false; }
            }
            let status = control_row.unwrap_or(id);
            if spec.except.iter().any(|except| key_of_status(status, except)) { return false; }
            let rule = match control_row { Some(row) => registry.status(row), None => registry.rule(id) };
            let class_ok = match (spec.mode, rule) {
                (PurgeMode::All, _) => true,
                (PurgeMode::Basic, Some(rule)) => rule.class == Class::Basic,
                (PurgeMode::Strong, Some(rule)) => matches!(rule.class, Class::Basic | Class::Strong),
                (_, None) => control,
            };
            if !class_ok { return false; }
            let source = if control { None } else { applier(world, id, &entry.payload) };
            let side = rule.and_then(|rule| rule.side).or_else(|| if control { Some(Purge::Debuffs) } else {
                source.map(|source| if hostile(source) { Purge::Debuffs } else { Purge::Buffs }) });
            if spec.side.is_some() && side != spec.side { return false; }
            match spec.team {
                SourceTeam::Any => true,
                SourceTeam::Enemy => control || source.is_some_and(|source| hostile(source)),
                SourceTeam::Ally => !control && source.is_some_and(|source| !hostile(source)),
            }
        }).map(|(id, _)| id.to_owned()).collect()
    };
    doomed.sort();
    if let Some(limit) = spec.limit {
        // The limit counts statuses, not their control rows or shared controls.
        let mut kept = 0u32;
        doomed.retain(|id| {
            if is_control(id) || id.starts_with(CONTROL_SOURCE_PREFIX) { return true; }
            kept += 1;
            kept <= limit
        });
    }
    follow_controls(world, entity, &mut doomed);
    doomed.sort();
    doomed.dedup();
    let mut buffs = world.write_resource::<BuffStore>();
    for id in &doomed { buffs.remove(entity, id); }
    doomed.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::comp::FactionType;
    use omoba_sim::Fixed64;
    use serde_json::json;
    use specs::Builder;

    #[test]
    fn patterns_match_caster_and_rank_placeholders() {
        let p = "a07_strongest_r{rank}:{caster}";
        assert!(runtime_id_matches(p, "a07_strongest_r3:12:1") && runtime_id_matches(p, "a07_strongest_r10:7:-2"));
        assert!(!runtime_id_matches(p, "a07_strongest_r:12:1") && !runtime_id_matches(p, "a07_strongest_r3:12") && !runtime_id_matches(p, "a07_strongest_r3:x:1"));
        let q = "generic_slow:b29_quake_axe:{caster}";
        assert!(runtime_id_matches(q, "generic_slow:b29_quake_axe:5:0") && !runtime_id_matches(q, "generic_slow:b29_quake_axe_x:5:0"));
        assert!(runtime_id_matches("stun", "stun") && !runtime_id_matches("stun", "stunned"));
    }

    #[test]
    fn compiled_content_gives_every_status_a_class_and_shared_keys_the_strictest() {
        let registry = registry();
        assert!(!registry.exact.is_empty() && !registry.patterns.is_empty(), "the catalog carries dispel classes");
        assert_eq!(registry.rule("stun"), Some(Rule { class: Class::Strong, side: Some(Purge::Debuffs) }), "basic and strong stuns share one key");
        assert_eq!(registry.rule("root"), Some(Rule { class: Class::Basic, side: Some(Purge::Debuffs) }));
        assert_eq!(registry.rule("c17_victory_blessing:4:1"), Some(Rule { class: Class::Basic, side: Some(Purge::Buffs) }));
        assert_eq!(registry.declared("c17_victory_blessing:{caster}"), registry.rule("c17_victory_blessing:4:1"));
        // No runtime_id: the executor's default key, per source or not.
        assert_eq!(registry.rule("b04w_trigger").map(|r| r.class), Some(Class::None));
        assert_eq!(registry.rule("b04w_aura:9:1"), registry.rule("b04w_aura"));
        assert_eq!(registry.rule("a07_strongest_r2:4:1").map(|r| r.class), Some(Class::None));
        // Statuses sharing a key never disagree on buff against debuff.
        let doc: Value = serde_json::from_str(omoba_template_ids::V3_CATALOG_JSON).unwrap();
        let mut seen: BTreeMap<String, Purge> = BTreeMap::new();
        for list in ["abilities", "items"] {
            for entry in doc[list].as_array().into_iter().flatten() {
                for status in entry["v3"]["statuses"].as_array().into_iter().flatten() {
                    assert!(status["dispel"].as_str().and_then(Class::parse).is_some(), "{} has a dispel class", status["id"]);
                    let key = status["runtime_id"].as_str().or_else(|| status["id"].as_str()).unwrap().to_owned();
                    let Some(side) = side_of(&status["visibility"]) else { continue };
                    assert_eq!(*seen.entry(key.clone()).or_insert(side), side, "{key}: statuses disagree on buff and debuff");
                }
            }
        }
    }

    #[test]
    fn dispels_take_their_class_and_side() {
        let doc = json!({"abilities": [{"v3": {"statuses": [
            {"id": "x_slow", "runtime_id": "x_slow:{caster}", "dispel": "basic", "visibility": "debuff"},
            {"id": "x_stun", "runtime_id": "x_stun", "dispel": "strong", "visibility": "debuff"},
            {"id": "x_mark", "runtime_id": "x_mark:{caster}", "dispel": "none", "visibility": "debuff"},
            {"id": "x_haste", "runtime_id": "x_haste", "dispel": "basic", "visibility": "buff"},
            {"id": "x_shield", "runtime_id": "x_shield:{caster}", "dispel": "strong", "visibility": "buff"},
            {"id": "x_form", "runtime_id": "x_form", "dispel": "death_only", "visibility": "buff"},
            {"id": "x_zone", "runtime_id": "x_zone:{caster}", "dispel": "basic", "visibility": "hidden"}]}}]});
        let registry = Registry::from_catalog(&doc);
        let mut world = World::new();
        world.register::<Faction>();
        world.insert(BuffStore::new());
        let hero = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let enemy = world.create_entity().with(Faction::new(FactionType::Enemy, 2)).build();
        let ally = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let tag = |e: Entity| format!("{}:{}", e.id(), e.gen().id());
        let put = |world: &World| {
            let mut buffs = world.write_resource::<BuffStore>();
            for key in [format!("x_slow:{}", tag(enemy)), "x_stun".into(), format!("x_mark:{}", tag(enemy)), "x_haste".into(),
                        format!("x_shield:{}", tag(ally)), "x_form".into(), format!("x_slow:{}", tag(ally)),
                        format!("x_zone:{}", tag(enemy)), format!("x_zone:{}", tag(ally))] {
                buffs.add(hero, &key, Fixed64::from_i32(5), json!({}));
            }
        };
        let left = |world: &World| { let mut v: Vec<String> = world.read_resource::<BuffStore>().iter_for(hero).map(|(k, _)| k.to_owned()).collect(); v.sort(); v };
        put(&world);
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Debuffs), 3, "basic: both slows (a debuff whoever applied it) and the enemy's hidden zone");
        assert!(left(&world).contains(&format!("x_zone:{}", tag(ally))), "an ally's hidden status is a buff");
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Strong, Purge::Debuffs), 1, "strong: the stun too; the none-class mark stays");
        assert!(left(&world).contains(&format!("x_mark:{}", tag(enemy))));
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Buffs), 2, "an enemy purge: the haste and the ally's hidden zone");
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Strong, Purge::Buffs), 1, "strong purge: the shield; death_only stays");
        assert_eq!(left(&world), vec!["x_form".to_owned(), format!("x_mark:{}", tag(enemy))]);
        // A hidden status whose applier is gone stays; a debuff still goes.
        for key in [format!("x_zone:{}", tag(enemy)), format!("x_slow:{}", tag(enemy))] {
            world.write_resource::<BuffStore>().add(hero, &key, Fixed64::from_i32(5), json!({}));
        }
        world.delete_entity(enemy).unwrap();
        world.maintain();
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Buffs), 0);
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Debuffs), 1);
        assert!(left(&world).contains(&format!("x_zone:{}", tag(enemy))));
    }
    /// conformance：stun 是共用 key（registry 取最嚴格的 strong），但各狀態的類別不同。帶控制效果的
    /// 狀態另記 `__ctl:<id>`，驅散依各自類別：basic 的暈眩被 basic 驅散移除；還有 strong 的暈眩時
    /// 共用 stun 留著；沒有來源列的舊 handler 暈眩照舊規則。
    #[test]
    fn a_shared_control_follows_the_classes_of_the_statuses_holding_it() {
        let doc = json!({"abilities": [{"v3": {"statuses": [
            {"id": "a_freeze", "runtime_id": "stun", "dispel": "basic", "visibility": "debuff"},
            {"id": "b_quake", "runtime_id": "stun", "dispel": "strong", "visibility": "debuff"}]}}]});
        let registry = Registry::from_catalog(&doc);
        assert_eq!(registry.rule("stun").map(|r| r.class), Some(Class::Strong));
        let mut world = World::new();
        world.register::<Faction>();
        world.insert(BuffStore::new());
        let hero = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let hold = |world: &World, status: &str| {
            let mut buffs = world.write_resource::<BuffStore>();
            buffs.add(hero, "stun", Fixed64::from_i32(3), json!({}));
            buffs.add(hero, &format!("{CONTROL_SOURCE_PREFIX}{status}"), Fixed64::from_i32(3), control_source_payload(&["stun"]));
        };
        let stunned = |world: &World| world.read_resource::<BuffStore>().has(hero, "stun");
        hold(&world, "a_freeze");
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Debuffs), 2, "the basic freeze and the shared stun");
        assert!(!stunned(&world));
        hold(&world, "a_freeze");
        hold(&world, "b_quake");
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Debuffs), 1, "only the freeze's row");
        assert!(stunned(&world), "the strong quake still stuns");
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Strong, Purge::Debuffs), 2);
        assert!(!stunned(&world));
        // A handler's stun with no source row: a basic dispel leaves it (the strictest class).
        world.write_resource::<BuffStore>().add(hero, "stun", Fixed64::from_i32(3), json!({}));
        assert_eq!(dispel_with(&registry, &world, hero, Strength::Basic, Purge::Debuffs), 0);
        assert!(stunned(&world));
    }
    /// v3 `remove_status` by filters: `dispel = "all"` with `team_of_source = "enemy"` removes every
    /// status an enemy applied whatever its class, plus controls; `except` keeps named ones; `limit`
    /// counts statuses; `states` takes only those controls.
    #[test]
    fn purges_follow_mode_side_source_team_except_states_and_limit() {
        let doc = json!({"abilities": [{"v3": {"statuses": [
            {"id": "y_slow", "dispel": "basic", "visibility": "debuff"},
            {"id": "y_mark", "dispel": "none", "visibility": "debuff"},
            {"id": "y_haste", "dispel": "basic", "visibility": "buff"},
            {"id": "y_freeze", "runtime_id": "stun", "dispel": "strong", "visibility": "debuff"}]}}]});
        let registry = Registry::from_catalog(&doc);
        let mut world = World::new();
        world.register::<Faction>();
        world.insert(BuffStore::new());
        let hero = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let enemy = world.create_entity().with(Faction::new(FactionType::Enemy, 2)).build();
        let tag = |e: Entity| format!("{}:{}", e.id(), e.gen().id());
        let put = |world: &World| {
            let mut buffs = world.write_resource::<BuffStore>();
            buffs.add(hero, &format!("y_slow:{}", tag(enemy)), Fixed64::from_i32(5), json!({}));
            buffs.add(hero, &format!("y_mark:{}", tag(enemy)), Fixed64::from_i32(5), json!({}));
            buffs.add(hero, &format!("y_haste:{}", tag(hero)), Fixed64::from_i32(5), json!({}));
            buffs.add(hero, "stun", Fixed64::from_i32(5), json!({}));
            buffs.add(hero, &format!("{CONTROL_SOURCE_PREFIX}y_freeze"), Fixed64::from_i32(5), control_source_payload(&["stun"]));
        };
        let left = |world: &World| { let mut v: Vec<String> = world.read_resource::<BuffStore>().iter_for(hero).map(|(k, _)| k.to_owned()).collect(); v.sort(); v };
        let spec = |mode, team, except: &[&str], states: &[&str], limit| PurgeSpec { mode, side: None, team,
            except: except.iter().map(|s| s.to_string()).collect(), states: states.iter().map(|s| s.to_string()).collect(), limit };
        put(&world);
        assert_eq!(purge_with(&registry, &world, hero, &spec(PurgeMode::All, SourceTeam::Enemy, &[], &[], None)), 4,
            "enemy slow, the undispellable mark, the stun and its row; the own haste stays");
        assert_eq!(left(&world), vec![format!("y_haste:{}", tag(hero))]);
        world.write_resource::<BuffStore>().remove(hero, &format!("y_haste:{}", tag(hero)));
        put(&world);
        assert_eq!(purge_with(&registry, &world, hero, &spec(PurgeMode::All, SourceTeam::Enemy, &["y_mark"], &["stun"], None)), 2,
            "states: only the stun and its row");
        assert!(left(&world).contains(&format!("y_slow:{}", tag(enemy))));
        assert_eq!(purge_with(&registry, &world, hero, &spec(PurgeMode::All, SourceTeam::Any, &["y_mark"], &[], Some(1))), 1,
            "limit 1, the mark excepted");
        let payload = spec(PurgeMode::Basic, SourceTeam::Ally, &["a"], &["root"], Some(2)).payload();
        assert_eq!(PurgeSpec::from_payload(PURGE_BUFF, Fixed64::ONE, &payload), Some(spec(PurgeMode::Basic, SourceTeam::Ally, &["a"], &["root"], Some(2))));
    }
}
