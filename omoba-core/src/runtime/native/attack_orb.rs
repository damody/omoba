//! Toggle orb (G-2 `toggle_mana_per_attack`). A toggle ability puts a marker
//! buff `attack_orb:<ability>` with payload `{KEY: {"ability": id}}` on its
//! caster. While present, each ordinary attack launch spends the rank's mana
//! cost (or switches the toggle off when mana is short) and the launched
//! projectile carries the orb; a hit that is not evaded adds the rank's bonus
//! damage and slows. Values always come from the caster's current rank.
use omoba_sim::Fixed64;
use serde::Serialize;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::ability_runtime::BuffStore;
use super::comp::*;

pub const KEY: &str = "__attack_orb_v1";

pub fn buff_id(ability: &str) -> String { format!("attack_orb:{ability}") }

pub fn payload(ability: &str) -> Value { json!({ (KEY): { "ability": ability } }) }

/// Strict marker: exactly `{KEY: {"ability": <id>}}` under the matching buff id.
pub fn orb_ability<'a>(buff: &str, payload: &'a Value) -> Option<&'a str> {
    if payload.as_object()?.len() != 1 { return None; }
    let row = payload.get(KEY)?.as_object()?;
    let ability = row.get("ability")?.as_str()?;
    (row.len() == 1 && buff == buff_id(ability)).then_some(ability)
}

/// Rank-resolved orb carried by one launched attack. Authority-private.
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct OrbHit {
    pub ability: String,
    pub amount_raw: i64,
    pub damage_kind: u8,
    pub move_slow_raw: Option<i64>,
    pub attack_slow_raw: Option<i64>,
    pub slow_duration_raw: i64,
}

#[derive(Clone, Debug, Default)]
pub struct OrbAttacks(pub BTreeMap<Entity, OrbHit>);

fn values(world: &World, source: Entity, ability: &str) -> Option<omoba_template_ids::AttackOrbValues> {
    let rank = world.read_storage::<Hero>().get(source)?.get_ability_level(ability);
    omoba_template_ids::ability_by_name(ability).and_then(omoba_template_ids::active_ability_const)?
        .attack_orb_at(u8::try_from(rank).ok().filter(|rank| *rank > 0)?)
}

/// Toggle-on admission: the marker targets its own caster, for a learned orb ability.
pub fn admitted(world: &World, target: Entity, buff: &str, payload: &Value, caster: Option<Entity>) -> bool {
    let Some(ability) = orb_ability(buff, payload) else { return false; };
    caster.is_none_or(|caster| caster == target) && values(world, target, ability).is_some()
}

/// Serial attack launch hook. Charges the first active orb (orbs never stack;
/// ordered by buff id) and records it on `projectile`, or switches it off.
pub fn on_launch(world: &mut World, source: Entity, projectile: Entity) {
    let orb = world.read_resource::<BuffStore>().iter_for(source)
        .filter_map(|(id, entry)| orb_ability(id, &entry.payload).map(|ability| (id.to_owned(), ability.to_owned())))
        .min();
    let Some((buff, ability)) = orb else { return; };
    let Some(values) = values(world, source, &ability) else {
        world.write_resource::<BuffStore>().remove(source, &buff);
        return;
    };
    let paid = {
        let mut heroes = world.write_storage::<Hero>();
        match heroes.get_mut(source).and_then(|hero| hero.mana_pool.as_mut()) {
            // Mana-disabled rulesets have no pool: the orb is free there.
            None => true,
            Some(pool) => pool.current() >= values.mana_per_attack && pool.spend(values.mana_per_attack).is_ok(),
        }
    };
    if !paid {
        // Auto-off: this attack is an ordinary one.
        world.write_resource::<BuffStore>().remove(source, &buff);
        return;
    }
    if world.try_fetch::<OrbAttacks>().is_none() { world.insert(OrbAttacks::default()); }
    world.write_resource::<OrbAttacks>().0.insert(projectile, OrbHit {
        ability, amount_raw: values.amount.raw(), damage_kind: values.damage_kind,
        move_slow_raw: values.move_slow.map(Fixed64::raw), attack_slow_raw: values.attack_slow.map(Fixed64::raw),
        slow_duration_raw: values.slow_duration.raw(),
    });
}

