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
    // The body takes the template's turn speed and attack projectile speed.
    let stats=*omoba_template_ids::summon_by_name("saika_gunner").and_then(omoba_template_ids::active_summon_stats).unwrap();
    assert_eq!(world.read_storage::<TurnSpeed>().get(entity).unwrap().0,stats.turn_speed);
    assert_eq!(world.read_storage::<TAttack>().get(entity).unwrap().bullet_speed,stats.projectile_speed);
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
fn a_dummy_summon_is_a_bodiless_holder_for_its_lifetime() {
    let (mut world, _owner, entity, mut outcome) = fixture();
    if let Outcome::SpawnSummoned {unit_id,duration,..}=&mut outcome {*unit_id="dummy".into();*duration=Fixed64::from_i32(4);}
    assert_eq!(settle(&mut world, outcome).len(), 1);
    assert!(world.read_storage::<SummonedUnit>().get(entity).is_some_and(|summon| summon.immobile));
    assert!(world.read_storage::<TAttack>().get(entity).is_none(), "no attack");
    assert_eq!(world.read_storage::<CollisionRadius>().get(entity).map(|r| r.0), Some(Fixed64::ZERO), "blocks nothing");
    assert!(world.read_storage::<CProperty>().get(entity).is_some_and(|p| p.hp > Fixed64::ZERO), "a living holder");
    let buffs = world.read_resource::<crate::runtime::BuffStore>();
    for id in super::super::dummy_unit::STATES {
        assert_eq!(buffs.get(entity, id).map(|row| row.remaining), Some(Fixed64::from_i32(4)), "{id} for its lifetime");
    }
}

#[test]
fn an_item_use_commits_its_summon_before_the_effects_on_it_and_a_refused_use_releases_it() {
    let pool = crate::runtime::StateInitializer::create_thread_pool();
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(crate::runtime::TowerTemplateRegistry::default());
    crate::runtime::setup_single_lane_match(&mut world, crate::runtime::SingleLaneConfig { warmup: Fixed64::ZERO, ..Default::default() }).unwrap();
    let (hero, pid) = { let mut state = world.write_resource::<crate::runtime::MobaMatch>();
        state.phase = crate::runtime::MobaMatchPhase::Playing; (state.heroes[0].entity.unwrap(), state.heroes[0].player_id) };
    if world.try_fetch::<ItemRegistry>().is_none() { world.insert(ItemRegistry::generated_moba()); }
    let item = { let registry = world.read_resource::<ItemRegistry>(); let mut ids: Vec<_> = registry.items.keys().cloned().collect(); ids.sort(); ids[0].clone() };
    world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0] = Some(crate::runtime::native::comp::inventory::ItemInstance { item_id: item.clone(), cooldown_remaining: 0.0 });
    let at = world.read_storage::<Pos>().get(hero).unwrap().0;
    let faction = world.read_storage::<Faction>().get(hero).unwrap().clone();
    let transaction = |world: &World| {
        let entity = world.entities().create();
        (entity, Outcome::ScriptItemTransaction { owner: hero, owner_pid: pid, slot: 0, item_id: item.clone(), target: None, target_point: None,
            refund: crate::comp::ItemRefund::default(),
            outcomes: vec![Outcome::SpawnSummoned { entity, pos: at, unit_id: "dummy".into(), owner: hero, faction: faction.clone(),
                player_owner: Some(pid), duration: Fixed64::from_i32(5) },
                Outcome::AddBuff { target: entity, buff_id: "fire_field".into(), duration: Fixed64::from_i32(5), payload: serde_json::Value::Null }] })
    };
    let (summon, used) = transaction(&world);
    world.write_resource::<Vec<Outcome>>().push(used);
    process_outcomes(&mut world, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    assert!(world.read_storage::<SummonedUnit>().get(summon).is_some(), "the item's summon is committed");
    assert!(world.read_resource::<BuffStore>().has(summon, "fire_field"), "the status after it lands on it");
    // A use whose slot is not ready is refused and releases its summon.
    world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0].as_mut().unwrap().cooldown_remaining = 9.0;
    let (refused, used) = transaction(&world);
    world.write_resource::<Vec<Outcome>>().push(used);
    process_outcomes(&mut world, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
    world.maintain();
    assert!(!world.entities().is_alive(refused), "a refused use releases its reservation");
}

