//! 一刀．空牙（H06，來源 B08T current）：朝方向斬出巨大刀牙（寬 500、飛行 3000、
//! 速度 3000），穿透沿途敵人，各受 800/1200/1600 魔法傷害一次。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_B08_SKY_FANG;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use super::b08_fang;

const ID:omoba_template_ids::AbilityId=ABILITY_B08_SKY_FANG;

pub struct SkyFangHandler;
impl AbilityScript for SkyFangHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match b08_fang::resolve(ID,level,json.as_str()).and_then(|spec|b08_fang::launch(world,ID,caster,target,spec)) {
            Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b08_sky_fang_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,SkyFangHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"projectile_speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage_amount",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;
    use omoba_core::runtime::native::ability_projectile as host;

    #[test]
    fn every_rank_launches_a_wide_fang_over_three_thousand_units() {
        for (rank,damage) in [(1u8,800),(2,1200),(3,1600)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&SkyFangHandler,Target::Point(p(0,100)),rank,&rank_json(ID,rank)).unwrap();
            let spec=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>
                host::Spec::from_payload(host::BUFF,*duration,payload),_=>None}).expect("launch");
            assert_eq!((spec.distance,spec.speed,spec.radius,spec.max_hits),
                (Fixed64::from_i32(3000),Fixed64::from_i32(3000),Fixed64::from_i32(250),64),"rank {rank}");
            assert_eq!(spec.amount,Fixed64::from_i32(damage),"rank {rank}");
        }
    }

    #[test]
    fn invalid_rank_is_rejected() {
        let mut t=TestWorld::new();
        assert!(t.cast(&SkyFangHandler,Target::Point(p(0,100)),4,&rank_json(ID,3)).is_err());
        assert!(t.cast(&SkyFangHandler,Target::None,1,&rank_json(ID,1)).is_err());
    }
}
