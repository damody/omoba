use super::*;
use crate::runtime::native::moba_match::{begin_moba_match_tick, setup_single_lane_match, MobaMatch, MobaMatchPhase, SingleLaneConfig};
use crate::runtime::{process_outcomes, CProperty, DeltaTime, Outcome, RuntimeEventVecSink, StateInitializer};
use specs::Builder;

fn map(id: &str) -> &'static omoba_template_ids::MobaMapConst { omoba_template_ids::moba_map_by_name(id).unwrap() }
fn v(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }
/// Canonical bytes: regions compare by what replicas receive.
fn enc(regions: &BlockedRegions) -> Vec<u8> { crate::runtime::encode_public_blocked_regions(regions) }

fn fixture(id: &str) -> World {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    setup_single_lane_match(&mut world, SingleLaneConfig { map_id: Some(id.into()),
        lane_length: Fixed64::from_i32(map(id).lane_length), authored_minion_waves: true, warmup: Fixed64::ZERO, ..SingleLaneConfig::default() }).unwrap();
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    world.write_resource::<DeltaTime>().0 = Fixed64::ONE / Fixed64::from_i32(60);
    world.insert(crate::runtime::ObservableFactBuffer::default());
    world
}

/// The request as a script's `destroy_trees` sends it, settled like any outcome.
fn request_cut(world: &mut World, at: Vec2, radius: i32) {
    let target = world.create_entity().build();
    world.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target, buff_id: BUFF.into(), duration: Fixed64::ONE,
        payload: payload(at, Fixed64::from_i32(radius)) });
    process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
    assert!(!world.read_resource::<crate::runtime::BuffStore>().has(target, BUFF), "never stored as a buff");
}

fn tree_changes(world: &World) -> Vec<(u64, i64)> {
    world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap().into_iter().filter_map(|fact| match fact.fact {
        crate::runtime::ObservableFact::Hud { team: 0, metric_id, value } if metric_id == CUT_METRIC_ID || metric_id == REGROW_METRIC_ID => {
            assert_eq!(fact.audience, crate::runtime::FactAudience::AllPlayers);
            assert_eq!(fact.key.phase, crate::runtime::FactPhase::PreStep);
            Some((metric_id, value))
        }
        _ => None,
    }).collect()
}

/// A tree with no other tree within 400 and open ground east and west of it.
fn lone_tree(world: &World) -> (u32, Vec2) {
    let nobu = map("nobu");
    let terrain = BlockedRegions(world.read_resource::<BlockedRegions>().0[..nobu.terrain.len()].to_vec());
    let points: Vec<Vec2> = nobu.trees().trees.iter().map(|t| v(t.position.0, t.position.1)).collect();
    points.iter().enumerate().find(|(_, c)| points.iter().filter(|o| (**o - **c).length_squared() < Fixed64::from_i32(400 * 400)).count() == 1
        && !crate::tick::hero_move_tick::path_hits_regions(**c - v(150, 0), **c + v(150, 0), Fixed64::from_i32(30), &terrain))
        .map(|(index, c)| (index as u32, *c)).expect("a lone tree")
}

