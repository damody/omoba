//! v3 `set_base_max_hp { to, value }` (E563, 原作 SetBaseMaxHealth) on a non-hero unit: a one-shot
//! envelope (never stored). Its base max HP becomes `value`; what modifiers added on top of the
//! base stays on top (max HP = new base + (old max - old base)). Current HP is kept, capped at the
//! new maximum (原作 keeps health; a script sets it with set_health). A hero's max HP comes from
//! its attributes and items, so a hero is left alone. The new vitals reach the team replicas at the
//! end of the tick (E551).
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{CProperty, Hero, Unit};

pub const BUFF: &str = "__base_max_hp_v1";
const LIMIT: i64 = 1_000_000 * 1024;

pub fn payload(value: Fixed64) -> Value { json!({ "value": value.raw() }) }

fn value_of(payload: &Value) -> Option<Fixed64> {
    let row = payload.as_object().filter(|row| row.len() == 1)?;
    row.get("value")?.as_i64().filter(|raw| *raw > 0 && *raw <= LIMIT).map(Fixed64::from_raw)
}

/// Applies the envelope to `target`. Returns whether its maximum changed.
pub fn apply(world: &mut World, target: Entity, payload: &Value) -> bool {
    let Some(value) = value_of(payload) else { return false };
    if !world.entities().is_alive(target) || world.read_storage::<Hero>().get(target).is_some() { return false; }
    let base = world.read_storage::<Unit>().get(target).map(|unit| Fixed64::from_i32(unit.max_hp));
    let mut properties = world.write_storage::<CProperty>();
    let Some(property) = properties.get_mut(target) else { return false };
    let bonus = base.map_or(Fixed64::ZERO, |base| property.mhp - base);
    property.mhp = (value + bonus).max(Fixed64::from_raw(1));
    if property.hp > property.mhp { property.hp = property.mhp; }
    drop(properties);
    if let Some(unit) = world.write_storage::<Unit>().get_mut(target) {
        unit.max_hp = (value.raw() / 1024) as i32;
        unit.current_hp = unit.current_hp.min(unit.max_hp);
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn body(world: &mut World, hp: i32, mhp: i32) -> Entity {
        world.create_entity().with(CProperty { hp: Fixed64::from_i32(hp), mhp: Fixed64::from_i32(mhp), msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build()
    }

    #[test]
    fn the_base_moves_modifiers_stay_on_top_and_health_is_capped() {
        let mut world = World::new();
        world.register::<CProperty>(); world.register::<Hero>(); world.register::<Unit>();
        // A body with a 1000 base and +200 from modifiers.
        let dog = body(&mut world, 900, 1200);
        let mut unit = Unit::new("dog".into(), "dog".into(), super::super::comp::UnitType::Summon);
        unit.max_hp = 1000; unit.current_hp = 900;
        world.write_storage::<Unit>().insert(dog, unit).unwrap();
        assert!(apply(&mut world, dog, &payload(Fixed64::from_i32(1500))));
        let p = world.read_storage::<CProperty>().get(dog).cloned().unwrap();
        assert_eq!((p.mhp, p.hp), (Fixed64::from_i32(1700), Fixed64::from_i32(900)));
        assert!(apply(&mut world, dog, &payload(Fixed64::from_i32(500))));
        let p = world.read_storage::<CProperty>().get(dog).cloned().unwrap();
        assert_eq!((p.mhp, p.hp), (Fixed64::from_i32(700), Fixed64::from_i32(700)), "health capped at the new maximum");
        // Without a Unit row the value is the maximum; a hero, zero or a malformed row: nothing.
        let plain = body(&mut world, 50, 100);
        assert!(apply(&mut world, plain, &payload(Fixed64::from_i32(300))));
        assert_eq!(world.read_storage::<CProperty>().get(plain).unwrap().mhp, Fixed64::from_i32(300));
        assert!(!apply(&mut world, plain, &payload(Fixed64::ZERO)));
        assert!(!apply(&mut world, plain, &json!({ "value": 1024, "extra": 1 })));
        world.write_storage::<Hero>().insert(plain, Hero::default()).unwrap();
        assert!(!apply(&mut world, plain, &payload(Fixed64::from_i32(400))));
    }
}
