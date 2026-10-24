use super::*;
use crate::runtime::input_scheme::{DirectionGesture,InputScheme,InputSchemeError,InputSchemeRules};
use specs::WorldExt;
use std::collections::BTreeSet;
fn world() -> World {
    let pool=crate::runtime::StateInitializer::create_thread_pool();
    let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    setup_single_lane_match(&mut world,SingleLaneConfig::default()).unwrap();world
}
#[test]
fn input_scheme_roster_defaults_and_owner_admission_are_atomic() {
    let world=world();let mut state=world.write_resource::<MobaMatch>();
    let owner=state.heroes[0].player_id;let other=state.heroes[1].player_id;
    for slot in &state.heroes { assert_eq!(slot.input_scheme().scheme(),InputScheme::PointAndClick);assert_eq!(slot.hero_content_id(),"training_luminary"); }
    assert_eq!(state.admit_input_scheme(owner,other,1),Err(InputSchemeError::WrongOwner));
    assert_eq!(state.admit_input_scheme(0,0,1),Err(InputSchemeError::WrongOwner));
    assert_eq!(state.admit_input_scheme(99,99,1),Err(InputSchemeError::WrongOwner));
    assert_eq!(state.admit_input_scheme(owner,owner,256),Err(InputSchemeError::InvalidScheme));
    state.admit_input_scheme(owner,owner,1).unwrap();assert_eq!(state.heroes[0].input_scheme().scheme(),InputScheme::Keyboard);
    assert_eq!(state.heroes[1].input_scheme().scheme(),InputScheme::PointAndClick);
    state.heroes[1].player_id=owner;
    assert_eq!(state.admit_input_scheme(owner,owner,0),Err(InputSchemeError::WrongOwner));
    assert_eq!(state.heroes[0].input_scheme().scheme(),InputScheme::Keyboard);
}

#[cfg(feature="kcp")]
fn direction_command(hero:Entity,press_id:u64,phase:u32,x:i32,y:i32) -> crate::runtime::PlayerInput {
    crate::runtime::PlayerInput {action:Some(crate::runtime::PlayerInputEnum::Direction(crate::game_proto::HeroDirectionInput {
        schema_version:1,source:None,sealed_source:crate::runtime::canonical_entity_id(hero),press_id,phase,x,y,
    }))}
}

