use super::*;
use crate::runtime::combat_controls::AttackAcquisition;
use specs::WorldExt;
use std::collections::BTreeSet;

fn world() -> World {
    let pool = crate::runtime::StateInitializer::create_thread_pool();
    let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    setup_single_lane_match(&mut world, SingleLaneConfig::default()).unwrap();
    world
}
#[test]
fn minion_cadence_spawned_profiles_modifiers_and_committed_public_periods() {
    let definitions=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap().minion_wave;
    for definition in definitions {
        let mut w=world();let mut state=w.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;state.config.warmup=Fixed64::ZERO;state.next_wave_at=Fixed64::from_i32(10000);state.config.map_id=Some("three_lane_training".into());state.config.authored_minion_waves=true;
        for _ in 0..3{state.spawn_wave(&mut w);}
        let actor=state.units.values().find(|u|u.team==0&&w.read_storage::<Unit>().get(u.entity).is_some_and(|b|b.id==definition.id)).unwrap().entity;let enemy=state.heroes[1].entity.unwrap();state.units.retain(|_,u|u.entity==actor||u.entity==enemy);let key=entity_key(actor);let id=crate::runtime::canonical_entity_id(actor);let base=Fixed64::from_raw(i64::from(definition.attack_interval_ms)*1024/1000);
        assert_eq!(state.units[&key].base_attack_interval,base);w.write_storage::<Pos>().get_mut(actor).unwrap().0=Vec2::ZERO;w.write_storage::<Pos>().get_mut(enemy).unwrap().0=Vec2::new(Fixed64::from_i32(50),Fixed64::ZERO);w.write_resource::<crate::runtime::TeamVisibilityRuntime>().ensure_team(1);w.write_resource::<crate::runtime::TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(crate::runtime::canonical_entity_id(enemy));w.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);w.insert(state);
        w.write_resource::<crate::runtime::BuffStore>().add(actor,"arbitrary_cadence",Fixed64::from_i32(10),serde_json::json!({"attack_speed_multiplier":2048}));assert!(begin_moba_match_tick(&mut w));let first=w.read_resource::<MobaMatch>().units[&key].attack_duration;assert_eq!(first,base/Fixed64::from_i32(2));w.write_resource::<crate::runtime::BuffStore>().remove(actor,"arbitrary_cadence");assert!(begin_moba_match_tick(&mut w));assert_eq!(w.read_resource::<MobaMatch>().units[&key].attack_duration,first);
        for _ in 0..640 {if w.read_resource::<MobaMatch>().units[&key].attack_sequence>=2{break;}assert!(begin_moba_match_tick(&mut w));}
        let s=w.read_resource::<MobaMatch>();assert_eq!(s.units[&key].attack_sequence,2);assert_eq!(s.units[&key].attack_duration,base);assert_eq!(s.disclosed_unit_attack_states(false)[&id].duration_raw,base.raw());
    }
}

