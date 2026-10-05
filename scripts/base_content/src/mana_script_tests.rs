//! Real manifest metadata + resource handlers through Production60Hz dispatch.
use abi_stable::{prefix_type::PrefixTypeTrait,sabi_trait::prelude::TD_Opaque,
    std_types::{RStr,RString,RResult,ROk,RErr,RVec}};
use omb_script_abi::{ability::{AbilityScript,AbilityScript_TO,AbilityDefFFI},
    script::{UnitScript,UnitScript_TO},manifest::{Manifest,UnitDef},
    types::{EntityHandle,Fixed64,Target},world::{GameWorldDyn,TowerCooldownAccessDyn}};
use omoba_core::runtime::*;
use specs::{World,WorldExt};

struct ResourceAbility {id:String}
impl AbilityScript for ResourceAbility {
    fn ability_id(&self)->RStr<'_> {self.id.as_str().into()}
    fn execute(&self,caster:EntityHandle,target:Target,rank:u8,_data:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let recipient=if let Target::Entity(entity)=target {entity} else {caster};
        world.restore_mana(recipient,Fixed64::from_i32(20));
        if !world.spend_mana(recipient,Fixed64::from_i32(10),"extra".into()) {return RErr("extra cost rejected".into());}
        if rank==2 {return RErr("fixture rollback".into());}
        ROk(())
    }
}
struct ResourceUnit;
impl UnitScript for ResourceUnit {
    fn unit_id(&self)->RStr<'_> {"training_ranger".into()}
    fn on_tower_tick(&self,owner:EntityHandle,_dt:Fixed64,_cd:&mut TowerCooldownAccessDyn<'_>,world:&mut GameWorldDyn<'_>) {
        let _=world.spend_mana(owner,Fixed64::ONE,"tick".into());
    }
}
extern "C" fn units()->RVec<UnitDef> {
    let mut defs=crate::units();
    defs.push(UnitDef {unit_id:"training_ranger".into(),
        script:UnitScript_TO::from_value(ResourceUnit,TD_Opaque)});
    defs
}
extern "C" fn abilities()->RVec<AbilityDefFFI> {
    crate::abilities().into_iter().map(|mut def| {
        let data:serde_json::Value=serde_json::from_str(def.def_json.as_str()).unwrap();
        let id=data["id"].as_str().unwrap();
        if matches!(id,"ranger_patch"|"ranger_finisher") {
            def.script=AbilityScript_TO::from_value(ResourceAbility {id:id.into()},TD_Opaque);
        }
        def
    }).collect()
}
fn fixture()->(World,SimulationDriver,specs::Entity) {fixture_with_abilities(abilities)}
fn fixture_with_abilities(ability_factory:extern "C" fn()->RVec<AbilityDefFFI>)->(World,SimulationDriver,specs::Entity) {
    fixture_with_mana(ability_factory,false)
}
fn fixture_with_mana(ability_factory:extern "C" fn()->RVec<AbilityDefFFI>,mana_enabled:bool)->(World,SimulationDriver,specs::Entity) {
    let mut world=StateInitializer::setup_campaign_ecs_world(&StateInitializer::create_thread_pool());
    world.insert(TowerTemplateRegistry::default());
    let mut registry=ScriptRegistry::new();
    registry.insert_manifest(Manifest {units,abilities:ability_factory,dev_reload_runtime_lua_content:crate::dev_reload_runtime_lua_content}.leak_into_prefix());
    populate_ability_registry(&mut world,&registry);world.insert(registry);
    setup_single_lane_match(&mut world,SingleLaneConfig {warmup:Fixed64::ZERO,mana_enabled,base_recovery_enabled:false,
        heroes:["training_ranger".into(),"training_luminary".into()],
        wave_interval:Fixed64::from_i32(10_000),..Default::default()}).unwrap();
    let mut driver=SimulationDriver::from_world(&mut world,SimulationTickProfile::Production60Hz).unwrap();
    driver.step(&mut world,[]).unwrap();
    let caster=world.read_resource::<MobaMatch>().heroes[0].entity.unwrap();
    world.write_storage::<Pos>().get_mut(caster).unwrap().0=
        omoba_sim::Vec2::new(Fixed64::ZERO,Fixed64::from_i32(2500));
    world.write_storage::<Hero>().get_mut(caster).unwrap().mana_pool=Some(
        ability_runtime::ManaPool::new(Fixed64::from_i32(90),Fixed64::from_i32(280)).unwrap());
    // Heroes do not implicitly acquire unit tick hooks. This fixture explicitly
    // binds its resource producer through the same tag used by native scripts.
    world.write_storage::<ScriptUnitTag>().insert(caster,ScriptUnitTag {
        unit_id:"training_ranger".into(),
    }).unwrap();
    (world,driver,caster)
}

