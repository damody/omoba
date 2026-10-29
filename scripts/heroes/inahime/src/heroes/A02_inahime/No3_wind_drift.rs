//! 遊風行身（H05，來源 A02R current）：被動魔法抗性與總攻擊傷害加成（見 v3.lua 的
//! magic_resist、damage_pct）。每次學習或升級以同一 buff id 取代，不疊加。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A02_WIND_DRIFT,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A02_WIND_DRIFT;
pub(crate) const BUFF_ID:&str="a02_wind_drift";
const RESIST:&str="magic_resist";
const DAMAGE:&str="damage_pct";

/// Raw payload for one rank, or None for an invalid rank. Both stats are
/// authored in percent: magic resistance stays in percent points, damage
/// outgoing is a fraction of one.
pub(crate) fn payload(rank:u8)->Option<String> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let resist=extra_at_id(ID,RESIST,rank);let damage=extra_at_id(ID,DAMAGE,rank);
    if resist<=Fixed64::ZERO || resist>=Fixed64::from_i32(100) || damage<=Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::MagicalResistanceBonus.as_str()):resist.raw(),
        (StatKey::BaseDamageOutgoingPercentage.as_str()):(damage.raw()+50)/100,"level":rank}).to_string())
}

pub struct WindDriftHandler;
impl AbilityScript for WindDriftHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(BUFF_ID),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn a02_wind_drift_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::MagicalResistanceBonus.as_str().into(),extra_at_id_f32(ID,RESIST,1));
    modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(),extra_at_id_f32(ID,DAMAGE,1)/100.0);
    build_ability_ffi(ID,WindDriftHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A02_inahime::a02_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_grants_resist_and_damage_and_replaces_the_previous_rank() {
        for (rank,resist,damage_raw) in [(1u8,9,61),(2,18,92),(3,27,123),(4,36,154)] {
            let mut t=TestWorld::new();
            let buffs:Vec<_>=t.learn(&WindDriftHandler,rank).iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}: one buff id, so an upgrade replaces instead of stacking");
            let (payload,duration)=&buffs[0];
            assert_eq!(*duration,Fixed64::from_raw(i64::MAX));
            assert_eq!(payload[StatKey::MagicalResistanceBonus.as_str()].as_i64(),Some(resist*1024),"rank {rank}");
            assert_eq!(payload[StatKey::BaseDamageOutgoingPercentage.as_str()].as_i64(),Some(damage_raw),"rank {rank}");
        }
    }

    #[test]
    fn passive_cannot_be_cast_and_invalid_rank_learns_nothing() {
        let mut t=TestWorld::new();
        assert!(t.cast(&WindDriftHandler,Target::None,1,"{}").is_err());
        assert!(t.learn(&WindDriftHandler,0).is_empty());
        assert!(t.learn(&WindDriftHandler,5).is_empty());
    }
}
