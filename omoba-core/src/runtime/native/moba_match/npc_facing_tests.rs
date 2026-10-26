//! One facing rule for lane and camp NPCs (work-creep-facing): the visible
//! front is the step just taken, or the target being attacked.
use super::*;
use crate::runtime::{SimulationDriver, SimulationTickProfile, StateInitializer};

fn driven(mut config: SingleLaneConfig) -> (World, SimulationDriver) {
    if let Some(map) = config.map_id.as_deref().and_then(omoba_template_ids::moba_map_by_name) {
        config.lane_length = Fixed64::from_i32(map.lane_length);
    }
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    setup_single_lane_match(&mut world, config).unwrap();
    world.insert(crate::runtime::ScriptRegistry::new());
    let driver = SimulationDriver::from_world(&mut world, SimulationTickProfile::Production60Hz).unwrap();
    (world, driver)
}

fn facing(world: &World, entity: Entity) -> Option<i32> {
    world.read_storage::<Facing>().get(entity).map(|facing| facing.0.ticks())
}

fn dir(from: Vec2, to: Vec2) -> Option<i32> { direction_angle(from, to).map(|angle| angle.ticks()) }

fn positions(world: &World) -> BTreeMap<Entity, Vec2> {
    use specs::Join;
    (&world.entities(), &world.read_storage::<Pos>()).join().map(|(entity, pos)| (entity, pos.0)).collect()
}

#[test]
fn lane_creeps_face_their_step_and_their_attack_target() {
    // Routes with turns: a creep that kept its spawn facing would fail here.
    let (mut world, mut driver) = driven(SingleLaneConfig { warmup: Fixed64::ZERO, map_id: Some("three_lane_training".into()),
        ..SingleLaneConfig::default() });
    let (mut moved, mut attacked) = (0, 0);
    for _ in 0..3600 {
        let before = positions(&world);
        let sequences: BTreeMap<Entity, u32> = world.read_resource::<MobaMatch>().units.values()
            .map(|unit| (unit.entity, unit.attack_sequence)).collect();
        driver.step(&mut world, []).unwrap();
        let after = positions(&world);
        let state = world.read_resource::<MobaMatch>();
        for unit in state.units.values().filter(|unit| unit.role == LaneRole::Creep) {
            let (Some(&was), Some(&now)) = (before.get(&unit.entity), after.get(&unit.entity)) else { continue };
            let current = facing(&world, unit.entity);
            assert!(current.is_some(), "a lane creep always has a facing");
            if let Some(step) = dir(was, now) {
                assert_eq!(current, Some(step), "a moving creep faces its step");
                moved += 1;
            } else if sequences.get(&unit.entity).is_some_and(|sequence| *sequence != unit.attack_sequence) {
                let target = unit.target.expect("an attack has a target");
                // Decided on the start-of-step position; a hero may have moved earlier in the step.
                let candidates = [before.get(&target), after.get(&target)].map(|pos| pos.and_then(|pos| dir(now, *pos)));
                assert!(candidates.contains(&current), "an attacking creep faces its target: {current:?} vs {candidates:?}");
                attacked += 1;
            }
        }
    }
    assert!(moved > 100 && attacked > 10, "both rules exercised: moved={moved} attacked={attacked}");
}

#[test]
fn lane_creeps_are_born_facing_their_next_waypoint() {
    let (mut world, mut driver) = driven(SingleLaneConfig { warmup: Fixed64::ZERO, map_id: Some("three_lane_training".into()),
        ..SingleLaneConfig::default() });
    driver.step(&mut world, []).unwrap();
    let state = world.read_resource::<MobaMatch>();
    let creeps: Vec<_> = state.units.values().filter(|unit| unit.role == LaneRole::Creep).collect();
    assert!(!creeps.is_empty(), "the first wave spawned");
    for unit in creeps {
        let route = &state.routes[unit.lane][unit.team as usize];
        let pos = world.read_storage::<Pos>().get(unit.entity).unwrap().0;
        let next = route[unit.next_waypoint.min(route.len() - 1)];
        let expected = dir(pos, next).expect("a creep is never born on its waypoint");
        let actual = facing(&world, unit.entity).expect("born with a facing");
        // The first step may already have turned it at a waypoint; it still faces down its lane.
        let delta = (actual - expected).rem_euclid(omoba_sim::trig::TAU_TICKS);
        assert!(delta.min(omoba_sim::trig::TAU_TICKS - delta) < omoba_sim::trig::TAU_TICKS / 8, "creep faces down its lane");
    }
}

#[test]
fn camp_monsters_face_their_chase_step_and_their_attack_target() {
    let (mut world, mut driver) = driven(SingleLaneConfig { warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(10_000),
        map_id: Some("three_lane_training".into()), ..SingleLaneConfig::default() });
    driver.step(&mut world, []).unwrap();
    let hero = world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let camp = world.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap();
    assert!(facing(&world, camp).is_some(), "a camp monster is born with a facing");
    let home = world.read_storage::<Pos>().get(camp).unwrap().0;
    // Fixture placement and one real hit: the camp aggroes on the hero, chases and attacks.
    world.write_storage::<Pos>().get_mut(hero).unwrap().0 = home + Vec2::new(Fixed64::ZERO, Fixed64::from_i32(250));
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { pos: Vec2::ZERO, phys: Fixed64::ZERO, magi: Fixed64::ZERO,
        real: Fixed64::from_i32(1), source: hero, target: camp, damage_profile: 0, predeclared: false });
    crate::runtime::process_outcomes(&mut world, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    let (mut moved, mut attacked) = (0, 0);
    for _ in 0..240 {
        let before = positions(&world);
        driver.step(&mut world, []).unwrap();
        let after = positions(&world);
        let state = world.read_resource::<MobaMatch>();
        let bodies: Vec<_> = std::iter::once(&state.jungle_camps[0]).chain(state.jungle_camps[0].members.iter())
            .filter_map(|body| body.entity).collect();
        for body in bodies {
            let (Some(&was), Some(&now)) = (before.get(&body), after.get(&body)) else { continue };
            if let Some(step) = dir(was, now) {
                assert_eq!(facing(&world, body), Some(step), "a chasing monster faces its step");
                moved += 1;
            } else if let Some(at) = after.get(&hero).copied().filter(|at| (*at - now).length() <= Fixed64::from_i32(400)) {
                assert_eq!(facing(&world, body), dir(now, at), "an attacking monster faces its target");
                attacked += 1;
            }
        }
    }
    assert!(moved > 0 && attacked > 0, "both rules exercised: moved={moved} attacked={attacked}");
}
