//! Real factories and the production projectile system, without network matches.
use super::*;
use specs::Join;
use crate::runtime::{process_outcomes,RuntimeEventVecSink,TowerTemplateRegistry};

fn fixture()->(World,Entity,Entity) {
    let pool=crate::runtime::StateInitializer::create_thread_pool();
    let mut world=crate::runtime::StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let map=omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
    setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,map_id:Some(map.id.into()),
        lane_length:Fixed64::from_i32(map.lane_length),
        building_reward_rules:BuildingRewardRules::training(),lane_creep_gold:20,
        additional_players:vec![SingleLanePlayerConfig {player_id:3,team_id:1,hero:"training_luminary".into()}],
        ..Default::default()}).unwrap();
    let (hero,enemy)={let mut state=world.write_resource::<MobaMatch>();state.phase=MobaMatchPhase::Playing;
        (state.heroes[0].entity.unwrap(),state.heroes[1].entity.unwrap())};
    for gold in (&mut world.write_storage::<Gold>()).join() {gold.0=0;}
    (world,hero,enemy)
}
fn source(world:&mut World,player:u32)->Entity {
    world.create_entity().with(Faction::new(FactionType::Ally,1)).with(PlayerOwner::new(player))
        .with(Pos(Vec2::ZERO)).build()
}
fn flush(world:&mut World) {
    process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();
}
fn launch(world:&mut World,owner:Entity,target:Entity)->Entity {
    let tpos=world.read_storage::<Pos>().get(target).unwrap().0;
    world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptProjectile {basic_attack:false,source_missed:false,source_critical:false,
        pos:tpos-Vec2::new(Fixed64::from_i32(300),Fixed64::ZERO),owner,target:Some(target),tpos,
        radius:Fixed64::ZERO,msd:Fixed64::from_i32(900),damage_phys:Fixed64::ZERO,
        damage_magi:Fixed64::ZERO,damage_real:Fixed64::from_i32(100_000),damage_profile:omb_script_abi::types::DamageProfile::TRUE.bits(),
        slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,hit_radius:Fixed64::ZERO,
        stun_duration:Fixed64::ZERO,kind_id:0,generation:0,
    });
    flush(world);
    (&world.entities(),&world.read_storage::<Projectile>()).join()
        .filter(|(_,p)|p.owner==owner).map(|(e,_)|e).max_by_key(|e|(e.id(),e.gen().id())).unwrap()
}
fn impact(world:&mut World) {
    let mut builder=specs::DispatcherBuilder::new();
    crate::comp::dispatch::<crate::runtime::native::tick::projectile_tick::Sys>(&mut builder,&[]);
    let mut dispatcher=builder.build();dispatcher.setup(world);
    world.write_resource::<DeltaTime>().0=Fixed64::ONE;
    dispatcher.dispatch(world);flush(world);flush(world);
    assert!(world.try_fetch::<ProjectileImpactOrigin>().is_none());
}
fn balance(world:&World,index:usize)->i32 {
    let state=world.read_resource::<MobaMatch>();let slot=&state.heroes[index];
    slot.entity.map_or(slot.gold.0,|e|world.read_storage::<Gold>().get(e).unwrap().0)
}

#[test]
fn retired_projectile_callback_damage_keeps_moba_launch_credit() {
    use abi_stable::{sabi_trait::prelude::TD_Opaque,std_types::{RStr,RSome}};
    use omb_script_abi::{script::{UnitScript,UnitScript_TO},types::{EntityHandle,ProjectileHitContext,DamageKind,DamageProfile},
        world::{GameWorldDyn,ProjectileQueryDyn}};
    struct Followup;
    impl UnitScript for Followup {
        fn unit_id(&self)->RStr<'_> {"credit_probe".into()}
        fn on_projectile_hit(&self,attacker:EntityHandle,victim:EntityHandle,_:ProjectileHitContext,
            _:&ProjectileQueryDyn<'_>,world:&mut GameWorldDyn<'_>) {
            assert!(!world.is_alive(attacker));
            world.deal_damage(victim,Fixed64::from_i32(100_000),DamageKind::Pure,DamageProfile::NORMAL,RSome(attacker));
        }
    }
    let (mut world,_,enemy)=fixture();let owner=source(&mut world,1);
    world.write_storage::<crate::runtime::ScriptUnitTag>().insert(owner,
        crate::runtime::ScriptUnitTag {unit_id:"credit_probe".into()}).unwrap();
    let projectile=launch(&mut world,owner,enemy);
    world.write_storage::<Projectile>().get_mut(projectile).unwrap().damage_real=Fixed64::ZERO;
    world.delete_entity(owner).unwrap();world.maintain();impact(&mut world);
    assert_eq!(balance(&world,0),0);
    let mut registry=crate::runtime::ScriptRegistry::new();
    registry.insert_unit_for_test("credit_probe",UnitScript_TO::from_value(Followup,TD_Opaque));
    crate::runtime::run_script_dispatch(&mut world,&registry,1,Fixed64::ZERO);flush(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills,1);
    assert_eq!(balance(&world,0),world.read_resource::<MobaMatch>().config.hero_kill_gold as i32);
    flush(&mut world);assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills,1);
    assert!(world.try_fetch::<ProjectileImpactOrigin>().is_none());
}