#[cfg(feature="kcp")]
fn direction_life(hero:Entity) -> crate::runtime::selective::CanonicalEntityKey {
    crate::runtime::selective::CanonicalEntityKey {id:hero.id(),generation:hero.gen().id() as u32}
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_direction_wire_preserves_order_and_moves_through_real_prehook() {
    for mode_first in [true,false] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        apply_input_scheme_update(&mut world,owner,0).unwrap();
        let mode=crate::runtime::PlayerInput {action:Some(crate::runtime::PlayerInputEnum::InputScheme(
            crate::game_proto::InputSchemeUpdate {schema_version:1,scheme:1}))};
        let direction=direction_command(hero,2,1,0,1);
        world.write_resource::<PendingPlayerInputs>().inputs=if mode_first {vec![(owner,mode),(owner,direction)]}
            else {vec![(owner,direction),(owner,mode)]};
        assert!(begin_moba_match_tick(&mut world));
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
        let row=world.read_resource::<MobaMatch>().input_step.as_ref().unwrap().rows[&owner];
        assert_eq!(row.direction!=Vec2::ZERO,mode_first);
        movement_tick(&world);
        let pos=world.read_storage::<Pos>().get(hero).unwrap().0;
        assert_eq!(pos.y>Fixed64::ZERO,mode_first);
        assert_eq!(pos.x,Fixed64::ZERO);
    }
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_direction_wire_recall_priority_requires_admitted_fresh_begin() {
    for direction_first in [true,false] {for kind in ["begin","update","invalid-begin"] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        let direction=match kind {"begin"=>direction_command(hero,2,1,1,0),
            "update"=>direction_command(hero,1,2,0,1),_=>direction_command(hero,2,1,32768,0)};
        let recall=crate::runtime::PlayerInput {action:Some(crate::runtime::PlayerInputEnum::Recall(crate::game_proto::Recall {}))};
        world.write_resource::<PendingPlayerInputs>().inputs=if direction_first {vec![(owner,direction),(owner,recall)]}
            else {vec![(owner,recall),(owner,direction)]};
        assert!(begin_moba_match_tick(&mut world));
        let state=world.read_resource::<MobaMatch>();
        assert_eq!(state.heroes[0].recall.is_some(),kind!="begin","{kind}, first={direction_first}");
        assert_eq!(state.direction_for_source(owner,hero)!=Vec2::ZERO,kind=="begin");
    }}
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_direction_wire_pause_and_zero_delta_allow_only_matching_end() {
    for paused in [true,false] {for phase in [1,2,3] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        if paused {world.write_resource::<GamePause>().is_paused=true;}
        else {world.write_resource::<DeltaTime>().0=Fixed64::ZERO;}
        let (press,x)=if phase==1 {(2,1)} else if phase==2 {(1,1)} else {(1,0)};
        world.write_resource::<PendingPlayerInputs>().inputs.push((owner,direction_command(hero,press,phase,x,0)));
        assert!(!begin_moba_match_tick(&mut world));
        let state=world.read_resource::<MobaMatch>();
        assert_eq!(state.heroes[0].input_scheme.active_press(direction_life(hero)),if phase==3 {0}else{1});
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
    }}
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_direction_wire_warmup_crossing_is_admitted_at_first_playing_step() {
    for crossing in [true,false] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        {let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Warmup;
            state.heroes[0].input_scheme.neutralize();
            state.elapsed=state.config.warmup-Fixed64::from_raw(if crossing {34}else{35});}
        world.write_resource::<PendingPlayerInputs>().inputs.push((owner,direction_command(hero,2,1,1,0)));
        assert_eq!(begin_moba_match_tick(&mut world),crossing);
        let state=world.read_resource::<MobaMatch>();
        assert_eq!(state.direction_for_source(owner,hero)!=Vec2::ZERO,crossing);
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
    }
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_direction_wire_rejects_wrong_owner_client_reference_and_old_life() {
    for kind in ["wrong-owner","unsealed","dead","old-life","wide-phase"] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        let mut input=direction_command(hero,2,1,1,0);
        let sender=if kind=="wrong-owner" {world.read_resource::<MobaMatch>().heroes[1].player_id}else{owner};
        if let Some(crate::runtime::PlayerInputEnum::Direction(value))=input.action.as_mut() {
            match kind {
                "unsealed"=>{value.sealed_source=0;value.source=Some(Default::default());},
                "wide-phase"=>value.phase=257,
                "old-life"=>value.sealed_source+=1_u64<<32,
                _=>{},
            }
        }
        if kind=="dead" {world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;}
        let before=world.read_resource::<MobaMatch>().heroes[0].input_scheme.last_press(direction_life(hero));
        world.write_resource::<PendingPlayerInputs>().inputs.push((sender,input));
        assert!(begin_moba_match_tick(&mut world));
        assert_eq!(world.read_resource::<MobaMatch>().heroes[0].input_scheme.last_press(direction_life(hero)),before,"{kind}");
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
    }
}
#[test]
fn input_scheme_match_rules_lock_at_playing_time_and_survive_clone() {
    let world=world();let mut state=world.write_resource::<MobaMatch>();let owner=state.heroes[0].player_id;
    state.config.input_scheme_rules=InputSchemeRules::new(Fixed64::from_i32(3),Fixed64::from_raw(307)).unwrap();
    state.phase=MobaMatchPhase::Playing;state.elapsed=state.config.warmup+Fixed64::from_raw(3*1024-1);
    state.admit_input_scheme(owner,owner,1).unwrap();state.elapsed+=Fixed64::from_raw(1);
    assert_eq!(state.admit_input_scheme(owner,owner,0),Err(InputSchemeError::Locked));
    let mut reconnected=state.clone();assert_eq!(reconnected.admit_input_scheme(owner,owner,0),Err(InputSchemeError::Locked));
    reconnected.admit_input_scheme(owner,owner,1).unwrap();
    state.phase=MobaMatchPhase::Finished{winner:Some(0),tick:100};
    assert_eq!(state.admit_input_scheme(owner,owner,1),Err(InputSchemeError::InvalidPhase));
}
#[test]
fn input_scheme_roster_direction_is_bounded_life_scoped_and_digest_bound() {
    let mut world=world();
    let (owner,old,other)={let s=world.read_resource::<MobaMatch>();(s.heroes[0].player_id,s.heroes[0].entity.unwrap(),s.heroes[1].entity.unwrap())};
    world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
    world.write_resource::<MobaMatch>().admit_input_scheme(owner,owner,1).unwrap();
    let before=single_lane_replay_digest(&world);
    assert_eq!(world.write_resource::<MobaMatch>().admit_direction(owner,other,DirectionGesture::Begin(1),1,0),Err(InputSchemeError::InvalidSource));
    assert_eq!(world.write_resource::<MobaMatch>().admit_direction(owner,old,DirectionGesture::Begin(1),32768,0),Err(InputSchemeError::InvalidAxis));
    assert_eq!(single_lane_replay_digest(&world),before);
    world.write_resource::<MobaMatch>().admit_direction(owner,old,DirectionGesture::Begin(1),1,1).unwrap();assert_ne!(single_lane_replay_digest(&world),before);
    assert_ne!(world.read_resource::<MobaMatch>().direction_for_source(owner,old),Vec2::ZERO);
    {let mut s=world.write_resource::<MobaMatch>();s.elapsed=s.config.warmup+s.config.input_scheme_rules.direction_lease();}
    assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner,old),Vec2::ZERO);
    world.write_resource::<MobaMatch>().admit_direction(owner,old,DirectionGesture::Update(1),1,0).unwrap();
    world.write_storage::<CProperty>().get_mut(old).unwrap().hp=Fixed64::ZERO;record_moba_death(&mut world,old);
    {let s=world.read_resource::<MobaMatch>();assert_eq!(s.heroes[0].input_scheme().scheme(),InputScheme::Keyboard);assert_eq!(s.heroes[0].hero_content_id(),"training_luminary");assert_eq!(s.direction_for_source(owner,old),Vec2::ZERO);}
    world.delete_entity(old).unwrap();world.maintain();
    let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_hero(&mut world,0);let replacement=state.heroes[0].entity.unwrap();
    assert_ne!(replacement,old);assert_eq!(state.heroes[0].input_scheme().scheme(),InputScheme::Keyboard);assert_eq!(state.direction_for_source(owner,replacement),Vec2::ZERO);
    assert_eq!(state.admit_direction(owner,old,DirectionGesture::Update(1),1,0),Err(InputSchemeError::InvalidSource));
    state.admit_direction(owner,replacement,DirectionGesture::Begin(1),0,1).unwrap();assert_ne!(state.direction_for_source(owner,replacement),Vec2::ZERO);world.insert(state);
}

