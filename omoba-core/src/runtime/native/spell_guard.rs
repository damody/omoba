//! Spell guard (E163, 真田幸村 心眼 B06R): a unit that carries a guard row and is
//! the target of an enemy unit-targeted ability gets a timed magic-resistance
//! buff, then cannot trip it again for a cooldown counted from when the guard
//! ends. A respawn starts over (buffs do not survive death; the passive's
//! on_learn re-adds the row).
//!
//! Tripped in the cast transaction (`dispatch` SkillCast, before the caster's
//! own hooks), so a rejected cast trips nothing. The rows are ordinary buffs,
//! the same on authority and the caster team's replica.
use super::ability_runtime::BuffStore;
use crate::runtime::comp::Faction;
use omb_script_abi::stat_keys::StatKey;
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::Entity;

/// Payload keys of the guard row (raw Fixed64): magic resistance points, guard seconds, cooldown seconds.
pub const MAGIC_RESIST: &str = "spell_guard_magic_resist";
pub const DURATION: &str = "spell_guard_duration";
pub const COOLDOWN: &str = "spell_guard_cooldown";
/// The buff the guard grants, and the marker that blocks tripping it again.
pub const ACTIVE: &str = "__spell_guard_active";
pub const BLOCKED: &str = "__spell_guard_blocked";

pub fn payload(magic_resist: Fixed64, duration: Fixed64, cooldown: Fixed64) -> Value {
    json!({ (MAGIC_RESIST): magic_resist.raw(), (DURATION): duration.raw(), (COOLDOWN): cooldown.raw() })
}

/// The buffs to add when `caster` targets `target` with an ability:
/// (buff id, duration, payload). None when nothing trips.
pub fn trip(buffs: &BuffStore, caster: Option<&Faction>, target_faction: Option<&Faction>, target: Entity)
    -> Option<Vec<(String, Fixed64, Value)>> {
    if !caster.zip(target_faction).is_some_and(|(c, t)| c.is_hostile_to(t)) || buffs.has(target, BLOCKED) { return None; }
    let read = |payload: &Value, key: &str| payload.get(key).and_then(Value::as_i64).map(Fixed64::from_raw);
    let (resist, duration, cooldown) = buffs.iter_for(target)
        .filter_map(|(_, entry)| Some((read(&entry.payload, MAGIC_RESIST)?, read(&entry.payload, DURATION)?, read(&entry.payload, COOLDOWN)?)))
        .max_by_key(|(resist, duration, cooldown)| (*resist, *duration, std::cmp::Reverse(*cooldown)))?;
    let bounded = |v: Fixed64, max: i32| v > Fixed64::ZERO && v <= Fixed64::from_i32(max);
    if !bounded(resist, 100) || !bounded(duration, 60) || cooldown < Fixed64::ZERO || cooldown > Fixed64::from_i32(600) { return None; }
    Some(vec![
        (ACTIVE.to_owned(), duration, json!({ (StatKey::MagicalResistanceBonus.as_str()): resist.raw() })),
        (BLOCKED.to_owned(), duration + cooldown, json!({})),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::comp::FactionType;
    use specs::{Builder, World, WorldExt};

    #[test]
    fn an_enemy_cast_trips_the_guard_once_until_the_cooldown_after_it_ends() {
        let mut world = World::new();
        let target = world.create_entity().build();
        let (enemy, ally, me) = (Faction::new(FactionType::Enemy, 2), Faction::new(FactionType::Player, 1), Faction::new(FactionType::Player, 1));
        let mut buffs = BuffStore::new();
        assert!(trip(&buffs, Some(&enemy), Some(&me), target).is_none(), "no guard row");
        buffs.add(target, "b06_mind_eye", Fixed64::from_raw(i64::MAX),
            payload(Fixed64::from_i32(75), Fixed64::from_i32(3), Fixed64::from_i32(23)));
        assert!(trip(&buffs, Some(&ally), Some(&me), target).is_none(), "an ally's cast does not trip it");
        let rows = trip(&buffs, Some(&enemy), Some(&me), target).unwrap();
        assert_eq!(rows[0], (ACTIVE.to_owned(), Fixed64::from_i32(3), json!({"magical_resistance_bonus": 75 * 1024})));
        assert_eq!((rows[1].0.as_str(), rows[1].1), (BLOCKED, Fixed64::from_i32(26)));
        for (id, duration, payload) in rows { buffs.add(target, &id, duration, payload); }
        assert!(trip(&buffs, Some(&enemy), Some(&me), target).is_none(), "blocked while active and cooling down");
        buffs.tick(Fixed64::from_i32(26));
        assert!(trip(&buffs, Some(&enemy), Some(&me), target).is_some(), "ready again");
    }
}
