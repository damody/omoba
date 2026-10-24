use crate::point_blast::*;
use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,world::GameWorld_TO,types::{Fixed64,Vec2,Target,DamageKind,DamageProfile}};
use omoba_content_model::ControlEffectKind;
use omoba_core::{Pos,Faction,FactionType,CProperty,Outcome};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,
    scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity};

fn p(x:i32,y:i32)->Vec2 {Vec2::new(Fixed64::from_i32(x),Fixed64::from_i32(y))}
fn body(world:&mut World,at:Vec2,team:i32)->Entity {
    world.create_entity().with(Pos(at)).with(Faction::new(if team==1 {FactionType::Player}else{FactionType::Enemy},team))
        .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
            def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build()
}
fn fixture()->(World,Entity) {
    let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
    world.insert(TowerTemplateRegistry::default());let source=body(&mut world,p(0,0),1);(world,source)
}
fn spec()->PointBlast {PointBlast {range:Fixed64::from_i32(700),radius:Fixed64::from_i32(100),damage:Fixed64::from_i32(200),
    kind:DamageKind::Magical,profile:DamageProfile::FIRE,control:None,zone:None}}

#[test]
fn point_blast_exact_circle_canonical_live_hits_preserve_fire_profile_and_all_control_kinds() {
    for (control,key) in [(ControlEffectKind::Stun,"stun"),(ControlEffectKind::Root,"root"),(ControlEffectKind::Silence,"silence")] {
        let (mut world,source)=fixture();let center=p(700,0);
        let inside=body(&mut world,center,2);let tangent=body(&mut world,p(800,0),2);
        let outside=body(&mut world,Vec2::new(Fixed64::from_raw(800*1024+1),Fixed64::ZERO),2);
        let ally=body(&mut world,center,1);let dead=body(&mut world,center,2);
        world.write_storage::<CProperty>().get_mut(dead).unwrap().hp=Fixed64::ZERO;
        let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let mut data=spec();data.control=Some((control,Fixed64::from_i32(2)));
        execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),Target::Point(center),data).unwrap();
        drop(dynworld);let outcomes=adapter.finish();drop(cache);
        let hits:Vec<_>=outcomes.iter().filter_map(|outcome|if let Outcome::RawDamage {target,magi,damage_profile,..}=outcome {
            assert_eq!(*magi,Fixed64::from_i32(200));assert_eq!(*damage_profile,DamageProfile::FIRE.bits());Some(*target)
        }else{None}).collect();assert_eq!(hits,vec![inside,tangent]);
        world.write_resource::<Vec<Outcome>>().extend(outcomes);
        omoba_core::runtime::process_outcomes(&mut world,&mut omoba_core::runtime::RuntimeEventVecSink::default()).unwrap();
        for entity in [inside,tangent] {assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(800));assert!(world.read_resource::<BuffStore>().has(entity,key));}
        for entity in [outside,ally,source] {assert_eq!(world.read_storage::<CProperty>().get(entity).unwrap().hp,Fixed64::from_i32(1000));assert!(!world.read_resource::<BuffStore>().has(entity,key));}
        assert_eq!(world.read_storage::<CProperty>().get(dead).unwrap().hp,Fixed64::ZERO);
    }
}

#[test]
fn point_blast_invalid_source_geometry_or_budget_emits_no_outcomes_and_self_point_is_legal() {
    for case in ["range","radius","damage","duration","dead","origin","extreme","outside","target"] {
        let (mut world,source)=fixture();let mut data=spec();
        if case=="range" {data.range=Fixed64::ZERO;}if case=="radius" {data.radius=Fixed64::from_i32(10001);}
        if case=="damage" {data.damage=-Fixed64::ONE;}if case=="duration" {data.control=Some((ControlEffectKind::Stun,Fixed64::ZERO));}
        if case=="dead" {world.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO;}
        if case=="origin" {world.write_storage::<Pos>().remove(source);}
        let target=match case {"target"=>Target::None,"extreme"=>Target::Point(Vec2::new(Fixed64::from_raw(i64::MAX),Fixed64::ZERO)),
            "outside"=>Target::Point(Vec2::new(Fixed64::from_raw(700*1024+1),Fixed64::ZERO)),_=>Target::Point(p(700,0))};
        let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        assert!(execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),target,data).is_err(),"{case}");
        drop(dynworld);assert!(adapter.finish().is_empty(),"{case}");
    }
    let (world,source)=fixture();let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
    let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
    execute(&mut dynworld,ParallelWorldAdapter::entity_to_handle(source),Target::Point(Vec2::ZERO),spec()).unwrap();
    drop(dynworld);assert!(!adapter.finish().is_empty());
}