fn moving_world(x: i32, y: i32) -> (World, specs::Entity, u32, Fixed64) {
    let mut world = world();
    let (owner, hero) = {
        let mut state = world.write_resource::<MobaMatch>();
        state.phase = MobaMatchPhase::Playing;
        state.elapsed = state.config.warmup;
        let owner = state.heroes[0].player_id;
        let hero = state.heroes[0].entity.unwrap();
        state.admit_input_scheme(owner, owner, 1).unwrap();
        state.admit_direction(owner, hero, DirectionGesture::Begin(1), x, y).unwrap();
        (owner, hero)
    };
    world.insert(DeltaTime(Fixed64::from_raw(34)));
    world.insert(Tick(1));
    world.insert(BlockedRegions::default());
    world.write_storage::<Pos>().get_mut(hero).unwrap().0 = Vec2::ZERO;
    let direction = crate::runtime::input_scheme::normalized_axes(x, y).unwrap();
    world.write_storage::<Facing>().get_mut(hero).unwrap().0 = omoba_sim::trig::atan2(direction.y, direction.x);
    let step = world.read_storage::<CProperty>().get(hero).unwrap().msd * Fixed64::from_raw(34);
    (world, hero, owner, step)
}

fn movement_tick(world: &World) {
    crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(world);
    crate::comp::run_now::<crate::tick::hero_move_tick::Sys>(world);
}

#[test]
fn input_scheme_snapshot_keeps_pre_dispatch_controls_after_same_tick_stop() {
    let (mut world, hero, owner, _) = moving_world(1, 0);
    apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0).unwrap();
    assert!(begin_moba_match_tick(&mut world));
    let frozen = world.read_resource::<MobaMatch>().input_step.clone().unwrap();
    let row = frozen.rows.get(&owner).unwrap();
    assert_eq!(frozen.tick, world.read_resource::<Tick>().0);
    assert_eq!(row.source, hero);
    assert_eq!(row.scheme, InputScheme::Keyboard);
    assert_eq!(row.press_id, 2);
    assert!(row.navigation_reset);
    assert!(!row.attack_reset,"direction Begin must never cancel windup");
    assert_eq!(row.direction, Vec2::new(Fixed64::ONE, Fixed64::ZERO));
    movement_tick(&world);
    assert!(world.read_storage::<Pos>().get(hero).unwrap().0.x > Fixed64::ZERO);
    world.write_resource::<PendingMoveQueue>().requests.push(PendingHeroCommand {
        owner_pid: owner, queued: false, kind: PendingHeroCommandKind::Stop,
    });
    crate::runtime::drain_pending_moves(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner, hero), Vec2::ZERO);
    assert_eq!(world.read_resource::<MobaMatch>().input_step.as_ref(), Some(&frozen),
        "post-movement retirement must not rewrite this tick's consumed controls");
    world.write_resource::<Tick>().0 += 1;
    assert!(begin_moba_match_tick(&mut world));
    let next = world.read_resource::<MobaMatch>().input_step.clone().unwrap();
    assert_eq!(next.tick, frozen.tick + 1);
    assert_eq!(next.rows.get(&owner).unwrap().direction, Vec2::ZERO);
    assert_eq!(next.rows.get(&owner).unwrap().press_id, 0);
    assert!(!next.rows.get(&owner).unwrap().navigation_reset);
    world.write_resource::<Tick>().0 += 1;
    world.write_resource::<GamePause>().is_paused = true;
    assert!(!begin_moba_match_tick(&mut world));
    let paused = world.read_resource::<MobaMatch>().input_step.clone().unwrap();
    assert_eq!(paused.tick, next.tick + 1);
    assert!(paused.rows.is_empty());
}

#[test]
fn input_scheme_mode_reset_is_projected_once_and_survives_inactive_pause() {
    let (mut world,hero,owner,_)=moving_world(1,0);
    { let mut attacks=world.write_storage::<TAttack>();let attack=attacks.get_mut(hero).unwrap();
        attack.attack_phase=AttackSequencePhase::Windup;attack.asd_count=Fixed64::from_raw(-100); }
    apply_input_scheme_update(&mut world,owner,0).unwrap();
    apply_input_scheme_update(&mut world,owner,1).unwrap();
    apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0).unwrap();
    assert!(begin_moba_match_tick(&mut world));
    let row=world.read_resource::<MobaMatch>().input_step.as_ref().unwrap().rows[&owner];
    assert!(row.navigation_reset && row.attack_reset);
    assert_eq!(world.read_resource::<AttackCancelFxQueue>().pending.len(),1);
    world.write_resource::<Tick>().0+=1;
    assert!(begin_moba_match_tick(&mut world));
    assert!(!world.read_resource::<MobaMatch>().input_step.as_ref().unwrap().rows[&owner].attack_reset);
    world.write_resource::<GamePause>().is_paused=true;
    apply_input_scheme_update(&mut world,owner,0).unwrap();
    apply_input_scheme_update(&mut world,owner,1).unwrap();
    world.write_resource::<Tick>().0+=1;
    assert!(!begin_moba_match_tick(&mut world));
    world.write_resource::<GamePause>().is_paused=false;
    world.write_resource::<Tick>().0+=1;
    assert!(begin_moba_match_tick(&mut world));
    assert!(world.read_resource::<MobaMatch>().input_step.as_ref().unwrap().rows[&owner].attack_reset);
}

