use abi_stable::{sabi_trait::prelude::TD_Opaque,RMut};
use omb_script_abi::{ability::AbilityScript,world::GameWorld_TO,types::{Fixed64,Vec2,Target},stat_keys::StatKey};
use omoba_core::{Pos,Faction,FactionType,CProperty,Outcome};
use omoba_core::runtime::{StateInitializer,TowerTemplateRegistry,BuffStore,
    scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter}};
use specs::{World,WorldExt,Builder,Entity};
use crate::heroes::{B01_saika_magoichi::No4_three_stage_technique::ThreeStageHandler,
    B02_date_masamune::No4_matchlock_gun::MatchlockGunHandler};

fn fixture()->(World,Entity) {
    let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
    world.insert(TowerTemplateRegistry::default());
    let source=world.create_entity().with(Pos(Vec2::ZERO)).with(Faction::new(FactionType::Player,1))
        .with(CProperty {hp:Fixed64::from_i32(1000),mhp:Fixed64::from_i32(1000),msd:Fixed64::ZERO,
            def_physic:Fixed64::ZERO,def_magic:Fixed64::ZERO}).build();
    (world,source)
}
fn cast(world:&World,source:Entity,matchlock:bool,rank:u8,json:&str,target:Target)->(bool,Vec<Outcome>) {
    let cache=ParallelAdapterCache::new(world,1);let mut adapter=ParallelWorldAdapter::new(&cache,source);
    let mut dynworld=GameWorld_TO::from_ptr(RMut::new(&mut adapter),TD_Opaque);
    let handle=ParallelWorldAdapter::entity_to_handle(source);
    let success=if matchlock {MatchlockGunHandler.execute(handle,target,rank,json.into(),&mut dynworld).is_ok()}
        else {ThreeStageHandler.execute(handle,target,rank,json.into(),&mut dynworld).is_ok()};
    drop(dynworld);(success,adapter.finish())
}
fn flush(world:&mut World,outcomes:Vec<Outcome>) {
    world.write_resource::<Vec<Outcome>>().extend(outcomes);
    omoba_core::runtime::process_outcomes(world,&mut omoba_core::runtime::RuntimeEventVecSink::default()).unwrap();
}
#[test]
fn self_transform_all_authored_ranks_preserve_encodings_refresh_without_stacking_and_expire() {
    for matchlock in [false,true] {for rank in 1..=if matchlock {3}else{4} {
        let (mut world,source)=fixture();
        let (success,outcomes)=cast(&world,source,matchlock,rank,r#"{"cooldown":0,"mana_cost":0}"#,Target::None);
        assert!(success);assert_eq!(outcomes.len(),1);flush(&mut world,outcomes);
        let id=if matchlock {"matchlock_gun"}else{"three_stage_transform"};
        let expected_time=Fixed64::from_i32(if matchlock {45}else{5});
        {
            let buffs=world.read_resource::<BuffStore>();let entry=buffs.iter_for(source).find(|(key,_)|*key==id).unwrap().1;
            assert_eq!(entry.remaining,expected_time);
            if matchlock {
                assert_eq!(buffs.sum_add(source,StatKey::AttackRangeBonus),Fixed64::from_i32(600+i32::from(rank)*100));
                assert_eq!(buffs.sum_add(source,StatKey::BaseAttackBonusDamage),Fixed64::from_i32(50+i32::from(rank)*40));
                let rate=crate::ability_builder::extra_at_id(omoba_template_ids::ABILITY_MATCHLOCK_GUN,"lifesteal",rank);
                assert_eq!(rate.raw(),[128,192,256][usize::from(rank-1)]);
                assert_eq!(buffs.sum_add(source,StatKey::Lifesteal),rate);
                assert_eq!(entry.payload["lifesteal"],rate.raw());
                assert!(entry.payload["attack_stun_chance"].as_f64().unwrap()>0.86);
                assert!(entry.payload["attack_stun_chance"].as_f64().unwrap()<0.88);
                assert_eq!(entry.payload["attack_stun_duration"].as_f64().unwrap(),
                    crate::ability_builder::extra_at_id(omoba_template_ids::ABILITY_MATCHLOCK_GUN,"stun_duration",rank).raw() as f64/1024.0);
            } else {
                assert_eq!(buffs.sum_add(source,StatKey::TotalDamageOutgoingPercentage),Fixed64::from_i32(2));
                assert_eq!(buffs.sum_add(source,StatKey::MultiShotVisual),Fixed64::from_i32(3));
                assert_eq!(entry.payload["visual_effect"],"three_stage_transform_red");
            }
        }
        world.write_resource::<BuffStore>().tick(Fixed64::ONE);
        let (_,outcomes)=cast(&world,source,matchlock,rank,r#"{"cooldown":0,"mana_cost":0}"#,Target::None);flush(&mut world,outcomes);
        let mut buffs=world.write_resource::<BuffStore>();
        assert_eq!(buffs.iter_for(source).filter(|(key,_)|*key==id).count(),1);
        assert_eq!(buffs.iter_for(source).find(|(key,_)|*key==id).unwrap().1.remaining,expected_time);
        if matchlock {assert_eq!(buffs.sum_add(source,StatKey::Lifesteal),crate::ability_builder::extra_at_id(omoba_template_ids::ABILITY_MATCHLOCK_GUN,"lifesteal",rank));}
        buffs.tick(expected_time-Fixed64::from_raw(1));assert!(buffs.has(source,id));
        buffs.tick(Fixed64::from_raw(1));assert!(!buffs.has(source,id));
        assert_eq!(buffs.sum_add(source,StatKey::AttackRangeBonus),Fixed64::ZERO);
        assert_eq!(buffs.sum_add(source,StatKey::TotalDamageOutgoingPercentage),Fixed64::ZERO);
        assert_eq!(buffs.sum_add(source,StatKey::Lifesteal),Fixed64::ZERO);
    }}
}
#[test]
fn self_transform_invalid_metadata_rank_target_and_life_emit_no_partial_buff() {
    for matchlock in [false,true] {for case in ["json","rank0","rank255","rankhigh","duration0","negative","quantum","huge",
        "type","null","count","count0","count17","probability","stun","leech_high","leech_negative","leech_type","leech_null","leech_quantum","mana","cooldown","casttime","range","point","entity","dead","retired"] {
        let (mut world,source)=fixture();
        let mut data=serde_json::json!({"cooldown":10,"mana_cost":20});
        match case {
            "duration0"=>data["duration"]=0.into(),"negative"=>data["duration"]=(-1).into(),
            "quantum"=>data["duration"]=serde_json::json!(0.0001),"huge"=>data["duration"]=3601.into(),
            "type"=>data["duration"]="5".into(),"null"=>data["duration"]=serde_json::Value::Null,
            "count" if !matchlock=>data["multi_shot_count"]=serde_json::json!(2.5),
            "count0" if !matchlock=>data["multi_shot_count"]=0.into(),
            "count17" if !matchlock=>data["multi_shot_count"]=17.into(),
            "probability" if matchlock=>data["stun_chance"]=serde_json::json!(1.01),
            "stun" if matchlock=>data["stun_duration"]=61.into(),
            "leech_high" if matchlock=>data["lifesteal"]=serde_json::json!(1.01),
            "leech_negative" if matchlock=>data["lifesteal"]=serde_json::json!(-0.1),
            "leech_type" if matchlock=>data["lifesteal"]="0.5".into(),
            "leech_null" if matchlock=>data["lifesteal"]=serde_json::Value::Null,
            "leech_quantum" if matchlock=>data["lifesteal"]=serde_json::json!(0.0001),
            "mana"=>data["mana_cost"]=(-1).into(),"cooldown"=>data["cooldown"]=(-1).into(),
            "casttime"=>data["cast_time"]=1.into(),"range"=>data["range"]=1.into(),
            "dead"=>world.write_storage::<CProperty>().get_mut(source).unwrap().hp=Fixed64::ZERO,
            "retired"=>{world.delete_entity(source).unwrap();world.maintain();},
            "count"|"count0"|"count17"|"probability"|"stun"|"leech_high"|"leech_negative"|"leech_type"|"leech_null"|"leech_quantum"=>continue,_=>{},
        }
        let json=if case=="json" {"{".into()}else{data.to_string()};
        let rank=match case {"rank0"=>0,"rank255"=>255,"rankhigh"=>if matchlock {4}else{5},_=>1};
        let target=match case {"point"=>Target::Point(Vec2::ZERO),"entity"=>Target::Entity(ParallelWorldAdapter::entity_to_handle(source)),_=>Target::None};
        let (success,outcomes)=cast(&world,source,matchlock,rank,&json,target);
        assert!(!success,"matchlock={matchlock} case={case}");assert!(outcomes.is_empty(),"{case}");
    }}
}
#[test]
fn self_transform_explicit_overrides_are_bounded_and_preserve_both_host_encodings() {
    let (mut world,source)=fixture();
    let (success,outcomes)=cast(&world,source,true,2,r#"{"cooldown":0,"mana_cost":0,"duration":2,"range_bonus":0,"damage_bonus":7.5,"stun_chance":1,"stun_duration":0.5,"lifesteal":0.5}"#,Target::None);
    assert!(success);flush(&mut world,outcomes);
    let buffs=world.read_resource::<BuffStore>();let payload=&buffs.iter_for(source).find(|(id,_)|*id=="matchlock_gun").unwrap().1.payload;
    assert_eq!(payload[StatKey::AttackRangeBonus.as_str()],0);assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()],7680);
    assert_eq!(payload["attack_stun_chance"].as_f64(),Some(1.0));assert_eq!(payload["attack_stun_duration"].as_f64(),Some(0.5));
    assert_eq!(payload["lifesteal"],512);
}
