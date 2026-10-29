//! 炎之迅捷（H02，來源 B04R current，B04.txt:241-271）：被動永久 +agility 敏捷（見 v3.lua）
//! （MODIFIER_PROPERTY_STATS_AGILITY_BONUS）。以 StatKey::StatsAgilityBonus 送出，由 host
//! attributes settle 依屬性規則換算護甲、攻速與主屬性攻擊；技能本身不換算。
//! 每次學習或升級以同一 buff id 取代，不疊加。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B04_FLAME_SWIFTNESS,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B04_FLAME_SWIFTNESS;
pub(crate) const BUFF_ID:&str="b04_flame_swiftness";
const AGILITY:&str="agility";

/// Raw payload for one rank, or None for an invalid rank.
pub(crate) fn payload(rank:u8)->Option<String> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let agility=extra_at_id(ID,AGILITY,rank);
    if agility<=Fixed64::ZERO || agility>Fixed64::from_i32(10_000) {return None;}
    Some(serde_json::json!({StatKey::StatsAgilityBonus.as_str():agility.raw(),"level":rank}).to_string())
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
    modifiers.insert(StatKey::StatsAgilityBonus.as_str().into(),extra_at_id_f32(ID,AGILITY,1));
    build_ability_ffi(ID,FlameSwiftnessHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B02_date_masamune::h02_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_grants_source_agility_and_replaces_the_previous_rank() {
        for (rank,agility) in [(1u8,10i64),(2,20),(3,30),(4,40)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&FlameSwiftnessHandler,rank);
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}: one buff id, so an upgrade replaces instead of stacking");
            let (payload,duration)=&buffs[0];
            assert_eq!(*duration,Fixed64::from_raw(i64::MAX));
            // B04.txt:254 add_agi; the attributes settle converts it (no armor/attack speed here).
            assert_eq!(payload[StatKey::StatsAgilityBonus.as_str()].as_i64(),Some(agility*1024),"rank {rank}");
            assert!(payload.get(StatKey::AttackSpeedBonusConstant.as_str()).is_none(),"rank {rank}");
            assert!(payload.get(StatKey::PhysicalArmorBonus.as_str()).is_none(),"rank {rank}");
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
