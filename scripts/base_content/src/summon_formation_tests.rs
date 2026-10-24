use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,world::GameWorld_TO,types::{Fixed64,Vec2,Target}};
use omoba_core::{Pos,Faction,FactionType,CProperty,Outcome,BlockedRegions,BlockedRegion};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity,Join};
use omoba_template_ids::ABILITY_SAIKA_REINFORCEMENTS as ID;
use crate::{summon_formation::resolve,heroes::B01_saika_magoichi::No2_saika_reinforcements::SaikaReinforcementsHandler};
fn fixture()->(World,Entity) {
 let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
 world.insert(TowerTemplateRegistry::default());
 let source=world.create_entity().with(Pos(Vec2::ZERO)).with(Faction::new(FactionType::Player,1))
  .with(CProperty{hp:Fixed64::ONE,mhp:Fixed64::ONE,msd:Fixed64::ZERO,def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
 (world,source)
}
fn execute(adapter:&mut ParallelWorldAdapter<'_>,source:Entity,rank:u8,json:&str)->bool {
 let mut world=GameWorld_TO::from_ptr(RMut::new(adapter),TD_Opaque);
 SaikaReinforcementsHandler.execute(ParallelWorldAdapter::entity_to_handle(source),Target::Point(Vec2::ZERO),rank,json.into(),&mut world).is_ok()
}
#[test]
fn summon_formation_all_ranks_use_current_content_and_rotate_row_major() {
 assert_eq!(omoba_template_ids::active_ability_const(ID).unwrap().facing_formation_rows,2);
 let other=resolve(ID,4,r#"{"cooldown":0,"mana_cost":0}"#,4).unwrap();
 let grid=other.positions(Vec2::ZERO,omoba_sim::Angle::ZERO).unwrap();assert_eq!(grid.len(),8);
 assert_eq!(grid[6].x,Fixed64::from_i32(300));
 for rank in 1..=4 {
  let spec=resolve(ID,rank,r#"{"cooldown":0,"mana_cost":0}"#,2).unwrap();
  assert_eq!(spec.count,u16::from(rank)*2);assert_eq!(spec.duration,Fixed64::from_i32(40+i32::from(rank)*5));
  let at=spec.positions(Vec2::ZERO,omoba_sim::Angle::ZERO).unwrap();assert_eq!(at.len(),usize::from(rank)*2);
  assert_eq!(at[0].x,Fixed64::from_i32(120));assert_eq!(at[usize::from(rank)].x,Fixed64::from_i32(180));
  assert_eq!(at[0].y,-Fixed64::from_i32((i32::from(rank)-1)*30));
  let rotated=spec.positions(Vec2::ZERO,omoba_sim::trig::atan2(Fixed64::ONE,Fixed64::ZERO)).unwrap();
  for error in [rotated[0].x+at[0].y,rotated[0].y-at[0].x] {assert!(error>= -Fixed64::from_raw(1)&&error<=Fixed64::from_raw(1));}
 }
}
#[test]
fn summon_formation_invalid_metadata_rank_extent_and_owner_emit_nothing() {
 for json in ["{",r#"{"cooldown":0,"mana_cost":0,"summon_count":2.5}"#,r#"{"cooldown":0,"mana_cost":0,"summon_count":3}"#,
  r#"{"cooldown":0,"mana_cost":0,"summon_count":130}"#,r#"{"cooldown":0,"mana_cost":0,"duration":0}"#,
  r#"{"cooldown":0,"mana_cost":0,"duration":null}"#,r#"{"cooldown":0,"mana_cost":0,"row_spacing":"60"}"#,
  r#"{"cooldown":0,"mana_cost":0,"col_spacing":-1}"#,r#"{"cooldown":0,"mana_cost":0,"front_row_distance":0.0001}"#,
  r#"{"cooldown":0,"mana_cost":0,"front_row_distance":10000,"row_spacing":1}"#] {
  let (world,source)=fixture();let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
  assert!(!execute(&mut adapter,source,1,json),"{json}");assert!(adapter.finish().is_empty());
 }
 for rank in [0,5,255] {assert!(resolve(ID,rank,r#"{"cooldown":0,"mana_cost":0}"#,2).is_err());}
 assert!(resolve(ID,1,r#"{"cooldown":0,"mana_cost":0}"#,0).is_err());
 let spec=resolve(ID,1,r#"{"cooldown":0,"mana_cost":0}"#,2).unwrap();
 assert!(spec.positions(Vec2::new(Fixed64::from_i32(1_000_000),Fixed64::ZERO),omoba_sim::Angle::ZERO).is_err());
 for missing in [false,true] {let (mut world,source)=fixture();
  if missing {world.write_storage::<Pos>().remove(source);}else{world.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
  let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
  assert!(!execute(&mut adapter,source,1,r#"{"cooldown":0,"mana_cost":0}"#));assert!(adapter.finish().is_empty());
 }
}
#[test]
fn summon_formation_mid_group_placement_failure_discards_reserved_entities_and_recovers() {
 let (mut world,source)=fixture();
 world.write_resource::<BlockedRegions>().0.push(BlockedRegion{name:"second-row-enclosure".into(),points:vec![
  vek::Vec2::new(1400.,-800.),vek::Vec2::new(2800.,-800.),vek::Vec2::new(2800.,800.),vek::Vec2::new(1400.,800.)]});
 let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
 assert!(!execute(&mut adapter,source,1,r#"{"cooldown":0,"mana_cost":0,"row_spacing":2000}"#));
 assert_eq!((&world.entities()).join().count(),2,"first row reserved before second row rejects");
 adapter.discard_cast();assert!(adapter.finish().is_empty());drop(cache);world.maintain();
 assert_eq!((&world.entities()).join().count(),1,"no partial entity survives rejected managed cast");
 world.write_resource::<BlockedRegions>().0.clear();
 let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
 assert!(execute(&mut adapter,source,1,r#"{"cooldown":0,"mana_cost":0}"#));
 assert_eq!(adapter.finish().iter().filter(|o|matches!(o,Outcome::SpawnSummoned{..})).count(),2);
}