#[test]
fn manual_point_blast_handlers_reject_bad_json_rank_extras_and_non_point_targets() {
    use crate::heroes::B02_date_masamune::{No1_flame_blade::FlameBladeHandler,No3_flame_assault::FlameAssaultHandler};
    for assault in [false,true] {for case in ["json","rank0","rank5","none","entity","range","radius","damage","type","quantum","duration"] {
        if !assault && case=="duration" {continue;}
        let (world,source)=fixture();let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let mut data=serde_json::json!({"cooldown":10,"mana_cost":80,"range":700,"damage":200,"swipe_radius":100,"radius":500,"stun_duration":0.3});
        if case=="range" {data["range"]=0.into();}if case=="damage" {data["damage"]=(-1).into();}
        if case=="radius" {data[if assault {"radius"}else{"swipe_radius"}]=0.into();}
        if case=="type" {data["damage"]="large".into();}if case=="quantum" {data["damage"]=serde_json::json!(0.0007);}
        if case=="duration" {data["stun_duration"]=61.into();}
        let json=if case=="json" {"{".to_owned()}else{data.to_string()};
        let target=match case {"none"=>Target::None,"entity"=>Target::Entity(ParallelWorldAdapter::entity_to_handle(source)),_=>Target::Point(p(400,0))};
        let rank=match case {"rank0"=>0,"rank5"=>5,_=>1};
        let result=if assault {FlameAssaultHandler.execute(ParallelWorldAdapter::entity_to_handle(source),target,rank,json.as_str().into(),&mut dynworld)}
            else {FlameBladeHandler.execute(ParallelWorldAdapter::entity_to_handle(source),target,rank,json.as_str().into(),&mut dynworld)};
        assert!(result.is_err(),"assault={assault} case={case}");drop(dynworld);assert!(adapter.finish().is_empty());
    }}
    for (id,key,control) in [(omoba_template_ids::ABILITY_FLAME_BLADE,"swipe_radius",None),
        (omoba_template_ids::ABILITY_FLAME_ASSAULT,"radius",Some((ControlEffectKind::Stun,"stun_duration")))] {
        let data=resolve(id,1,r#"{"cooldown":10,"mana_cost":80,"range":700}"#,"damage",key,control,DamageKind::Magical,DamageProfile::FIRE).unwrap();
        assert_eq!(data.damage,Fixed64::from_i32(200));
        assert_eq!(data.radius,crate::ability_builder::extra_at_id(id,key,1));
    }
}

#[test]
fn point_blast_previews_match_circle_damage_and_control_without_single_target_claim() {
    use omoba_core::ability_meta::{AbilityDef,EffectSpec,TargetSelector};
    for (ffi,radius,stun) in [(crate::heroes::B02_date_masamune::flame_blade_ffi(),100.0,false),
        (crate::heroes::B02_date_masamune::flame_assault_ffi(),500.0,true)] {
        let data:AbilityDef=serde_json::from_str(ffi.def_json.as_str()).unwrap();
        assert_eq!(data.effects_preview.len(),if stun {2}else{1});
        let EffectSpec::AreaEffect {radius:actual,effects,..}=&data.effects_preview[0] else {panic!("circle preview");};
        assert_eq!(*actual,radius);
        assert!(matches!(&effects[0],EffectSpec::Damage {target:TargetSelector::AllEnemies,amount,..} if *amount==200.0));
        assert_eq!(effects.len(),if stun {2}else{1});
        if stun {assert!(matches!(&effects[1],EffectSpec::StatusModifier {modifier_type,duration:Some(time),..} if modifier_type=="stun" && *time>0.0));}
        if stun {let EffectSpec::AreaEffect {radius,duration,tick_interval,effects}=&data.effects_preview[1]else{panic!("zone preview");};
            assert_eq!((*radius,*duration,*tick_interval),(500.0,3.0,0.5));
            assert!(matches!(effects.as_slice(),[EffectSpec::Damage {target:TargetSelector::AllEnemies,amount,damage_type:omoba_core::ability_meta::DamageType::Magical}] if *amount==25.0));}
    }
}

#[test]
fn ground_zone_rank_metadata_invalid_values_reject_before_any_blast_or_buff() {
    use crate::heroes::B02_date_masamune::No3_flame_assault::FlameAssaultHandler;
    for (key,value) in [("zone_damage",serde_json::json!(-1)),("zone_interval",serde_json::json!(0.1)),
        ("zone_duration",serde_json::json!(31)),("zone_duration",serde_json::json!(0.25)),("zone_damage",serde_json::json!("25"))] {
        let (world,source)=fixture();let cache=ParallelAdapterCache::new(&world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
        let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
        let mut data=serde_json::json!({"range":700});data[key]=value;
        assert!(FlameAssaultHandler.execute(ParallelWorldAdapter::entity_to_handle(source),Target::Point(p(400,0)),1,
            data.to_string().as_str().into(),&mut dynworld).is_err());drop(dynworld);assert!(adapter.finish().is_empty());
    }
}
