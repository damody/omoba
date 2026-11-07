//! 碎擊槍（本多忠勝，來源 A07R；行為依 history_nobu A07_Honda_Tadakatsu.lua A07R_old）：
//! 被動。每次攻擊擲 RandomInt(0,crit_roll_max)，≤crit_roll_at_most 時暴擊；連續 misses 次沒暴擊，
//! 下一次必定暴擊。暴擊倍率 crit_multiplier（數值見 v3.lua）。
//! 原作在攻擊開始時決定當次是否暴擊；本遊戲在每次普攻命中時（英雄 UnitScript
//! on_attack_hit → [`on_owner_attack_hit`]）決定下一次普攻：暴擊時放
//! PreattackCriticalStrike 1.0＋CritMultiplier 的 buff，由 host 普攻暴擊結算，否則移除。
//! 機率與保底序列和原作相同。學習時（[`on_learn`]）擲第一次並放 rank 標記；
//! 奮起依這個 rank 選攻速（A07R_Learn_Skill 把奮起設為碎擊槍等級 +1）。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_A07_SHATTERING_SPEAR,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

pub(crate) const ID:omoba_template_ids::AbilityId=ABILITY_A07_SHATTERING_SPEAR;

/// The next attack crits while this buff is on the caster.
pub(crate) fn crit_buff(caster:EntityHandle)->String {format!("a07_crit_next:{}:{}",caster.id,caster.gen)}
/// Consecutive non-crit decisions so far: `a07_noncrit:<n>:<id>:<gen>` (at most one present).
pub(crate) fn miss_marker(caster:EntityHandle,n:u32)->String {format!("a07_noncrit:{n}:{}:{}",caster.id,caster.gen)}
const CRIT_HOLD:Fixed64=Fixed64::from_raw(60*1024);

#[derive(Clone,Copy,Debug)]
pub(crate) struct Spear {pub roll_max:u32,pub roll_at_most:u32,pub misses:u32,pub multiplier:Fixed64}

pub(crate) fn spec(rank:u8)->Option<Spear> {
    let content=active_ability_const(ID)?;
    if rank==0 || rank>content.max_level || usize::from(rank)>content.levels.len() {return None;}
    let int=|key:&str|u32::try_from(extra_at_id(ID,key,rank).raw()/1024).ok();
    let spec=Spear {roll_max:int("crit_roll_max")?,roll_at_most:int("crit_roll_at_most")?,
        misses:int("misses")?,
        multiplier:Fixed64::from_raw((extra_at_id(ID,"crit_multiplier",rank).raw()+50)/100)};
    (spec.roll_max>0 && spec.roll_max<=10_000 && spec.roll_at_most<=spec.roll_max && spec.misses<=64
        && spec.multiplier>Fixed64::ONE && spec.multiplier<=Fixed64::from_i32(10)).then_some(spec)
}

/// RandomInt(0, roll_max) from a deterministic unit roll in [0, 1).
fn roll(world:&mut GameWorldDyn<'_>,max:u32)->u32 {
    let unit=world.rand_unit().raw().clamp(0,1023) as u64;
    (unit*u64::from(max+1)/1024) as u32
}

/// Decide whether the caster's next attack crits (A07R_old run one attack early).
pub(crate) fn decide_next(world:&mut GameWorldDyn<'_>,caster:EntityHandle,spec:&Spear) {
    let held=(1..=spec.misses).find(|n|world.has_buff(caster,miss_marker(caster,*n).as_str().into())).unwrap_or(0);
    let ran=roll(world,spec.roll_max);
    let misses=if ran>spec.roll_at_most {held+1} else {held};
    let crit=misses>spec.misses || ran<=spec.roll_at_most;
    if held>0 {world.remove_buff(caster,miss_marker(caster,held).as_str().into());}
    if crit {
        let payload=serde_json::json!({(StatKey::PreattackCriticalStrike.as_str()):Fixed64::ONE.raw(),
            (StatKey::CritMultiplier.as_str()):spec.multiplier.raw()}).to_string();
        world.add_stat_buff(caster,crit_buff(caster).as_str().into(),CRIT_HOLD,payload.as_str().into());
    } else {
        world.remove_buff(caster,crit_buff(caster).as_str().into());
        world.add_buff(caster,miss_marker(caster,misses).as_str().into(),Fixed64::from_raw(i64::MAX));
    }
}

