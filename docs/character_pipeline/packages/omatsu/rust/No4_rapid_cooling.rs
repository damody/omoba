//! 急速冷卻（H03，來源 A34T current，A34.txt:509-664、A34_Matsu.lua:10-123）：
//! 數值見 v3.lua（mana_bonus、cooldown_refund、duration、cooldown_cap、mana_on_cast、shred*）。
//! - 被動：最大法力 +mana_bonus（學習／升級以同一 buff id 取代，不重灌法力）。
//! - 被動：施放基礎技能時，其他基礎技能剩餘冷卻 −cooldown_refund。由 content
//!   `cast_cooldown_refund` 宣告、host 在施法結算時處理，本 handler 不參與。
//! - 主動 duration（modifier_A34T）：
//!   - host cooldown_cap：每 tick 把非大招技能的剩餘冷卻壓到 ≤ 該等級全冷卻 ×(1−cooldown_cap)，
//!     含施放前已在冷卻的（A34T_OnIntervalThink 100-113）。
//!   - host hit_stack 規則：W/E/R 命中英雄疊魔抗減免（A34T_OnHit 10-40）。
//!   - 施放技能回復法力（OnAbilityExecuted → Mana_regen 120-123）：hero UnitScript
//!     `on_skill_cast` 讀取同時期掛上的 rank 標記 buff。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::{cooldown_cap,hit_stack};
use omoba_template_ids::{ABILITY_A34_RAPID_COOLING,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A34_RAPID_COOLING;
pub(crate) const PASSIVE_BUFF:&str="a34_rapid_cooling_mana";
/// modifier_A34T: carries the cooldown cap and the hit-stack rule.
pub(crate) const ACTIVE_BUFF:&str="a34_rapid_cooling";
/// modifier_A34T_debuff on the unit hit.
pub(crate) const SHRED_BUFF:&str="a34_rapid_cooling_resist_shred";
/// Rank markers lasting as long as the active buff: the hero UnitScript cannot
/// read payloads, so the cast mana restore reads the rank from the marker id.
pub(crate) const MANA_MARKERS:[&str;3]=["a34_rapid_cooling_mana_on_cast_1","a34_rapid_cooling_mana_on_cast_2","a34_rapid_cooling_mana_on_cast_3"];
const MANA:&str="mana_bonus";
const DURATION:&str="duration";
const CAP:&str="cooldown_cap";
const MANA_ON_CAST:&str="mana_on_cast";
const SHRED:&str="shred";
const SHRED_STACKS:&str="shred_max_stacks";
const SHRED_DURATION:&str="shred_duration";

fn rank_ok(rank:u8)->bool {
    active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())
}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let mana=extra_at_id(ID,MANA,rank);
    if mana<=Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::ManaBonus.as_str()):mana.raw(),"level":rank}).to_string())
}

/// A34T_OnHit: −reduce magic resistance points per stack (MagicalResistanceBonus
/// sums points like the source's `magical_resistance` value), at most `shred_max_stacks`
/// stacks for `shred_duration`, no refresh at full stacks, heroes only.
pub(crate) fn shred_rule(rank:u8)->Option<hit_stack::Rule> {
    if !rank_ok(rank) {return None;}
    let stacks=extra_at_id(ID,SHRED_STACKS,rank);
    if stacks.raw()%1024!=0 {return None;}
    let rule=hit_stack::Rule {buff:SHRED_BUFF.into(),stat:StatKey::MagicalResistanceBonus,per_stack:-extra_at_id(ID,SHRED,rank),
        max:u8::try_from(stacks.raw()/1024).ok()?,duration:extra_at_id(ID,SHRED_DURATION,rank),refresh_at_max:false,heroes_only:true};
    rule.valid().then_some(rule)
}