#[test]
fn mana_script_formal_cast_and_tick_share_post_cost_balance_and_notifications() {
    let (mut world,mut driver,caster)=fixture();
    driver.step(&mut world,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:None}))})]).unwrap();
    // Metadata45 once, restore20, explicit extra10, then tick1. A stale tick
    // cache would overwrite this with89; omitting metadata would yield99.
    assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(54));
    let events=world.write_resource::<ScriptEventQueue>().drain();
    let costs:Vec<_>=events.iter().filter_map(|event|match event {
        ScriptEvent::SpentMana {caster:e,cost,..} if *e==caster=>Some(cost.raw()),_=>None}).collect();
    assert_eq!(costs,vec![45*1024,10*1024,1024]);
    assert!(events.iter().any(|event|matches!(event,ScriptEvent::ManaGained {e,amount} if *e==caster && *amount==Fixed64::from_i32(20))));
}

#[test]
fn mana_script_same_batch_is_ordered_and_failed_cast_rolls_back_resources() {
    for reject in [false,true] {
        let (mut world,mut driver,caster)=fixture();
        if reject {world.write_storage::<Hero>().get_mut(caster).unwrap().ability_levels.insert("ranger_patch".into(),2);}
        {let mut queue=world.write_resource::<ScriptEventQueue>();
            for id in if reject {vec!["ranger_patch"]} else {vec!["ranger_patch","ranger_finisher","ranger_patch"]} {
                queue.push(ScriptEvent::SkillCast {caster,skill_id:id.into(),target:SkillTarget::None});
            }}
        driver.step(&mut world,[]).unwrap();
        let heroes=world.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
        assert_eq!(hero.mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(if reject {89} else {19}));
        assert_eq!(hero.is_on_cooldown("ranger_patch"),!reject);
        drop(heroes);
        let events=world.write_resource::<ScriptEventQueue>().drain();
        if reject {
            assert!(!events.iter().any(|event|matches!(event,ScriptEvent::ManaGained {..})));
            assert_eq!(events.iter().filter(|event|matches!(event,ScriptEvent::SpentMana {..})).count(),1,"only successful tick spend survives");
        }
    }
}

#[test]
fn mana_script_legacy_caster_failure_rolls_back_managed_recipient() {
    let (mut world,mut driver,caster)=fixture();
    let recipient=world.read_resource::<MobaMatch>().heroes[1].entity.unwrap();
    {
        let mut heroes=world.write_storage::<Hero>();
        let hero=heroes.get_mut(caster).unwrap();
        hero.mana_pool=None;
        hero.ability_levels.insert("ranger_patch".into(),2);
        heroes.get_mut(recipient).unwrap().mana_pool=Some(
            ability_runtime::ManaPool::new(Fixed64::from_i32(50),Fixed64::from_i32(100)).unwrap());
    }
    world.write_resource::<ScriptEventQueue>().push(ScriptEvent::SkillCast {
        caster,skill_id:"ranger_patch".into(),target:SkillTarget::Entity(recipient),
    });
    driver.step(&mut world,[]).unwrap();
    let heroes=world.read_storage::<Hero>();
    assert_eq!(heroes.get(recipient).unwrap().mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(50));
    assert!(!heroes.get(caster).unwrap().is_on_cooldown("ranger_patch"));
    drop(heroes);
    let events=world.write_resource::<ScriptEventQueue>().drain();
    assert!(!events.iter().any(|event|matches!(event,
        ScriptEvent::ManaGained {e,..} if *e==recipient)));
    assert!(!events.iter().any(|event|matches!(event,
        ScriptEvent::SpentMana {caster:e,..} if *e==recipient)));
}

extern "C" fn declarative_mana_abilities()->RVec<AbilityDefFFI> {
    crate::abilities().into_iter().map(|def| {
        let data:serde_json::Value=serde_json::from_str(def.def_json.as_str()).unwrap();
        if data["id"]=="ranger_patch" {
            use crate::generic_effects::EffectOp;
            // Existing Lua rank data supplies all amounts (55 at rank1).
            crate::generic_effects::generic_effect_ffi(
                omoba_template_ids::ability_by_name("ranger_patch").unwrap(),&[
                    EffectOp::HealSelf {amount_key:"heal"},
                    EffectOp::RestoreManaSelf {amount_key:"heal"},
                    EffectOp::SpendManaSelf {amount_key:"heal"},
                    EffectOp::SpendManaSelf {amount_key:"heal"},
                ])
        } else {def}
    }).collect()
}

extern "C" fn declarative_buff_abilities()->RVec<AbilityDefFFI> {
    crate::abilities().into_iter().map(|def| {
        let data:serde_json::Value=serde_json::from_str(def.def_json.as_str()).unwrap();
        if data["id"]=="ranger_patch" {
            use crate::generic_effects::EffectOp;
            crate::generic_effects::generic_effect_ffi(omoba_template_ids::ability_by_name("ranger_patch").unwrap(),&[
                EffectOp::ManaBuffSelf {stat:omoba_content_model::ManaBuffStat::ManaBonus,value_key:"heal",duration_key:"heal"},
            ])
        } else {def}
    }).collect()
}

