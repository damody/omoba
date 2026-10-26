//! 急速冷卻（H03，來源 A34T current）：
//! - 被動：最大法力 +300/400/500（學習／升級以同一 buff id 取代，不重灌法力）。
//! - 被動：施放基礎技能時，其他基礎技能剩餘冷卻 −2 秒。由 content
//!   `cast_cooldown_refund` 宣告、host 在施法結算時處理，本 handler 不參與。
//! - 主動：22 秒內冷卻 ×(1−40/50/60%)。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A34_RAPID_COOLING,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_RAPID_COOLING;
pub(crate) const PASSIVE_BUFF:&str="a34_rapid_cooling_mana";
pub(crate) const ACTIVE_BUFF:&str="a34_rapid_cooling";
const MANA:&str="stat_bonus_permanent_mana_bonus";
const REDUCTION:&str="stat_buff_cooldown_reduction_pct";
const DURATION:&str="stat_buff_duration";

fn rank_ok(rank:u8)->bool {
    active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())
}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let mana=extra_at_id(ID,MANA,rank);
    if mana<=Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::ManaBonus.as_str()):mana.raw(),"level":rank}).to_string())
}

/// (payload, duration) of the active cooldown buff; the reduction is
/// authored in percent and the stat is a (negative) fraction of one.
pub(crate) fn active_payload(rank:u8)->Option<(String,Fixed64)> {
    if !rank_ok(rank) {return None;}
    let pct=extra_at_id(ID,REDUCTION,rank);let duration=extra_at_id(ID,DURATION,rank);
    if pct<=Fixed64::ZERO || pct>=Fixed64::from_i32(90) || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60) {return None;}
    Some((serde_json::json!({(StatKey::CooldownPercentage.as_str()):-((pct.raw()+50)/100)}).to_string(),duration))
}

pub struct RapidCoolingHandler;
impl AbilityScript for RapidCoolingHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,_:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        if !matches!(target,Target::None) {return RErr("rapid cooling is a self cast".into());}
        let Some((payload,duration))=active_payload(level) else {return RErr("invalid rapid cooling rank".into());};
        if !world.is_alive(caster) {return RErr("caster is not alive".into());}
        world.add_stat_buff(caster,RStr::from_str(ACTIVE_BUFF),duration,payload.as_str().into());
        ROk(())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn a34_rapid_cooling_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::CooldownPercentage.as_str().into(),-extra_at_id_f32(ID,REDUCTION,1)/100.0);
    build_ability_ffi(ID,RapidCoolingHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,
        duration:Some(extra_at_id_f32(ID,DURATION,1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A34_omatsu::a34_test_support::*;
    use omoba_core::Outcome;

    fn buffs(outcomes:&[Outcome],id:&str)->Vec<(serde_json::Value,Fixed64)> {
        outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==id=>Some((payload.clone(),*duration)),_=>None}).collect()
    }

    #[test]
    fn learning_grants_rank_mana_and_content_declares_the_cast_refund() {
        for (rank,mana) in [(1u8,300),(2,400),(3,500)] {
            let mut t=TestWorld::new();
            let rows=buffs(&t.learn(&RapidCoolingHandler,rank),PASSIVE_BUFF);
            assert_eq!(rows.len(),1,"rank {rank}: one id, upgrades replace");
            assert_eq!(rows[0].0[StatKey::ManaBonus.as_str()].as_i64(),Some(mana*1024),"rank {rank}");
            let content=active_ability_const(ID).unwrap();
            assert_eq!(content.cast_cooldown_refund_at(rank),Some(Fixed64::from_i32(2)),"rank {rank}");
        }
    }

    #[test]
    fn casting_grants_twenty_two_seconds_of_rank_cooldown_reduction() {
        for (rank,raw) in [(1u8,-410),(2,-512),(3,-614)] {
            let mut t=TestWorld::new();
            let rows=buffs(&t.cast(&RapidCoolingHandler,Target::None,rank,&rank_json(ID,rank)).unwrap(),ACTIVE_BUFF);
            assert_eq!(rows,vec![(serde_json::json!({(StatKey::CooldownPercentage.as_str()):raw}),Fixed64::from_i32(22))],"rank {rank}");
        }
        let mut t=TestWorld::new();
        assert!(t.cast(&RapidCoolingHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RapidCoolingHandler,Target::None,4,"{}").is_err());
        assert!(t.learn(&RapidCoolingHandler,0).is_empty());
    }
}