#[test]
fn input_scheme_step_projection_never_attaches_old_controls_to_replacement_life() {
    let (mut world,hero,owner,_)=moving_world(1,0);
    assert!(begin_moba_match_tick(&mut world));
    { let state=world.read_resource::<MobaMatch>();
        assert!(crate::runtime::input_scheme_transport::project(&state,&state.heroes[0]).unwrap().direction_step.is_some()); }
    world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;
    record_moba_death(&mut world,hero);
    { let state=world.read_resource::<MobaMatch>();
        assert!(crate::runtime::input_scheme_transport::project(&state,&state.heroes[0]).unwrap().direction_step.is_none()); }
    world.delete_entity(hero).unwrap();world.maintain();
    let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_hero(&mut world,0);
    assert_ne!(state.heroes[0].entity,Some(hero));
    assert!(crate::runtime::input_scheme_transport::project(&state,&state.heroes[0]).unwrap().direction_step.is_none());
    world.insert(state);world.write_resource::<Tick>().0+=1;
    assert!(begin_moba_match_tick(&mut world));
    let state=world.read_resource::<MobaMatch>();
    let step=crate::runtime::input_scheme_transport::project(&state,&state.heroes[0]).unwrap().direction_step.unwrap();
    assert_eq!((step.direction_x_raw,step.direction_y_raw,step.press_id),(0,0,0));
    assert_eq!(state.heroes[0].player_id,owner);
}

#[test]
fn input_scheme_snapshot_admits_only_current_unique_live_owned_roster_bodies() {
    let (mut world, hero, owner, _) = moving_world(1, 0);
    let mut state = world.remove::<MobaMatch>().unwrap();
    capture_input_scheme_step(&world, &mut state);
    assert!(state.input_step.as_ref().unwrap().rows.contains_key(&owner));
    state.heroes[1].player_id = owner;
    capture_input_scheme_step(&world, &mut state);
    assert!(!state.input_step.as_ref().unwrap().rows.contains_key(&owner));
    state.heroes[1].player_id = owner + 100;
    world.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = owner + 1;
    capture_input_scheme_step(&world, &mut state);
    assert!(!state.input_step.as_ref().unwrap().rows.contains_key(&owner));
    world.write_storage::<PlayerOwner>().get_mut(hero).unwrap().player_id = owner;
    world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ZERO;
    capture_input_scheme_step(&world, &mut state);
    assert!(!state.input_step.as_ref().unwrap().rows.contains_key(&owner));
    world.write_storage::<CProperty>().get_mut(hero).unwrap().hp = Fixed64::ONE;
    state.heroes[0].recall = Some((Vec2::ZERO, Fixed64::from_i32(20)));
    capture_input_scheme_step(&world, &mut state);
    assert_eq!(state.input_step.as_ref().unwrap().rows[&owner].direction, Vec2::ZERO);
    world.insert(state);
}

#[test]
fn input_scheme_authority_digest_binds_private_commitment_and_step_controls() {
    let world = world();
    world.write_resource::<MobaMatch>().phase=MobaMatchPhase::Playing;
    let (hero, target) = { let state = world.read_resource::<MobaMatch>();
        (state.heroes[0].entity.unwrap(), state.heroes[1].entity.unwrap()) };
    let before = single_lane_replay_digest(&world);
    world.write_storage::<TAttack>().get_mut(hero).unwrap().committed_target = Some(target);
    let committed = single_lane_replay_digest(&world);
    assert_ne!(committed, before);
    world.write_storage::<TAttack>().get_mut(hero).unwrap().committed_target = None;
    assert_eq!(single_lane_replay_digest(&world), before);
    world.write_resource::<MobaMatch>().input_step = Some(crate::runtime::input_scheme::InputSchemeStepSnapshot {
        tick: 9, rows: BTreeMap::new(),
    });
    assert_ne!(single_lane_replay_digest(&world), before);
    world.write_resource::<GamePause>().is_paused=true;
    let paused=single_lane_replay_digest(&world);
    world.write_resource::<MobaMatch>().input_step.as_mut().unwrap().tick=10;
    assert_eq!(single_lane_replay_digest(&world),paused,"inactive wire tick is not a gameplay clock");
    world.write_storage::<TAttack>().get_mut(hero).unwrap().committed_target=Some(target);
    assert_ne!(single_lane_replay_digest(&world),paused,"pause still binds real private commitments");
}