#[test]
fn mana_buff_declaration_formal_cast_refreshes_one_identity_and_failure_preserves_existing_buff() {
    let (mut world,mut driver,caster)=fixture_with_mana(declarative_buff_abilities,true);
    world.write_storage::<ScriptUnitTag>().remove(caster);
    world.write_resource::<BuffStore>().add(caster,"fixture_no_regen",Fixed64::from_i32(100),
        serde_json::json!({"mana_regen_percentage":-1024}));
    let key=format!("generic_mana:ranger_patch:mana_bonus:{}:{}",caster.id(),caster.gen().id());
    let cast=||[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
        ability_index:1,target_entity:None,target_pos:None}))})];
    driver.step(&mut world,cast()).unwrap();
    assert_eq!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().payload["mana_bonus"],55*1024);
    assert_eq!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining,Fixed64::from_i32(55));
    assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().raw_state(),
        (45*1024,335*1024,0),"deferred capacity buff commits without restoring mana");
    {
        let mut heroes=world.write_storage::<Hero>();let hero=heroes.get_mut(caster).unwrap();
        hero.start_cooldown("ranger_patch",Fixed64::ZERO);hero.mana_pool.as_mut().unwrap().restore(Fixed64::from_i32(100)).unwrap();
    }
    driver.step(&mut world,cast()).unwrap();
    assert_eq!(world.read_resource::<BuffStore>().iter_for(caster).filter(|(id,_)|id.starts_with("generic_mana:")).count(),1);
    assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap().maximum(),Fixed64::from_i32(335));
    assert_eq!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining,Fixed64::from_i32(55));
    {
        let mut heroes=world.write_storage::<Hero>();let hero=heroes.get_mut(caster).unwrap();
        hero.start_cooldown("ranger_patch",Fixed64::ZERO);hero.ability_levels.insert("ranger_patch".into(),2);
        hero.mana_pool.as_mut().unwrap().restore(Fixed64::from_i32(100)).unwrap();
    }
    let before=world.read_storage::<Hero>().get(caster).unwrap().mana_pool.clone().unwrap();
    driver.step(&mut world,cast()).unwrap(); // invalid duration85; no new buff or cost
    assert_eq!(world.read_storage::<Hero>().get(caster).unwrap().mana_pool.as_ref().unwrap(),&before);
    assert!(!world.read_storage::<Hero>().get(caster).unwrap().is_on_cooldown("ranger_patch"));
    assert_eq!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().payload["mana_bonus"],55*1024);
    assert!(world.read_resource::<BuffStore>().get(caster,&key).unwrap().remaining<Fixed64::from_i32(55));
}

#[test]
fn mana_effect_formal_60hz_capacity_order_admission_and_host_rollback() {
    for success in [true,false] {
        let (mut world,mut driver,caster)=fixture_with_abilities(declarative_mana_abilities);
        // Isolate this declaration from the independent tick producer.
        world.write_storage::<ScriptUnitTag>().remove(caster);
        {
            let mut heroes=world.write_storage::<Hero>();
            heroes.get_mut(caster).unwrap().mana_pool=Some(ability_runtime::ManaPool::new(
                Fixed64::from_i32(if success {145} else {50}),Fixed64::from_i32(150)).unwrap());
        }
        world.write_storage::<CProperty>().get_mut(caster).unwrap().hp=Fixed64::from_i32(100);
        driver.step(&mut world,[(1,PlayerInput {action:Some(PlayerInputEnum::CastAbility(CastAbility {
            ability_index:1,target_entity:None,target_pos:None}))})]).unwrap();
        let heroes=world.read_storage::<Hero>();let hero=heroes.get(caster).unwrap();
        // 145-45+55 capped150-55-55 = 40; insufficient second spend
        // after 50-45+55-55 rolls back pool, metadata/CD and pending heal.
        assert_eq!(hero.mana_pool.as_ref().unwrap().current(),Fixed64::from_i32(if success {40} else {50}));
        assert_eq!(hero.is_on_cooldown("ranger_patch"),success);
        drop(heroes);
        assert_eq!(world.read_storage::<CProperty>().get(caster).unwrap().hp,Fixed64::from_i32(if success {155} else {100}));
        let events=world.write_resource::<ScriptEventQueue>().drain();
        let costs:Vec<_>=events.iter().filter_map(|event|match event {
            ScriptEvent::SpentMana {caster:e,cost,..} if *e==caster=>Some(cost.raw()),_=>None}).collect();
        assert_eq!(costs,if success {vec![45*1024,55*1024,55*1024]} else {vec![]});
        let gains:Vec<_>=events.iter().filter_map(|event|match event {
            ScriptEvent::ManaGained {e,amount} if *e==caster=>Some(amount.raw()),_=>None}).collect();
        assert_eq!(gains,if success {vec![50*1024]} else {vec![]});
    }
}
