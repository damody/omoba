//! 士氣提升（村上義清，來源 B29R current）：被動。
//! - 光環：自身周圍 600 內的友軍（含自身，不含建築）攻擊傷害 +7/14/21/28%。
//!   英雄 UnitScript tick（hero_unit.rs → [`on_owner_tick`]）每 0.5 秒送一個
//!   host `ally_area_buff` 封包：同一個 buff id 刷新 0.75 秒，所以離開 600 後
//!   最多 0.75 秒失效；施法者死亡後不再刷新。
//! - 永久力量 +4/8/12/16，換算為 +80/160/240/320 最大生命、+0.4/0.8/1.2/1.6
//!   回血、+4/8/12/16 攻擊（strength_conversion），學習／升級以同一 buff id 取代。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::ally_area_buff;
use omoba_template_ids::{ABILITY_B29_MORALE_BOOST,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B29_MORALE_BOOST;
pub(crate) const PASSIVE_BUFF:&str="b29_morale_strength";
/// Seconds between aura packets, and how long each refresh lasts.
const REFRESH:Fixed64=Fixed64::from_raw(512);
const LINGER:Fixed64=Fixed64::from_raw(768);

fn max_rank()->u8 {active_ability_const(ID).map(|c|c.max_level).unwrap_or(0)}
fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}
/// Rank marker on the caster; the rank is in the id because a hook cannot read payloads.
pub(crate) fn rank_marker(caster:EntityHandle,rank:u8)->String {format!("b29_morale_r{rank}:{}:{}",caster.id,caster.gen)}
pub(crate) fn think_marker(caster:EntityHandle)->String {format!("b29_morale_think:{}:{}",caster.id,caster.gen)}
/// The buff the aura keeps refreshed on allies.
pub(crate) fn aura_buff(caster:EntityHandle)->String {format!("b29_morale:{}:{}",caster.id,caster.gen)}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let hp=extra_at_id(ID,"stat_bonus_permanent_max_hp_bonus",rank);
    let regen=extra_at_id(ID,"stat_bonus_permanent_health_regen",rank);
    let attack=extra_at_id(ID,"stat_bonus_permanent_attack_damage_bonus",rank);
    if hp<=Fixed64::ZERO || regen<Fixed64::ZERO || attack<Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::HealthBonus.as_str()):hp.raw(),(StatKey::HealthRegenConstant.as_str()):regen.raw(),
        (StatKey::BaseAttackBonusDamage.as_str()):attack.raw(),"level":rank}).to_string())
}

/// The aura packet for one refresh, or None for an invalid rank.
pub(crate) fn aura_packet(caster:EntityHandle,center:omb_script_abi::types::Vec2,rank:u8)->Option<ally_area_buff::Spec> {
    if !rank_ok(rank) {return None;}
    let pct=extra_at_id(ID,"stat_buff_base_damage_outgoing_pct",rank);
    let radius=extra_at_id(ID,"stat_buff_radius",rank);
    // Percent (7 = +7%) becomes a fraction of one, rounded.
    let fraction=(pct.raw()+50)/100;
    ally_area_buff::Spec {source_id:caster.id,source_generation:caster.gen,center,radius,buff:aura_buff(caster),seconds:LINGER,
        stats:vec![(StatKey::BaseDamageOutgoingPercentage.as_str().to_owned(),fraction)]}.checked()
}

/// Hero UnitScript tick: one aura packet every REFRESH seconds while the caster
/// lives and has the passive learned.
pub(crate) fn on_owner_tick(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
    let Some(rank)=(1..=max_rank()).rev().find(|rank|world.has_buff(caster,rank_marker(caster,*rank).as_str().into())) else {return};
    if world.has_buff(caster,think_marker(caster).as_str().into()) {return;}
    let Some(center)=world.get_pos(caster).into_option() else {return};
    let Some(packet)=aura_packet(caster,center,rank) else {return};
    world.add_buff(caster,think_marker(caster).as_str().into(),REFRESH);
    world.add_stat_buff(caster,ally_area_buff::BUFF.into(),Fixed64::ONE,packet.payload().to_string().as_str().into());
}

