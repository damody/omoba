//! 炎之迅捷（H02，來源 B04R current）：被動永久靈活，換算為攻速點與護甲。
//! 每次學習或升級以同一 buff id 取代，不疊加。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B04_FLAME_SWIFTNESS,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_FLAME_SWIFTNESS;
pub(crate) const BUFF_ID:&str="b04_flame_swiftness";
const ATTACK_SPEED:&str="stat_bonus_permanent_attack_speed_bonus";
const ARMOR:&str="stat_bonus_permanent_armor_bonus";

/// Raw payload for one rank, or None for an invalid rank.
pub(crate) fn payload(rank:u8)->Option<String> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let attack_speed=extra_at_id(ID,ATTACK_SPEED,rank);let armor=extra_at_id(ID,ARMOR,rank);
    if attack_speed<Fixed64::ZERO || armor<Fixed64::ZERO {return None;}
    Some(serde_json::json!({StatKey::AttackSpeedBonusConstant.as_str():attack_speed.raw(),
        StatKey::PhysicalArmorBonus.as_str():armor.raw(),"level":rank}).to_string())
}

pub struct FlameSwiftnessHandler;
impl AbilityScript for FlameSwiftnessHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(BUFF_ID),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn b04_flame_swiftness_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(),extra_at_id_f32(ID,ATTACK_SPEED,1));
    modifiers.insert(StatKey::PhysicalArmorBonus.as_str().into(),extra_at_id_f32(ID,ARMOR,1));
    build_ability_ffi(ID,FlameSwiftnessHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omoba_core::Outcome;

    fn q(v:f64)->i64 {(v*1024.0).round() as i64}

    #[test]
    fn every_rank_grants_converted_agility_and_replaces_the_previous_rank() {
        for (rank,attack_speed,armor) in [(1u8,10.0,1.667),(2,20.0,3.333),(3,30.0,5.0),(4,40.0,6.667)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&FlameSwiftnessHandler,rank);
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}: one buff id, so an upgrade replaces instead of stacking");
            let (payload,duration)=&buffs[0];
            assert_eq!(*duration,Fixed64::from_raw(i64::MAX));
            assert_eq!(payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64(),Some(q(attack_speed)),"rank {rank}");
            assert_eq!(payload[StatKey::PhysicalArmorBonus.as_str()].as_i64(),Some(q(armor)),"rank {rank}");
        }
    }

    #[test]
    fn passive_cannot_be_cast_and_invalid_rank_learns_nothing() {
        let mut t=TestWorld::new();
        assert!(t.cast(&FlameSwiftnessHandler,Target::None,1,"{}").is_err());
        assert!(t.learn(&FlameSwiftnessHandler,0).is_empty());
        assert!(t.learn(&FlameSwiftnessHandler,5).is_empty());
    }
}
