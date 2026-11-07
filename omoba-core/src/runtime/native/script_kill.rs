//! v3 `kill { to, silent }` of a generated script: a one-shot envelope (never stored). `to` dies
//! at once without a damage packet, so invulnerability, shields and damage immunity do not stop
//! it. The kill is credited to the named killer through the same records a lethal hit makes
//! (moba_match hero / jungle / lane / building damage, `lethal`); death_tick then settles the
//! death like any other. `silent` removes a non-hero unit instead (no bounty, no death; a
//! summon's own end); a hero always dies normally.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use super::comp::{CProperty, Hero, Outcome};

pub const BUFF: &str = "__script_kill";

/// The envelope payload: the killer's handle (u32::MAX for none) and `silent`.
pub fn payload(killer_id: u32, killer_generation: u32, silent: bool) -> Value {
    json!({ "killer_id": killer_id, "killer_generation": killer_generation, "silent": silent })
}

fn killer_of(world: &World, payload: &Value) -> Option<Entity> {
    let id = u32::try_from(payload.get("killer_id")?.as_u64()?).ok()?;
    let generation = u32::try_from(payload.get("killer_generation")?.as_u64()?).ok()?;
    if id == u32::MAX || generation == 0 { return None; }
    let entity = world.entities().entity(id);
    (entity.gen().id() as u32 == generation && world.entities().is_alive(entity)).then_some(entity)
}

/// Settles one envelope on `target`; false when it is malformed or `target` is already dead.
pub fn apply(world: &mut World, target: Entity, payload: &Value) -> bool {
    let row = match payload.as_object() { Some(row) if row.len() == 3 => row, _ => return false };
    let Some(silent) = row.get("silent").and_then(Value::as_bool) else { return false };
    if !world.entities().is_alive(target) || world.read_storage::<CProperty>().get(target).is_none_or(|p| p.hp <= Fixed64::ZERO) {
        return false;
    }
    if silent && world.read_storage::<Hero>().get(target).is_none() {
        world.write_resource::<Vec<Outcome>>().push(Outcome::EntityRemoved { entity: target });
        return true;
    }
    let killer = killer_of(world, payload);
    if let Some(props) = world.write_storage::<CProperty>().get_mut(target) { props.hp = Fixed64::ZERO; }
    super::moba_match::record_moba_hero_damage(world, killer, target, true);
    super::moba_match::record_moba_jungle_damage(world, killer, target, true);
    super::moba_match::record_moba_lane_damage(world, killer, target, true);
    super::moba_match::record_moba_building_damage(world, killer, target, true);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn the_envelope_is_closed() {
        let mut world = World::new();
        world.register::<CProperty>();
        world.register::<Hero>();
        world.insert(Vec::<Outcome>::new());
        let target = world.create_entity().build();
        assert!(!apply(&mut world, target, &json!({ "killer_id": 1 })), "malformed");
        assert!(!apply(&mut world, target, &payload(u32::MAX, 0, true)), "no hp: not a living unit");
    }
}
