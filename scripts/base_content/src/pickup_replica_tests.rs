//! E243: a gold bag spawned on the authority reaches the team replicas as an ordinary disclosed
//! body, and a hero walking onto it takes it with every replica identical to the authority.
use super::*;
use omoba_core::runtime::native::pickup;

#[test]
fn a_dropped_gold_bag_is_taken_the_same_on_the_authority_and_every_replica() {
    use prost::Message;
    use std::collections::BTreeSet;
    let config = SingleLaneConfig { wave_interval: Fixed64::from_i32(10_000), ..three_lane_config() };
    let seed = config.seed;
    let (mut authority, mut driver) = world(config, SimulationTickProfile::Production60Hz);
    let first = driver.step(&mut authority, []).unwrap();
    project_tick(&mut authority, first);
    let v = |x: i32, y: i32| omoba_sim::Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y));
    let hero = authority.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    let player = authority.read_resource::<MobaMatch>().config.players[0];
    let start = authority.read_storage::<Pos>().get(hero).unwrap().0;
    let allow = TeamProjectorConfig::default().component_allowlist;
    let starts = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick() + 1, 60, seed);
    let mut replicas: Vec<_> = starts.into_iter().map(|(team, start)| {
        let replica = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow.clone(), BTreeSet::new()).unwrap();
        let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
        stepper.script_registry.insert_manifest(crate::get_manifest());
        populate_ability_registry(&mut stepper.filtered.world, &stepper.script_registry);
        stepper.bootstrap_membership(replica.world()).unwrap();
        (team, replica, stepper)
    }).collect();
    // The script's request settles now; the bag appears 400 ahead at the next step.
    let at = start + v(400, 0);
    pickup::request(&authority, hero, &pickup::Spec { at, gold: 50, who: pickup::Who::Any, killable: true, lifetime: Fixed64::ZERO }.payload());
    let gold_before = authority.read_storage::<Gold>().get(hero).unwrap().0;
    let mut seen = false;
    for step in 0..180 {
        let inputs = if step == 1 {
            vec![(player, PlayerInput { action: Some(PlayerInputEnum::MoveTo(MoveTo {
                target: Some(Vec2I { x: at.x.raw() as i32, y: at.y.raw() as i32 }), queued: false })) })]
        } else { Vec::new() };
        let accepted: Vec<_> = inputs.iter().map(|(player, input)| {
            let state = authority.read_resource::<MobaMatch>();
            CanonicalAcceptedInput::from_authoritative_acceptance(state.config.teams[0], *player, (driver.tick() + 1) * 2,
                1, canonical_entity_id(hero), None, input.encode_to_vec())
        }).collect();
        let result = driver.step(&mut authority, inputs).unwrap();
        authority.write_resource::<TeamProjectionRuntime>().pending_accepted_inputs.extend(accepted);
        project_tick(&mut authority, result);
        seen |= !authority.read_resource::<MobaMatch>().pickup_bodies().is_empty();
        let frames = authority.read_resource::<TeamProjectionRuntime>().latest_frames.clone();
        let expected = authority.write_resource::<TeamProjectionRuntime>().build_team_bootstraps(driver.tick() + 1, 60, seed);
        for (team, replica, stepper) in &mut replicas {
            let frame = frames[team].frame.clone();
            assert!(frame.post_step.as_ref().unwrap().component_repairs.is_empty());
            assert!(matches!(replica.apply_frame(frame, stepper).unwrap(), FrameApplyResult::Applied { .. }), "team {team} step {step}");
            let expected = SelectiveReplicaRuntime::bootstrap_from_team_game_start(&expected[team], allow.clone(), BTreeSet::new()).unwrap();
            assert_eq!(replica.canonical_team_hash(), expected.canonical_team_hash(), "team {team} step {step}");
        }
    }
    assert!(seen, "the bag stood on the ground");
    assert!(authority.read_resource::<MobaMatch>().pickup_bodies().is_empty(), "the hero took it");
    assert_eq!(authority.read_storage::<Gold>().get(hero).unwrap().0, gold_before + 50, "the bag's 50 gold");
}
