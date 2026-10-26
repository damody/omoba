//! 精準之光（玉子，來源 C15R current）：被動；攻擊傷害 +10/18/26/34%，並提升
//! 5/10/15/20 點靈活（換算為攻擊、攻速點與護甲，見 assumptions precise_light_agility）。
//! 每次學習或升級以同一 buff id 取代，不疊加。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_C15_PRECISE_LIGHT,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C15_PRECISE_LIGHT;
pub(crate) const BUFF_ID:&str="c15_precise_light";
const OUTGOING:&str="stat_bonus_permanent_base_damage_outgoing_pct";
const DAMAGE:&str="stat_bonus_permanent_attack_damage_bonus";
const ATTACK_SPEED:&str="stat_bonus_permanent_attack_speed_bonus";
const ARMOR:&str="stat_bonus_permanent_armor_bonus";

/// Raw payload for one rank, or None for an invalid rank or value.
pub(crate) fn payload(rank:u8)->Option<String> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let outgoing=extra_at_id(ID,OUTGOING,rank);let damage=extra_at_id(ID,DAMAGE,rank);
    let attack_speed=extra_at_id(ID,ATTACK_SPEED,rank);let armor=extra_at_id(ID,ARMOR,rank);
    let bounded=|v:Fixed64,max:i32|v>=Fixed64::ZERO && v<=Fixed64::from_i32(max);
    if !bounded(outgoing,1_000) || !bounded(damage,100_000) || !bounded(attack_speed,1_000) || !bounded(armor,1_000) {return None;}
    // Percent authored as 10 → engine fraction 0.10 (rounded raw).
    let fraction=(outgoing.raw()+50)/100;
    Some(serde_json::json!({
        StatKey::BaseDamageOutgoingPercentage.as_str():fraction,
        StatKey::BaseAttackBonusDamage.as_str():damage.raw(),
        StatKey::AttackSpeedBonusConstant.as_str():attack_speed.raw(),
        StatKey::PhysicalArmorBonus.as_str():armor.raw(),
        "level":rank}).to_string())
}

pub struct PreciseLightHandler;
impl AbilityScript for PreciseLightHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(BUFF_ID),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn c15_precise_light_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(),extra_at_id_f32(ID,OUTGOING,1)/100.0);
    modifiers.insert(StatKey::BaseAttackBonusDamage.as_str().into(),extra_at_id_f32(ID,DAMAGE,1));
    modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(),extra_at_id_f32(ID,ATTACK_SPEED,1));
    modifiers.insert(StatKey::PhysicalArmorBonus.as_str().into(),extra_at_id_f32(ID,ARMOR,1));
    build_ability_ffi(ID,PreciseLightHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C15_tamako::c15_test_support::*;
    use omoba_core::Outcome;

    fn q(v:f64)->i64 {(v*1024.0).round() as i64}

    #[test]
    fn every_rank_grants_outgoing_damage_and_converted_agility_and_replaces_the_previous_rank() {
        for (rank,pct,agi,armor) in [(1u8,10.0,5.0,0.833),(2,18.0,10.0,1.667),(3,26.0,15.0,2.5),(4,34.0,20.0,3.333)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&PreciseLightHandler,rank);
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {
                Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>Some((payload.clone(),*duration)),_=>None}).collect();
            assert_eq!(buffs.len(),1,"rank {rank}: one buff id, so an upgrade replaces instead of stacking");
            assert_eq!(outcomes.len(),1,"rank {rank}: nothing else is emitted");
            let (payload,duration)=&buffs[0];
            assert_eq!(*duration,Fixed64::from_raw(i64::MAX));
            // (raw + 50) / 100 of the authored percent.
            assert_eq!(payload[StatKey::BaseDamageOutgoingPercentage.as_str()].as_i64(),Some((q(pct)+50)/100),"rank {rank}");
            assert_eq!(payload[StatKey::BaseDamageOutgoingPercentage.as_str()].as_i64(),Some(q(pct/100.0)),"rank {rank}: ≈ pct/100");
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(q(agi)),"rank {rank}");
            assert_eq!(payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64(),Some(q(agi)),"rank {rank}");
            assert_eq!(payload[StatKey::PhysicalArmorBonus.as_str()].as_i64(),Some(q(armor)),"rank {rank}");
        }
    }

    #[test]
    fn passive_cannot_be_cast_and_invalid_rank_learns_nothing() {
        let mut t=TestWorld::new();
        assert!(t.cast(&PreciseLightHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&PreciseLightHandler,Target::Point(p(100,0)),1,&rank_json(ID,1)).is_err());
        assert!(t.learn(&PreciseLightHandler,0).is_empty());
        assert!(t.learn(&PreciseLightHandler,5).is_empty());
    }
}
