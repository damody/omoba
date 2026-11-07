//! 一刀．牙（H06，來源 B08W current，B08.txt:143-287）：朝方向斬出刀牙（半徑、固定
//! 飛行距離與速度見 v3.lua 的 radius／distance／speed，與施法距離分開），穿透沿途敵人，
//! 各受 damage 魔法傷害一次。施放後自身 command_lock 秒 command_restricted（modifier_B08W，253-261）。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_B08_FANG_SLASH;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};
use super::b08_fang;

const ID:omoba_template_ids::AbilityId=ABILITY_B08_FANG_SLASH;

pub struct FangSlashHandler;
impl AbilityScript for FangSlashHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match b08_fang::resolve(ID,level,json.as_str()).and_then(|spec|b08_fang::launch(world,ID,caster,target,spec)) {
            Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn b08_fang_slash_ffi()->AbilityDefFFI {
    build_ability_ffi(ID,FangSlashHandler,vec![EffectSpec::Projectile {speed:extra_at_id_f32(ID,"speed",1),on_hit:vec![
        EffectSpec::Damage {target:TargetSelector::Target,amount:extra_at_id_f32(ID,"damage",1),damage_type:DamageType::Magical}]}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omb_script_abi::types::Fixed64;
    use omoba_core::Outcome;
    use omoba_core::runtime::native::ability_projectile as host;

    #[test]
    fn every_rank_launches_a_piercing_fang_with_rank_damage() {
        for (rank,damage) in [(1u8,200),(2,300),(3,400),(4,500)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&FangSlashHandler,Target::Point(p(100,0)),rank,&rank_json(ID,rank)).unwrap();
            let spec=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==host::BUFF=>
                host::Spec::from_payload(host::BUFF,*duration,payload),_=>None}).expect("launch");
            assert!(matches!(spec.motion,host::Motion::Linear {..}) && !spec.buildings && spec.dot.is_none());
            assert_eq!((spec.distance,spec.speed,spec.max_hits),(Fixed64::from_i32(1200),Fixed64::from_i32(1500),64),"rank {rank}: flies 1200");
            assert_eq!(spec.radius,Fixed64::from_i32(175),"source StartRadius/EndRadius 175 is a radius");
            assert_eq!(spec.amount,Fixed64::from_i32(damage),"rank {rank}");
            assert!(!outcomes.iter().any(|o|matches!(o,Outcome::RawDamage {..})),"damage only on contact");
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {target,buff_id,duration,..}
                if *target==t.source && buff_id==omoba_core::runtime::native::command_restriction::BUFF && *duration==Fixed64::from_raw(102))),
                "0.1 s command restriction on the caster");
        }
        let content=omoba_template_ids::active_ability_const(ID).unwrap();
        assert!(content.levels.iter().all(|l|l.range==Fixed64::from_i32(1100) && l.cast_time==Fixed64::from_raw(102)),"cast range 1100, cast point 0.1");
    }

    #[test]
    fn no_direction_or_wrong_target_is_rejected() {
        let mut t=TestWorld::new();let enemy=t.body(p(300,0),2);
        assert!(t.cast(&FangSlashHandler,Target::Point(p(0,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&FangSlashHandler,Target::Entity(t.handle(enemy)),1,&rank_json(ID,1)).is_err());
    }
}