#[test]
fn input_scheme_committed_attack_survives_direction_press_without_retargeting() {
    use crate::runtime::native::combat_acquisition::test_support::{CombatFixture, IndexedBody};
    // Both the successful impact and every invalidated original life must keep
    // its attack clock; a nearer replacement is never an impact fallback.
    for invalidation in ["none", "hidden", "dead", "recycled", "range"] {
        let mut fix = CombatFixture::legacy();
        fix.owner(7);
        fix.world.insert(GamePause::default());
        fix.world.insert(AttackCancelFxQueue::default());
        fix.world.insert(DeltaTime(Fixed64::from_raw(34)));
        fix.world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
        let original = fix.spawn_units(IndexedBody::Creep, 24, 2, 100);
        let mut state = stop_command_match(fix.hero, MobaMatchPhase::Playing);
        state.admit_input_scheme(7, 7, 1).unwrap();
        fix.world.insert(state);
        fix.commit(1, &[original]);
        fix.command(HeroCommand::AttackTarget { target: original, chase_origin: None });
        fix.rebuild();
        crate::comp::run_now::<crate::tick::hero_tick::Sys>(&fix.world);
        let before = *fix.world.read_storage::<TAttack>().get(fix.hero).unwrap();
        assert_eq!(before.attack_phase, AttackSequencePhase::Windup);
        assert_eq!(before.committed_target, Some(original));
        assert!(before.asd_count < Fixed64::ZERO);
        // Private ECS identity cannot escape through snapshots or be injected.
        let mut encoded = serde_json::to_value(before).unwrap();
        assert!(encoded.get("committed_target").is_none());
        encoded["committed_target"] = serde_json::json!({"id": 99, "generation": 1});
        assert!(serde_json::from_value::<TAttack>(encoded).unwrap().committed_target.is_none());
        fix.world.write_resource::<Vec<Outcome>>().clear();
        apply_direction_input(&mut fix.world, 7, fix.hero, DirectionGesture::Begin(1), 1, 0).unwrap();
        let after = *fix.world.read_storage::<TAttack>().get(fix.hero).unwrap();
        assert_eq!(after.asd_count, before.asd_count);
        assert_eq!(after.attack_seq, before.attack_seq);
        assert_eq!(after.committed_target, Some(original));
        assert!(fix.world.read_resource::<AttackCancelFxQueue>().pending.is_empty());
        let nearer = fix.spawn_units(IndexedBody::Creep, 12, 2, 100);
        fix.commit(1, &[nearer]);
        match invalidation {
            "hidden" => {
                fix.world.remove::<crate::runtime::TeamVisibilityRuntime>();
                fix.commit(1, &[nearer]);
            }
            "dead" => fix.world.write_storage::<CProperty>().get_mut(original).unwrap().hp = Fixed64::ZERO,
            "recycled" => {
                fix.delete(original);
                let replacement = fix.spawn_units(IndexedBody::Creep, 24, 2, 100);
                assert_ne!(replacement, original);
                fix.commit(1, &[replacement]);
            }
            "range" => fix.world.write_storage::<Pos>().get_mut(original).unwrap().0.x = Fixed64::from_i32(100),
            _ => {}
        }
        fix.rebuild();
        let start = fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0;
        for _ in 0..20 {
            movement_tick(&fix.world);
            crate::comp::run_now::<crate::tick::hero_tick::Sys>(&fix.world);
            assert_eq!(fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0, start,
                "direction must wait through impact: {invalidation}");
            if fix.world.read_storage::<TAttack>().get(fix.hero).unwrap().attack_phase == AttackSequencePhase::Backswing { break; }
        }
        let attack = *fix.world.read_storage::<TAttack>().get(fix.hero).unwrap();
        assert_eq!(attack.attack_phase, AttackSequencePhase::Backswing, "{invalidation}");
        assert!(attack.committed_target.is_none());
        assert!(attack.asd_count > Fixed64::ZERO && attack.asd_count < attack.asd.v);
        let hits: Vec<_> = fix.world.read_resource::<Vec<Outcome>>().iter().filter_map(|outcome| {
            if let Outcome::ProjectileLine2 { source: Some(source), target, .. } = outcome {
                (*source == fix.hero).then_some(*target)
            } else { None }
        }).collect();
        assert_eq!(hits, if invalidation == "none" { vec![Some(original)] } else { vec![] }, "{invalidation}");
        movement_tick(&fix.world);
        assert!(fix.world.read_storage::<Pos>().get(fix.hero).unwrap().0.x > start.x,
            "backswing must allow direction movement: {invalidation}");
    }
}

#[test]
fn input_scheme_consumer_moves_new_hero_without_point_command_and_bounds_diagonals() {
    for (x, y) in [(1,0),(-1,0),(0,1),(0,-1),(1,1),(-1,1),(1,-1),(-1,-1),(32767,32767)] {
        let (world, hero, _, step) = moving_world(x, y);
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_none());
        movement_tick(&world);
        let pos = world.read_storage::<Pos>().get(hero).unwrap().0;
        let squared = i128::from(pos.x.raw()).pow(2) + i128::from(pos.y.raw()).pow(2);
        let budget = i128::from(step.raw()).pow(2);
        assert!(squared <= budget, "overspeed for {x},{y}: {pos:?}");
        assert!(squared * 1000 > budget * 995, "unexpectedly slow for {x},{y}: {pos:?}");
        assert!(world.read_storage::<HeroCommandQueue>().get(hero).is_some());
    }
}

#[test]
fn input_scheme_consumer_neutral_and_expiry_retire_previous_waypoint() {
    for neutral in [true, false] {
        let (world, hero, owner, _) = moving_world(1, 0);
        movement_tick(&world);
        let before = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert!(before.x > Fixed64::ZERO);
        {
            let mut state = world.write_resource::<MobaMatch>();
            if neutral { state.admit_direction(owner, hero, DirectionGesture::End(1), 0, 0).unwrap(); }
            else { let lease = state.config.input_scheme_rules.direction_lease(); state.elapsed += lease; }
        }
        movement_tick(&world);
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, before);
        assert!(world.read_storage::<MoveTarget>().get(hero).is_none());
        assert!(world.read_resource::<crate::runtime::input_scheme::DirectionalStepPlan>().directions.is_empty());
    }
}

