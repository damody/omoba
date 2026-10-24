//! Real serial settlement coverage; no script or presentation substitutes.
use super::*;
use specs::{Builder, WorldExt};
use omoba_sim::Vec2;
use crate::comp::BlockedRegion;

fn fixture() -> (World, Entity, Entity, Outcome) {
    let pool = crate::runtime::StateInitializer::create_thread_pool();
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    let faction = Faction::new(FactionType::Player, 1);
    let owner = world.create_entity().with(Pos(Vec2::ZERO)).with(faction.clone())
        .with(PlayerOwner::new(7)).with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),
            msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
    let entity = world.entities().create();
    let pos = Vec2::new(Fixed64::from_i32(200), Fixed64::ZERO);
    world.read_resource::<super::super::summon_navigation::PlacementReservations>().0.lock().unwrap()
        .insert(entity, (1, pos, Fixed64::from_i32(16)));
    let outcome = Outcome::SpawnSummoned {entity, pos, unit_id:"saika_gunner".into(), owner,
        faction, player_owner:Some(7), duration:Fixed64::from_i32(30)};
    (world, owner, entity, outcome)
}

fn settle(world:&mut World, outcome:Outcome) -> Vec<crate::runtime::OrderedFact> {
    world.write_resource::<Vec<Outcome>>().push(outcome);
    process_outcomes(world, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    assert!(world.read_resource::<super::super::summon_navigation::PlacementReservations>().0.lock().unwrap().is_empty());
    world.read_resource::<crate::runtime::ObservableFactBuffer>().drain_ordered().unwrap()
}

fn rejected(world:&mut World, entity:Entity, outcome:Outcome) {
    assert!(settle(world, outcome).is_empty(), "rejected intent must not announce a spawn");
    assert!(world.read_storage::<Unit>().get(entity).is_none());
    assert!(world.read_storage::<SummonedUnit>().get(entity).is_none());
    world.maintain(); // specs retirement settles at the actual frame boundary.
    assert!(!world.entities().is_alive(entity));
    assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
}

#[test]
fn summon_settlement_commits_once_and_duplicate_preserves_the_existing_body() {
    let (mut world, owner, entity, outcome) = fixture();
    let facts = settle(&mut world, outcome.clone());
    assert_eq!(facts.len(), 1);
    assert!(matches!(facts[0].fact, crate::runtime::ObservableFact::Spawn {source:Some(source),team:1,..}
        if source==crate::runtime::canonical_entity_id(owner)));
    assert_eq!(world.read_storage::<PlayerOwner>().get(entity).unwrap().player_id,7);
    assert_eq!(world.read_storage::<SummonedUnit>().get(entity).unwrap().summoner,owner);
    let events=world.write_resource::<ScriptEventQueue>().drain();
    assert_eq!(events.len(),1);assert!(matches!(events[0],ScriptEvent::Spawn {e} if e==entity));
    world.write_storage::<CProperty>().get_mut(entity).unwrap().hp=Fixed64::ONE;
    world.write_storage::<Pos>().get_mut(entity).unwrap().0.x=Fixed64::from_i32(500);
    assert!(settle(&mut world,outcome).is_empty());
    assert!(world.entities().is_alive(entity));
    assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::ONE);
    assert_eq!(world.read_storage::<Pos>().get(entity).unwrap().0.x,Fixed64::from_i32(500));
    assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
}

#[test]
fn summon_settlement_rechecks_life_faction_and_optional_player_ownership() {
    for case in 0..7 {
        let (mut world, owner, entity, outcome)=fixture();
        match case {
            0 => world.write_storage::<CProperty>().get_mut(owner).unwrap().hp=Fixed64::ZERO,
            1 => {world.write_storage::<CProperty>().remove(owner);},
            2 => world.write_storage::<Faction>().get_mut(owner).unwrap().team_id=2,
            3 => world.write_storage::<Faction>().get_mut(owner).unwrap().faction_id=FactionType::Enemy,
            4 => world.write_storage::<PlayerOwner>().get_mut(owner).unwrap().player_id=8,
            5 => {world.write_storage::<PlayerOwner>().remove(owner);},
            6 => {world.delete_entity(owner).unwrap();world.maintain();let replacement=world.create_entity().build();
                assert_eq!(replacement.id(),owner.id());assert_ne!(replacement.gen(),owner.gen());},
            _ => unreachable!(),
        }
        rejected(&mut world,entity,outcome);
    }
    let (mut world,owner,entity,mut outcome)=fixture();
    if let Outcome::SpawnSummoned {player_owner,..}=&mut outcome {*player_owner=None;}
    // Capturing no player is not permission to bind a now player-owned body.
    rejected(&mut world,entity,outcome);
    assert!(world.entities().is_alive(owner));
}

#[test]
fn summon_settlement_lifetime_bounds_and_permanent_semantics() {
    for duration in [Fixed64::ZERO,Fixed64::from_i32(3600),Fixed64::from_raw(-1),Fixed64::from_i32(3600)+Fixed64::from_raw(1)] {
        let (mut world,_,entity,mut outcome)=fixture();
        if let Outcome::SpawnSummoned {duration:value,..}=&mut outcome {*value=duration;}
        if duration<Fixed64::ZERO||duration>Fixed64::from_i32(3600) {rejected(&mut world,entity,outcome);}
        else {
            assert_eq!(settle(&mut world,outcome).len(),1);
            assert_eq!(world.read_storage::<SummonedUnit>().get(entity).unwrap().time_remaining,
                (duration>Fixed64::ZERO).then_some(duration));
        }
    }
}

#[test]
fn summon_settlement_blocked_or_unknown_template_has_no_success_fact() {
    for blocked in [false,true] {
        let (mut world,_,entity,mut outcome)=fixture();
        if blocked {world.write_resource::<BlockedRegions>().0.push(BlockedRegion {name:"enclosed".into(),points:vec![
            vek::Vec2::new(-2000.0,-2000.0),vek::Vec2::new(2000.0,-2000.0),
            vek::Vec2::new(2000.0,2000.0),vek::Vec2::new(-2000.0,2000.0)]});}
        else if let Outcome::SpawnSummoned {unit_id,..}=&mut outcome {*unit_id="missing-template".into();}
        rejected(&mut world,entity,outcome);
    }
}

#[test]
fn summon_settlement_cannot_overwrite_or_delete_an_existing_owner() {
    let (mut world,owner,entity,mut outcome)=fixture();
    // Retire the genuinely reserved body separately; forge an already-materialized target.
    world.read_resource::<super::super::summon_navigation::PlacementReservations>().release(entity);
    world.entities().delete(entity).unwrap();
    if let Outcome::SpawnSummoned {entity,..}=&mut outcome {*entity=owner;}
    assert!(settle(&mut world,outcome).is_empty());
    assert!(world.entities().is_alive(owner));
    assert_eq!(world.read_storage::<CProperty>().get(owner).unwrap().hp,Fixed64::from_i32(100));
    assert!(world.read_storage::<SummonedUnit>().get(owner).is_none());
}
