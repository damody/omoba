//! A team replica derives a hero's attribute max HP (`Hero::attribute_hp`) like the authority, also
//! on the tick it buys an attribute item: the 1+9 demo diverged by the item's strength (E353). The
//! other team, which sees the hero but never its inventory, takes the authority's derived
//! attributes as disclosed (CommittedAttributes) instead of deriving them without the item (E374).
use std::collections::BTreeSet;

use omoba_sim::Fixed64;
use prost::Message;
use specs::{World, WorldExt};

use crate::game_proto::{player_input::Action, ItemBuy, PlayerInput};
use crate::runtime::{
    canonical_entity_id, run_committed_visibility_wave_b, run_team_projection_after_wave_b, CanonicalAcceptedInput,
    CommittedProjectionBatch, FrameApplyResult, Gold, Hero, MobaMatch, SelectiveReplicaRuntime, SimulationDriver,
    SimulationTickProfile, SimulationTickResult, SingleLaneConfig, SpecsDisclosedWorldStepper, StateInitializer,
    TeamProjectionRuntime,
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
    // Damage settles (a base tower shoots the visiting hero): its log reads tower names.
    authority.insert(crate::runtime::TowerTemplateRegistry::default());
    let config = SingleLaneConfig {
        heroes: ["saika_magoichi".into(), "saika_magoichi".into()], map_id: Some("three_lane_quick_training".into()), lane_length: Fixed64::from_i32(omoba_template_ids::moba_map_by_name("three_lane_quick_training").unwrap().lane_length),
        warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(10_000), passive_gold_per_second: 0,
        ..SingleLaneConfig::default()
    };
    let seed = config.seed;
    crate::runtime::setup_single_lane_match(&mut authority, config).unwrap();
    let mut driver = SimulationDriver::from_world(&mut authority, SimulationTickProfile::Production60Hz).unwrap();
    let heroes: Vec<_> = authority.read_resource::<MobaMatch>().heroes.iter().map(|slot| slot.entity.unwrap()).collect();
    for hero in &heroes { authority.write_storage::<Gold>().get_mut(*hero).unwrap().0 = 5000; }
    // The second hero stands beside the first from the start (disarmed, so they do not fight): the
    // first buys while the other team's replica sees it, which never receives its items.
    let at = authority.read_storage::<crate::runtime::Pos>().get(heroes[0]).unwrap().0;
    authority.write_storage::<crate::runtime::Pos>().get_mut(heroes[1]).unwrap().0 = omoba_sim::Vec2::new(at.x + Fixed64::from_i32(200), at.y);
    for hero in &heroes {
        authority.write_resource::<crate::runtime::BuffStore>().add(*hero, crate::runtime::native::rules::vocab::state::DISARM,
            Fixed64::from_i32(60), serde_json::Value::Null);
    }
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let allow = crate::runtime::secure_replica_component_allowlist();
    let starts = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick() + 1, 60, seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), crate::runtime::secure_replica_resource_allowlist()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), crate::runtime::secure_replica_resource_allowlist());
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    let attribute_hp = |world: &World, hero| world.read_storage::<Hero>().get(hero).unwrap().attribute_hp;
    let before = attribute_hp(&authority, heroes[0]);
    for step in 0..30u64 {
        let inputs: Vec<(u32, PlayerInput)> = if step == 2 {
            [1].into_iter().map(|player| (player, PlayerInput {
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
    // Each team's replica holds the other team's hero, without its inventory, with the
    // authority's derived attributes.
    let teams: Vec<u32> = { let state = authority.read_resource::<MobaMatch>(); heroes.iter().enumerate().map(|(side, _)| state.config.teams[side]).collect() };
    for (team, replica, _) in &replicas {
        let (side, _) = teams.iter().enumerate().find(|(_, owner)| *owner != team).unwrap();
        let enemy = replica.world().entities.values().find(|entity| entity.components.contains_key(&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID)
            && entity.components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| crate::runtime::decode_demo_render_state(bytes)).is_some_and(|render| render.team_id == teams[side]))
            .unwrap_or_else(|| panic!("team {team} sees the other hero"));
        assert!(!enemy.components.contains_key(&crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID), "never its items");
        let seen: Hero = serde_json::from_slice(&enemy.components[&crate::runtime::DISCLOSED_HERO_COMPONENT_SCHEMA_ID]).unwrap();
        let truth = authority.read_storage::<Hero>().get(heroes[side]).unwrap().clone();
        assert_eq!((seen.attribute_hp, seen.attribute_totals), (truth.attribute_hp, truth.attribute_totals), "team {team}");
        // Only the first hero buys (the second stands away from its own shop).
        let bought = if side == 0 { Fixed64::from_i32(5) } else { Fixed64::ZERO };
        assert_eq!(seen.attribute_totals[0], truth.get_total_strength() + bought, "the item's strength is in the disclosed total");
    }
}