#[test]
fn input_scheme_consumer_sweeps_thin_public_wall_and_slides_without_pathfinding() {
    for diagonal in [false, true] {
        let (mut world, hero, _, _) = moving_world(1, i32::from(diagonal));
        world.insert(BlockedRegions(vec![BlockedRegion {
            name: "thin-wall".into(),
            points: vec![vek::Vec2::new(25.0,-100.0),vek::Vec2::new(26.0,-100.0),vek::Vec2::new(26.0,100.0),vek::Vec2::new(25.0,100.0)],
        }]));
        movement_tick(&world);
        let pos = world.read_storage::<Pos>().get(hero).unwrap().0;
        assert_eq!(pos.x, Fixed64::ZERO);
        assert_eq!(pos.y > Fixed64::ZERO, diagonal);
    }
}

#[test]
fn input_scheme_consumer_root_and_attack_windup_pause_held_direction() {
    for root in [true, false] {
        let (world, hero, _, _) = moving_world(1, 0);
        if root {
            world.write_resource::<crate::runtime::BuffStore>().add(hero,
                omb_script_abi::buff_ids::BuffId::Root.as_str(), Fixed64::from_i32(5), serde_json::json!({}));
        } else {
            world.write_storage::<TAttack>().get_mut(hero).unwrap().asd_count = Fixed64::from_raw(-1);
        }
        movement_tick(&world);
        assert_eq!(world.read_storage::<Pos>().get(hero).unwrap().0, Vec2::ZERO);
        if !root {
            world.write_storage::<TAttack>().get_mut(hero).unwrap().asd_count = Fixed64::ZERO;
            movement_tick(&world);
            assert!(world.read_storage::<Pos>().get(hero).unwrap().0.x > Fixed64::ZERO);
        }
    }
}

#[test]
fn input_scheme_consumer_attack_arbitration_respects_disclosure_and_ready_clock() {
    use crate::runtime::native::combat_acquisition::test_support::{CombatFixture, IndexedBody};
    for (clock, disclosed, in_range, expect_move) in [
        (-100, true, true, false), (-1, true, true, false),
        (0, true, true, true), (1024, true, true, false),
        (1024, false, true, true), (1024, true, false, true),
    ] {
        let mut fix = CombatFixture::legacy();
        fix.owner(7);
        let enemy = fix.spawn_units(IndexedBody::Creep, if in_range {24} else {100}, 2, 100);
        fix.command(HeroCommand::AttackTarget { target: enemy, chase_origin: None });
        let mut state = stop_command_match(fix.hero, MobaMatchPhase::Playing);
        state.admit_input_scheme(7, 7, 1).unwrap();
        state.admit_direction(7, fix.hero, DirectionGesture::Begin(1), 1, 0).unwrap();
        fix.world.insert(state);
        fix.world.insert(crate::runtime::input_scheme::DirectionalStepPlan::default());
        fix.world.insert(DeltaTime(Fixed64::from_raw(34)));
        if disclosed { fix.commit(1, &[enemy]); } else { fix.commit(1, &[]); }
        fix.world.write_storage::<TAttack>().get_mut(fix.hero).unwrap().asd_count = Fixed64::from_raw(clock);
        fix.rebuild();
        crate::comp::run_now::<crate::tick::hero_command_tick::Sys>(&fix.world);
        assert_eq!(fix.world.read_storage::<MoveTarget>().get(fix.hero).is_some(), expect_move,
            "clock={clock}, disclosed={disclosed}, in_range={in_range}");
        let active = fix.world.read_storage::<HeroCommandQueue>().get(fix.hero).unwrap().active;
        assert_eq!(active.is_some(), disclosed, "hidden targets must retire, legal targets remain");
        // Command arbitration predicts attack readiness without advancing its clock.
        assert_eq!(fix.world.read_storage::<TAttack>().get(fix.hero).unwrap().asd_count.raw(), clock);
    }
}

#[cfg(feature = "kcp")]
#[test]
fn combat_preferences_reaches_roster_through_real_match_input_gate_and_dispatcher() {
    let mut world = world();
    world.insert(DeltaTime(Fixed64::from_raw(34)));
    let owner = {
        let mut state = world.write_resource::<MobaMatch>();
        state.phase = MobaMatchPhase::Playing;
        state.elapsed = state.config.warmup;
        state.heroes[0].player_id
    };
    let update = crate::game_proto::CombatPreferencesUpdate {
        schema_version: 1, acquisition: 1, idle_auto_attack: false,
    };
    world.write_resource::<PendingPlayerInputs>().inputs.push((owner, crate::runtime::PlayerInput {
        action: Some(crate::runtime::PlayerInputEnum::CombatPreferences(update)),
    }));
    assert!(begin_moba_match_tick(&mut world));
    assert_eq!(world.read_resource::<PendingPlayerInputs>().inputs.len(), 1,
        "valid owner setting must survive the real match input allowlist");
    crate::comp::run_now::<crate::tick::player_input_tick::Sys>(&world);
    let state = world.read_resource::<MobaMatch>();
    let preferences = state.heroes[0].combat_preferences();
    assert_eq!(preferences.acquisition().raw(), 1);
    assert!(!preferences.idle_auto_attack());
    assert_eq!(state.heroes[1].combat_preferences().acquisition().raw(), 0);
    assert!(state.heroes[1].combat_preferences().idle_auto_attack());
    assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
}

