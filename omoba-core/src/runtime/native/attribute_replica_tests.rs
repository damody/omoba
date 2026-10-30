//! A team replica derives a hero's attribute max HP (`Hero::attribute_hp`) like the authority, also
//! on the tick it buys an attribute item: the 1+9 demo diverged by the item's strength (E353).
use std::collections::BTreeSet;

use omoba_sim::Fixed64;
use prost::Message;
use specs::{World, WorldExt};

use crate::game_proto::{player_input::Action, ItemBuy, PlayerInput};
use crate::runtime::{
    canonical_entity_id, run_committed_visibility_wave_b, run_team_projection_after_wave_b, CanonicalAcceptedInput,
    CommittedProjectionBatch, FrameApplyResult, Gold, Hero, MobaMatch, SelectiveReplicaRuntime, SimulationDriver,
    SimulationTickProfile, SimulationTickResult, SingleLaneConfig, SpecsDisclosedWorldStepper, StateInitializer,
    TeamProjectionRuntime, TeamProjectorConfig,
};

fn project_tick(world: &mut World, result: SimulationTickResult) {
    *world.write_resource::<CommittedProjectionBatch>() = CommittedProjectionBatch {
        tick: result.tick, facts: result.facts, barrier_reached: true, ..Default::default()
    };
    run_committed_visibility_wave_b(world, result.tick, 0);
    run_team_projection_after_wave_b(world, result.tick).unwrap();
}

#[test]
fn a_replica_settles_the_attribute_hp_of_a_bought_strength_item_like_the_authority() {
    let pool = StateInitializer::create_thread_pool();
    let mut authority = StateInitializer::setup_campaign_ecs_world(&pool);
    authority.insert(crate::runtime::ScriptRegistry::new());
    let config = SingleLaneConfig {
        heroes: ["saika_magoichi".into(), "saika_magoichi".into()],
        warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(10_000), passive_gold_per_second: 0,
        ..SingleLaneConfig::default()
    };
    let seed = config.seed;
    crate::runtime::setup_single_lane_match(&mut authority, config).unwrap();
    let mut driver = SimulationDriver::from_world(&mut authority, SimulationTickProfile::Production60Hz).unwrap();
    let heroes: Vec<_> = authority.read_resource::<MobaMatch>().heroes.iter().map(|slot| slot.entity.unwrap()).collect();
    for hero in &heroes { authority.write_storage::<Gold>().get_mut(*hero).unwrap().0 = 5000; }
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick() + 1, 60, seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let attribute_hp = |world: &World, hero| world.read_storage::<Hero>().get(hero).unwrap().attribute_hp;
    let before = attribute_hp(&authority, heroes[0]);
    for step in 0..30u64 {
        let inputs: Vec<(u32, PlayerInput)> = if step == 2 {
            [1, 2].into_iter().map(|player| (player, PlayerInput {
                action: Some(Action::ItemBuy(ItemBuy { item_id: "strength_ingot".into() })) })).collect()
        } else { Vec::new() };
        let accepted: Vec<_> = inputs.iter().enumerate().map(|(ordinal, (player, input))| {
            let state = authority.read_resource::<MobaMatch>();
            let side = state.config.players.iter().position(|id| id == player).unwrap();
            CanonicalAcceptedInput::from_authoritative_acceptance(state.config.teams[side], *player,
                (driver.tick() + 1) * 16 + ordinal as u64, 17, canonical_entity_id(heroes[side]), None, input.encode_to_vec())
        }).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick() + 1, 60, seed);
        for (team, replica, stepper) in &mut replicas {
            assert!(matches!(replica.apply_frame(frames[team].frame.clone(), stepper).unwrap(), FrameApplyResult::Applied { .. }));
            let expected = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(), expected.canonical_team_hash(), "team {team} tick {}", driver.tick());
        }
    }
    // strength_ingot: +5 strength, 25 HP each (attribute_rules).
    assert_eq!(attribute_hp(&authority, heroes[0]) - before, Fixed64::from_i32(125), "the item's strength reached attribute_hp");
}