#[test]
fn a_cut_fells_the_trees_in_its_radius_at_the_next_tick_and_they_regrow_after_300_s() {
    let mut world = fixture("nobu");
    let nobu = map("nobu");
    let (index, c) = lone_tree(&world);
    let hits = |world: &World| crate::tick::hero_move_tick::path_hits_regions(c - v(150, 0), c + v(150, 0), Fixed64::from_i32(30), &world.read_resource::<BlockedRegions>());
    assert!(hits(&world));
    request_cut(&mut world, c + v(40, 0), 100);
    // Nothing changes inside the settling tick.
    assert_eq!(world.read_resource::<BlockedRegions>().0.len(), nobu.terrain.len() + nobu.trees().trees.len());
    assert_eq!(world.read_resource::<PendingTreeCuts>().0.len(), 1);
    begin_moba_match_tick(&mut world);
    let felled_at = world.read_resource::<MobaMatch>().elapsed;
    let fallen = BTreeSet::from([index]);
    assert_eq!(world.read_resource::<MobaMatch>().trees.regrow_at, BTreeMap::from([(index, felled_at + Fixed64::from_i32(REGROW_SECONDS))]));
    assert_eq!(enc(&world.read_resource::<BlockedRegions>()), enc(&regions(nobu, &fallen)));
    assert_eq!(*world.read_resource::<VisionOccluderSet>(), occluders(nobu, &fallen));
    assert_eq!(world.read_resource::<VisionOccluderSet>().committed().line_of_sight(c - v(150, 0), c + v(150, 0)), crate::runtime::LosResult::Clear);
    assert!(!hits(&world), "the ground through the fallen tree is open");
    assert_eq!(tree_changes(&world), vec![(CUT_METRIC_ID, i64::from(index))]);
    // A second cut over a fallen tree changes nothing and discloses nothing.
    request_cut(&mut world, c, 100);
    begin_moba_match_tick(&mut world);
    assert!(tree_changes(&world).is_empty());
    // A body on the square keeps it down past its time; once it leaves, the tree stands again.
    let body = world.create_entity().with(Pos(c)).with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
    world.write_resource::<MobaMatch>().elapsed = felled_at + Fixed64::from_i32(REGROW_SECONDS);
    begin_moba_match_tick(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().trees.fallen(), fallen);
    assert!(tree_changes(&world).is_empty());
    world.delete_entity(body).unwrap();
    world.maintain();
    begin_moba_match_tick(&mut world);
    assert!(world.read_resource::<MobaMatch>().trees.is_empty());
    assert_eq!(enc(&world.read_resource::<BlockedRegions>()), enc(&crate::runtime::moba_map_layout::compiled_blocked_regions(nobu)));
    assert_eq!(*world.read_resource::<VisionOccluderSet>(), crate::runtime::moba_map_layout::compiled_vision_occluders(nobu));
    assert!(hits(&world));
    assert_eq!(tree_changes(&world), vec![(REGROW_METRIC_ID, i64::from(index))]);
}

#[test]
fn a_wide_cut_fells_every_tree_whose_point_is_within_the_radius() {
    let mut world = fixture("nobu");
    let nobu = map("nobu");
    let at = v(nobu.trees().trees[0].position.0, nobu.trees().trees[0].position.1);
    let expected: BTreeSet<u32> = nobu.trees().trees.iter().enumerate()
        .filter(|(_, t)| (v(t.position.0, t.position.1) - at).length_squared() <= Fixed64::from_i32(500 * 500)).map(|(i, _)| i as u32).collect();
    assert!(expected.len() > 1);
    request_cut(&mut world, at, 500);
    begin_moba_match_tick(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().trees.fallen(), expected);
    assert_eq!(tree_changes(&world), expected.iter().map(|i| (CUT_METRIC_ID, i64::from(*i))).collect::<Vec<_>>());
}