#[test]
fn input_scheme_world_press_replaces_navigation_but_preserves_committed_attack() {
    use crate::runtime::input_scheme::DirectionAdmission;
    let (mut world, hero, owner, _) = moving_world(1, 0);
    world.write_storage::<HeroCommandQueue>().insert(hero, HeroCommandQueue {
        active: Some(HeroCommand::HoldPosition), queued: vec![],
    }).unwrap();
    world.write_resource::<MobaMatch>().heroes[0].recall=Some((Vec2::ZERO,Fixed64::from_i32(20)));
    {
        let mut attacks=world.write_storage::<TAttack>();let attack=attacks.get_mut(hero).unwrap();
        attack.attack_phase=AttackSequencePhase::Windup;attack.asd_count=Fixed64::from_raw(-100);attack.attack_seq=9;
    }
    let before=single_lane_replay_digest(&world);
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),32768,0),Err(InputSchemeError::InvalidAxis));
    assert_eq!(single_lane_replay_digest(&world),before);
    assert_eq!(apply_direction_input(&mut world,owner+100,hero,DirectionGesture::Begin(2),1,0),Err(InputSchemeError::InvalidSource));
    assert_eq!(single_lane_replay_digest(&world),before);
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(1),1,1),Ok(DirectionAdmission::Updated));
    assert!(world.read_resource::<MobaMatch>().is_recalling(hero));
    assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,Some(HeroCommand::HoldPosition));
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().asd_count.raw(),-100);
    assert!(world.read_resource::<AttackCancelFxQueue>().pending.is_empty());
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0),Ok(DirectionAdmission::Began));
    assert!(!world.read_resource::<MobaMatch>().is_recalling(hero));
    assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,None);
    assert!(world.read_resource::<AttackCancelFxQueue>().pending.is_empty());
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().attack_phase,AttackSequencePhase::Windup);
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().asd_count.raw(),-100);
    apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(2),1,0).unwrap();
    assert!(world.read_resource::<AttackCancelFxQueue>().pending.is_empty());
    let current=single_lane_replay_digest(&world);
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::End(1),0,0),Err(InputSchemeError::RetiredGesture));
    assert_eq!(single_lane_replay_digest(&world),current);
    world.write_resource::<GamePause>().is_paused=true;
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(2),1,0),Err(InputSchemeError::InvalidPhase));
    apply_direction_input(&mut world,owner,hero,DirectionGesture::End(2),0,0).unwrap();
    assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner,hero),Vec2::ZERO);
}

#[test]
fn input_scheme_retired_release_does_not_remove_later_point_waypoint() {
    let (mut world,hero,owner,_)=moving_world(1,0);
    world.write_resource::<MobaMatch>().neutralize_direction(owner,hero);
    let waypoint=Vec2::new(Fixed64::from_i32(90),Fixed64::from_i32(20));
    world.write_storage::<MoveTarget>().insert(hero,MoveTarget(waypoint)).unwrap();
    apply_direction_input(&mut world,owner,hero,DirectionGesture::End(1),0,0).unwrap();
    assert_eq!(world.read_storage::<MoveTarget>().get(hero).unwrap().0,waypoint);
    world.write_resource::<DeltaTime>().0=Fixed64::ZERO;
    let before=single_lane_replay_digest(&world);
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0),Err(InputSchemeError::InvalidPhase));
    assert_eq!(single_lane_replay_digest(&world),before);
    assert_eq!(world.read_storage::<MoveTarget>().get(hero).unwrap().0,waypoint);
}

#[test]
fn input_scheme_world_stop_and_hold_retire_gesture_without_heartbeat_resurrection() {
    for kind in [PendingHeroCommandKind::Stop,PendingHeroCommandKind::HoldPosition,
        PendingHeroCommandKind::MoveTo{pos:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO)},
        PendingHeroCommandKind::AttackMove{pos:Vec2::new(Fixed64::from_i32(100),Fixed64::ZERO)}] {
        let (mut world,hero,owner,_)=moving_world(1,0);
        world.write_resource::<PendingMoveQueue>().requests.push(PendingHeroCommand{owner_pid:owner,queued:false,kind});
        crate::runtime::drain_pending_moves(&mut world);
        assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner,hero),Vec2::ZERO);
        assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(1),1,0),Err(InputSchemeError::RetiredGesture));
        apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0).unwrap();
        assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,None);
        movement_tick(&world);
        assert!(world.read_storage::<Pos>().get(hero).unwrap().0.x>Fixed64::ZERO);
    }
}

#[cfg(feature = "kcp")]
#[test]
fn input_scheme_world_recall_retires_held_press_through_real_match_input_gate() {
    let (mut world,hero,owner,_)=moving_world(1,0);
    world.write_resource::<PendingPlayerInputs>().inputs.push((owner,crate::runtime::PlayerInput {
        action:Some(crate::runtime::PlayerInputEnum::Recall(crate::game_proto::Recall {})),
    }));
    assert!(begin_moba_match_tick(&mut world));
    assert!(world.read_resource::<MobaMatch>().is_recalling(hero));
    assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner,hero),Vec2::ZERO);
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(1),1,0),Err(InputSchemeError::RetiredGesture));
    assert!(world.read_resource::<MobaMatch>().is_recalling(hero));
    apply_direction_input(&mut world,owner,hero,DirectionGesture::Begin(2),1,0).unwrap();
    assert!(!world.read_resource::<MobaMatch>().is_recalling(hero));
}

