//! 種子島（H02，來源 B04T current）：限時換上鐵砲，最終射程 800、基礎攻擊加成、
//! 普攻機率停頓（沿用 matchlock_gun 的 host 攻擊暈眩機制，不含吸血）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_B04_TANEGASHIMA;
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use crate::self_transform::{Encoding,Field};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_TANEGASHIMA;
pub(crate) const BUFF_ID:&str="b04_tanegashima";
const FIELDS:&[Field]=&[
    Field {key:"stat_buff_attack_range_bonus",stat:StatKey::AttackRangeBonus,maximum:10_000,encoding:Encoding::Raw},
    Field {key:"stat_buff_attack_damage_bonus",stat:StatKey::BaseAttackBonusDamage,maximum:1_000_000,encoding:Encoding::Raw},
    Field {key:"stat_buff_attack_stun_chance",stat:StatKey::AttackStunChance,maximum:1,encoding:Encoding::Normalized},
    Field {key:"stat_buff_attack_stun_duration",stat:StatKey::AttackStunDuration,maximum:60,encoding:Encoding::Normalized},
];

pub struct TanegashimaHandler;
impl AbilityScript for TanegashimaHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::self_transform::resolve_keyed(ID,level,json.as_str(),FIELDS,None,"stat_buff_duration",true)
            .and_then(|spec|crate::self_transform::execute(world,caster,target,BUFF_ID,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn b04_tanegashima_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    for field in FIELDS {modifiers.insert(field.stat.as_str().into(),extra_at_id_f32(ID,field.key,1));}
    build_ability_ffi(ID,TanegashimaHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,"stat_buff_duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_equips_the_gun_for_its_duration_with_final_range_800() {
        for (rank,duration,damage) in [(1u8,28,50),(2,32,70),(3,36,90)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&TanegashimaHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}");
            let (payload,got)=&buffs[0];
            assert_eq!(*got,Fixed64::from_i32(duration),"rank {rank}");
            // 250 game melee base + 550 = the described final range 800.
            assert_eq!(payload[StatKey::AttackRangeBonus.as_str()].as_i64(),Some(550*1024),"rank {rank}");
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(i64::from(damage)*1024),"rank {rank}");
            let chance=payload[StatKey::AttackStunChance.as_str()].as_f64().unwrap();
            let stun=payload[StatKey::AttackStunDuration.as_str()].as_f64().unwrap();
            assert!((chance-0.38).abs()<1.0/1024.0 && (stun-0.2).abs()<1.0/1024.0,"rank {rank}: {chance} {stun}");
            assert!(payload.get(StatKey::Lifesteal.as_str()).is_none(),"source has no lifesteal");
        }
    }

    #[test]
    fn windup_metadata_is_accepted_but_targets_and_ranks_are_strict() {
        let mut t=TestWorld::new();
        assert!(t.cast(&TanegashimaHandler,Target::None,1,&rank_json(ID,1)).is_ok());
        assert!(t.cast(&TanegashimaHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&TanegashimaHandler,Target::None,4,&rank_json(ID,3)).is_err());
        let ranged=serde_json::json!({"cooldown":110,"mana_cost":150,"cast_time":0.1,"range":800}).to_string();
        assert!(t.cast(&TanegashimaHandler,Target::None,1,&ranged).is_err(),"self transform keeps rejecting a cast range");
    }
}
