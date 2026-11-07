//! 士氣提升（村上義清，來源 B29R current，B29.txt:311-391）：被動。
//! 數值見 v3.lua（radius／aura_radius、damage_pct、strength、aura_interval、aura_window）。
//! - 光環 `modifier_B29R_aura`：自身周圍 aura_radius 內的友軍（含自身，不含建築）攻擊傷害
//!   +damage_pct。以通用光環發射器（aura_emitter，E173，與道具光環同一機制）實作：英雄
//!   UnitScript tick（hero_unit.rs → [`on_owner_tick`]）每 aura_interval 在自身刷新 aura_window 的發射器；
//!   核心每次 settle 把 `b29_morale` 刷新到 aura_radius 內友軍，離開後殘留 AURA_LINGER（Dota
//!   光環預設）。收到者的 buff id 不含施法者：同名光環在 Dota 不疊加，兩位村上只算一份。
//!   施法者死亡後發射器不再刷新。
//! - `modifier_B29R_str`：永久力量 +strength（StatKey::StatsStrengthBonus），由三圍
//!   settle（omoba-core attributes.rs）換算生命、回復與主屬性攻擊；學習／升級以同一 buff id 取代。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::aura_emitter;
use omoba_template_ids::{ABILITY_B29_MORALE_BOOST,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B29_MORALE_BOOST;
pub(crate) const PASSIVE_BUFF:&str="b29_morale_strength";

fn max_rank()->u8 {active_ability_const(ID).map(|c|c.max_level).unwrap_or(0)}
fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}
/// Rank marker on the caster; the rank is in the id because a hook cannot read payloads.
pub(crate) fn rank_marker(caster:EntityHandle,rank:u8)->String {format!("b29_morale_r{rank}:{}:{}",caster.id,caster.gen)}
pub(crate) fn think_marker(caster:EntityHandle)->String {format!("b29_morale_think:{}:{}",caster.id,caster.gen)}
/// The buff the aura keeps refreshed on allies (one per name: Dota auras of one modifier do not stack).
pub(crate) const AURA_BUFF:&str="b29_morale";

/// Seconds between emitter refreshes (v3.lua `aura_interval`).
fn refresh(rank:u8)->Fixed64 {extra_at_id(ID,"aura_interval",rank)}
/// The emitter window each refresh declares (v3.lua `aura_window`).
fn linger(rank:u8)->Fixed64 {extra_at_id(ID,"aura_window",rank)}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let strength=extra_at_id(ID,"strength",rank);
    if strength<=Fixed64::ZERO || strength>Fixed64::from_i32(1_000) {return None;}
    Some(serde_json::json!({(StatKey::StatsStrengthBonus.as_str()):strength.raw(),"level":rank}).to_string())
}

/// The aura at `rank` (aura_emitter spec), or None for an invalid rank.
pub(crate) fn aura_spec(rank:u8)->Option<aura_emitter::Spec> {
    if !rank_ok(rank) {return None;}
    let pct=extra_at_id(ID,"damage_pct",rank);
    let radius=extra_at_id(ID,"aura_radius",rank);
    if linger(rank)<=Fixed64::ZERO || refresh(rank)<=Fixed64::ZERO || refresh(rank)>linger(rank) {return None;}
    // Percent (kit `damage_pct`, unit pct) becomes a fraction of one, rounded.
    let fraction=(pct.raw()+50)/100;
    aura_emitter::Spec {name:AURA_BUFF.to_owned(),radius,
        stats:vec![(StatKey::BaseDamageOutgoingPercentage.as_str().to_owned(),fraction)]}.checked()
}

/// Hero UnitScript tick: every REFRESH seconds, while the caster lives and has the
/// passive learned, re-arms its aura emitter for the linger window.
pub(crate) fn on_owner_tick(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    if !world.is_alive(caster) || !world.get_hp(caster).into_option().is_some_and(|hp|hp>Fixed64::ZERO) {return;}
    let Some(rank)=(1..=max_rank()).rev().find(|rank|world.has_buff(caster,rank_marker(caster,*rank).as_str().into())) else {return};
    if world.has_buff(caster,think_marker(caster).as_str().into()) {return;}
    let Some(aura)=aura_spec(rank) else {return};
    world.add_buff(caster,think_marker(caster).as_str().into(),refresh(rank));
    world.add_stat_buff(caster,aura_emitter::buff_id(&aura.name).as_str().into(),linger(rank),aura.payload().to_string().as_str().into());
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
    modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(),extra_at_id_f32(ID,"damage_pct",1)/100.0);
    modifiers.insert(StatKey::StatsStrengthBonus.as_str().into(),extra_at_id_f32(ID,"strength",1));
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
    fn learning_grants_source_strength_and_replaces_the_previous_rank() {
        for (rank,strength) in [(1u8,4),(2,8),(3,12),(4,16)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&MoraleBoostHandler,rank);
            let payload=outcomes.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==PASSIVE_BUFF=>Some(payload.clone()),_=>None}).expect("strength");
            // modifier_B29R_str: STATS_STRENGTH_BONUS only; the attribute settle converts it.
            assert_eq!(payload[StatKey::StatsStrengthBonus.as_str()].as_i64(),Some(strength*1024),"rank {rank}");
            for converted in [StatKey::HealthBonus,StatKey::BaseAttackBonusDamage,StatKey::HealthRegenConstant] {
                assert!(payload.get(converted.as_str()).is_none(),"rank {rank}: no hand conversion");
            }
            let source=t.source;
            assert!(outcomes.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,..} if *buff_id==rank_marker(t.handle(source),rank))));
        }
    }

    #[test]
    fn the_aura_rearms_one_emitter_per_refresh_with_the_rank_bonus() {
        for (rank,fraction) in [(1u8,72),(2,143),(3,215),(4,287)] {
            let mut t=TestWorld::new();
            assert!(owner_tick(&mut t).is_empty(),"not learned: no aura");
            let learned=t.learn(&MoraleBoostHandler,rank);settle(&mut t,&learned);
            let first=owner_tick(&mut t);
            let (window,emitter)=first.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}
                if *buff_id==aura_emitter::buff_id(AURA_BUFF)=>Some((*duration,aura_emitter::Spec::from_payload(payload))),_=>None}).expect("aura emitter");
            let emitter=emitter.expect("valid emitter payload");
            assert_eq!((emitter.name.as_str(),emitter.radius,window),(AURA_BUFF,Fixed64::from_i32(600),q(0.5)),"rank {rank}: one shared aura name");
            let think=think_marker(t.handle(t.source));
            assert!(first.iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,duration,..} if *buff_id==think && *duration==q(0.25))),
                "rank {rank}: re-armed every 0.25 s, inside the 0.5 s window");
            assert_eq!(emitter.stats,vec![(StatKey::BaseDamageOutgoingPercentage.as_str().to_owned(),fraction)],"rank {rank}");
            settle(&mut t,&first);
            assert!(owner_tick(&mut t).is_empty(),"waits for the refresh interval");
        }
        let mut t=TestWorld::new();
        assert!(t.cast(&MoraleBoostHandler,Target::None,1,"{}").is_err());
    }
}
