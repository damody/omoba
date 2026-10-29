//! Short command restriction (source `MODIFIER_STATE_COMMAND_RESTRICTED`, e.g.
//! B08W / B08E: 0.1 s after the cast). While a hero holds the buff [`BUFF`]
//! its owner's new orders are refused at admission: hero commands (move,
//! attack, attack-move, hold, stop; queued or not), ability casts and item
//! uses. The current order keeps running, nothing is cancelled, and the hero
//! still auto-attacks; orders to owned summons are not the hero's own orders
//! and pass. Direction gestures (input scheme) are a continuous stream, not
//! discrete orders, and are not restricted.
//!
//! The buff is an ordinary BuffStore entry (payload `{}`) a script puts on its
//! own caster inside the cast, so the authority and the caster's team replica,
//! which runs the same cast, hold it for the same ticks and refuse the same
//! inputs. A refused cast reports `HERO_INACTIVE` (an owner-state code).
use specs::{Entity, World};

use super::ability_runtime::BuffStore;

pub const BUFF: &str = "command_restricted";

/// Whether `hero` currently refuses new orders.
pub fn restricted(world: &World, hero: Entity) -> bool {
    world.try_fetch::<BuffStore>().is_some_and(|buffs| buffs.has(hero, BUFF))
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;
    use specs::WorldExt;

    #[test]
    fn the_buff_restricts_only_its_holder_until_it_expires() {
        let mut world = World::new();
        world.insert(BuffStore::default());
        let (hero, other) = (world.entities().create(), world.entities().create());
        assert!(!restricted(&world, hero));
        world.write_resource::<BuffStore>().add(hero, BUFF, Fixed64::from_raw(102), serde_json::json!({}));
        assert!(restricted(&world, hero) && !restricted(&world, other));
        world.write_resource::<BuffStore>().tick(Fixed64::from_raw(102));
        assert!(!restricted(&world, hero), "0.1 s later orders are accepted again");
    }
}
