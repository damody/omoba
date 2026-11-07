//! Damage tally for headless statistics (E352): every settled packet's credited source, target and
//! the health it removed. Attacks and abilities settle in the rules layer (rules/damage.rs
//! `after_mitigation`) and items and script direct damage in `apply_checked_direct_damage`; neither
//! publishes a DirectCombat fact, so a report counting those facts saw no hero damage at all.
//! A harness inserts [`DamageTally`] and drains it; without it nothing is recorded.
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};

/// (credited source, target, health removed), in settlement order since the last drain.
#[derive(Clone, Debug, Default)]
pub struct DamageTally(pub Vec<(Entity, Entity, Fixed64)>);

pub(crate) fn record(world: &World, source: Option<Entity>, target: Entity, removed: Fixed64) {
    let Some(source) = source else { return };
    if removed <= Fixed64::ZERO { return; }
    if let Some(mut tally) = world.try_fetch_mut::<DamageTally>() { tally.0.push((source, target, removed)); }
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn records_only_with_the_resource_and_only_credited_positive_damage() {
        let mut world = World::new();
        let a = world.create_entity().build();
        let b = world.create_entity().build();
        record(&world, Some(a), b, Fixed64::from_i32(5));
        world.insert(DamageTally::default());
        record(&world, Some(a), b, Fixed64::from_i32(5));
        record(&world, None, b, Fixed64::from_i32(5));
        record(&world, Some(a), b, Fixed64::ZERO);
        assert_eq!(world.read_resource::<DamageTally>().0, vec![(a, b, Fixed64::from_i32(5))]);
    }
}
