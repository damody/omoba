//! v3 `status.forced_move = { toward, locks }` (原作 A11E 陰邪凝視, A21T 假情報: MOVE_TO_POSITION and
//! a locked order filter). The holder carries the row `<status runtime id>:forced_move` beside its
//! status, payload `{"forced_move_to": [x_raw, y_raw]}`. When serial settlement stores it, a hero
//! holder drops what it was doing and is ordered to move there (the ordinary pathing move); while
//! it holds a row its owner's new orders are refused (command_restriction.rs, every lock at once).
//! The row lasts as long as its status and goes with it (dispel::remove_status). When it ends the
//! move order simply stays, like any order. Authority and replicas run the same settlement, so they
//! issue the same order on the same tick. Other units (creeps, summons) keep their own AI.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::ability_runtime::BuffStore;
use super::comp::{HeroCommand, HeroCommandQueue};

pub const KEY: &str = "forced_move_to";
pub const SUFFIX: &str = ":forced_move";

/// The row of the status `status_id` (its runtime id) forcing its holder toward `to`.
pub fn row(status_id: &str) -> String { format!("{status_id}{SUFFIX}") }

pub fn payload(to: Vec2) -> Value { json!({ (KEY): [to.x.raw(), to.y.raw()] }) }

/// The destination a row's payload names, inside the map's geometry.
pub fn destination(payload: &Value) -> Option<Vec2> {
    let row = payload.as_object().filter(|object| object.len() == 1)?.get(KEY)?.as_array().filter(|row| row.len() == 2)?;
    let raw = |value: &Value| value.as_i64().filter(|raw| raw.abs() <= omoba_sim::terrain::MAX_GEOMETRY_RAW);
    Some(Vec2::new(Fixed64::from_raw(raw(&row[0])?), Fixed64::from_raw(raw(&row[1])?)))
}

/// Whether `unit` holds a forced move now.
pub fn forced(buffs: &BuffStore, unit: Entity) -> bool {
    buffs.iter_for(unit).any(|(id, entry)| id.ends_with(SUFFIX) && entry.remaining > Fixed64::ZERO && entry.payload.get(KEY).is_some())
}

/// After the row was stored on `holder`: a hero holder is ordered to move to the destination.
pub fn order(world: &mut World, holder: Entity, payload: &Value) {
    let Some(pos) = destination(payload) else { return };
    if world.read_storage::<super::comp::Hero>().get(holder).is_none() { return; }
    super::game_processor::clear_hero_command_queue(world, holder);
    super::cast_windup::cancel(world, holder);
    super::game_processor::interrupt_attack_for_accepted_command(world, holder);
    if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(holder) {
        queue.replace(HeroCommand::MoveTo { pos });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// E464：被強制移動的英雄改走向目的地，期間拒絕擁有者的新指令；狀態結束後指令照常。
    #[test]
    fn a_forced_hero_walks_to_the_destination_and_refuses_orders() {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
        }).unwrap();
        let hero = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().find_map(|slot| slot.entity).unwrap();
        let to = Vec2::new(Fixed64::from_i32(1200), Fixed64::from_i32(800));
        assert_eq!(destination(&payload(to)), Some(to));
        assert!(destination(&json!({ (KEY): [1, 2], "other": 1 })).is_none() && destination(&json!({ (KEY): [1] })).is_none());
        assert!(!super::super::command_restriction::restricted(&world, hero));
        world.write_resource::<Vec<crate::comp::Outcome>>().push(crate::comp::Outcome::AddBuff {
            target: hero, buff_id: row("a11e_mesmerize"), duration: Fixed64::from_i32(2), payload: payload(to) });
        let mut events = crate::runtime::native::events::RuntimeEventVecSink::default();
        crate::runtime::native::game_processor::process_outcomes(&mut world, &mut events).unwrap();
        assert!(super::super::command_restriction::restricted(&world, hero), "the holder refuses orders");
        assert!(matches!(world.read_storage::<HeroCommandQueue>().get(hero).and_then(|queue| queue.active.as_ref()),
            Some(HeroCommand::MoveTo { pos }) if *pos == to), "ordered to the destination");
        world.write_resource::<BuffStore>().tick(Fixed64::from_i32(2));
        assert!(!super::super::command_restriction::restricted(&world, hero), "orders are accepted once it ends");
        world.write_resource::<BuffStore>().add(hero, &row("a11e_mesmerize"), Fixed64::from_i32(2), payload(to));
        super::super::dispel::remove_status(&world, hero, "a11e_mesmerize");
        assert!(!super::super::command_restriction::restricted(&world, hero), "the row goes with its status");
    }
}
