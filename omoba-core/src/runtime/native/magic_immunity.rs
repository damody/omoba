//! Magic immunity and strong dispel (E179), one rule for hero skills and items, as the source
//! (history_nobu, Dota rules) has it:
//! * the status is the buff `BUFF` on the unit; its payload zeroes magical damage through the
//!   one damage choke point (damage_immunity::settle, `absolute_no_damage_magical`);
//! * while it lasts, hostile debuffs do not land: controls (stun/root/silence), ability slows,
//!   damage-over-time, mana drain, the legacy slow and healing reduction (`blocks`, called by
//!   BuffStore::add for every buff);
//! * an enemy cannot target the unit with a unit-targeted ability (dispatch rejects the cast)
//!   or item, unless the ability pierces magic immunity (content `pierces_magic_immunity`);
//! * a debuff that pierces carries `PIERCE_KEY` in its payload (`pierce_payload`), so a hero
//!   script marks its piercing stun as `add_stat_buff(target, "stun", seconds, pierce_payload())`;
//! * physical and pure damage and basic attacks are unaffected;
//! * granting it first strong-dispels the unit: every debuff an enemy applied is removed.
//! The status is an ordinary timed buff in the catalog (disclosed with the owner's buffs), and
//! nothing but expiry or death removes it, so the source's "re-apply if removed" never triggers.
use super::ability_runtime::BuffStore;
use super::comp::Faction;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

pub const BUFF: &str = "magic_immune";
pub const PIERCE_KEY: &str = "__pierces_magic_immunity";

/// The status payload: magical damage absorbed in full by the damage settlement.
pub fn status_payload() -> Value { json!({ (StatKey::AbsoluteNoDamageMagical.as_str()): 1024 }) }

/// A debuff payload that lands through magic immunity (extend it with the debuff's own keys).
pub fn pierce_payload() -> Value { json!({ (PIERCE_KEY): 1 }) }

/// `pierce_payload` as the JSON text a hero script passes to `add_stat_buff`.
pub const PIERCE_PAYLOAD_JSON: &str = r#"{"__pierces_magic_immunity":1}"#;

pub fn is_magic_immune(buffs: &BuffStore, entity: Entity) -> bool {
    buffs.get(entity, BUFF).is_some_and(|entry| entry.remaining > Fixed64::ZERO)
}

fn is_control(buff_id: &str) -> bool {
    use omb_script_abi::buff_ids::BuffId;
    [BuffId::Stun.as_str(), BuffId::Root.as_str(), BuffId::Silence.as_str()].contains(&buff_id)
}

/// A hostile debuff kind magic immunity keeps out.
fn is_debuff(buff_id: &str, payload: &Value) -> bool {
    is_control(buff_id) || buff_id == "slow" || buff_id == "healing_reduction"
        || buff_id.starts_with("generic_slow:") || buff_id.starts_with("generic_dot:") || buff_id.starts_with("generic_mana:")
        || payload.get("__aggregation_family").and_then(Value::as_str) == Some("generic_ability_slow")
}

/// BuffStore::add: the debuff does not land on a magic-immune unit unless it pierces.
pub fn blocks(buffs: &BuffStore, entity: Entity, buff_id: &str, payload: &Value) -> bool {
    payload.get(PIERCE_KEY).is_none() && is_debuff(buff_id, payload) && is_magic_immune(buffs, entity)
}

/// Whether `caster` may target `target` with an enemy unit-targeted ability or item.
pub fn admits_target(buffs: &BuffStore, caster: Option<&Faction>, target_faction: Option<&Faction>, target: Entity, pierces: bool) -> bool {
    pierces || !caster.zip(target_faction).is_some_and(|(c, t)| c.is_hostile_to(t)) || !is_magic_immune(buffs, target)
}

/// The ability's authored `pierces_magic_immunity` (compiled content); unknown abilities do not pierce.
pub fn ability_pierces(skill_id: &str) -> bool {
    omoba_template_ids::ability_by_name(skill_id).and_then(omoba_template_ids::ability_const)
        .is_some_and(|ability| ability.pierces_magic_immunity)
        // a v3 document says it in its ability flags (B29T, flags = { "pierce_magic_immune" })
        || super::rules::status::has_ability_flag(skill_id, "pierce_magic_immune")
}

