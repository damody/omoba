//! 雨鐵炮（H01，來源 B01R current）：被動；真正普攻命中時依目標類別追加物理傷害
//! （建築／英雄／部隊三檔百分比，以攻擊者當下攻擊力計）。召喚物算部隊。
use abi_stable::std_types::{RErr,RResult,RSome,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_template_ids::{ABILITY_B01_RAIN_IRON_CANNON,active_ability_const};
use crate::ability_builder::build_ability_ffi;

const ID:omoba_template_ids::AbilityId=ABILITY_B01_RAIN_IRON_CANNON;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum TargetClass {Building,Hero,Creep}

/// The host reports a hero's template id (MOBA spawns) or the legacy
/// `hero_<template id>` script tag; both identify a hero template.
pub(crate) fn classify(world:&GameWorldDyn<'_>,victim:EntityHandle)->TargetClass {
    if world.is_building(victim) {return TargetClass::Building;}
    let hero=world.unit_id_of(victim).into_option().and_then(|id| {
        let id=id.as_str();
        omoba_template_ids::hero_by_name(id.strip_prefix("hero_").unwrap_or(id))
    }).is_some();
    if hero {TargetClass::Hero} else {TargetClass::Creep}
}

/// Bonus fraction (1.0 = +100%) for `rank` and `class`.
pub(crate) fn bonus(rank:u8,class:TargetClass)->Option<Fixed64> {
    let content=active_ability_const(ID)?;
    let index=usize::from(rank.checked_sub(1)?);
    if index>=content.levels.len() {return None;}
    let key=match class {
        TargetClass::Building=>"building_pct",
        TargetClass::Hero=>"hero_pct",
        TargetClass::Creep=>"creep_pct",
    };
    let percent=content.extras.iter().find(|(name,_)|*name==key)?.1.get(index).copied()?;
    if percent<Fixed64::ZERO || percent>Fixed64::from_i32(1000) {return None;}
    Some(Fixed64::from_raw(percent.raw()/100))
}

pub struct RainIronCannonClassHandler;
impl AbilityScript for RainIronCannonClassHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_attack_hit(&self,owner:EntityHandle,attacker:EntityHandle,victim:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        if owner!=attacker || attacker==victim {return;}
        if !world.is_alive(victim) || !world.get_hp(victim).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
        let Some(fraction)=bonus(rank,classify(world,victim)) else {return;};
        let attack=world.get_final_atk(attacker);
        if attack<=Fixed64::ZERO || attack>Fixed64::from_i32(1_000_000) {return;}
        let raw=i128::from(attack.raw())*i128::from(fraction.raw())/1024;
        if raw<=0 || raw>i128::from(1_000_000*1024) {return;}
        // AMHC:Damage (B01.lua:335): nothing when Magoichi has died before the shot lands (amhc.lua:540).
        world.deal_damage(victim,Fixed64::from_raw(raw as i64),DamageKind::Physical,
            DamageProfile::NORMAL.union(DamageProfile::REQUIRE_SOURCE_ALIVE),RSome(attacker));
    }
}

pub fn b01_rain_iron_cannon_ffi()->AbilityDefFFI {build_ability_ffi(ID,RainIronCannonClassHandler,Vec::new())}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_rank_and_class_uses_the_source_percentages() {
        for (rank,building,hero,creep) in [(1u8,10,10,50),(2,15,18,70),(3,20,24,90),(4,25,30,110)] {
            for (class,percent) in [(TargetClass::Building,building),(TargetClass::Hero,hero),(TargetClass::Creep,creep)] {
                assert_eq!(bonus(rank,class),Some(Fixed64::from_raw(i64::from(percent)*1024/100)),"rank {rank} {class:?}");
            }
        }
        assert_eq!(bonus(0,TargetClass::Hero),None);
        assert_eq!(bonus(5,TargetClass::Hero),None);
    }
}
