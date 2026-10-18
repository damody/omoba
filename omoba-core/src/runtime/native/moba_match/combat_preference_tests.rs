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
