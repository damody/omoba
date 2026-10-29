use super::*;
use crate::runtime::native::moba_match::{begin_moba_match_tick, setup_single_lane_match, MobaMatch, MobaMatchPhase, SingleLaneConfig};
use crate::runtime::{process_outcomes, DeltaTime, Outcome, RuntimeEventVecSink, StateInitializer};

fn v(x: i32, y: i32) -> Vec2 { Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }

fn fixture() -> World {
    let pool = StateInitializer::create_thread_pool();
    let mut world = StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
    setup_single_lane_match(&mut world, SingleLaneConfig { map_id: Some(map.id.into()), lane_length: Fixed64::from_i32(map.lane_length),
        authored_minion_waves: true, warmup: Fixed64::ZERO, wave_interval: Fixed64::from_i32(10_000), ..SingleLaneConfig::default() }).unwrap();
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    world.write_resource::<DeltaTime>().0 = Fixed64::ONE / Fixed64::from_i32(60);
    world.insert(crate::runtime::ObservableFactBuffer::default());
    world
}

fn heroes(world: &World) -> [Entity; 2] {
    let state = world.read_resource::<MobaMatch>();
    [state.heroes[0].entity.unwrap(), state.heroes[1].entity.unwrap()]
}
fn place(world: &mut World, unit: Entity, at: Vec2) { world.write_storage::<Pos>().get_mut(unit).unwrap().0 = at; }
fn gold(world: &World, unit: Entity) -> i32 { world.read_storage::<Gold>().get(unit).unwrap().0 }

/// The request as a script's `create_pickup` sends it, settled like any outcome.
fn drop_pickup(world: &mut World, caster: Entity, spec: Spec) {
    world.write_resource::<Vec<Outcome>>().push(Outcome::AddBuff { target: caster, buff_id: BUFF.into(), duration: Fixed64::ONE, payload: spec.payload() });
    process_outcomes(world, &mut RuntimeEventVecSink::default()).unwrap();
    assert!(!world.read_resource::<crate::runtime::BuffStore>().has(caster, BUFF), "never stored as a buff");
}

fn bag(at: Vec2) -> Spec { Spec { at, gold: 50, who: Who::Any, killable: true, lifetime: Fixed64::ZERO } }

#[test]
fn a_bag_appears_at_the_next_tick_and_the_nearest_hero_takes_its_gold() {
    let mut world = fixture();
    let [a, b] = heroes(&world);
    place(&mut world, a, v(3000, 3000));
    place(&mut world, b, v(3600, 3000));
    drop_pickup(&mut world, a, bag(v(3300, 3000)));
    assert!(world.read_resource::<MobaMatch>().pickup_bodies().is_empty(), "nothing inside the settling tick");
    begin_moba_match_tick(&mut world);
    let body = world.read_resource::<MobaMatch>().pickup_bodies()[0];
    assert_eq!(world.read_storage::<Pos>().get(body).unwrap().0, v(3300, 3000));
    assert_eq!(world.read_storage::<crate::runtime::Unit>().get(body).unwrap().name, IDENTITY);
    assert_eq!(world.read_storage::<Faction>().get(body).unwrap().faction_id, FactionType::HostileNeutral);
    assert_eq!(world.read_storage::<ReplicationScope>().get(body).unwrap().kind, ReplicationScopeKind::Vision);
    // Out of reach: it stays.
    let (gold_a, gold_b) = (gold(&world, a), gold(&world, b));
    begin_moba_match_tick(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().pickup_bodies(), vec![body]);
    // The enemy hero walks within 150 first: any team may take it.
    place(&mut world, b, v(3300 + PICKUP_RADIUS, 3000));
    begin_moba_match_tick(&mut world);
    assert!(world.read_resource::<MobaMatch>().pickup_bodies().is_empty());
    world.maintain();
    assert!(!world.entities().is_alive(body));
    assert_eq!((gold(&world, a), gold(&world, b)), (gold_a, gold_b + 50));
}

#[test]
fn ties_go_to_the_nearest_then_the_lowest_canonical_id_and_who_limits_the_takers() {
    let mut world = fixture();
    let [a, b] = heroes(&world);
    let (low, high) = if crate::runtime::canonical_entity_id(a) < crate::runtime::canonical_entity_id(b) { (a, b) } else { (b, a) };
    place(&mut world, a, v(3000, 3100));
    place(&mut world, b, v(3000, 2900));
    drop_pickup(&mut world, a, bag(v(3000, 3000)));
    let (gold_low, gold_high) = (gold(&world, low), gold(&world, high));
    begin_moba_match_tick(&mut world);
    begin_moba_match_tick(&mut world);
    assert_eq!((gold(&world, low), gold(&world, high)), (gold_low + 50, gold_high), "equal distance: lowest canonical id");
    // `allies`: only the dropper's team, even when an enemy stands nearer.
    place(&mut world, b, v(3000, 3010));
    drop_pickup(&mut world, a, Spec { who: Who::Allies, ..bag(v(3000, 3000)) });
    let (gold_a, gold_b) = (gold(&world, a), gold(&world, b));
    begin_moba_match_tick(&mut world);
    begin_moba_match_tick(&mut world);
    assert_eq!((gold(&world, a), gold(&world, b)), (gold_a + 50, gold_b));
}