#[test]
fn npc_damage_lane_commits_consume_once_refusals_preserve_and_buildings_ignore() {
    for mode in 0..5 {
        let mut world=world();let mut state=world.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;state.config.warmup=Fixed64::ZERO;state.next_wave_at=Fixed64::from_i32(10000);state.spawn_wave(&mut world);
        let actor=state.units.values().find(|u|u.team==0&&if mode==4{u.role==LaneRole::Tower}else{u.role==LaneRole::Creep}).unwrap().entity;let enemy=state.heroes[1].entity.unwrap();state.units.retain(|_,u|u.entity==actor||u.entity==enemy);
        world.write_storage::<Unit>().get_mut(actor).unwrap().base_damage=100;world.write_storage::<Pos>().get_mut(actor).unwrap().0=Vec2::ZERO;world.write_storage::<Pos>().get_mut(enemy).unwrap().0=Vec2::new(Fixed64::from_i32(if mode==2{600}else{50}),Fixed64::ZERO);
        world.write_resource::<crate::runtime::TeamVisibilityRuntime>().ensure_team(1);world.write_resource::<crate::runtime::TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(crate::runtime::canonical_entity_id(enemy));
        {let mut b=world.write_resource::<crate::runtime::BuffStore>();b.add(actor,"arbitrary_outgoing",Fixed64::from_i32(10),serde_json::json!({"preattack_bonus_damage":20480,"totaldamageoutgoing_percentage":512}));b.arm_next_attack_bonus(actor,Fixed64::from_i32(30));if mode==1{b.add(actor,"stun",Fixed64::ONE,serde_json::json!({}));}}
        if mode==3{state.units.get_mut(&entity_key(actor)).unwrap().attack_remaining=Fixed64::ONE;}world.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);world.write_resource::<Vec<DamageInstance>>().clear();world.insert(state);assert!(begin_moba_match_tick(&mut world));
        let packets=world.read_resource::<Vec<DamageInstance>>();let own:Vec<_>=packets.iter().filter(|p|p.source.source_entity==actor).collect();
        if matches!(mode,1|2|3){assert!(own.is_empty());}else{assert_eq!(own.len(),1);assert_eq!(own[0].damage_types.physical,Fixed64::from_i32(if mode==4{100}else{210}));}drop(packets);
        let b=world.read_resource::<crate::runtime::BuffStore>();assert_eq!(b.next_attack_bonus(actor),if mode==0{Fixed64::ZERO}else{Fixed64::from_i32(30)});assert!(b.has(actor,"arbitrary_outgoing"));drop(b);
        if mode==0 {for (remove,expected) in [(false,180),(true,100)]{if remove{world.write_resource::<crate::runtime::BuffStore>().remove(actor,"arbitrary_outgoing");}world.write_resource::<MobaMatch>().units.get_mut(&entity_key(actor)).unwrap().attack_remaining=Fixed64::ZERO;world.write_resource::<Vec<DamageInstance>>().clear();world.write_resource::<Tick>().0+=1;assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<Vec<DamageInstance>>().iter().find(|p|p.source.source_entity==actor).unwrap().damage_types.physical,Fixed64::from_i32(expected));}}
    }
}

