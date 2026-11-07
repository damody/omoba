//! Taunt (原作 MODIFIER_STATE_TAUNT 類，C06T 豪奪的強制攻擊). The victim carries the status
//! `taunt` whose payload names its taunter: `{"taunt_target": [id, generation]}`. When serial
//! settlement stores it, a hero victim drops what it was doing and is ordered to attack the
//! taunter; while it holds the status its owner's new orders are refused (as
//! `command_restricted`, command_restriction.rs). When the status ends the attack order simply
//! stays, like any order. A dead or vanished taunter orders nothing. Authority and replicas run
//! the same settlement, so they issue the same order on the same tick.
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{HeroCommand, HeroCommandQueue};

pub const BUFF: &str = "taunt";
pub const KEY: &str = "taunt_target";

/// The status payload naming the taunter.
pub fn payload(taunter_id: u32, taunter_generation: i32) -> Value {
    json!({ (KEY): [taunter_id, taunter_generation] })
}

/// The live taunter a payload names.
pub fn taunter(world: &World, payload: &Value) -> Option<Entity> {
    let row = payload.get(KEY)?.as_array()?;
    let (id, generation) = (u32::try_from(row.first()?.as_u64()?).ok()?, i32::try_from(row.get(1)?.as_i64()?).ok()?);
    let entities = world.entities();
    let entity = entities.entity(id);
    (row.len() == 2 && entity.gen().id() == generation && entities.is_alive(entity)).then_some(entity)
}

/// After the status was stored on `victim`: a hero victim is ordered to attack its taunter.
pub fn order(world: &mut World, victim: Entity, payload: &Value) {
    let Some(target) = taunter(world, payload) else { return };
    if target == victim || world.read_storage::<super::comp::Hero>().get(victim).is_none() { return; }
    super::game_processor::clear_hero_command_queue(world, victim);
    super::cast_windup::cancel(world, victim);
    super::game_processor::interrupt_attack_for_accepted_command(world, victim);
    if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(victim) {
        queue.replace(HeroCommand::AttackTarget { target, chase_origin: None });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;

    /// 嘲諷：英雄被命令攻擊嘲諷者，期間拒絕擁有者的新指令；嘲諷者不在了就不下命令。
    #[test]
    fn a_taunted_hero_attacks_its_taunter_and_refuses_orders() {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
        }).unwrap();
        let heroes: Vec<Entity> = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().filter_map(|slot| slot.entity).collect();
        let (victim, taunter) = (heroes[0], heroes[1]);
        assert!(!super::super::command_restriction::restricted(&world, victim));
        let payload = super::payload(taunter.id(), taunter.gen().id());
        world.write_resource::<crate::runtime::BuffStore>().add(victim, BUFF, Fixed64::from_i32(2), payload.clone());
        order(&mut world, victim, &payload);
        let active = world.read_storage::<HeroCommandQueue>().get(victim).and_then(|queue| queue.active);
        assert!(matches!(active, Some(HeroCommand::AttackTarget { target, .. }) if target == taunter));
        assert!(super::super::command_restriction::restricted(&world, victim), "orders refused while taunted");
        assert_eq!(taunter_of(&world, &json!({ (KEY): [taunter.id(), taunter.gen().id() + 1] })), None, "a stale generation names no one");
    }

    fn taunter_of(world: &World, payload: &Value) -> Option<Entity> { taunter(world, payload) }
}