#[test]
fn retired_summon_projectile_pays_shutdown_to_dead_owner_wallet_once() {
    let (mut world,hero,enemy)=fixture();
    {let mut state=world.write_resource::<MobaMatch>();state.config.kill_bounty_rules=KillBountyRules::training();state.heroes[1].kill_streak=9;}
    let owner=source(&mut world,1);launch(&mut world,owner,enemy);
    record_moba_death(&mut world,hero);world.delete_entity(hero).unwrap();world.delete_entity(owner).unwrap();world.maintain();
    impact(&mut world);assert_eq!(balance(&world,0),1000);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kill_streak,1);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[1].kill_streak,0);
    flush(&mut world);assert_eq!(balance(&world,0),1000);
    let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_hero(&mut world,0);world.insert(state);
    assert_eq!(balance(&world,0),1000);
}

#[test]
fn real_flight_keeps_each_launch_owner_after_source_retirement_and_generation_reuse() {
    let (mut world,_,enemy)=fixture();let owner=source(&mut world,1);
    let first=launch(&mut world,owner,enemy);
    world.write_storage::<PlayerOwner>().get_mut(owner).unwrap().player_id=3;
    let tower=world.read_resource::<MobaMatch>().towers[1].unwrap();
    let second=launch(&mut world,owner,tower);
    assert_eq!(world.read_resource::<ProjectileOrigins>().0[&first].credit_player,Some(1));
    assert_eq!(world.read_resource::<ProjectileOrigins>().0[&second].credit_player,Some(3));
    world.delete_entity(owner).unwrap();world.maintain();
    let replacement=source(&mut world,3);assert_eq!(replacement.id(),owner.id());assert_ne!(replacement.gen(),owner.gen());
    impact(&mut world);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills,1);
    assert_eq!(world.read_resource::<MobaMatch>().heroes[2].kills,0);
    assert!(balance(&world,0)>=300);assert!(balance(&world,2)>=100);
    flush(&mut world);assert!(world.read_resource::<ProjectileOrigins>().0.is_empty());
}

#[test]
fn real_flight_credits_dead_owner_wallet_and_survives_respawn() {
    let (mut world,hero,enemy)=fixture();let owner=source(&mut world,1);launch(&mut world,owner,enemy);
    record_moba_death(&mut world,hero);world.delete_entity(hero).unwrap();world.delete_entity(owner).unwrap();world.maintain();
    impact(&mut world);
    let reward=world.read_resource::<MobaMatch>().config.hero_kill_gold as i32;
    assert_eq!(balance(&world,0),reward);assert_eq!(world.read_resource::<MobaMatch>().heroes[0].kills,1);
    let mut state=world.remove::<MobaMatch>().unwrap();state.elapsed=Fixed64::from_i32(100);
    state.spawn_hero(&mut world,0);world.insert(state);assert_eq!(balance(&world,0),reward);
}

#[test]
fn real_lane_last_hit_and_jungle_objective_use_retired_sources_without_duplicate_credit() {
    for jungle in [false,true] {
        let (mut world,_,_)=fixture();let owner=source(&mut world,1);
        let target=if jungle {world.read_resource::<MobaMatch>().jungle_camps[0].entity.unwrap()}
            else {let mut state=world.remove::<MobaMatch>().unwrap();state.spawn_wave(&mut world);
                let target=state.units.values().find(|u|u.role==LaneRole::Creep && u.team==1).unwrap().entity;
                world.insert(state);target};
        launch(&mut world,owner,target);world.delete_entity(owner).unwrap();world.maintain();impact(&mut world);
        assert!(balance(&world,0)>0);let paid=balance(&world,0);
        world.write_resource::<Vec<Outcome>>().push(Outcome::Death {ent:target,pos:Vec2::ZERO});flush(&mut world);
        assert_eq!(balance(&world,0),paid);
    }
}

#[test]
fn origin_is_authority_digest_state_and_expired_projectiles_release_it() {
    let (mut world,_,enemy)=fixture();let owner=source(&mut world,1);let shot=launch(&mut world,owner,enemy);
    let before=single_lane_replay_digest(&world);
    world.write_resource::<ProjectileOrigins>().0.get_mut(&shot).unwrap().credit_player=Some(3);
    assert_ne!(before,single_lane_replay_digest(&world));
    world.delete_entity(shot).unwrap();world.maintain();flush(&mut world);
    assert!(world.read_resource::<ProjectileOrigins>().0.is_empty());
}
