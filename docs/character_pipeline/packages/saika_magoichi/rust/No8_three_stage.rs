//! 三段（H01，來源 B01T current）：7/11/13 秒基礎傷害 +200%；每次普攻只判定一次。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_B01_THREE_STAGE;
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use crate::self_transform::{Encoding,Field};

const ID:omoba_template_ids::AbilityId=ABILITY_B01_THREE_STAGE;
pub(crate) const BUFF_ID:&str="b01_three_stage";
const FIELDS:&[Field]=&[
    Field {key:"stat_buff_base_damage_outgoing_pct",stat:StatKey::BaseDamageOutgoingPercentage,maximum:100_000,encoding:Encoding::Percent},
];

pub struct ThreeStageHandler;
impl AbilityScript for ThreeStageHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::self_transform::resolve_keyed(ID,level,json.as_str(),FIELDS,None,"stat_buff_duration",true)
            .and_then(|spec|crate::self_transform::execute(world,caster,target,BUFF_ID,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn b01_three_stage_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(),
        extra_at_id_f32(ID,"stat_buff_base_damage_outgoing_pct",1)/100.0);
    build_ability_ffi(ID,ThreeStageHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"stat_buff_duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B01_saika_magoichi::b01_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_doubles_base_damage_bonus_to_plus_200_percent_for_its_duration() {
        for (rank,duration) in [(1u8,7),(2,11),(3,13)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&ThreeStageHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}");
            assert_eq!(buffs[0].1,Fixed64::from_i32(duration),"rank {rank}");
            // +200% = raw fraction 2.0 (Q10), the same encoding sniper_mode uses for +15%.
            assert_eq!(buffs[0].0[StatKey::BaseDamageOutgoingPercentage.as_str()].as_i64(),Some(2*1024),"rank {rank}");
        }
    }

    #[test]
    fn rank_four_does_not_exist_and_targets_are_rejected() {
        let mut t=TestWorld::new();
        assert!(t.cast(&ThreeStageHandler,Target::None,4,&rank_json(ID,3)).is_err());
        assert!(t.cast(&ThreeStageHandler,Target::Point(p(10,0)),1,&rank_json(ID,1)).is_err());
    }
}