/// On-hit payload for a non-evaded empowered hit, attributed to `source`.
pub fn hit_outcomes(hit: &OrbHit, source: Entity, target: Entity, pos: omoba_sim::Vec2) -> Vec<Outcome> {
    let amount = Fixed64::from_raw(hit.amount_raw);
    let zero = Fixed64::ZERO;
    let (phys, magi, real, profile) = match hit.damage_kind {
        0 => (amount, zero, zero, omb_script_abi::types::DamageProfile::NORMAL),
        1 => (zero, amount, zero, omb_script_abi::types::DamageProfile::ENERGY),
        _ => (zero, zero, amount, omb_script_abi::types::DamageProfile::TRUE),
    };
    let mut outcomes = Vec::new();
    if amount > zero {
        outcomes.push(Outcome::RawDamage { pos, phys, magi, real, source, target, damage_profile: profile.bits(), origin: None });
    }
    let duration = Fixed64::from_raw(hit.slow_duration_raw);
    let caster = omb_script_abi::types::EntityHandle { id: source.id(), gen: source.gen().id() as u32 };
    if let Some(reduction) = hit.move_slow_raw.filter(|_| duration > zero) {
        outcomes.push(Outcome::AddBuff { target, buff_id: omb_script_abi::buff_ids::generic_slow_buff_id(&hit.ability, caster),
            duration, payload: json!({"move_speed_bonus": -reduction, "__aggregation_family": "generic_ability_slow"}) });
    }
    if let Some(reduction) = hit.attack_slow_raw.filter(|_| duration > zero) {
        outcomes.push(Outcome::AddBuff { target, buff_id: format!("orb_attack_slow:{}:{}:{}", hit.ability, caster.id, caster.gen),
            duration, payload: json!({"attack_speed_multiplier": 1024 - reduction}) });
    }
    outcomes
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;
    #[test]
    fn marker_is_strict_and_bound_to_its_buff_id() {
        assert_eq!(orb_ability("attack_orb:venom", &payload("venom")), Some("venom"));
        assert_eq!(orb_ability("attack_orb:other", &payload("venom")), None);
        let mut extra = payload("venom"); extra[KEY]["mana"] = json!(10);
        assert_eq!(orb_ability("attack_orb:venom", &extra), None);
        let mut sibling = payload("venom"); sibling["move_speed_bonus"] = json!(1);
        assert_eq!(orb_ability("attack_orb:venom", &sibling), None);
    }
    #[test]
    fn hit_outcomes_carry_damage_kind_and_both_slows() {
        let mut world = World::new();
        let (source, target) = (world.create_entity().build(), world.create_entity().build());
        let hit = OrbHit { ability: "venom".into(), amount_raw: 30 * 1024, damage_kind: 1, move_slow_raw: Some(92),
            attack_slow_raw: Some(92), slow_duration_raw: 4 * 1024 };
        let rows = hit_outcomes(&hit, source, target, omoba_sim::Vec2::ZERO);
        assert!(matches!(&rows[0], Outcome::RawDamage { magi, phys, .. } if magi.raw() == 30 * 1024 && *phys == Fixed64::ZERO));
        assert!(matches!(&rows[1], Outcome::AddBuff { payload, .. } if payload["move_speed_bonus"] == -92));
        assert!(matches!(&rows[2], Outcome::AddBuff { payload, .. } if payload["attack_speed_multiplier"] == 1024 - 92));
        let none = OrbHit { amount_raw: 0, move_slow_raw: None, attack_slow_raw: None, ..hit };
        assert!(hit_outcomes(&none, source, target, omoba_sim::Vec2::ZERO).is_empty());
    }
}
