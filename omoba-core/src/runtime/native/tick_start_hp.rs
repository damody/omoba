//! Tick-start life for script reads (E194). A team replica settles no damage
//! on a disclosed MOBA entity (`rules::damage::admit`, m2-disclosure-policy.md): its
//! life arrives as committed vitals after the step. The authority settles the
//! pre-script outcome batch first, so a handler that read life mid-tick (a target
//! killed by an attack earlier in the same tick) refused a cast its team's replica
//! accepted, and only the replica recorded the cooldown. Script dispatch on both
//! sides therefore reads every entity's life as it was before the tick's first
//! outcome batch; the batch result returns right after dispatch.
use omoba_sim::Fixed64;
use specs::{Entity, Join, World, WorldExt};

use super::comp::{CProperty, Tick};

/// Life of every entity before the first outcome batch of `tick`, in entity order.
#[derive(Default)]
pub struct TickStartHp {
    tick: Option<u64>,
    rows: Vec<(Entity, Fixed64)>,
}

/// The authority of a MOBA match and every team replica (which heals locally).
fn applies(world: &World) -> bool {
    world.try_fetch::<crate::runtime::MobaMatch>().is_some()
        || world.try_fetch::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>().is_some()
}

/// Called at the start of every outcome batch; only the tick's first one records.
pub fn capture(world: &mut World) {
    if !applies(world) { return; }
    let tick = world.try_fetch::<Tick>().map(|tick| tick.0);
    if world.try_fetch::<TickStartHp>().is_some_and(|start| start.tick == tick) { return; }
    let rows = (&world.entities(), &world.read_storage::<CProperty>()).join()
        .map(|(entity, property)| (entity, property.hp)).collect();
    world.insert(TickStartHp { tick, rows });
}

/// Entities whose life the tick's batches changed, with their batch value, after
/// restoring the tick-start value for the script phase.
pub fn enter(world: &mut World) -> Vec<(Entity, Fixed64, Fixed64)> {
    if !applies(world) { return Vec::new(); }
    let tick = world.try_fetch::<Tick>().map(|tick| tick.0);
    let Some(start) = world.try_fetch::<TickStartHp>() else { return Vec::new() };
    if start.tick != tick { return Vec::new(); }
    let entities = world.entities();
    let mut properties = world.write_storage::<CProperty>();
    let mut swapped = Vec::new();
    for (entity, hp) in &start.rows {
        if !entities.is_alive(*entity) { continue; }
        let Some(property) = properties.get_mut(*entity) else { continue };
        if property.hp != *hp {
            swapped.push((*entity, property.hp, *hp));
            property.hp = *hp;
        }
    }
    swapped
}

/// Restores the batch values. A value the script phase changed itself is kept.
pub fn leave(world: &mut World, swapped: Vec<(Entity, Fixed64, Fixed64)>) {
    let entities = world.entities();
    let mut properties = world.write_storage::<CProperty>();
    for (entity, settled, start) in swapped {
        if !entities.is_alive(entity) { continue; }
        if let Some(property) = properties.get_mut(entity).filter(|property| property.hp == start) {
            property.hp = settled;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn world() -> (World, Entity) {
        let mut world = World::new();
        world.register::<CProperty>();
        world.insert(Tick(7));
        world.insert(crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets(Default::default()));
        let entity = world.create_entity().with(CProperty { hp: Fixed64::from_i32(34), mhp: Fixed64::from_i32(500), msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
        (world, entity)
    }
    fn hp(world: &World, entity: Entity) -> Fixed64 { world.read_storage::<CProperty>().get(entity).unwrap().hp }
    fn set(world: &World, entity: Entity, value: i32) { world.write_storage::<CProperty>().get_mut(entity).unwrap().hp = Fixed64::from_i32(value); }

    #[test]
    fn script_phase_reads_life_from_before_the_ticks_first_batch() {
        let (mut world, unit) = world();
        capture(&mut world);
        set(&world, unit, 0); // killed by the pre-script batch
        capture(&mut world); // the tick's second batch does not re-record
        let settled = enter(&mut world);
        assert_eq!(hp(&world, unit), Fixed64::from_i32(34), "handlers see the life the team replica holds");
        leave(&mut world, settled);
        assert_eq!(hp(&world, unit), Fixed64::ZERO, "the settled batch result returns after dispatch");
        // A change made during the script phase itself is kept.
        let settled = enter(&mut world);
        set(&world, unit, 20);
        leave(&mut world, settled);
        assert_eq!(hp(&world, unit), Fixed64::from_i32(20));
        // The next tick records afresh.
        world.insert(Tick(8));
        capture(&mut world);
        assert!(enter(&mut world).is_empty());
    }

    #[test]
    fn worlds_without_a_moba_match_or_replica_keep_live_reads() {
        let (mut world, unit) = world();
        world.remove::<crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets>();
        capture(&mut world);
        set(&world, unit, 0);
        assert!(enter(&mut world).is_empty());
        assert_eq!(hp(&world, unit), Fixed64::ZERO);
    }
}