/// v3 `refund { mana, cooldown }` in an item's cast (魂蛇 soul_adder on a target it cannot put to
/// sleep): the use commits, but spends no mana and starts no cooldown; without the refund it does.
#[test]
fn a_refunded_item_use_commits_without_its_mana_and_cooldown() {
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
    crate::runtime::setup_single_lane_match(&mut world, crate::runtime::SingleLaneConfig::default()).unwrap();
    let (hero, pid) = { let state = world.read_resource::<crate::runtime::MobaMatch>(); (state.heroes[0].entity.unwrap(), state.heroes[0].player_id) };
    world.write_resource::<crate::runtime::MobaMatch>().phase = crate::runtime::MobaMatchPhase::Playing;
    if world.try_fetch::<ItemRegistry>().is_none() { world.insert(ItemRegistry::generated_moba()); }
    // an item with a cooldown and a mana cost
    let item = omoba_template_ids::MOBA_ITEM_CATALOG.iter()
        .filter(|row| row.mana_cost > Fixed64::ZERO && world.read_resource::<ItemRegistry>().get(row.id).is_some_and(|cfg| cfg.cooldown > 0.0))
        .map(|row| row.id).next().expect("a catalog item with mana and a cooldown").to_owned();
    let cooldown = world.read_resource::<ItemRegistry>().get(&item).unwrap().cooldown;
    // a match with mana, the hero's pool full enough for two uses
    world.write_resource::<crate::runtime::MobaMatch>().config.mana_enabled = true;
    world.write_storage::<Hero>().get_mut(hero).unwrap().mana_pool = Some(crate::runtime::ability_runtime::ManaPool::full(Fixed64::from_i32(10_000)).unwrap());
    let mana = |world: &World| world.read_storage::<Hero>().get(hero).unwrap().mana_pool.as_ref().map(|pool| pool.current());
    let use_with = |world: &mut World, refund: crate::comp::ItemRefund| {
        world.write_storage::<Inventory>().get_mut(hero).unwrap().slots[0] =
            Some(crate::runtime::native::comp::inventory::ItemInstance { item_id: item.clone(), cooldown_remaining: 0.0 });
        let before = mana(world);
        world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptItemTransaction { owner: hero, owner_pid: pid, slot: 0,
            item_id: item.clone(), target: None, target_point: None, outcomes: Vec::new(), refund });
        process_outcomes(world, &mut crate::runtime::RuntimeEventVecSink::default()).unwrap();
        let left = world.read_storage::<Inventory>().get(hero).unwrap().slots[0].as_ref().map(|slot| slot.cooldown_remaining);
        (before, mana(world), left)
    };
    let (before, after, left) = use_with(&mut world, crate::comp::ItemRefund { mana: true, cooldown: true });
    assert_eq!((after, left), (before, Some(0.0)), "a refunded use spends no mana and starts no cooldown");
    let (before, after, left) = use_with(&mut world, crate::comp::ItemRefund::default());
    assert_eq!(left, Some(cooldown), "an ordinary use starts its cooldown");
    assert!(before.is_some() && after < before, "and spends its mana: {before:?} -> {after:?}");
}