/// (payload, duration) of the active buff: cooldown cap fraction and the shred rule.
pub(crate) fn active_payload(rank:u8)->Option<(String,Fixed64)> {
    if !rank_ok(rank) {return None;}
    let pct=extra_at_id(ID,CAP,rank);let duration=extra_at_id(ID,DURATION,rank);
    if pct<=Fixed64::ZERO || pct>=Fixed64::from_i32(100) || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60) {return None;}
    let fraction=Fixed64::from_raw((pct.raw()+50)/100);
    Some((serde_json::json!({(cooldown_cap::CAP_KEY):cooldown_cap::cap_value(fraction),
        (hit_stack::RULE_KEY):shred_rule(rank)?.value()}).to_string(),duration))
}

/// Mana restored per cast while the active buff lasts, from the rank marker.
pub(crate) fn mana_on_cast(world:&GameWorldDyn<'_>,caster:EntityHandle)->Option<Fixed64> {
    if !world.has_buff(caster,ACTIVE_BUFF.into()) {return None;}
    let rank=(1..=MANA_MARKERS.len()).rev().find(|rank|world.has_buff(caster,MANA_MARKERS[rank-1].into()))?;
    let amount=extra_at_id(ID,MANA_ON_CAST,u8::try_from(rank).ok()?);
    (amount>Fixed64::ZERO).then_some(amount)
}

pub struct RapidCoolingHandler;
impl AbilityScript for RapidCoolingHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,_:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        if !matches!(target,Target::None) {return RErr("rapid cooling is a self cast".into());}
        let Some((payload,duration))=active_payload(level) else {return RErr("invalid rapid cooling rank".into());};
        if !world.is_alive(caster) {return RErr("caster is not alive".into());}
        world.add_stat_buff(caster,RStr::from_str(ACTIVE_BUFF),duration,payload.as_str().into());
        world.add_stat_buff(caster,RStr::from_str(MANA_MARKERS[usize::from(level)-1]),duration,"{}".into());
        ROk(())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn a34_rapid_cooling_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::ManaBonus.as_str().into(),extra_at_id_f32(ID,MANA,1));
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
    fn casting_grants_twenty_two_seconds_of_cooldown_cap_shred_rule_and_rank_marker() {
        for (rank,cap,shred) in [(1u8,410,5),(2,512,10),(3,614,15)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&RapidCoolingHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let rows=buffs(&outcomes,ACTIVE_BUFF);
            assert_eq!(rows.len(),1);
            let (payload,duration)=&rows[0];
            assert_eq!(*duration,Fixed64::from_i32(22),"rank {rank}");
            assert_eq!(payload[cooldown_cap::CAP_KEY],cooldown_cap::cap_value(Fixed64::from_raw(cap)),"rank {rank}");
            assert!(payload.get(StatKey::CooldownPercentage.as_str()).is_none(),"a cap, not a new-cooldown multiplier");
            let rule=hit_stack::rule_of(payload).expect("shred rule");
            assert_eq!((rule.stat,rule.per_stack,rule.max,rule.duration,rule.refresh_at_max,rule.heroes_only),
                (StatKey::MagicalResistanceBonus,Fixed64::from_i32(-shred),3,Fixed64::from_i32(3),false,true),"rank {rank}");
            let marker=buffs(&outcomes,MANA_MARKERS[usize::from(rank)-1]);
            assert_eq!(marker.len(),1);assert_eq!(marker[0].1,Fixed64::from_i32(22));
        }
        let mut t=TestWorld::new();
        assert!(t.cast(&RapidCoolingHandler,Target::Point(p(1,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&RapidCoolingHandler,Target::None,4,"{}").is_err());
        assert!(t.learn(&RapidCoolingHandler,0).is_empty());
    }

    #[test]
    fn mana_on_cast_follows_the_active_rank_marker() {
        for (rank,mana) in [(1u8,100),(2,150),(3,200)] {
            let mut t=TestWorld::new();
            assert_eq!(t.read(|w,caster|mana_on_cast(w,caster)),None,"inactive: nothing");
            t.source_buff(ACTIVE_BUFF);t.source_buff(MANA_MARKERS[usize::from(rank)-1]);
            assert_eq!(t.read(|w,caster|mana_on_cast(w,caster)),Some(Fixed64::from_i32(mana)),"rank {rank}");
        }
    }
}