#[test]
fn replicas_rebuild_the_same_terrain_from_the_disclosed_changes() {
    let nobu = map("nobu");
    let hud = |metric: u64, value: i64| [0_u32.to_le_bytes().as_slice(), &metric.to_le_bytes(), &value.to_le_bytes()].concat();
    let mut world = World::new();
    world.insert(crate::runtime::moba_map_layout::compiled_blocked_regions(nobu));
    let trees = replica_trees(Some(nobu), &world.read_resource::<BlockedRegions>()).unwrap();
    world.insert(trees);
    let cuts = [hud(CUT_METRIC_ID, 3), hud(CUT_METRIC_ID, 900), hud(0x1234, 5)];
    apply_disclosed(&world, &cuts.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap();
    assert_eq!(enc(&world.read_resource::<BlockedRegions>()), enc(&regions(nobu, &BTreeSet::from([3, 900]))));
    apply_disclosed(&world, &[hud(REGROW_METRIC_ID, 3).as_slice()]).unwrap();
    assert_eq!(enc(&world.read_resource::<BlockedRegions>()), enc(&regions(nobu, &BTreeSet::from([900]))));
    // A change the replica cannot hold is a broken stream, not a silent skip.
    for bad in [hud(CUT_METRIC_ID, 900), hud(REGROW_METRIC_ID, 3), hud(CUT_METRIC_ID, 1564), hud(CUT_METRIC_ID, -1)] {
        assert!(apply_disclosed(&world, &[bad.as_slice()]).is_err());
    }
    // A replica without the map's trees ignores tree changes.
    let mut training = World::new();
    training.insert(crate::runtime::moba_map_layout::compiled_blocked_regions(map("three_lane_training")));
    assert!(replica_trees(Some(map("three_lane_training")), &training.read_resource::<BlockedRegions>()).is_none());
    apply_disclosed(&training, &[hud(0x1234, 5).as_slice()]).unwrap();
    apply_disclosed(&training, &[hud(CUT_METRIC_ID, 0).as_slice()]).unwrap();
    assert_eq!(enc(&training.read_resource::<BlockedRegions>()), enc(&crate::runtime::moba_map_layout::compiled_blocked_regions(map("three_lane_training"))));
    training.maintain();
}

#[test]
fn a_mid_match_start_carries_the_fallen_trees_and_nothing_else() {
    let nobu = map("nobu");
    let fallen = BTreeSet::from([0, 7, 1563]);
    assert_eq!(cut_set_of(nobu, &regions(nobu, &fallen)), Some(fallen.clone()));
    assert_eq!(cut_set_of(nobu, &regions(nobu, &BTreeSet::new())), Some(BTreeSet::new()));
    let mut no_wall = crate::runtime::moba_map_layout::compiled_blocked_regions(nobu);
    no_wall.0.remove(0);
    assert_eq!(cut_set_of(nobu, &no_wall), None, "terrain never falls");
    let mut moved = regions(nobu, &fallen);
    moved.0.last_mut().unwrap().points[0].x += 1.0;
    assert_eq!(cut_set_of(nobu, &moved), None);
    let with = |regions: &BlockedRegions| {
        let mut metadata = crate::runtime::moba_map_layout::bootstrap_metadata(nobu);
        metadata[1].value = crate::runtime::encode_public_blocked_regions(regions);
        metadata
    };
    assert_eq!(crate::runtime::moba_map_layout::validate_metadata(&with(&regions(nobu, &fallen))).unwrap().unwrap().id, "nobu");
    assert!(crate::runtime::moba_map_layout::validate_metadata(&with(&no_wall)).is_err());
    // The replica recovers the same fallen set from the start's regions.
    assert_eq!(replica_trees(Some(nobu), &regions(nobu, &fallen)).unwrap().fallen, fallen);
}

#[test]
fn a_map_without_trees_ignores_the_request() {
    let mut world = fixture("three_lane_training");
    assert!(world.try_fetch::<PendingTreeCuts>().is_none());
    let before = enc(&world.read_resource::<BlockedRegions>());
    request_cut(&mut world, v(1000, 1000), 500);
    begin_moba_match_tick(&mut world);
    assert_eq!(enc(&world.read_resource::<BlockedRegions>()), before);
    assert!(world.read_resource::<MobaMatch>().trees.is_empty() && tree_changes(&world).is_empty());
}

#[test]
fn the_request_payload_is_bounded() {
    assert_eq!(decode(&payload(v(10, -20), Fixed64::from_i32(300))), Some((v(10, -20), Fixed64::from_i32(300))));
    for radius in [0, -5, MAX_RADIUS + 1] {
        assert!(decode(&json!({KEY: {"x": 0, "y": 0, "radius": i64::from(radius) * omoba_sim::fixed::SCALE}})).is_none());
    }
    assert!(decode(&json!({KEY: {"x": omoba_sim::terrain::MAX_GEOMETRY_RAW + 1, "y": 0, "radius": 1024}})).is_none());
    assert!(decode(&json!({"other": 1})).is_none());
}