/// Hero UnitScript `on_attack_hit`: the landed attack consumed the previous
/// decision; decide the next one.
pub(crate) fn on_owner_attack_hit(world:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    let Some(spec)=spec(super::a07_ranks::learned(world,ID,caster)) else {return};
    decide_next(world,caster,&spec);
}

pub struct ShatteringSpearHandler;
impl AbilityScript for ShatteringSpearHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(spec)=spec(rank) else {return};
        super::a07_ranks::learn(world,ID,caster,rank);
        // A07R_Levelup / A07R_old: the count starts over and the next attack is decided now.
        for n in 1..=spec.misses {world.remove_buff(caster,miss_marker(caster,n).as_str().into());}
        decide_next(world,caster,&spec);
    }
}

pub fn a07_shattering_spear_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::PreattackCriticalStrike.as_str().into(),
        (extra_at_id_f32(ID,"crit_roll_at_most",1)+1.0)/(extra_at_id_f32(ID,"crit_roll_max",1)+1.0));
    modifiers.insert(StatKey::CritMultiplier.as_str().into(),extra_at_id_f32(ID,"crit_multiplier",1)/100.0);
    build_ability_ffi(ID,ShatteringSpearHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::A07_honda_tadakatsu::a07_test_support::*;
    use crate::heroes::A07_honda_tadakatsu::hero_unit::hero_unit_script;
    use omoba_core::runtime::{BuffStore,UnitStats};
    use specs::WorldExt;

    fn crit_next(t:&TestWorld)->Option<Fixed64> {
        let b=t.world.read_resource::<BuffStore>();
        b.has(t.source,&crit_buff(t.handle(t.source))).then(||UnitStats::from_refs(&b,false).crit(t.source)).map(|(chance,m)|{assert_eq!(chance,Fixed64::ONE);m})
    }

    #[test]
    fn every_rank_learns_its_multiplier_and_never_misses_more_than_four_times_in_a_row() {
        for (rank,multiplier) in [(1u8,1.2),(2,1.4),(3,1.6),(4,1.8)] {
            let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
            let learned=t.learn(&ShatteringSpearHandler,rank);t.apply(learned);
            let unit=hero_unit_script();
            let (mut crits,mut run,mut longest)=(0,0,0);
            for _ in 0..400 {
                match crit_next(&t) {
                    Some(m)=>{assert_eq!(m,q(multiplier),"rank {rank}");crits+=1;run=0;}
                    None=>{run+=1;longest=longest.max(run);}
                }
                let hit=t.attack_hit(&unit,enemy);t.apply(hit);
                t.world.write_resource::<omoba_core::Tick>().0+=1;
            }
            assert!(longest<=4,"at most 4 attacks in a row without a crit, got {longest}");
            assert!((100..=220).contains(&crits),"about 31% plus the guarantee, got {crits}/400");
        }
    }

    #[test]
    fn upgrading_replaces_the_rank_marker() {
        let mut t=TestWorld::new();
        let first=t.learn(&ShatteringSpearHandler,1);t.apply(first);
        let second=t.learn(&ShatteringSpearHandler,4);t.apply(second);
        let b=t.world.read_resource::<BuffStore>();let h=t.handle(t.source);
        assert!(b.has(t.source,&super::super::a07_ranks::marker(ID,h,4)));
        assert!(!b.has(t.source,&super::super::a07_ranks::marker(ID,h,1)));
    }

    #[test]
    fn passive_cannot_be_cast_and_bad_ranks_grant_nothing() {
        let mut t=TestWorld::new();
        assert!(t.cast(&ShatteringSpearHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.learn(&ShatteringSpearHandler,0).is_empty());
        assert!(t.learn(&ShatteringSpearHandler,5).is_empty());
    }
}
