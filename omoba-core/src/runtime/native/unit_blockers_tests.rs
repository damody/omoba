use super::*;
use omoba_sim::Vec2;

use crate::runtime::StateInitializer;

fn b(x: i32, y: i32, r: i32) -> Blocker { Blocker { x, y, r } }
fn terrain() -> BlockedRegions {
    BlockedRegions(vec![BlockedRegion { name: "wall".into(), points: vec![vek::Vec2::new(900.0, 900.0), vek::Vec2::new(901.0, 900.0),
        vek::Vec2::new(901.0, 901.0), vek::Vec2::new(900.0, 901.0)] }])
}
fn hud(metric: u64, value: i64) -> Vec<u8> {
    [0_u32.to_le_bytes().as_slice(), metric.to_le_bytes().as_slice(), value.to_le_bytes().as_slice()].concat()
}

#[test]
fn a_blocker_packs_into_one_fact_value_and_back() {
    for blocker in [b(0, 0, 1), b(-1234, 5678, 96), b(-MAX_COORD, MAX_COORD, MAX_RADIUS)] {
        assert_eq!(Blocker::unpack(blocker.pack()), Some(blocker));
        assert_eq!(Blocker::parse(&blocker.name()), Some(blocker));
    }
    assert_eq!(Blocker::unpack(b(0, 0, 0).pack()), None, "no empty square");
    assert_eq!(Blocker::unpack(b(0, 0, MAX_RADIUS + 1).pack()), None);
    assert_eq!(Blocker::parse("blocker_1_2"), None);
}

#[test]
fn blocker_squares_follow_the_terrain_in_canonical_order_and_read_back() {
    let set = BTreeMap::from([(b(10, 0, 5), 2), (b(-10, 0, 5), 1)]);
    let regions = compose(terrain(), &set);
    let names: Vec<_> = regions.0.iter().map(|region| region.name.as_str()).collect();
    assert_eq!(names, vec!["wall", "blocker_-10_0_5", "blocker_10_0_5", "blocker_10_0_5"]);
    assert_eq!(current(&regions), Some(set.clone()));
    assert_eq!(without(&regions).0.len(), 1);
    assert_eq!(compose(regions.clone(), &set).0.len(), regions.0.len(), "composing again changes nothing");
    let mut bent = regions.clone(); bent.0[1].points[0].x = 99.0;
    assert_eq!(current(&bent), None, "a blocker region is exactly its square");
    let path = |from: (i32, i32), to: (i32, i32), regions: &BlockedRegions| crate::tick::hero_move_tick::path_hits_regions(
        Vec2::new(Fixed64::from_i32(from.0), Fixed64::from_i32(from.1)), Vec2::new(Fixed64::from_i32(to.0), Fixed64::from_i32(to.1)),
        Fixed64::from_i32(10), regions);
    assert!(path((-50, 0), (50, 0), &regions) && !path((-50, 0), (50, 0), &terrain()), "a blocker stops a walk through it");
}

#[test]
fn the_authority_discloses_each_change_and_a_replica_walks_the_same_terrain() {
    let pool = StateInitializer::create_thread_pool();
    let mut authority = StateInitializer::setup_campaign_ecs_world(&pool);
    let mut replica = StateInitializer::setup_campaign_ecs_world(&pool);
    authority.insert(crate::runtime::ObservableFactBuffer::default());
    // Two blockers whose summons are gone (a mid-match start carried them): the authority removes both.
    let stale = compose(terrain(), &BTreeMap::from([(b(30, 40, 96), 1), (b(-30, 40, 80), 1)]));
    *authority.write_resource::<BlockedRegions>() = stale.clone();
    *replica.write_resource::<BlockedRegions>() = stale;
    tick(&authority);
    assert_eq!(current(&authority.read_resource::<BlockedRegions>()), Some(BTreeMap::new()));
    let facts: Vec<Vec<u8>> = authority.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap().into_iter()
        .filter_map(|fact| match fact.fact {
            crate::runtime::ObservableFact::Hud { team: 0, metric_id, value } => {
                assert_eq!((fact.audience, fact.key.phase), (crate::runtime::FactAudience::AllPlayers, crate::runtime::FactPhase::PreStep));
                Some(hud(metric_id, value))
            }
            _ => None,
        }).collect();
    assert_eq!(facts.len(), 2);
    apply_disclosed(&replica, &facts.iter().map(Vec::as_slice).collect::<Vec<_>>()).unwrap();
    assert_eq!(crate::runtime::encode_public_blocked_regions(&replica.read_resource::<BlockedRegions>()),
        crate::runtime::encode_public_blocked_regions(&authority.read_resource::<BlockedRegions>()));
    // A replica adds what it is told, and refuses to remove what it does not hold.
    apply_disclosed(&replica, &[hud(ADD_METRIC_ID, b(1, 2, 3).pack()).as_slice()]).unwrap();
    assert_eq!(current(&replica.read_resource::<BlockedRegions>()), Some(BTreeMap::from([(b(1, 2, 3), 1)])));
    assert!(apply_disclosed(&replica, &[hud(REMOVE_METRIC_ID, b(9, 9, 9).pack()).as_slice()]).is_err());
    replica.maintain();
}

#[test]
fn trees_are_read_without_the_blocker_squares() {
    let map = omoba_template_ids::moba_map_by_name("nobu").unwrap();
    let with = compose(super::super::tree_cut::regions(map, &Default::default()), &BTreeMap::from([(b(0, 0, 50), 1)]));
    assert_eq!(super::super::tree_cut::cut_set_of(map, &with), Some(Default::default()));
}
