//! 狙擊模式（H01，來源 B01W current）：12 秒限時自身增益，攻擊力與攻擊距離加成，
//! 沒有本地 toggle 版的攻速／移速懲罰。夜晚視野不適用（無日夜循環）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_B01_SNIPER_MODE;
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use crate::self_transform::{Encoding,Field};

const ID:omoba_template_ids::AbilityId=ABILITY_B01_SNIPER_MODE;
pub(crate) const BUFF_ID:&str="b01_sniper_mode";
const FIELDS:&[Field]=&[
    Field {key:"stat_buff_attack_damage_bonus",stat:StatKey::BaseAttackBonusDamage,maximum:1_000_000,encoding:Encoding::Raw},
    Field {key:"stat_buff_attack_range_bonus",stat:StatKey::AttackRangeBonus,maximum:10_000,encoding:Encoding::Raw},
];

pub struct SniperModeTimedHandler;
impl AbilityScript for SniperModeTimedHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::self_transform::resolve_keyed(ID,level,json.as_str(),FIELDS,None,"stat_buff_duration",true)
            .and_then(|spec|crate::self_transform::execute(world,caster,target,BUFF_ID,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn b01_sniper_mode_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    for field in FIELDS {modifiers.insert(field.stat.as_str().into(),extra_at_id_f32(ID,field.key,1));}
    build_ability_ffi(ID,SniperModeTimedHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"stat_buff_duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B01_saika_magoichi::b01_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_grants_its_attack_and_range_for_twelve_seconds() {
        for (rank,damage,range) in [(1u8,0,200),(2,10,350),(3,20,500),(4,30,700)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&SniperModeTimedHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}");
            let (payload,duration)=&buffs[0];
            assert_eq!(*duration,Fixed64::from_i32(12),"rank {rank}");
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(i64::from(damage)*1024),"rank {rank}");
            assert_eq!(payload[StatKey::AttackRangeBonus.as_str()].as_i64(),Some(i64::from(range)*1024),"rank {rank}");
            // The source current version has no attack/move speed penalty (unlike the local toggle).
            assert!(payload.get(StatKey::AttackSpeedBonusConstant.as_str()).is_none());
            assert!(payload.get(StatKey::MoveSpeedBonusPercentage.as_str()).is_none());
        }
    }

    #[test]
    fn wrong_target_or_rank_is_rejected_without_outcomes() {
        let mut t=TestWorld::new();
        assert!(t.cast(&SniperModeTimedHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&SniperModeTimedHandler,Target::None,5,&rank_json(ID,4)).is_err());
    }
}
