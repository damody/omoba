//! Idle retaliation (LoL / Dota): a hero with no move or attack order that is hit fights back at
//! its attacker while the attacker is in attack range. Jungle camps first: a camp that is
//! attacking the hero is not replaced by a lane unit's hit, because a camp is otherwise never
//! acquired automatically (combat_acquisition skips HostileNeutral for idle and attack-move).
//!
//! Authority only. The attacker is recorded where damage is applied (game_processor
//! handle_damage, every source: attacks, projectiles, abilities, camps, towers); the authority's
//! hero attack then commits its attack clock, position and facing to the team replicas each tick
//! (CommittedAttack, Movement), so replicas follow the same attack.
use omoba_sim::Fixed64;
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::comp::{Faction, FactionType, Hero};

/// How long a hit keeps the hero retaliating without a new one.
pub const WINDOW_SECONDS: i32 = 3;

/// Last attacker of each hero and the match time of the hit.
#[derive(Clone, Debug, Default)]
pub struct HeroRetaliation(pub BTreeMap<Entity, (Entity, Fixed64)>);

fn neutral(factions: &specs::ReadStorage<'_, Faction>, entity: Entity) -> bool {
    factions.get(entity).is_some_and(|f| f.faction_id == FactionType::HostileNeutral)
}

/// Record that `source` hit `target` at match time `now`. Only heroes are tracked; a live camp
/// attacker is kept over a lane unit's hit inside the window.
pub fn record(world: &mut World, source: Entity, target: Entity, now: Fixed64) {
    if source == target || world.read_storage::<Hero>().get(target).is_none() || !world.entities().is_alive(source) {
        return;
    }
    if world.try_fetch::<HeroRetaliation>().is_none() {
        world.insert(HeroRetaliation::default());
    }
    let factions = world.read_storage::<Faction>();
    let mut retaliation = world.write_resource::<HeroRetaliation>();
    if let Some(&(previous, at)) = retaliation.0.get(&target) {
        let recent = now - at <= Fixed64::from_i32(WINDOW_SECONDS);
        if recent && previous != source && neutral(&factions, previous) && !neutral(&factions, source)
            && world.entities().is_alive(previous) {
            return;
        }
    }
    retaliation.0.insert(target, (source, now));
}

/// The attacker an idle `hero` should answer at match time `now`, if its last hit is recent.
/// The caller still checks range and legality.
pub fn attacker(retaliation: &HeroRetaliation, hero: Entity, now: Fixed64) -> Option<Entity> {
    let &(attacker, at) = retaliation.0.get(&hero)?;
    (now - at <= Fixed64::from_i32(WINDOW_SECONDS)).then_some(attacker)
}

/// Forget heroes and attackers that are gone.
pub fn retire(world: &World) {
    let Some(mut retaliation) = world.try_fetch_mut::<HeroRetaliation>() else { return };
    let entities = world.entities();
    retaliation.0.retain(|hero, (attacker, _)| entities.is_alive(*hero) && entities.is_alive(*attacker));
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    fn world() -> World {
        let mut world = World::new();
        world.register::<Hero>();
        world.register::<Faction>();
        world
    }

    #[test]
    fn a_camp_attacker_is_kept_over_a_lane_hit_inside_the_window() {
        let mut world = world();
        let hero = world.create_entity().with(Hero::default()).with(Faction::new(FactionType::Player, 1)).build();
        let camp = world.create_entity().with(Faction::new(FactionType::HostileNeutral, 0)).build();
        let creep = world.create_entity().with(Faction::new(FactionType::Enemy, 2)).build();
        record(&mut world, camp, hero, Fixed64::from_i32(10));
        record(&mut world, creep, hero, Fixed64::from_i32(11));
        let r = (*world.read_resource::<HeroRetaliation>()).clone();
        assert_eq!(attacker(&r, hero, Fixed64::from_i32(12)), Some(camp), "the camp first");
        assert_eq!(attacker(&r, hero, Fixed64::from_i32(14)), None, "the window closes");
        // After the window a lane hit replaces it.
        record(&mut world, creep, hero, Fixed64::from_i32(20));
        let r = (*world.read_resource::<HeroRetaliation>()).clone();
        assert_eq!(attacker(&r, hero, Fixed64::from_i32(20)), Some(creep));
        // A non-hero victim is not tracked.
        record(&mut world, hero, creep, Fixed64::from_i32(20));
        assert!(world.read_resource::<HeroRetaliation>().0.get(&creep).is_none());
    }
}