#[test]
fn a_destroyed_bag_gives_nothing_and_an_unkillable_one_cannot_be_hit() {
    let mut world = fixture();
    let [a, b] = heroes(&world);
    place(&mut world, a, v(3000, 3000));
    place(&mut world, b, v(4000, 3000));
    drop_pickup(&mut world, a, bag(v(3500, 3000)));
    drop_pickup(&mut world, a, Spec { killable: false, ..bag(v(3500, 3400)) });
    begin_moba_match_tick(&mut world);
    let bodies = world.read_resource::<MobaMatch>().pickup_bodies();
    let (killable, sturdy) = {
        let pos = world.read_storage::<Pos>();
        if pos.get(bodies[0]).unwrap().0 == v(3500, 3000) { (bodies[0], bodies[1]) } else { (bodies[1], bodies[0]) }
    };
    assert!(world.read_resource::<MobaMatch>().current_match_target_attackable(killable));
    assert!(!world.read_resource::<MobaMatch>().current_match_target_attackable(sturdy));
    // One hit destroys a killable bag (原作 ItemKillable).
    world.write_resource::<Vec<Outcome>>().push(Outcome::Damage { source: a, target: killable, pos: Vec2::ZERO, phys: Fixed64::from_i32(5),
        magi: Fixed64::ZERO, real: Fixed64::ZERO, damage_profile: 0, predeclared: false });
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    process_outcomes(&mut world, &mut RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    let gold_b = gold(&world, b);
    place(&mut world, b, v(3500, 3000));
    begin_moba_match_tick(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().pickup_bodies(), vec![sturdy]);
    assert_eq!(gold(&world, b), gold_b, "a destroyed bag gives nothing");
}

#[test]
fn a_lifetime_removes_an_unpicked_bag() {
    let mut world = fixture();
    let [a, _] = heroes(&world);
    place(&mut world, a, v(3000, 3000));
    drop_pickup(&mut world, a, Spec { lifetime: Fixed64::from_i32(2), ..bag(v(3800, 3000)) });
    begin_moba_match_tick(&mut world);
    let body = world.read_resource::<MobaMatch>().pickup_bodies()[0];
    let born = world.read_resource::<MobaMatch>().elapsed;
    world.write_resource::<MobaMatch>().elapsed = born + Fixed64::from_i32(2) - Fixed64::from_raw(20);
    begin_moba_match_tick(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().pickup_bodies(), vec![body]);
    begin_moba_match_tick(&mut world);
    world.maintain();
    assert!(world.read_resource::<MobaMatch>().pickup_bodies().is_empty() && !world.entities().is_alive(body));
}

#[test]
fn the_request_payload_is_bounded_and_replicas_drop_it() {
    let spec = Spec { at: v(10, -20), gold: 50, who: Who::Enemies, killable: false, lifetime: Fixed64::from_i32(30) };
    assert_eq!(Spec::from_payload(&spec.payload()), Some(spec));
    for bad in [Spec { gold: -1, ..spec }, Spec { gold: MAX_GOLD + 1, ..spec }, Spec { lifetime: Fixed64::from_i32(-1), ..spec },
        Spec { lifetime: Fixed64::from_i32(MAX_LIFETIME_SECONDS + 1), ..spec }] {
        assert!(Spec::from_payload(&bad.payload()).is_none(), "{bad:?}");
    }
    let mut odd = spec.payload();
    odd[KEY]["who"] = json!("everyone");
    assert!(Spec::from_payload(&odd).is_none());
    // A world without the authority queue (a replica) drops the request.
    let mut replica = World::new();
    replica.register::<Faction>();
    let caster = replica.create_entity().with(Faction::new(FactionType::Player, 1)).build();
    request(&replica, caster, &spec.payload());
    assert!(replica.try_fetch::<PendingPickups>().is_none());
}

#[test]
fn scripts_read_the_owner_gold_pool_through_the_gold_unit_attribute() {
    use crate::runtime::native::scripting::parallel_world_adapter::{ParallelAdapterCache, ParallelWorldAdapter};
    use abi_stable::std_types::{RResult, RSlice};
    use omb_script_abi::ext::{op, unit_attr, ExtArg, ExtValue};
    use omb_script_abi::world::GameWorld;
    let mut world = fixture();
    let [a, _] = heroes(&world);
    world.write_storage::<Gold>().get_mut(a).unwrap().0 = 777;
    let player = world.read_storage::<crate::runtime::PlayerOwner>().get(a).unwrap().player_id;
    let summon = world.create_entity().with(crate::runtime::PlayerOwner::new(player)).build();
    let stranger = world.create_entity().build();
    let cache = ParallelAdapterCache::new(&world, 1);
    let mut adapter = ParallelWorldAdapter::new(&cache, a);
    let mut read = |unit: Entity| match adapter.ext(op::UNIT_ATTR, RSlice::from_slice(&[ExtArg::Unit(ParallelWorldAdapter::entity_to_handle(unit)), ExtArg::Str(unit_attr::GOLD.into())])) {
        RResult::ROk(ExtValue::Num(gold)) => gold, other => panic!("{other:?}"),
    };
    assert_eq!(read(a), Fixed64::from_i32(777));
    assert_eq!(read(summon), Fixed64::from_i32(777), "a summon reads its owner's pool");
    assert_eq!(read(stranger), Fixed64::ZERO);
}
