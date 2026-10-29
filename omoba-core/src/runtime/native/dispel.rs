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

/// Runtime ids to dispel rules: exact keys, then `{caster}`/`{rank}` patterns.
#[derive(Debug, Default)]
pub struct Registry { exact: HashMap<String, Rule>, patterns: Vec<(String, Rule)> }

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
        for list in ["abilities", "items"] {
            for entry in doc[list].as_array().into_iter().flatten() {
                for status in entry["v3"]["statuses"].as_array().into_iter().flatten() {
                    let Some(class) = status["dispel"].as_str().and_then(Class::parse) else { continue };
                    let side = side_of(&status["visibility"]);
                    for key in runtime_keys(status) {
                        rules.entry(key)
                            .and_modify(|rule| { rule.class = rule.class.max(class); rule.side = rule.side.or(side); })
                            .or_insert(Rule { class, side });
                    }
                }
            }
        }
        let mut registry = Registry::default();
        for (key, rule) in rules {
            if is_pattern(&key) { registry.patterns.push((key, rule)); } else { registry.exact.insert(key, rule); }
        }
        registry
    }

    /// The rule for a BuffStore key.
    pub fn rule(&self, key: &str) -> Option<Rule> {
        self.exact.get(key).copied().or_else(|| self.patterns.iter().find(|(id, _)| runtime_id_matches(id, key)).map(|(_, rule)| *rule))
    }

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
    doomed.sort();
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
}
