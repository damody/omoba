//! v3 `create_unit { overrides = { hp, attack_damage, attack_damage_max, move_speed } }`: a
//! one-shot envelope (never stored) on a summon the cast spawns, settled right after the spawn
//! in the same batch. The template's values are replaced; later modifiers apply on top as usual.
//! The engine attack deals one amount, so a `attack_damage`..`attack_damage_max` range (the
//! original rolls each hit uniformly in it) becomes its mean; `attack_damage` alone sets both.
use omoba_sim::Fixed64;
use serde_json::{json, Map, Value};
use specs::{Entity, World, WorldExt};
use super::comp::{CProperty, SummonedUnit, TAttack, Tick, Unit, Vf32};

pub const BUFF: &str = "__unit_overrides_v1";
const KEYS: [&str; 4] = ["hp", "attack_damage", "attack_damage_max", "move_speed"];
const LIMIT: i64 = 1_000_000 * 1024;

/// The envelope payload: raw Fixed64 per given key (absent = the template's value).
pub fn payload(rows: &[(&str, Fixed64)]) -> Option<Value> {
    let mut map = Map::new();
    for (key, value) in rows {
        if !KEYS.contains(key) || map.contains_key(*key) { return None; }
        map.insert((*key).to_owned(), json!(value.raw()));
    }
    (!map.is_empty()).then_some(Value::Object(map))
}

fn read(row: &Map<String, Value>, key: &str) -> Result<Option<Fixed64>, ()> {
    match row.get(key) {
        None => Ok(None),
        Some(value) => value.as_i64().filter(|raw| (0..=LIMIT).contains(raw)).map(|raw| Some(Fixed64::from_raw(raw))).ok_or(()),
    }
}

/// Settles one envelope on `target`, a summon spawned this tick; false when it is malformed or
/// `target` is anything else.
pub fn apply(world: &mut World, target: Entity, payload: &Value) -> bool {
    let Some(row) = payload.as_object().filter(|row| !row.is_empty() && row.keys().all(|key| KEYS.contains(&key.as_str()))) else { return false };
    let (Ok(hp), Ok(low), Ok(high), Ok(speed)) = (read(row, "hp"), read(row, "attack_damage"), read(row, "attack_damage_max"), read(row, "move_speed"))
        else { return false };
    if hp.is_some_and(|hp| hp <= Fixed64::ZERO) { return false; }
    let tick = world.read_resource::<Tick>().0;
    if !world.entities().is_alive(target)
        || world.read_storage::<SummonedUnit>().get(target).is_none_or(|summon| summon.spawn_tick != tick) {
        return false;
    }
    let damage = match (low, high) {
        (Some(low), Some(high)) if high >= low => Some(Fixed64::from_raw((low.raw() + high.raw()) / 2)),
        (Some(_), Some(_)) => return false,
        (Some(one), None) | (None, Some(one)) => Some(one),
        (None, None) => None,
    };
    if let Some(props) = world.write_storage::<CProperty>().get_mut(target) {
        if let Some(hp) = hp { props.hp = hp; props.mhp = hp; }
        if let Some(speed) = speed { props.msd = speed; }
    }
    if let (Some(damage), Some(attack)) = (damage, world.write_storage::<TAttack>().get_mut(target)) {
        attack.atk_physic = Vf32::new(damage);
    }
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        if let Some(hp) = hp { unit.max_hp = (hp.raw() / 1024) as i32; unit.current_hp = unit.max_hp; }
        if let Some(damage) = damage { unit.base_damage = (damage.raw() / 1024) as i32; }
        if let Some(speed) = speed { unit.move_speed = speed; }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn summon(world: &mut World, tick: u64) -> Entity {
        let mut summoned = SummonedUnit::new_fixed(world.create_entity().build(), None);
        summoned.spawn_tick = tick;
        world.create_entity().with(summoned)
            .with(CProperty { hp: Fixed64::from_i32(100), mhp: Fixed64::from_i32(100), msd: Fixed64::from_i32(300),
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(TAttack::new(Fixed64::from_i32(10), Fixed64::ONE, Fixed64::from_i32(150), Fixed64::from_i32(900)))
            .build()
    }

    #[test]
    fn overrides_replace_the_template_values_of_a_summon_spawned_this_tick() {
        let mut world = World::new();
        world.register::<CProperty>();world.register::<TAttack>();world.register::<SummonedUnit>();world.register::<Unit>();
        world.insert(Tick(7));
        let fresh = summon(&mut world, 7);
        let s = Fixed64::from_i32;
        assert!(apply(&mut world, fresh, &payload(&[("hp", s(450)), ("attack_damage", s(20)), ("attack_damage_max", s(30)),
            ("move_speed", s(522))]).unwrap()));
        let props = world.read_storage::<CProperty>().get(fresh).unwrap().clone();
        assert_eq!((props.hp, props.mhp, props.msd), (s(450), s(450), s(522)));
        assert_eq!(world.read_storage::<TAttack>().get(fresh).unwrap().atk_physic.bv, s(25), "a damage range is its mean");
        // One damage key sets the amount; nothing else changes.
        assert!(apply(&mut world, fresh, &payload(&[("attack_damage", s(40))]).unwrap()));
        assert_eq!(world.read_storage::<TAttack>().get(fresh).unwrap().atk_physic.bv, s(40));
        assert_eq!(world.read_storage::<CProperty>().get(fresh).unwrap().msd, s(522));
        // Closed: unknown keys, a reversed range, zero hp, an older summon, a non-summon.
        assert!(payload(&[("armor", s(1))]).is_none() && payload(&[]).is_none());
        assert!(!apply(&mut world, fresh, &json!({ "armor": 1024 })));
        assert!(!apply(&mut world, fresh, &payload(&[("attack_damage", s(30)), ("attack_damage_max", s(20))]).unwrap()));
        assert!(!apply(&mut world, fresh, &payload(&[("hp", Fixed64::ZERO)]).unwrap()));
        let old = summon(&mut world, 3);
        assert!(!apply(&mut world, old, &payload(&[("hp", s(1))]).unwrap()));
        let other = world.create_entity().build();
        assert!(!apply(&mut world, other, &payload(&[("hp", s(1))]).unwrap()));
    }
}
