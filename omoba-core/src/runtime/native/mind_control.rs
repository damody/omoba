//! Mind control (B23T_old 心靈控制), reduced scope (D22, line B 2026-11-07). The holder of the
//! status `mind_control`, whose payload names its controller (`{"mind_controller": [id,
//! generation]}`), counts as the controller's team: the authority gives it the controller's
//! `Faction` while the status lasts and puts its own back when the status ends, the holder dies or
//! the controller dies (dispel.rs `remove_owned_by`). Every hostility check (acquisition, towers,
//! selections, scripts) follows the `Faction`, and team replicas take it from the disclosed combat
//! relation each step. A hero holder drops what it was doing, attack-moves where it stands (the
//! host AI attacks whatever is hostile to it now) and refuses its owner's orders
//! (command_restriction.rs; its own team's replica through owner_control.rs).
//!
//! Not in scope (the full version would be its own task): the controller does not command it, its
//! `PlayerOwner`, vision, render team, private information, kill credit and lane copies
//! (`LaneUnit.team`, the hero slot) stay its own team's. The status itself is not dispelled with
//! the v3 status that applied it (source B23T_old: a dispel did not end the control either).
use std::collections::BTreeMap;

use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};

use super::comp::{Faction, HeroCommand, HeroCommandQueue, Pos};
use super::BuffStore;

pub const BUFF: &str = "mind_control";
pub const KEY: &str = "mind_controller";

/// The status payload naming the controller.
pub fn payload(controller_id: u32, controller_generation: i32) -> Value {
    json!({ (KEY): [controller_id, controller_generation] })
}

/// The live controller a payload names.
pub fn controller(world: &World, payload: &Value) -> Option<Entity> {
    let row = payload.get(KEY)?.as_array()?;
    let (id, generation) = (u32::try_from(row.first()?.as_u64()?).ok()?, i32::try_from(row.get(1)?.as_i64()?).ok()?);
    let entities = world.entities();
    let entity = entities.entity(id);
    (row.len() == 2 && entity.gen().id() == generation && entities.is_alive(entity)).then_some(entity)
}

/// Authority: each controlled unit's own `Faction`, put back when its control ends.
#[derive(Default)]
pub struct MindControls(pub BTreeMap<Entity, Faction>);

/// The team a controlled unit really belongs to (its render team, information and vision).
pub fn home_team(world: &World, entity: Entity) -> Option<i32> {
    world.try_fetch::<MindControls>()?.0.get(&entity).map(|home| home.team_id)
}

fn is_replica(world: &World) -> bool { world.has_value::<crate::runtime::filtered_specs::ReplicaPhaseTrace>() }

/// After the status was stored on `target`: the authority moves it to its controller's team; a
/// hero holder is ordered to attack-move where it stands.
pub fn take(world: &mut World, target: Entity, payload: &Value) {
    let Some(by) = controller(world, payload) else { return };
    if by == target { return; }
    if !is_replica(world) {
        let Some(team) = world.read_storage::<Faction>().get(by).cloned() else { return };
        let home = match world.write_storage::<Faction>().get_mut(target) {
            Some(faction) => std::mem::replace(faction, team),
            None => return,
        };
        if world.try_fetch::<MindControls>().is_none() { world.insert(MindControls::default()); }
        world.write_resource::<MindControls>().0.entry(target).or_insert(home);
    }
    if world.read_storage::<super::comp::Hero>().get(target).is_none() { return; }
    let Some(at) = world.read_storage::<Pos>().get(target).map(|pos| pos.0) else { return };
    super::game_processor::clear_hero_command_queue(world, target);
    super::cast_windup::cancel(world, target);
    super::game_processor::interrupt_attack_for_accepted_command(world, target);
    if let Some(queue) = world.write_storage::<HeroCommandQueue>().get_mut(target) {
        queue.replace(HeroCommand::AttackMove { pos: at });
    }
}

/// Authority, after every outcome batch: a unit whose status is gone (ended, removed, its holder
/// dead) gets its own `Faction` back.
pub fn settle(world: &mut World) {
    if is_replica(world) { return; }
    let ended: Vec<(Entity, Faction)> = {
        let Some(rows) = world.try_fetch::<MindControls>() else { return };
        if rows.0.is_empty() { return; }
        let buffs = world.read_resource::<BuffStore>();
        let entities = world.entities();
        rows.0.iter().filter(|(entity, _)| !(entities.is_alive(**entity) && buffs.has(**entity, BUFF)))
            .map(|(entity, home)| (*entity, home.clone())).collect()
    };
    for (entity, home) in ended {
        world.write_resource::<MindControls>().0.remove(&entity);
        if world.entities().is_alive(entity) {
            if let Some(faction) = world.write_storage::<Faction>().get_mut(entity) { *faction = home; }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoba_sim::Fixed64;

    /// 心靈控制（縮小版）：英雄換到控制者的隊伍、拒絕擁有者的指令並原地攻擊移動；
    /// 狀態結束時換回原隊伍；基本驅散不會解除。
    #[test]
    fn a_controlled_hero_joins_its_controllers_team_until_the_status_ends() {
        use crate::runtime::{SingleLaneConfig, StateInitializer};
        let pool = StateInitializer::create_thread_pool();
        let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
        crate::runtime::setup_single_lane_match(&mut world, SingleLaneConfig {
            warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(1_000_000), ..SingleLaneConfig::default()
        }).unwrap();
        let heroes: Vec<Entity> = world.read_resource::<crate::runtime::MobaMatch>().heroes.iter().filter_map(|slot| slot.entity).collect();
        let (victim, caster) = (heroes[0], heroes[1]);
        let faction = |world: &World, e| world.read_storage::<Faction>().get(e).map(|f| (f.faction_id.clone(), f.team_id)).unwrap();
        let (home, team) = (faction(&world, victim), faction(&world, caster));
        assert_ne!(home.1, team.1);
        let payload = super::payload(caster.id(), caster.gen().id());
        world.write_resource::<BuffStore>().add(victim, BUFF, Fixed64::from_i32(2), payload.clone());
        take(&mut world, victim, &payload);
        assert_eq!(faction(&world, victim), team, "on the controller's team");
        assert_eq!(home_team(&world, victim), Some(home.1));
        assert!(super::super::command_restriction::restricted(&world, victim), "orders refused while controlled");
        let active = world.read_storage::<HeroCommandQueue>().get(victim).and_then(|queue| queue.active);
        assert!(matches!(active, Some(HeroCommand::AttackMove { .. })), "{active:?}");
        for side in [super::super::dispel::Purge::Debuffs, super::super::dispel::Purge::Buffs] {
            super::super::dispel::dispel(&world, victim, super::super::dispel::Strength::Strong, side);
        }
        settle(&mut world);
        assert_eq!(faction(&world, victim), team, "a dispel does not end the control");
        world.write_resource::<BuffStore>().tick(Fixed64::from_i32(2));
        settle(&mut world);
        assert_eq!(faction(&world, victim), home, "back on its own team");
        assert_eq!(home_team(&world, victim), None);
        assert!(!super::super::command_restriction::restricted(&world, victim));
        // the controller's death ends it
        world.write_resource::<BuffStore>().add(victim, BUFF, Fixed64::from_i32(5), payload.clone());
        take(&mut world, victim, &payload);
        super::super::dispel::remove_owned_by(&world, caster);
        settle(&mut world);
        assert_eq!(faction(&world, victim), home, "the controller died");
        assert_eq!(controller(&world, &json!({ (KEY): [caster.id(), caster.gen().id() + 1] })), None, "a stale generation names no one");
    }
}
