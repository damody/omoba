use crate::passive_cone::*;
use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,world::GameWorld_TO,types::{Fixed64,Vec2,Target,DamageKind,DamageProfile,ProjectileSpec,PathSpec}};
use omoba_core::{Pos,Faction,FactionType,CProperty,TAttack,Hero,Outcome,ProjectileOrigins};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,ScriptRegistry,ScriptEventQueue,
 scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity};
const ID:omoba_template_ids::AbilityId=omoba_template_ids::ABILITY_RAIN_IRON_CANNON;
fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
fn body(world:&mut World,at:Vec2,team:i32)->Entity {
 world.create_entity().with(Pos(at)).with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
  .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build()
}
fn fixture()->(World,Entity,Entity) {
 let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());world.insert(TowerTemplateRegistry::default());
 let source=body(&mut world,p(-1000,0),1);let victim=body(&mut world,Vec2::ZERO,2);
 world.write_storage::<TAttack>().insert(source,TAttack::new(Fixed64::from_i32(200),Fixed64::ONE,Fixed64::from_i32(1500),Fixed64::from_i32(1000))).unwrap();
 (world,source,victim)
}
fn spec()->PassiveCone {PassiveCone {radius:Fixed64::from_i32(150),half_ticks:512,attack_fraction:Fixed64::from_raw(512)}}
fn flush(world:&mut World) {omoba_core::runtime::process_outcomes(world,&mut omoba_core::runtime::RuntimeEventVecSink::default()).unwrap();world.maintain();}
#[test]
fn passive_cone_impact_apex_exact_edges_live_filter_and_canonical_true_damage() {
 let (mut world,source,victim)=fixture();let front=body(&mut world,p(100,0),2);let behind=body(&mut world,p(-10,0),2);
 let edge=body(&mut world,p(50,50),2);let edge_out=body(&mut world,p(50,50)+Vec2::new(Fixed64::ZERO,Fixed64::from_raw(1)),2);
 let tangent=body(&mut world,p(150,0),2);let outside=body(&mut world,p(150,0)+Vec2::new(Fixed64::from_raw(1),Fixed64::ZERO),2);
 let ally=body(&mut world,p(50,0),1);let dead=body(&mut world,p(60,0),2);
 world.write_storage::<CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
 let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
 let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
 assert!(execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),ParallelWorldAdapter::entity_to_handle(victim),spec(),DamageKind::Pure,DamageProfile::TRUE));
 drop(dynworld);let outcomes=adapter.finish();drop(cache);
 let hits:Vec<_>=outcomes.iter().filter_map(|o|if let Outcome::RawDamage {target,real,phys,magi,damage_profile,..}=o {
  assert_eq!(*real,Fixed64::from_i32(100));assert_eq!((*phys,*magi),(Fixed64::ZERO,Fixed64::ZERO));assert_eq!(*damage_profile,DamageProfile::TRUE.bits());Some(*target)
 }else{None}).collect();
 let mut expected=vec![victim,front,edge,tangent];expected.sort_by_key(|e|(e.id(),e.gen().id()));assert_eq!(hits,expected);
 world.write_resource::<Vec<Outcome>>().extend(outcomes);flush(&mut world);
 for e in expected {assert_eq!(world.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(900));}
 for e in [source,behind,edge_out,outside,ally] {assert_eq!(world.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(1000));}
}
#[test]
fn passive_cone_invalid_budgets_geometry_and_zero_direction_emit_nothing() {
 for case in 0..8 {
  let (mut world,source,victim)=fixture();let mut data=spec();
  if case==0 {data.radius=Fixed64::ZERO;}if case==1 {data.half_ticks=2049;}if case==2 {data.attack_fraction=Fixed64::from_i32(5);}
  if case==3 {world.write_storage::<Pos>().get_mut(source).unwrap().0=Vec2::ZERO;}
  if case==4 {world.write_storage::<Pos>().remove(victim);}
  if case==5 {world.write_storage::<Pos>().get_mut(source).unwrap().0.x=Fixed64::from_raw(i64::MAX);}
  if case==6 {world.write_storage::<TAttack>().get_mut(source).unwrap().atk_physic.v=Fixed64::ZERO;}
  if case==7 {world.write_storage::<TAttack>().get_mut(source).unwrap().atk_physic.v=Fixed64::from_i32(1_000_001);}
  let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
  let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
  assert!(!execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),ParallelWorldAdapter::entity_to_handle(victim),data,DamageKind::Pure,DamageProfile::TRUE),"{case}");
  drop(dynworld);assert!(adapter.finish().is_empty(),"{case}");
 }
}
#[test]
fn passive_cone_checked_ranks_passive_rejection_and_canonical_permanent_marker() {
 use crate::heroes::B01_saika_magoichi::No3_rain_iron_cannon::RainIronCannonHandler as Handler;
 for rank in [0,5,255] {assert!(resolve(ID,rank).is_none());}
 for rank in 1..=4 {let spec=resolve(ID,rank).unwrap();assert_eq!(spec.radius,Fixed64::from_i32(150));assert_eq!(spec.half_ticks,512);}
 let (mut world,source,_)=fixture();let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
 let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);let handle=ParallelWorldAdapter::entity_to_handle(source);
 for rank in [0,5,255] {Handler.on_learn(handle,rank,&mut dynworld);}
 assert!(Handler.execute(handle,Target::None,1,"{}".into(),&mut dynworld).is_err());
 drop(dynworld);assert!(adapter.finish().is_empty());drop(cache);
 let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
 let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);Handler.on_learn(handle,4,&mut dynworld);drop(dynworld);
 let outcomes=adapter.finish();drop(cache);world.write_resource::<Vec<Outcome>>().extend(outcomes);flush(&mut world);
 let buffs=world.read_resource::<BuffStore>();let entry=buffs.iter_for(source).find(|(id,_)|*id=="rain_iron_cannon_passive").unwrap().1;
 assert_eq!(entry.remaining,Fixed64::from_raw(i64::MAX));assert_eq!(entry.payload["level"],4);
}
fn normal_launch(world:&mut World,source:Entity,victim:Entity)->omoba_core::ProjectileOrigin {
 let mut hero=Hero::new("arbitrary_hero".into(),"probe".into(),String::new());hero.ability_levels.insert("rain_iron_cannon".into(),1);
 world.write_storage::<Hero>().insert(source,hero).unwrap();
 world.write_resource::<BuffStore>().add(source,"probe_visual",Fixed64::from_i32(10),serde_json::json!({"multi_shot_visual":3072}));
 let pos=world.read_storage::<Pos>().get(source).unwrap().0;
 world.write_resource::<Vec<Outcome>>().push(Outcome::ProjectileLine2 {pos,source:Some(source),target:Some(victim)});flush(world);
 let origins=world.read_resource::<ProjectileOrigins>();assert_eq!(origins.0.len(),3);assert_eq!(origins.0.values().filter(|o|o.normal_attack).count(),1);
 let origin=origins.0.values().find(|o|o.normal_attack).unwrap().clone();
 assert_eq!(origin.script.as_ref().unwrap().ability_levels,vec![("rain_iron_cannon".into(),1)]);origin
}
#[test]
fn passive_cone_real_normal_launch_retired_hero_and_visual_provenance_do_not_resurrect_or_retrigger() {
 for normal in [false,true] {
  let (mut world,source,victim)=fixture();let front=body(&mut world,p(100,0),2);let behind=body(&mut world,p(-10,0),2);
  let mut origin=normal_launch(&mut world,source,victim);origin.normal_attack=normal;
  world.delete_entity(source).unwrap();world.maintain();let replacement=body(&mut world,p(-1000,0),1);
  assert_eq!(source.id(),replacement.id());assert_ne!(source.gen(),replacement.gen());
  world.write_resource::<Vec<Outcome>>().push(Outcome::ProjectileContext {origin,outcome:Box::new(Outcome::ProjectileHit {source,target:victim,kind_id:0,generation:0})});flush(&mut world);
  let mut registry=ScriptRegistry::new();registry.insert_manifest(crate::get_manifest());
  omoba_core::runtime::run_script_dispatch(&mut world,&registry,5,Fixed64::from_raw(17));flush(&mut world);
  let expected=Fixed64::from_i32(1000)-if normal {Fixed64::from_i32(200)*resolve(ID,1).unwrap().attack_fraction}else{Fixed64::ZERO};
  for e in [victim,front] {assert_eq!(world.read_storage::<CProperty>().get(e).unwrap().hp,expected);}
  for e in [replacement,behind] {assert_eq!(world.read_storage::<CProperty>().get(e).unwrap().hp,Fixed64::from_i32(1000));}
  assert!(!world.entities().is_alive(source));
 }
}
#[test]
fn passive_cone_script_child_clears_normal_attack_provenance_even_with_normal_damage_profile() {
 let (mut world,source,victim)=fixture();let origin=normal_launch(&mut world,source,victim);
 let previous=world.read_resource::<ProjectileOrigins>().0.keys().copied().collect::<std::collections::BTreeSet<_>>();
 let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);adapter.set_projectile_origin(Some(origin));
 let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
 dynworld.spawn_projectile_ex(ProjectileSpec {owner:ParallelWorldAdapter::entity_to_handle(source),from:p(-1000,0),
  path:PathSpec::Homing {target:ParallelWorldAdapter::entity_to_handle(victim)},speed:Fixed64::from_i32(1000),damage:Fixed64::ZERO,
  damage_profile:DamageProfile::NORMAL,splash_radius:Fixed64::ZERO,hit_radius:Fixed64::ZERO,slow_factor:Fixed64::ZERO,
  slow_duration:Fixed64::ZERO,stun_duration:Fixed64::ZERO,kind_id:42});
 drop(dynworld);let outcomes=adapter.finish();drop(cache);world.write_resource::<Vec<Outcome>>().extend(outcomes);flush(&mut world);
 let origins=world.read_resource::<ProjectileOrigins>();let new:Vec<_>=origins.0.iter().filter(|(e,_)|!previous.contains(e)).collect();
 assert_eq!(new.len(),1);assert!(!new[0].1.normal_attack);
}
