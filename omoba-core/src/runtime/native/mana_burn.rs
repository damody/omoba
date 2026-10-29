//! v3 `burn_mana`: removes up to `amount` mana from a unit (B08E、本願寺顯如、石田三成、丹羽長秀、
//! 島左近、立花宗茂). Unlike the B08E drain packet (`mana_drain`, source-ranged and settling its own
//! damage), a burn only removes mana: the document deals any damage with its own `damage` action.
//! A unit without a mana pool loses nothing. One-shot AddBuff packet, never stored.
use omoba_sim::Fixed64;
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::Hero;

pub const KEY: &str = "__mana_burn_v1";
pub const BUFF: &str = "__mana_burn";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub amount: Fixed64,
}

impl Spec {
    pub fn checked(amount: Fixed64) -> Option<Self> {
        (amount > Fixed64::ZERO && amount <= Fixed64::from_i32(1_000_000)).then_some(Self { amount })
    }
    pub fn payload(self) -> Value {
        json!({ (KEY): { "amount_raw": self.amount.raw() } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 {
            return None;
        }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 1 {
            return None;
        }
        Self::checked(Fixed64::from_raw(row.get("amount_raw")?.as_i64()?))
    }
}

/// The packet is well formed and the target is a live entity.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    world.entities().is_alive(target).then_some(spec)
}

/// Removes the mana and returns how much was removed (0 without a pool).
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> Fixed64 {
    let Some(spec) = admitted(world, target, buff_id, duration, payload) else { return Fixed64::ZERO };
    world.write_storage::<Hero>().get_mut(target).and_then(|hero| hero.mana_pool.as_mut())
        .and_then(|pool| pool.drain(spec.amount).ok()).unwrap_or(Fixed64::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::Builder;

    #[test]
    fn packets_round_trip_and_bad_ones_are_refused() {
        let spec = Spec::checked(Fixed64::from_i32(40)).unwrap();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &spec.payload()), Some(spec));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(2), &spec.payload()), None);
        assert_eq!(Spec::from_payload("other", Fixed64::ONE, &spec.payload()), None);
        assert!(Spec::checked(Fixed64::ZERO).is_none());
    }

    #[test]
    fn a_burn_removes_up_to_the_pool_and_nothing_from_a_unit_without_mana() {
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        let plain = world.create_entity().build();
        let payload = Spec::checked(Fixed64::from_i32(40)).unwrap().payload();
        assert_eq!(apply(&mut world, plain, BUFF, Fixed64::ONE, &payload), Fixed64::ZERO);
        let pool = crate::runtime::native::ability_runtime::mana_pool::ManaPool::new(Fixed64::from_i32(60), Fixed64::from_i32(100)).unwrap();
        let hero = world.create_entity().with(Hero { mana_pool: Some(pool), ..Hero::default() }).build();
        assert_eq!(apply(&mut world, hero, BUFF, Fixed64::ONE, &payload), Fixed64::from_i32(40));
        assert_eq!(apply(&mut world, hero, BUFF, Fixed64::ONE, &payload), Fixed64::from_i32(20), "only what is left");
        let left = world.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().unwrap().current();
        assert_eq!(left, Fixed64::ZERO);
    }
}