#[test]
fn npc_attack_lane_modifiers_commit_period_range_cap_pause_and_building_rules() {
    fn fixture(building:bool,distance:i32)->(World,Entity) {
        let mut world=world();let mut state=world.remove::<MobaMatch>().unwrap();state.phase=MobaMatchPhase::Playing;state.config.warmup=Fixed64::ZERO;state.next_wave_at=Fixed64::from_i32(10000);state.spawn_wave(&mut world);
        let actor=state.units.values().find(|u|u.team==0&&if building{u.role==LaneRole::Tower}else{u.role==LaneRole::Creep}).unwrap().entity;
        let enemy=state.heroes[1].entity.unwrap();state.units.retain(|_,u|u.entity==actor||u.entity==enemy);
        world.write_storage::<Unit>().get_mut(actor).unwrap().attack_range=Fixed64::from_i32(100);
        world.write_storage::<Pos>().get_mut(actor).unwrap().0=Vec2::ZERO;world.write_storage::<Pos>().get_mut(enemy).unwrap().0=Vec2::new(Fixed64::from_i32(distance),Fixed64::ZERO);
        world.write_resource::<crate::runtime::TeamVisibilityRuntime>().ensure_team(1);world.write_resource::<crate::runtime::TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(crate::runtime::canonical_entity_id(enemy));
        world.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);world.insert(state);(world,actor)
    }
    for (payload,expected) in [(serde_json::json!({"attackspeed_bonus_constant":102400}),512),
        (serde_json::json!({"attackspeed_bonus_constant":-51200}),2048),
        (serde_json::json!({"fixed_attack_rate":4096}),256),(serde_json::json!({"attack_speed_multiplier":2048}),512),
        (serde_json::json!({"fixed_attack_rate":i64::MAX}),1)] {
        let (mut world,actor)=fixture(false,50);world.write_resource::<crate::runtime::BuffStore>().add(actor,"arbitrary_tempo",Fixed64::from_i32(10),payload.clone());
        assert!(begin_moba_match_tick(&mut world));let key=entity_key(actor);let id=crate::runtime::canonical_entity_id(actor);
        assert_eq!(world.read_resource::<MobaMatch>().units[&key].attack_duration.raw(),expected);
        let phase=world.read_resource::<MobaMatch>().disclosed_unit_attack_states(false)[&id];assert!(phase.is_valid());assert_eq!(phase.duration_raw,expected);assert_eq!(phase.elapsed_raw,0);
        let digest=single_lane_replay_digest(&world);world.write_resource::<MobaMatch>().units.get_mut(&key).unwrap().attack_duration+=Fixed64::from_raw(1);assert_ne!(single_lane_replay_digest(&world),digest);world.write_resource::<MobaMatch>().units.get_mut(&key).unwrap().attack_duration=Fixed64::from_raw(expected);
        world.write_resource::<GamePause>().is_paused=true;assert!(!begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&key].attack_remaining.raw(),expected);assert!(world.read_resource::<MobaMatch>().disclosed_unit_attack_states(true)[&id].paused);world.write_resource::<GamePause>().is_paused=false;
        world.write_resource::<crate::runtime::BuffStore>().remove(actor,"arbitrary_tempo");world.write_resource::<Tick>().0+=1;
        if expected>16 {assert!(begin_moba_match_tick(&mut world));let state=world.read_resource::<MobaMatch>();assert_eq!(state.units[&key].attack_sequence,1);let phase=state.disclosed_unit_attack_states(false)[&id];assert_eq!((phase.duration_raw,phase.elapsed_raw),(expected,16));drop(state);world.write_resource::<MobaMatch>().units.get_mut(&key).unwrap().attack_remaining=Fixed64::from_raw(16);world.write_resource::<Tick>().0+=1;}
        assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&key].attack_sequence,2);assert_eq!(world.read_resource::<MobaMatch>().units[&key].attack_duration,Fixed64::ONE);
        let (mut world,tower)=fixture(true,50);world.write_resource::<crate::runtime::BuffStore>().add(tower,"same_tempo",Fixed64::from_i32(10),payload);assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(tower)].attack_duration,Fixed64::ONE);
    }
    for (payload,expected) in [(serde_json::json!({}),0),(serde_json::json!({"attack_range_bonus":102400}),1),
        (serde_json::json!({"attack_range_bonus":102400,"max_attack_range":102400}),0),
        (serde_json::json!({"attack_range_bonus":-204800}),0)] {
        let (mut world,actor)=fixture(false,150);world.write_resource::<crate::runtime::BuffStore>().add(actor,"arbitrary_range",Fixed64::from_i32(10),payload.clone());assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_sequence,expected);
        let (mut world,tower)=fixture(true,150);world.write_resource::<crate::runtime::BuffStore>().add(tower,"arbitrary_range",Fixed64::from_i32(10),payload);assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(tower)].attack_sequence,0);
    }
}

