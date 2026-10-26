//! 風之祈（H05，來源 A02E current）：立即回復 100/200/300/400 生命，10 秒內攻速
//! +40/70/100/130（攻速點）。重複施放以同一 buff id 刷新，不疊加。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A02_WIND_PRAYER,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_A02_WIND_PRAYER;
pub(crate) const BUFF_ID:&str="a02_wind_prayer";

pub(crate) struct Prayer {pub heal:Fixed64,pub attack_speed:Fixed64,pub duration:Fixed64}

pub(crate) fn resolve(rank:u8)->Result<Prayer,&'static str> {
    let content=active_ability_const(ID).ok_or("missing wind prayer content")?;
    let index=usize::from(rank.checked_sub(1).ok_or("invalid wind prayer rank")?);
    if index>=content.levels.len() || rank>content.max_level {return Err("invalid wind prayer rank");}
    let extra=|key:&str|content.extras.iter().find(|(name,_)|*name==key).and_then(|(_,rows)|rows.get(index)).copied()
        .ok_or("missing wind prayer extra");
    let spec=Prayer {heal:extra("heal_amount")?,attack_speed:extra("stat_buff_attack_speed_bonus")?,duration:extra("stat_buff_duration")?};
    if spec.heal<=Fixed64::ZERO || spec.attack_speed<=Fixed64::ZERO || spec.duration<=Fixed64::ZERO || spec.duration>Fixed64::from_i32(60) {
        return Err("invalid wind prayer values");
    }
    Ok(spec)
}

pub(crate) fn execute(world:&mut GameWorldDyn<'_>,caster:EntityHandle,target:Target,spec:Prayer)->Result<(),&'static str> {
    if !matches!(target,Target::None) {return Err("wind prayer is a self cast");}
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return Err("caster is not alive");}
    world.heal(caster,spec.heal);
    let payload=serde_json::json!({(StatKey::AttackSpeedBonusConstant.as_str()):spec.attack_speed.raw()}).to_string();
    world.add_stat_buff(caster,RStr::from_str(BUFF_ID),spec.duration,payload.as_str().into());
    Ok(())
}

pub struct WindPrayerHandler;
impl AbilityScript for WindPrayerHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,_:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        match resolve(level).and_then(|spec|execute(world,caster,target,spec)) {Ok(())=>ROk(()),Err(e)=>RErr(e.into())}
    }
}

pub fn a02_wind_prayer_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(),extra_at_id_f32(ID,"stat_buff_attack_speed_bonus",1));
    build_ability_ffi(ID,WindPrayerHandler,vec![
        EffectSpec::Heal {target:TargetSelector::SelfUnit,amount:extra_at_id_f32(ID,"heal_amount",1)},
        EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:Some(extra_at_id_f32(ID,"stat_buff_duration",1)),modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A02_inahime::a02_test_support::*;
    use omoba_core::Outcome;

    #[test]
    fn every_rank_heals_and_grants_attack_speed_for_ten_seconds() {
        for (rank,heal,speed) in [(1u8,100,40),(2,200,70),(3,300,100),(4,400,130)] {
            let mut t=TestWorld::new();
            let outcomes=t.cast(&WindPrayerHandler,Target::None,rank,&rank_json(ID,rank)).unwrap();
            let source=t.source;
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::ScriptHeal {target,amount} if *target==source && *amount==Fixed64::from_i32(heal))),"rank {rank}");
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,duration,..} if buff_id==BUFF_ID=>
                Some((payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64(),*duration)),_=>None}).collect();
            assert_eq!(buffs,vec![(Some(i64::from(speed)*1024),Fixed64::from_i32(10))],"rank {rank}");
        }
    }

    #[test]
    fn wrong_target_or_rank_is_rejected() {
        let mut t=TestWorld::new();let other=t.body(p(100,0),1);
        assert!(t.cast(&WindPrayerHandler,Target::Entity(t.handle(other)),1,&rank_json(ID,1)).is_err());
        assert!(t.cast(&WindPrayerHandler,Target::None,0,"{}").is_err());
        assert!(t.cast(&WindPrayerHandler,Target::None,5,"{}").is_err());
    }
}