/// Strong dispel of what enemies applied (dispel.rs, E214): statuses of class `basic` or
/// `strong`; keys the content does not list keep the older rules (controls, the legacy slow and
/// healing reduction, hostile source-tagged buffs). Buffs from allies and the unit itself stay.
pub fn strong_dispel(world: &World, entity: Entity) {
    super::dispel::dispel(world, entity, super::dispel::Strength::Strong, super::dispel::Purge::Debuffs);
}

/// Grant magic immunity for `duration` (≤ 60 s): strong dispel, then the status.
pub fn grant(world: &World, entity: Entity, duration: Fixed64) -> bool {
    if duration <= Fixed64::ZERO || duration > Fixed64::from_i32(60) || !world.entities().is_alive(entity) { return false; }
    strong_dispel(world, entity);
    world.write_resource::<BuffStore>().add(entity, BUFF, duration, status_payload());
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::comp::FactionType;
    use specs::Builder;

    #[test]
    fn immunity_keeps_hostile_debuffs_out_unless_they_pierce_and_dispels_enemy_ones() {
        let mut world = World::new();
        world.register::<Faction>();
        world.insert(BuffStore::new());
        let hero = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let enemy = world.create_entity().with(Faction::new(FactionType::Enemy, 2)).build();
        let ally = world.create_entity().with(Faction::new(FactionType::Player, 1)).build();
        let tag = |e: Entity| format!("generic_slow:skill:{}:{}", e.id(), e.gen().id());
        {
            let mut buffs = world.write_resource::<BuffStore>();
            buffs.add(hero, "stun", Fixed64::from_i32(2), json!({}));
            buffs.add(hero, &tag(enemy), Fixed64::from_i32(2), json!({"move_speed_bonus": -100, "__aggregation_family": "generic_ability_slow"}));
            buffs.add(hero, &format!("ally_haste:{}:{}", ally.id(), ally.gen().id()), Fixed64::from_i32(2), json!({"movespeed_bonus_percentage": 100}));
            buffs.add(hero, "modifier_fire_attack", Fixed64::from_i32(8), json!({"basedamageoutgoing_percentage": 205}));
            buffs.add(hero, &format!("b29_breaker_armor:{}:{}", enemy.id(), enemy.gen().id()), Fixed64::ONE, json!({"physical_armor_bonus": -5120}));
            buffs.add(hero, "b08_living_soul_rank:1", Fixed64::ONE, json!({}));
        }
        assert_eq!(serde_json::from_str::<Value>(PIERCE_PAYLOAD_JSON).unwrap(), pierce_payload());
        assert!(grant(&world, hero, Fixed64::from_i32(5)));
        let mut buffs = world.write_resource::<BuffStore>();
        assert!(is_magic_immune(&buffs, hero));
        assert!(!buffs.has(hero, "stun") && !buffs.has(hero, &tag(enemy)), "enemy debuffs are dispelled");
        assert!(!buffs.iter_for(hero).any(|(id, _)| id.starts_with("b29_breaker_armor:")), "an id naming an enemy applier is dispelled too");
        assert!(buffs.has(hero, "b08_living_soul_rank:1"), "a rank suffix is not an applier");
        assert!(buffs.has(hero, "modifier_fire_attack") && buffs.iter_for(hero).any(|(id, _)| id.starts_with("ally_haste")), "ally buffs stay");
        buffs.add(hero, "stun", Fixed64::from_i32(2), json!({}));
        buffs.add(hero, "healing_reduction", Fixed64::from_i32(2), json!({"heal_received_multiplier": 512}));
        assert!(!buffs.is_stunned(hero) && !buffs.has(hero, "healing_reduction"), "debuffs do not land");
        buffs.add(hero, "stun", Fixed64::from_i32(2), pierce_payload());
        assert!(buffs.is_stunned(hero), "a piercing control lands");
        assert!(!admits_target(&buffs, Some(&Faction::new(FactionType::Enemy, 2)), Some(&Faction::new(FactionType::Player, 1)), hero, false));
        assert!(admits_target(&buffs, Some(&Faction::new(FactionType::Enemy, 2)), Some(&Faction::new(FactionType::Player, 1)), hero, true));
        assert!(admits_target(&buffs, Some(&Faction::new(FactionType::Player, 1)), Some(&Faction::new(FactionType::Player, 1)), hero, false), "allies may target");
        let (_, magical, _, _) = super::super::damage_immunity::settle(&buffs, hero, Fixed64::from_i32(10), Fixed64::from_i32(100), Fixed64::from_i32(5));
        assert_eq!(magical, Fixed64::ZERO, "magical damage is zero");
    }
}