#[test]
fn npc_control_lane_stun_root_slow_silence_clocks_recovery_and_building_rules() {
    fn fixture(control:Option<&str>,near:bool,building:bool)->(World,Entity) {
        let mut world=world();let mut state=world.remove::<MobaMatch>().unwrap();
        state.config.warmup=Fixed64::ZERO;state.phase=MobaMatchPhase::Playing;state.next_wave_at=Fixed64::from_i32(10000);state.spawn_wave(&mut world);
        let actor=state.units.values().find(|u|u.team==0&&if building{u.role==LaneRole::Tower}else{u.role==LaneRole::Creep}).unwrap().entity;
        let enemy=state.heroes[1].entity.unwrap();state.units.retain(|_,u|u.entity==actor||u.entity==enemy);
        state.units.get_mut(&entity_key(actor)).unwrap().attack_remaining=Fixed64::from_raw(8);
        world.write_storage::<Pos>().get_mut(actor).unwrap().0=Vec2::ZERO;
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0=Vec2::new(Fixed64::from_i32(if near{80}else{600}),Fixed64::ZERO);
        world.write_resource::<crate::runtime::TeamVisibilityRuntime>().ensure_team(1);
        world.write_resource::<crate::runtime::TeamVisibilityRuntime>().teams.get_mut(&1).unwrap().index.current.insert(crate::runtime::canonical_entity_id(enemy));
        if let Some(control)=control {world.write_resource::<crate::runtime::BuffStore>().add(actor,control,Fixed64::from_i32(10),
            if control=="slow"{serde_json::json!({"move_speed_bonus":-0.5,"slow_factor":0.5})}else{serde_json::json!({})});}
        world.write_resource::<DeltaTime>().0=Fixed64::from_raw(16);world.insert(state);(world,actor)
    }
    let mut baseline=Fixed64::ZERO;
    for control in [None,Some("stun"),Some("root"),Some("silence"),Some("slow")] {
        let (mut world,actor)=fixture(control,false,false);let clock=world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_remaining;
        world.write_resource::<GamePause>().is_paused=true;assert!(!begin_moba_match_tick(&mut world));assert_eq!(world.read_storage::<Pos>().get(actor).unwrap().0,Vec2::ZERO);assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_remaining,clock);world.write_resource::<GamePause>().is_paused=false;
        assert!(begin_moba_match_tick(&mut world));let moved=world.read_storage::<Pos>().get(actor).unwrap().0.x;
        assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_remaining,Fixed64::ZERO);
        match control {None=>{assert!(moved>Fixed64::ZERO);baseline=moved;},Some("stun"|"root")=>assert_eq!(moved,Fixed64::ZERO),Some("slow")=>assert_eq!(moved*Fixed64::from_i32(2),baseline),_=>assert_eq!(moved,baseline)}
        if let Some(control)=control {world.write_resource::<crate::runtime::BuffStore>().remove(actor,control);world.write_resource::<Tick>().0+=1;let before=world.read_storage::<Pos>().get(actor).unwrap().0;assert!(begin_moba_match_tick(&mut world));assert!(world.read_storage::<Pos>().get(actor).unwrap().0.x>before.x);}
        let (mut world,actor)=fixture(control,true,false);assert!(begin_moba_match_tick(&mut world));let sequence=world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_sequence;
        assert_eq!(sequence,if control==Some("stun"){0}else{1},"control={control:?}");
    }
    for control in [Some("stun"),Some("root"),Some("silence")] {let (mut world,actor)=fixture(control,true,true);assert!(begin_moba_match_tick(&mut world));assert_eq!(world.read_resource::<MobaMatch>().units[&entity_key(actor)].attack_sequence,1,"buildings keep their rules");}
}

#[test]
fn combat_roster_defaults_are_real_constructor_values() {
    let world = world();
    let state = world.read_resource::<MobaMatch>();
    assert_eq!(state.heroes.len(), 2);
    for slot in &state.heroes {
        let preferences = slot.combat_preferences();
        assert_eq!(preferences.owner_id(), slot.player_id);
        assert_eq!(preferences.acquisition(), AttackAcquisition::Hero);
        assert!(preferences.idle_auto_attack());
        assert!(!preferences.champion_only_direct());
        assert_eq!(state.combat_preferences_for_source(slot.player_id, slot.entity.unwrap()), Some(preferences));
    }
}

