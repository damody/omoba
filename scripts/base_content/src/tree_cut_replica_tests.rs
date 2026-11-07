//! E242: a tree cut on the authority reaches every team replica on the same step, so a hero
//! walking through the gap stays identical on the authority and its own replica.
use super::*;
use omoba_core::runtime::native::tree_cut;

#[test]
fn a_cut_tree_gap_walks_the_same_on_the_authority_and_every_replica() {
    use prost::Message;
    use std::collections::BTreeSet;
    let map = omoba_template_ids::moba_map_by_name("nobu").unwrap();
    let config = SingleLaneConfig { map_id: Some("nobu".into()), lane_length: Fixed64::from_i32(map.lane_length),
        wave_interval: Fixed64::from_i32(10_000), ..fast_config() };
    let seed = config.seed;
    let (mut authority, mut driver) = world(config, SimulationTickProfile::Production60Hz);
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    // A lone tree with open terrain 400 either side; the side-0 hero stands west of it.
    let v = |x: i32, y: i32| omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
    let points: Vec<_> = map.trees().trees.iter().map(|t| v(t.position.0, t.position.1)).collect();
    let (index, c) = points.iter().enumerate().find(|(_, c)| points.iter().filter(|o| (**o - **c).length_squared() < Fixed64::from_i32(500 * 500)).count() == 1
        && !map.terrain.iter().any(|r| r.min.1 <= ((c.y.raw() >> 10) as i32) + 80 && r.max.1 >= ((c.y.raw() >> 10) as i32) - 80
            && r.min.0 <= ((c.x.raw() >> 10) as i32) + 450 && r.max.0 >= ((c.x.raw() >> 10) as i32) - 450))
        .map(|(i, c)| (i as u32, *c)).expect("a lone tree");
    let hero = authority.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let player = authority.read_resource::<MobaMatch>().config.players[0];
    authority.write_storage::<Pos>().get_mut(hero).unwrap().0 = c - v(300, 0);
    let settle = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, settle);
    let allow = TeamProjectorConfig::default().component_allowlist;
    // The server attaches the current terrain to every start (omb state/core.rs); so does this harness.
    let starts_with_terrain = |authority: &mut World, tick: u64| {
        let regions = omoba_core::runtime::encode_public_blocked_regions(&authority.read_resource::<BlockedRegions>());
        let mut starts = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(tick, 60, seed);
        for start in starts.values_mut() {
            let mut metadata = omoba_core::runtime::moba_map_layout::bootstrap_metadata(map);
            metadata[1].value = regions.clone();
            start.public_metadata.extend(metadata);
        }
        starts
    };
    let starts = starts_with_terrain(&mut authority, driver.tick() + 1);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    // The script's request settles now; the tree falls at the next step's start.
    authority.write_resource::<tree_cut::PendingTreeCuts>().0.push((c, Fixed64::from_i32(100)));
    let destination = c + v(300, 0);
    for step in 0..240 {
        let inputs = if step == 0 {
            vec![(player, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo {
                target: Some(Vec2I { x: destination.x.raw() as i32, y: destination.y.raw() as i32 }), queued: false })) })]
        } else { Vec::new() };
        let accepted: Vec<_> = inputs.iter().map(|(player, input)| {
            let state = authority.read_resource::<MobaMatch>();
            CanonicalAcceptedInput::from_authoritative_acceptance(state.config.teams[0], *player, (driver.tick() + 1) * 2,
                1, canonical_entity_id(hero), None, input.encode_to_vec())
        }).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority, result);
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = starts_with_terrain(&mut authority, driver.tick() + 1);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(matches!(replica.apply_frame(frame, stepper).unwrap(), FrameApplyResult::Applied { .. }), "team {team} step {step}");
            assert_eq!(omoba_core::runtime::encode_public_blocked_regions(&stepper.filtered.world.read_resource::<BlockedRegions>()),
                omoba_core::runtime::encode_public_blocked_regions(&authority.read_resource::<BlockedRegions>()), "team {team} step {step} terrain");
            let expected = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(), expected.canonical_team_hash(), "team {team} step {step}");
        }
    }
    assert_eq!(authority.read_resource::<MobaMatch>().fallen_trees(), BTreeSet::from([index]));
    let at = authority.read_storage::<Pos>().get(hero).unwrap().0;
    assert!((at - destination).length_squared() < Fixed64::from_i32(10 * 10), "the hero walked through the gap: {at:?}");
}
