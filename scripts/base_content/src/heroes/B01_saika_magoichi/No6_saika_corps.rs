//! 雜賀眾（H01，來源 B01E current）：在指定地點召喚 1 位不能移動的雜賀眾。
//! 儲存次數（每 15 秒 +1，最多 2/3/4/5）由 host 的 G-2 charges 管理：施放成功才扣
//! 一次；被拒絕（超出距離、放置失敗）不扣次數、不扣魔。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{AbilityLevelData,EffectSpec};
use omoba_template_ids::{ABILITY_B01_SAIKA_CORPS,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B01_SAIKA_CORPS;
pub(crate) const UNIT:&str="b01_saika_corps_gunner";

pub(crate) struct Corps {pub range:Fixed64,pub lifetime:Fixed64}

pub(crate) fn resolve(rank:u8,json:&str)->Result<Corps,&'static str> {
    let content=active_ability_const(ID).ok_or("missing saika corps content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid saika corps rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid saika corps rank");}
    if content.charges.is_none() {return Err("saika corps requires host charges");}
    let data:AbilityLevelData=serde_json::from_str(json).map_err(|_|"invalid saika corps metadata")?;
    if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return Err("invalid saika corps range");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied();
    let lifetime=extra("summon_lifetime").ok_or("missing saika corps lifetime")?;
    if lifetime<=Fixed64::ZERO || lifetime>Fixed64::from_i32(600) {return Err("invalid saika corps lifetime");}
    if extra("summon_count")!=Some(Fixed64::ONE) {return Err("saika corps summons exactly one gunner");}
    if omoba_template_ids::summon_by_name(UNIT).is_none() {return Err("missing saika corps summon template");}
    Ok(Corps {range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),lifetime})
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Corps)->Result<(),&'static str> {
    let Target::Point(point)=target else {return Err("saika corps requires a ground point");};
    let origin=world.get_pos(caster).into_option().ok_or("missing caster position")?;
    if !omoba_sim::terrain::point_in_swept_circle(point,origin,origin,spec.range) {return Err("saika corps point outside cast range");}
    // The host admits placement; a rejected spawn emits nothing, so the cast fails cleanly.
    if !world.spawn_summoned_unit(point,UNIT.into(),caster,spec.lifetime).is_valid() {return Err("saika corps placement rejected");}
    Ok(())
}

pub struct SaikaCorpsHandler;
impl AbilityScript for SaikaCorpsHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level,json.as_str()).and_then(|spec|execute(world,caster,target,spec)) {
            Ok(())=>ROk(()),Err(error)=>RErr(error.into()),
        }
    }
}

pub fn b01_saika_corps_ffi()->AbilityDefFFI {
    let effects=vec![EffectSpec::Summon {unit_type:UNIT.into(),count:1,duration:Some(extra_at_id_f32(ID,"summon_lifetime",1))}];
    build_ability_ffi(ID,SaikaCorpsHandler,effects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B01_saika_magoichi::b01_test_support::*;

    #[test]
    fn authored_charges_and_summon_template_match_the_source() {
        let content=active_ability_const(ID).unwrap();
        let charges=content.charges.expect("host charges");let _=(charges.max_key,charges.recharge_key);
        for (rank,max) in [(1u8,2u16),(2,3),(3,4),(4,5)] {
            let (got_max,recharge)=content.charges_at(rank).unwrap();
            assert_eq!((got_max,recharge),(max,Fixed64::from_i32(15)),"rank {rank}");
        }
        let stats=omoba_template_ids::summon_by_name(UNIT).and_then(omoba_template_ids::active_summon_stats).unwrap();
        assert!(stats.immobile,"source: 雜賀眾無法移動");
        assert_eq!(stats.damage_per_owner_level,Fixed64::from_i32(10),"source: 依雜賀的等級上升攻擊 10 點");
    }

    #[test]
    fn every_rank_summons_one_gunner_inside_range_and_rejects_outside() {
        for rank in 1..=4u8 {
            let mut t=TestWorld::new();
            let inside=resolve(rank,&rank_json(ID,rank)).unwrap();
            assert_eq!(inside.lifetime,Fixed64::from_i32(60),"rank {rank}");
            assert_eq!(inside.range,Fixed64::from_i32(500),"rank {rank}");
            assert!(t.cast(&SaikaCorpsHandler,Target::Point(p(501,0)),rank,&rank_json(ID,rank)).is_err(),"rank {rank} outside 500");
            assert!(t.cast(&SaikaCorpsHandler,Target::None,rank,&rank_json(ID,rank)).is_err());
        }
    }
}