#[cfg(feature = "kcp")]
#[test]
fn combat_roster_wire_router_uses_authenticated_owner_and_preserves_commands() {
    use crate::runtime::{PlayerInput, PlayerInputEnum, PendingPlayerInputs};
    use crate::game_proto::CombatPreferencesUpdate;
    let world = world();
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    let (left, right) = { let state = world.read_resource::<MobaMatch>(); (state.heroes[0].player_id, state.heroes[1].player_id) };
    let update = |raw| PlayerInput { action: Some(PlayerInputEnum::CombatPreferences(CombatPreferencesUpdate {
        schema_version: 1, acquisition: raw, idle_auto_attack: false,
    })) };
    world.write_resource::<PendingPlayerInputs>().inputs = vec![(left, update(1)), (right, update(0))];
    crate::comp::run_now::<crate::tick::player_input_tick::Sys>(&world);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].combat_preferences().acquisition().raw(), 1);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[1].combat_preferences().acquisition().raw(), 0);
    for owner in [left, right] {
        let state = world.read_resource::<MobaMatch>();
        let slot = state.heroes.iter().find(|slot| slot.player_id == owner).unwrap();
        assert!(!slot.combat_preferences().idle_auto_attack());
        assert!(world.read_storage::<HeroCommandQueue>().get(slot.entity.unwrap()).is_none_or(|queue| queue.active.is_none() && queue.queued.is_empty()));
    }
    let before = single_lane_replay_digest(&world);
    world.write_resource::<PendingPlayerInputs>().inputs = vec![(left, update(256)), (left + right + 100, update(0))];
    crate::comp::run_now::<crate::tick::player_input_tick::Sys>(&world);
    assert_eq!(single_lane_replay_digest(&world), before);
    assert!(world.read_resource::<PendingPlayerInputs>().inputs.is_empty());
}

#[test]
fn combat_roster_wire_projection_keeps_own_roster_and_only_disclosed_opponents() {
    use crate::runtime::combat_transport::{project, decode_projection};
    let world = world();
    let state = world.read_resource::<MobaMatch>();
    let own = state.heroes[0].player_id;
    let enemy = &state.heroes[1];
    let team = state.config.teams[state.heroes[0].side];
    let hidden = project(&state, team, &BTreeSet::new(), 4).unwrap();
    assert_eq!(hidden.rows.iter().map(|row| row.player_id).collect::<Vec<_>>(), vec![own]);
    let visible = BTreeSet::from([crate::runtime::canonical_entity_id(enemy.entity.unwrap())]);
    let disclosed = project(&state, team, &visible, 4).unwrap();
    assert_eq!(decode_projection(Some(&disclosed), 4).unwrap().len(), 2);
    let mut retired = (*state).clone();
    retired.heroes[0].entity = None;
    assert_eq!(project(&retired, team, &BTreeSet::new(), 4).unwrap(), hidden);
}

#[test]
fn combat_roster_invalid_updates_preserve_complete_replay_digest() {
    let world = world();
    let owner = world.read_resource::<MobaMatch>().heroes[0].player_id;
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    for (authenticated, requested, acquisition, expected) in [
        (0, owner, 0, CombatPreferenceAdmissionError::WrongOwner),
        (owner, 0, 0, CombatPreferenceAdmissionError::WrongOwner),
        (owner, owner + 100, 0, CombatPreferenceAdmissionError::WrongOwner),
        (owner + 100, owner + 100, 0, CombatPreferenceAdmissionError::WrongOwner),
        (owner, owner, 2, CombatPreferenceAdmissionError::InvalidAcquisition),
        (owner, owner, 255, CombatPreferenceAdmissionError::InvalidAcquisition),
    ] {
        let before = single_lane_replay_digest(&world);
        assert_eq!(world.write_resource::<MobaMatch>().admit_combat_preferences(authenticated, requested, acquisition, false), Err(expected));
        assert_eq!(single_lane_replay_digest(&world), before);
    }
    for phase in [MobaMatchPhase::Warmup, MobaMatchPhase::Finished { winner: None, tick: 1 }] {
        world.write_resource::<MobaMatch>().phase = phase;
        let before = single_lane_replay_digest(&world);
        assert_eq!(world.write_resource::<MobaMatch>().admit_combat_preferences(owner, owner, 1, false), Err(CombatPreferenceAdmissionError::InvalidPhase));
        assert_eq!(single_lane_replay_digest(&world), before);
    }
    let mut state = world.write_resource::<MobaMatch>();
    state.phase = MobaMatchPhase::Playing;
    let duplicate = state.heroes[0].clone(); state.heroes.push(duplicate); drop(state);
    let before = single_lane_replay_digest(&world);
    assert_eq!(world.write_resource::<MobaMatch>().admit_combat_preferences(owner, owner, 1, false), Err(CombatPreferenceAdmissionError::WrongOwner));
    assert_eq!(single_lane_replay_digest(&world), before);
}

