//! 真田勇士（真田幸村，來源 B06W；history_nobu npc/B_Unified/B06.txt）：以自身為中心
//! radius 內的友方英雄與一般單位（含自己）duration 秒內攻擊 +attack_bonus（PREATTACK_BONUS_DAMAGE）、
//! 每秒回復 hp_regen 生命與 mana_regen 法力（數值見 v3.lua）。以 host ally_area_buff 封包套用（同 id 刷新不疊加）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec,TargetSelector};
use omoba_core::runtime::native::ally_area_buff;
use omoba_template_ids::{ABILITY_B06_BRAVE_WARRIORS,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B06_BRAVE_WARRIORS;

/// The buff the warriors packet puts on every ally hit (one per caster: refresh, no stacking).
pub(crate) fn buff_id(caster:EntityHandle)->String {format!("b06_brave_warriors:{}:{}",caster.id,caster.gen)}

pub(crate) struct Warriors {pub radius:Fixed64,pub seconds:Fixed64,pub attack:Fixed64,pub health_regen:Fixed64,pub mana_regen:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Warriors,&'static str> {
    let content=active_ability_const(ID).ok_or("missing brave warriors content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid brave warriors rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid brave warriors rank");}
    let _:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid brave warriors metadata")?;
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing brave warriors extra");
    Ok(Warriors {radius:extra("radius")?,seconds:extra("duration")?,attack:extra("attack_bonus")?,
        health_regen:extra("hp_regen")?,mana_regen:extra("mana_regen")?})
}

pub(crate) fn packet(caster:EntityHandle,center:omb_script_abi::types::Vec2,spec:&Warriors)->Option<ally_area_buff::Spec> {
    ally_area_buff::Spec {source_id:caster.id,source_generation:caster.gen,center,radius:spec.radius,buff:buff_id(caster),seconds:spec.seconds,
        stats:vec![(StatKey::PreattackBonusDamage.as_str().to_owned(),spec.attack.raw()),
            (StatKey::HealthRegenConstant.as_str().to_owned(),spec.health_regen.raw()),
            (StatKey::ManaRegenConstant.as_str().to_owned(),spec.mana_regen.raw())]}.checked()
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Warriors)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("brave warriors has no target");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    let center=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    let packet=packet(caster,center,&spec).ok_or("invalid brave warriors packet")?;
    world.add_stat_buff(caster,ally_area_buff::BUFF.into(),Fixed64::ONE,packet.payload().to_string().as_str().into());
    Ok(())
}

pub struct BraveWarriorsHandler;
impl AbilityScript for BraveWarriorsHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b06_brave_warriors_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::PreattackBonusDamage.as_str().into(),extra_at_id_f32(ID,"attack_bonus",1));
    modifiers.insert(StatKey::HealthRegenConstant.as_str().into(),extra_at_id_f32(ID,"hp_regen",1));
    modifiers.insert(StatKey::ManaRegenConstant.as_str().into(),extra_at_id_f32(ID,"mana_regen",1));
    build_ability_ffi(ID,BraveWarriorsHandler,vec![EffectSpec::AreaEffect {radius:extra_at_id_f32(ID,"radius",1),duration:0.0,tick_interval:0.0,
        effects:vec![EffectSpec::Buff {target:TargetSelector::AllAllies,duration:Some(extra_at_id_f32(ID,"duration",1)),modifiers}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B06_sanada_yukimura::b06_test_support::*;
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    #[test]
    fn every_rank_buffs_allies_within_500_including_the_caster_for_12_seconds() {
        for (rank,attack,regen) in [(1u8,20,10),(2,30,15),(3,40,20),(4,50,25)] {
            let mut t=TestWorld::new();
            let near=t.body(p(300,0),1);let edge=t.body(p(0,500),1);let out=t.body(p(0,-501),1);let enemy=t.body(p(100,0),2);
            let outcomes=t.cast(&BraveWarriorsHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();t.apply(outcomes);
            let key=buff_id(t.handle(t.source));
            let b=t.world.read_resource::<BuffStore>();
            for e in [t.source,near,edge] {
                let entry=b.get(e,&key).unwrap_or_else(||panic!("rank {rank}: ally {e:?} buffed"));
                assert_eq!(entry.remaining,Fixed64::from_i32(12));
                assert_eq!(entry.payload[StatKey::PreattackBonusDamage.as_str()].as_i64(),Some(i64::from(attack)*1024));
                assert_eq!(entry.payload[StatKey::HealthRegenConstant.as_str()].as_i64(),Some(i64::from(regen)*1024));
                assert_eq!(entry.payload[StatKey::ManaRegenConstant.as_str()].as_i64(),Some(i64::from(regen)*1024));
            }
            for e in [out,enemy] {assert!(!b.has(e,&key),"rank {rank}: {e:?} untouched");}
        }
    }

    #[test]
    fn wrong_target_dead_caster_and_bad_rank_are_rejected_without_outcomes() {
        let mut t=TestWorld::new();let ally=t.body(p(100,0),1);
        assert!(t.cast(&BraveWarriorsHandler,Target::Entity(t.handle(ally)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&BraveWarriorsHandler,Target::None,0,&rank_json(ID,1)).is_err());
        assert!(t.cast(&BraveWarriorsHandler,Target::None,5,&rank_json(ID,1)).is_err());
        let source=t.source;t.kill(source);
        assert!(t.cast(&BraveWarriorsHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }

    #[test]
    fn content_carries_the_history_cooldown_and_mana() {
        let content=active_ability_const(ID).unwrap();
        for level in content.levels.iter() {assert_eq!(level.cooldown,Fixed64::from_i32(10));assert_eq!(level.mana_cost,Fixed64::from_i32(100));}
    }
}