#[test]
fn input_scheme_owner_projection_keeps_dead_identity_and_filters_opponents() {
    use crate::runtime::input_scheme_transport::for_owner;
    let mut world=world();
    let (owner,other,hero,enemy,team)={let state=world.read_resource::<MobaMatch>();
        (state.heroes[0].player_id,state.heroes[1].player_id,state.heroes[0].entity.unwrap(),
            state.heroes[1].entity.unwrap(),state.config.teams[0])};
    world.write_resource::<MobaMatch>().admit_input_scheme(owner,owner,1).unwrap();
    let project=|world:&World,visible:&BTreeSet<u64>| {
        crate::runtime::combat_transport::project(&world.read_resource::<MobaMatch>(),team,visible,1).unwrap()
    };
    let hidden=project(&world,&BTreeSet::new());
    assert_eq!(for_owner(Some(&hidden),1,owner).unwrap().hero_content_id,"training_luminary");
    assert!(for_owner(Some(&hidden),1,other).is_none());
    let revealed=project(&world,&BTreeSet::from([crate::runtime::canonical_entity_id(enemy)]));
    assert!(for_owner(Some(&revealed),1,other).is_some());
    world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;
    record_moba_death(&mut world,hero);
    let dead=project(&world,&BTreeSet::new());
    let dead_owner=for_owner(Some(&dead),1,owner).unwrap();
    assert_eq!(dead_owner.hero_content_id,"training_luminary");
    assert_eq!(dead_owner.scheme,InputScheme::Keyboard);
    assert_eq!(dead_owner.last_press_id,0);
    assert!(for_owner(Some(&dead),2,owner).is_none());
}

#[test]
fn input_scheme_world_mode_change_is_atomic_and_idempotent_retries_are_inert() {
    let (mut world,hero,owner,_)=moving_world(1,0);
    world.write_storage::<HeroCommandQueue>().insert(hero,HeroCommandQueue {
        active:Some(HeroCommand::HoldPosition),queued:vec![],
    }).unwrap();
    world.write_resource::<MobaMatch>().heroes[0].recall=Some((Vec2::ZERO,Fixed64::from_i32(20)));
    let before=single_lane_replay_digest(&world);
    assert_eq!(apply_input_scheme_update(&mut world,owner,256),Err(InputSchemeError::InvalidScheme));
    assert_eq!(single_lane_replay_digest(&world),before);
    assert_eq!(apply_input_scheme_update(&mut world,owner,1),Ok(false));
    assert_eq!(single_lane_replay_digest(&world),before);
    assert_eq!(apply_input_scheme_update(&mut world,owner,0),Ok(true));
    assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero).unwrap().active,None);
    assert!(!world.read_resource::<MobaMatch>().is_recalling(hero));
    assert_eq!(world.read_resource::<MobaMatch>().direction_for_source(owner,hero),Vec2::ZERO);
    apply_input_scheme_update(&mut world,owner,1).unwrap();
    assert_eq!(apply_direction_input(&mut world,owner,hero,DirectionGesture::Update(1),1,0),Err(InputSchemeError::RetiredGesture));
    {
        let mut state=world.write_resource::<MobaMatch>();
        state.elapsed=state.config.warmup+state.config.input_scheme_rules.lock_after();
    }
    let locked=single_lane_replay_digest(&world);
    assert_eq!(apply_input_scheme_update(&mut world,owner,0),Err(InputSchemeError::Locked));
    assert_eq!(single_lane_replay_digest(&world),locked);
    assert_eq!(apply_input_scheme_update(&mut world,owner,1),Ok(false));
    assert_eq!(single_lane_replay_digest(&world),locked);
}

#[cfg(feature="kcp")]
#[test]
fn input_scheme_wire_update_uses_real_match_gate_during_warmup_pause_death_and_lock() {
    for scenario in [0,1,2,3] {
        let mut world=world();world.insert(DeltaTime(Fixed64::from_raw(34)));
        let (owner,hero)={let mut state=world.write_resource::<MobaMatch>();
            if scenario!=0 {state.phase=MobaMatchPhase::Playing;state.elapsed=state.config.warmup;}
            if scenario==3 {let lock=state.config.input_scheme_rules.lock_after();state.elapsed+=lock-Fixed64::from_raw(34);}
            (state.heroes[0].player_id,state.heroes[0].entity.unwrap())};
        if scenario==1 {world.write_resource::<GamePause>().is_paused=true;}
        if scenario==2 {
            world.write_storage::<CProperty>().get_mut(hero).unwrap().hp=Fixed64::ZERO;
            record_moba_death(&mut world,hero);
        }
        world.write_resource::<PendingPlayerInputs>().inputs.push((owner,crate::runtime::PlayerInput {
            action:Some(crate::runtime::PlayerInputEnum::InputScheme(crate::game_proto::InputSchemeUpdate {schema_version:1,scheme:1})),
        }));
        let _=begin_moba_match_tick(&mut world);
        let state=world.read_resource::<MobaMatch>();
        assert_eq!(state.heroes[0].input_scheme().scheme(),if scenario==3 {InputScheme::PointAndClick}else{InputScheme::Keyboard},"scenario {scenario}");
        assert_eq!(state.heroes[1].input_scheme().scheme(),InputScheme::PointAndClick);
        assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
        if scenario==2 {assert!(state.heroes[0].entity.is_none());}
    }
}