pub struct MoraleBoostHandler;
impl AbilityScript for MoraleBoostHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
        for r in 1..=max_rank() {
            if r!=rank {world.remove_buff(caster,rank_marker(caster,r).as_str().into());}
        }
        world.add_stat_buff(caster,rank_marker(caster,rank).as_str().into(),Fixed64::from_raw(i64::MAX),"{}".into());
    }
}

pub fn b29_morale_boost_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(),extra_at_id_f32(ID,"stat_buff_base_damage_outgoing_pct",1)/100.0);
    modifiers.insert(StatKey::HealthBonus.as_str().into(),extra_at_id_f32(ID,"stat_bonus_permanent_max_hp_bonus",1));
    build_ability_ffi(ID,MoraleBoostHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B29_murakami_yoshikiyo::b29_test_support::*;
    use omoba_core::Outcome;
    use omoba_core::runtime::BuffStore;
    use specs::WorldExt;

    fn owner_tick(t:&mut TestWorld)->Vec<Outcome> {
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let cache=ParallelAdapterCache::new(&t.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let caster=ParallelWorldAdapter::entity_to_handle(t.source);
        let mut w=omb_script_abi::world::GameWorld_TO::from_ptr(abi_stable::RMut::new(&mut adapter),abi_stable::sabi_trait::prelude::TD_Opaque);
        on_owner_tick(&mut w,caster);
        drop(w);adapter.finish()
    }
    fn settle(t:&mut TestWorld,outcomes:&[Outcome]) {
        let mut buffs=t.world.write_resource::<BuffStore>();
        for o in outcomes {if let Outcome::AddBuff {target,buff_id,duration,payload}=o {buffs.add(*target,buff_id,*duration,payload.clone());}}
    }

    #[test]
    fn learning_grants_converted_strength_and_replaces_the_previous_rank() {
        for (rank,hp,regen,attack) in [(1u8,80,0.4f64,4),(2,160,0.8,8),(3,240,1.2,12),(4,320,1.6,16)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&MoraleBoostHandler,rank);
            let payload=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==PASSIVE_BUFF=>Some(payload.clone()),_=>None}).expect("strength");
            assert_eq!(payload[StatKey::HealthBonus.as_str()].as_i64(),Some(hp*1024),"rank {rank}");
            assert_eq!(payload[StatKey::BaseAttackBonusDamage.as_str()].as_i64(),Some(attack*1024),"rank {rank}");
            let regen_raw=payload[StatKey::HealthRegenConstant.as_str()].as_i64().unwrap();
            assert!((regen_raw-(regen*1024.0).round() as i64).abs()<=1,"rank {rank}");
            let source=t.source;
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,..} if *buff_id==rank_marker(t.handle(source),rank))));
        }
    }

    #[test]
    fn the_aura_sends_one_ally_packet_per_refresh_with_the_rank_bonus() {
        for (rank,fraction) in [(1u8,72),(2,143),(3,215),(4,287)] {
            let mut t=TestWorld::new();
            assert!(owner_tick(&mut t).is_empty(),"not learned: no aura");
            let learned=t.learn(&MoraleBoostHandler,rank);settle(&mut t,&learned);
            let first=owner_tick(&mut t);
            let packet=first.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
                ally_area_buff::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("aura packet");
            assert_eq!((packet.radius,packet.seconds,packet.center),(Fixed64::from_i32(600),LINGER,p(0,0)),"rank {rank}");
            assert_eq!(packet.stats,vec![(StatKey::BaseDamageOutgoingPercentage.as_str().to_owned(),fraction)],"rank {rank}");
            settle(&mut t,&first);
            assert!(owner_tick(&mut t).is_empty(),"waits for the refresh interval");
        }
        let mut t=TestWorld::new();
        assert!(t.cast(&MoraleBoostHandler,Target::None,1,"{}").is_err());
    }
}