#[test]
fn combat_roster_two_owners_update_without_touching_commands_or_attack_clocks() {
    let world = world();
    let (left, right, hero) = {
        let state = world.read_resource::<MobaMatch>();
        (state.heroes[0].player_id, state.heroes[1].player_id, state.heroes[0].entity.unwrap())
    };
    let queue = HeroCommandQueue { active: Some(HeroCommand::HoldPosition), queued: vec![HeroCommand::MoveTo { pos: Vec2::ZERO }] };
    world.write_storage::<HeroCommandQueue>().insert(hero, queue.clone()).unwrap();
    let clock = world.read_storage::<TAttack>().get(hero).unwrap().asd_count;
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    let before = single_lane_replay_digest(&world);
    world.write_resource::<MobaMatch>().admit_combat_preferences(left, left, 1, false).unwrap();
    assert_ne!(single_lane_replay_digest(&world), before);
    world.write_resource::<MobaMatch>().admit_combat_preferences(right, right, 0, true).unwrap();
    let state = world.read_resource::<MobaMatch>();
    assert_eq!(state.heroes[0].combat_preferences().acquisition(), AttackAcquisition::Cursor);
    assert!(!state.heroes[0].combat_preferences().idle_auto_attack());
    assert_eq!(state.heroes[1].combat_preferences().acquisition(), AttackAcquisition::Hero);
    assert!(state.heroes[1].combat_preferences().idle_auto_attack());
    assert_eq!(state.clone().heroes[0].combat_preferences(), state.heroes[0].combat_preferences());
    assert_eq!(world.read_storage::<HeroCommandQueue>().get(hero), Some(&queue));
    assert_eq!(world.read_storage::<TAttack>().get(hero).unwrap().asd_count, clock);
}

#[test]
fn combat_roster_actual_death_and_spawn_preserve_policy_and_retire_old_source() {
    let mut world = world();
    let (owner, old) = { let state = world.read_resource::<MobaMatch>(); (state.heroes[0].player_id, state.heroes[0].entity.unwrap()) };
    world.write_resource::<MobaMatch>().phase = MobaMatchPhase::Playing;
    world.write_resource::<MobaMatch>().admit_combat_preferences(owner, owner, 1, false).unwrap();
    let expected = world.read_resource::<MobaMatch>().heroes[0].combat_preferences();
    world.write_storage::<CProperty>().get_mut(old).unwrap().hp = Fixed64::ZERO;
    record_moba_death(&mut world, old);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].combat_preferences(), expected);
    assert!(world.read_resource::<MobaMatch>().combat_preferences_for_source(owner, old).is_none());
    world.delete_entity(old).unwrap(); world.maintain();
    let mut state = world.remove::<MobaMatch>().unwrap();
    state.spawn_hero(&mut world, 0);
    let replacement = state.heroes[0].entity.unwrap();
    assert_ne!(replacement, old);
    assert_eq!(state.combat_preferences_for_source(owner, replacement), Some(expected));
    assert!(state.combat_preferences_for_source(owner, old).is_none());
    assert!(state.combat_preferences_for_source(0, replacement).is_none());
    world.insert(state);
}