/// v3 `target.type = "unit_or_point"` (巨龍): a named unit is judged as a unit target (alive, of
/// the item's team, in range), anything else as an explicit point in range.
#[test]
fn a_unit_or_point_item_takes_a_unit_or_a_point() {
    use omoba_template_ids::{MobaItemOutOfRangeConst, MobaItemTargetConst, MobaItemTargetShapeConst as Shape, MobaItemTeamConst};
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
    crate::runtime::setup_single_lane_match(&mut world, crate::runtime::SingleLaneConfig::default()).unwrap();
    let (hero, ally) = { let state = world.read_resource::<crate::runtime::MobaMatch>(); (state.heroes[0].entity.unwrap(), state.heroes[0].entity.unwrap()) };
    let at = world.read_storage::<Pos>().get(hero).unwrap().0;
    let faction = world.read_storage::<Faction>().get(hero).unwrap().clone();
    let rule = MobaItemTargetConst { shape: Shape::UnitOrPoint, range: Fixed64::from_i32(600), team: MobaItemTeamConst::Any,
        requires_visible: false, out_of_range: MobaItemOutOfRangeConst::Reject, include_magic_immune: true, exclude_heroes: false };
    let judge = |world: &World, point: Option<Vec2>, unit: Option<Entity>, explicit: bool, named: bool|
        scripted_item_target(world, rule, at, &faction, point, unit, explicit, named).ok();
    let near = at + Vec2::new(Fixed64::from_i32(300), Fixed64::ZERO);
    let far = at + Vec2::new(Fixed64::from_i32(900), Fixed64::ZERO);
    assert!(matches!(judge(&world, Some(near), None, true, false), Some(SkillTarget::Point { x, y }) if x == near.x && y == near.y), "a point in range");
    assert!(judge(&world, Some(far), None, true, false).is_none(), "a point out of range");
    assert!(matches!(judge(&world, None, Some(ally), false, true), Some(SkillTarget::Entity(unit)) if unit == ally), "a unit (its owner itself)");
    assert!(judge(&world, Some(near), Some(ally), true, true).is_none(), "not both");
    // a unit item still takes no point, a point item no unit
    let unit_only = MobaItemTargetConst { shape: Shape::Unit, ..rule };
    assert!(scripted_item_target(&world, unit_only, at, &faction, Some(near), None, true, false).is_err());
    let point_only = MobaItemTargetConst { shape: Shape::Point, ..rule };
    assert!(scripted_item_target(&world, point_only, at, &faction, None, Some(ally), false, true).is_err());
}

/// 千里大劍 (v3 target types creep, neutral, summon, courier, building; no range): an allied unit
/// anywhere on the map that is not a hero; its own user and other heroes are refused.
#[test]
fn a_non_hero_item_target_refuses_heroes_and_a_rangeless_one_reaches_anywhere() {
    use omoba_template_ids::{MobaItemOutOfRangeConst, MobaItemTargetConst, MobaItemTargetShapeConst as Shape, MobaItemTeamConst};
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
    crate::runtime::setup_single_lane_match(&mut world, crate::runtime::SingleLaneConfig::default()).unwrap();
    let hero = world.read_resource::<crate::runtime::MobaMatch>().heroes[0].entity.unwrap();
    let at = world.read_storage::<Pos>().get(hero).unwrap().0;
    let faction = world.read_storage::<Faction>().get(hero).unwrap().clone();
    let far = at + Vec2::new(Fixed64::from_i32(9000), Fixed64::ZERO);
    let unit = world.create_entity().with(Pos(far)).with(faction.clone())
        .with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO }).build();
    let rule = MobaItemTargetConst { shape: Shape::Unit, range: Fixed64::ZERO, team: MobaItemTeamConst::Ally,
        requires_visible: false, out_of_range: MobaItemOutOfRangeConst::Reject, include_magic_immune: false, exclude_heroes: true };
    let judge = |world: &World, rule: MobaItemTargetConst, target: Entity| scripted_item_target(world, rule, at, &faction, None, Some(target), false, true);
    assert!(matches!(judge(&world, rule, unit), Ok(SkillTarget::Entity(target)) if target == unit), "an allied unit 9000 away");
    assert!(judge(&world, rule, hero).is_err(), "never a hero, its user included");
    assert!(judge(&world, MobaItemTargetConst { exclude_heroes: false, ..rule }, hero).is_ok(), "heroes allowed without the flag");
    assert!(judge(&world, MobaItemTargetConst { range: Fixed64::from_i32(600), ..rule }, unit).is_err(), "a ranged one stays in range");
    // a visible-only rule: the authority judges by its vision (nothing sees 9000 away); a team
    // replica runs only accepted uses and has no vision sources, so its target passes
    let seen = MobaItemTargetConst { requires_visible: true, ..rule };
    assert!(judge(&world, seen, unit).is_err(), "unseen on the authority");
    world.insert(crate::runtime::filtered_specs::DisclosedAuthorityCombatTargets::default());
    assert!(judge(&world, seen, unit).is_ok(), "a replica follows the accepted use");
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
