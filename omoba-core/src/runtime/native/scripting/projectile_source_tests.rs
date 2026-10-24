//! Real launch -> flight -> outcome -> script dispatch, without a network match.
use super::*;
use crate::comp::*;
use crate::runtime::{process_outcomes, RuntimeEventVecSink, StateInitializer, BuffStore};
use abi_stable::{sabi_trait::prelude::TD_Opaque, std_types::{RStr, RSome}};
use omb_script_abi::{script::{UnitScript, UnitScript_TO}, stat_keys::StatKey,
    types::{EntityHandle, ProjectileHitContext, Fixed64, Vec2, DamageKind, DamageProfile, ProjectileSpec, PathSpec},
    world::{GameWorldDyn, ProjectileQueryDyn}};
use specs::{Builder, Entity, Join, World, WorldExt};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
struct Hit { alive:bool, flag:bool, attack:Fixed64, bonus:Fixed64, active:Fixed64,
    identity:String, team:String, pos:Vec2, generation:u8, candidate:Option<EntityHandle>,
    world_candidates:Vec<EntityHandle> }
struct Probe(Arc<Mutex<Vec<Hit>>>);
impl UnitScript for Probe {
    fn unit_id(&self)->RStr<'_> {"source_probe".into()}
    fn on_projectile_hit(&self,attacker:EntityHandle,victim:EntityHandle,context:ProjectileHitContext,
        query:&ProjectileQueryDyn<'_>,world:&mut GameWorldDyn<'_>) {
        let at=world.get_pos(victim).unwrap();
        let candidate=query.enemy_candidates_bounded(at,Fixed64::from_i32(500),attacker,RSome(victim),1)
            .first().copied();
        self.0.lock().unwrap().push(Hit {alive:world.is_alive(attacker),
            flag:world.has_tower_flag(attacker,"probe_flag".into()), attack:world.get_final_atk(attacker),
            bonus:world.get_stat_bonus(attacker,StatKey::CritBonus),active:world.get_buff_remaining(attacker,"probe_buff".into()),
            identity:world.unit_id_of(attacker).unwrap().as_str().into(),team:world.faction_of(attacker).unwrap().as_str().into(),
            pos:world.get_pos(attacker).unwrap(),generation:context.generation,candidate,
            world_candidates:world.query_enemies_in_range(at,Fixed64::from_i32(500),attacker).into_iter().collect()});
        world.deal_damage(victim,Fixed64::from_i32(7),DamageKind::Physical,DamageProfile::NORMAL,RSome(attacker));
        if context.generation==0 {
            if let Some(target)=candidate {world.spawn_projectile_ex(ProjectileSpec {
                owner:attacker,from:at,path:PathSpec::Homing {target},speed:Fixed64::from_i32(1000),
                damage:Fixed64::ZERO,damage_profile:DamageProfile::NORMAL,splash_radius:Fixed64::ZERO,
                hit_radius:Fixed64::ZERO,slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,
                stun_duration:Fixed64::ZERO,kind_id:42,
            });}
        }
    }
}
fn flush(world:&mut World) {process_outcomes(world,&mut RuntimeEventVecSink::default()).unwrap();world.maintain();}
fn fixture()->(World,Entity,Entity,Entity,ScriptRegistry,Arc<Mutex<Vec<Hit>>>) {
    let pool=StateInitializer::create_thread_pool();
    let mut world=StateInitializer::setup_campaign_ecs_world(&pool);
    world.insert(TowerTemplateRegistry::default());
    let mut tower=Tower::new();tower.upgrade_flags.push("probe_flag".into());
    let source=world.create_entity().with(Pos(Vec2::ZERO)).with(Faction::new(FactionType::Ally,1))
        .with(TAttack::new(Fixed64::from_i32(20),Fixed64::ONE,Fixed64::from_i32(500),Fixed64::from_i32(1000)))
        .with(ScriptUnitTag {unit_id:"source_probe".into()}).with(tower).build();
    world.write_resource::<BuffStore>().add(source,"probe_buff",Fixed64::from_i32(3),
        serde_json::json!({"crit_bonus":Fixed64::from_i32(11).raw()}));
    let mut targets=Vec::new();
    for (team,x) in [(2,100),(2,200),(1,101)] {
        targets.push(world.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(x),Fixed64::ZERO)))
            .with(Faction::new(FactionType::Enemy,team))
            .with(Creep {name:"probe".into(),label:None,path:"probe".into(),pidx:0,path_remaining_distance:Fixed64::ZERO,
                block_tower:None,status:CreepStatus::Walk,td_layer:None})
            .with(CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),msd:Fixed64::ZERO,
                def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build());
    }
    let positions:Vec<_>=targets.iter().map(|e|(*e,vek::Vec2::new(world.read_storage::<Pos>().get(*e).unwrap().0.x.to_f32_for_render(),0.0))).collect();
    world.write_resource::<Searcher>().creep.rebuild_from(positions);
    world.write_resource::<GamePause>().is_paused=true;
    let hits=Arc::new(Mutex::new(Vec::new()));let mut registry=ScriptRegistry::new();
    registry.insert_unit_for_test("source_probe",UnitScript_TO::from_value(Probe(hits.clone()),TD_Opaque));
    (world,source,targets[0],targets[1],registry,hits)
}
fn launch(world:&mut World,source:Entity,target:Entity) {
    let tpos=world.read_storage::<Pos>().get(target).unwrap().0;
    world.write_resource::<Vec<Outcome>>().push(Outcome::ScriptProjectile {basic_attack:false,source_missed:false,source_critical:false,pos:Vec2::ZERO,owner:source,
        target:Some(target),tpos,radius:Fixed64::ZERO,msd:Fixed64::from_i32(1000),
        damage_phys:Fixed64::ZERO,damage_magi:Fixed64::ZERO,damage_real:Fixed64::ZERO,damage_profile:0,
        slow_factor:Fixed64::ZERO,slow_duration:Fixed64::ZERO,hit_radius:Fixed64::ZERO,stun_duration:Fixed64::ZERO,
        kind_id:41,generation:0});flush(world);
}
fn impact(world:&mut World,registry:&ScriptRegistry) {
    let mut builder=specs::DispatcherBuilder::new();
    crate::comp::dispatch::<crate::runtime::native::tick::projectile_tick::Sys>(&mut builder,&[]);
    let mut dispatcher=builder.build();dispatcher.setup(world);
    world.write_resource::<DeltaTime>().0=Fixed64::ONE;dispatcher.dispatch(world);flush(world);
    run_script_dispatch(world,registry,1,Fixed64::ZERO);flush(world);
}
#[test]
fn retired_projectile_callback_recovers_readonly_context_and_child_lineage() {
    let (mut world,source,target,second,registry,hits)=fixture();launch(&mut world,source,target);
    let snapshot=world.read_resource::<ProjectileOrigins>().0.values().next().unwrap().script.clone().unwrap();
    world.delete_entity(source).unwrap();world.maintain();
    let replacement=world.create_entity().with(Pos(Vec2::new(Fixed64::from_i32(999),Fixed64::ZERO)))
        .with(Faction::new(FactionType::Enemy,2)).with(ScriptUnitTag {unit_id:"wrong_script".into()}).build();
    assert_eq!(replacement.id(),source.id());assert_ne!(replacement.gen(),source.gen());
    impact(&mut world,&registry);
    {let hits=hits.lock().unwrap();assert_eq!(hits.len(),1);let hit=&hits[0];
        assert!(!hit.alive);assert!(hit.flag);assert_eq!(hit.attack,Fixed64::from_i32(20));
        assert_eq!(hit.bonus,Fixed64::from_i32(11));assert_eq!(hit.active,Fixed64::from_i32(3));
        assert_eq!(hit.identity,"source_probe");assert_eq!(hit.team,"team:1");assert_eq!(hit.pos,Vec2::ZERO);
        assert_eq!(hit.candidate,Some(ParallelWorldAdapter::entity_to_handle(second)));}
    assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(93));
    {let origins=world.read_resource::<ProjectileOrigins>();assert_eq!(origins.0.len(),1);
        let child=origins.0.values().next().unwrap();assert_eq!(child.source,source);
        assert!(Arc::ptr_eq(&snapshot,child.script.as_ref().unwrap()),"children share immutable provenance");}
    impact(&mut world,&registry);assert_eq!(hits.lock().unwrap().len(),2);
    assert_eq!(hits.lock().unwrap()[1].generation,1);
    assert_eq!(world.read_storage::<CProperty>().get(second).unwrap().hp,Fixed64::from_i32(93));
    assert_eq!(world.read_storage::<Pos>().get(replacement).unwrap().0.x,Fixed64::from_i32(999));
    flush(&mut world);assert!(world.read_resource::<ProjectileOrigins>().0.is_empty());
}
#[test]
fn living_projectile_callback_keeps_current_source_values() {
    let (mut world,source,target,_,registry,hits)=fixture();launch(&mut world,source,target);
    world.write_storage::<Tower>().get_mut(source).unwrap().upgrade_flags.clear();
    world.write_storage::<TAttack>().get_mut(source).unwrap().atk_physic.v=Fixed64::from_i32(77);
    impact(&mut world,&registry);let hits=hits.lock().unwrap();assert_eq!(hits.len(),1);
    assert!(hits[0].alive);assert!(!hits[0].flag);assert_eq!(hits[0].attack,Fixed64::from_i32(77));
}
#[test]
fn retired_summon_projectile_callback_keeps_target_admission_without_source_resurrection() {
    for retired in [true,false] {
    let (mut world,source,target,second,registry,hits)=fixture();
    world.write_storage::<Faction>().get_mut(source).unwrap().faction_id=FactionType::Player;
    world.write_storage::<Unit>().insert(source,Unit::new("probe".into(),"probe".into(),UnitType::Summon)).unwrap();
    world.write_storage::<CProperty>().insert(source,CProperty {hp:Fixed64::from_i32(100),mhp:Fixed64::from_i32(100),
        msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).unwrap();
    launch(&mut world,source,target);
    if retired {world.delete_entity(source).unwrap();world.maintain();}
    else {world.write_resource::<BuffStore>().add(source,"stun",Fixed64::from_i32(10),serde_json::json!({}));}
    // Keep a live hostile in the spatial index but deny automatic acquisition.
    let mut gate=crate::runtime::filtered_specs::DisclosedAttackabilityGate::default();
    gate.ward_automatic_disabled.insert(target);world.insert(gate);
    impact(&mut world,&registry);let hits=hits.lock().unwrap();assert_eq!(hits.len(),1);
    assert_eq!(hits[0].alive,!retired);
    assert_eq!(hits[0].world_candidates,vec![ParallelWorldAdapter::entity_to_handle(second)]);
    }
}
#[test]
fn projectile_context_rejects_mismatched_sources_and_nested_envelopes() {
    let (mut world,source,target,other,_,_)=fixture();launch(&mut world,source,target);
    let origin=world.read_resource::<ProjectileOrigins>().0.values().next().unwrap().clone();
    let hit=Outcome::ProjectileHit {source:other,target,kind_id:41,generation:0};
    world.write_resource::<Vec<Outcome>>().extend([
        Outcome::ProjectileContext {origin:origin.clone(),outcome:Box::new(hit)},
        Outcome::ProjectileContext {origin:origin.clone(),outcome:Box::new(Outcome::ProjectileContext {origin,
            outcome:Box::new(Outcome::ProjectileHit {source,target,kind_id:41,generation:0})})}]);
    flush(&mut world);assert!(world.write_resource::<ScriptEventQueue>().drain().is_empty());
    assert_eq!(world.read_storage::<CProperty>().get(target).unwrap().hp,Fixed64::from_i32(100));
}
