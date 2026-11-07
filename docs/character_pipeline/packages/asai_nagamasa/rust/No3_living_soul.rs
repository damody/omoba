//! 活心．魂（H06，來源 B08R current，B08.txt:621-678；B08_Asai_Nagamasa.lua:B08R_Action:250-278）：
//! - 被動：攻擊速度 +attack_speed（見 v3.lua；學習／升級以同一 buff id 取代）。
//! - OnAbilityExecuted：每次成功使用技能或物品（切換技與 logging／tpscroll／samurai_cloth
//!   除外，252-255）套 modifier_B08R_buff duration 秒：攻擊 +attack_bonus、受到傷害 damage_taken（KV 固定值，
//!   不乘層數，655-656）。已有時只刷新；層數 +1（上限 max_stacks，260）只計數。
//!   由英雄 UnitScript 的 on_skill_cast 呼叫（技能施放與物品使用，`owner_cast`）；施法
//!   被拒絕時整筆交易捨棄，所以不會多算。
//! 層數是不帶數值的標記 buff（`STACK_PREFIX` + 序號），rank 以標記 buff 表示，
//! 全部可由 has_buff 判定，不讀 payload。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_template_ids::{ABILITY_B08_LIVING_SOUL,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};
use crate::owner_cast;

const ID:omoba_template_ids::AbilityId=ABILITY_B08_LIVING_SOUL;
pub(crate) const PASSIVE_BUFF:&str="b08_living_soul";
/// modifier_B08R_buff: the one stat buff (attack_bonus, damage_taken; see v3.lua).
pub(crate) const SOUL_BUFF:&str="b08_living_soul_buff";
/// Display-only stack markers (payload `{}`).
pub(crate) const STACK_PREFIX:&str="b08_living_soul_stack:";
const RANK_PREFIX:&str="b08_living_soul_rank:";
const SPEED:&str="attack_speed";
const DAMAGE:&str="attack_bonus";
const GUARD:&str="damage_taken";
const DURATION:&str="duration";
const STACKS:&str="max_stacks";

fn max_level()->u8 {active_ability_const(ID).map_or(0,|c|c.max_level)}
fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let speed=extra_at_id(ID,SPEED,rank);
    if speed<=Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::AttackSpeedBonusConstant.as_str()):speed.raw(),"level":rank}).to_string())
}

/// Learned rank from the marker buff, 0 when not learned.
fn learned(w:&GameWorldDyn<'_>,caster:EntityHandle)->u8 {
    (1..=max_level()).rev().find(|r|w.has_buff(caster,format!("{RANK_PREFIX}{r}").as_str().into())).unwrap_or(0)
}

/// One committed ability cast or item use by the owner: (re)apply the timed buff
/// and count one more stack (up to the rank cap; display only).
pub(crate) fn on_owner_cast(w:&mut GameWorldDyn<'_>,caster:EntityHandle,skill:&str) {
    if !owner_cast::counts(skill,owner_cast::B08R_EXCLUDED_ITEMS) {return;}
    let rank=learned(w,caster);
    if !rank_ok(rank) {return;}
    let (damage,guard,duration,cap)=(extra_at_id(ID,DAMAGE,rank),extra_at_id(ID,GUARD,rank),extra_at_id(ID,DURATION,rank),extra_at_id(ID,STACKS,rank));
    let cap=u32::try_from(cap.raw()/1024).unwrap_or(0).min(16);
    if cap==0 || duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60) {return;}
    let payload=serde_json::json!({(StatKey::PreattackBonusDamage.as_str()):damage.raw(),
        (StatKey::IncomingDamagePercentage.as_str()):(guard.raw()-50)/100}).to_string();
    w.add_stat_buff(caster,SOUL_BUFF.into(),duration,payload.as_str().into());
    let held=(1..=cap).take_while(|k|w.has_buff(caster,format!("{STACK_PREFIX}{k}").as_str().into())).count() as u32;
    for k in 1..=(held+1).min(cap) {
        w.add_stat_buff(caster,format!("{STACK_PREFIX}{k}").as_str().into(),duration,"{}".into());
    }
}

pub struct LivingSoulHandler;
impl AbilityScript for LivingSoulHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
        for r in 1..=max_level() {
            if r!=rank {world.remove_buff(caster,format!("{RANK_PREFIX}{r}").as_str().into());}
        }
        world.add_stat_buff(caster,format!("{RANK_PREFIX}{rank}").as_str().into(),Fixed64::from_raw(i64::MAX),"{}".into());
    }
}

pub fn b08_living_soul_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(),extra_at_id_f32(ID,SPEED,1));
    build_ability_ffi(ID,LivingSoulHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B08_asai_nagamasa::b08_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;
    use omoba_core::runtime::BuffStore;

    /// Applies outcome buffs into the world so the next hook sees them.
    fn settle(t:&mut TestWorld,outcomes:&[Outcome]) {
        let mut buffs=t.world.write_resource::<BuffStore>();
        for o in outcomes {
            match o {
                Outcome::AddBuff {target,buff_id,duration,payload}=>{buffs.add(*target,buff_id,*duration,payload.clone());}
                Outcome::ScriptRemoveBuff {target,buff_id}=>{buffs.remove(*target,buff_id);}
                _=>{}
            }
        }
    }

    fn owner_cast(t:&mut TestWorld,skill:&str)->Vec<Outcome> {
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let cache=ParallelAdapterCache::new(&t.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let caster=ParallelWorldAdapter::entity_to_handle(t.source);
        let mut w=omb_script_abi::world::GameWorld_TO::from_ptr(abi_stable::RMut::new(&mut adapter),abi_stable::sabi_trait::prelude::TD_Opaque);
        on_owner_cast(&mut w,caster,skill);
        drop(w);adapter.finish()
    }

    #[test]
    fn learning_grants_rank_attack_speed() {
        for (rank,speed) in [(1u8,20),(2,35),(3,50),(4,65)] {
            let mut t=TestWorld::new();
            let rows:Vec<_>=t.learn(&LivingSoulHandler,rank).iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==PASSIVE_BUFF=>
                payload[StatKey::AttackSpeedBonusConstant.as_str()].as_i64(),_=>None}).collect();
            assert_eq!(rows,vec![i64::from(speed)*1024],"rank {rank}");
        }
    }

    #[test]
    fn every_trigger_refreshes_one_fixed_buff_and_counts_stacks_up_to_the_cap() {
        let mut t=TestWorld::new();
        let learned=t.learn(&LivingSoulHandler,1);settle(&mut t,&learned);
        for expected in [1usize,2,3,4,4] {
            let outcomes=owner_cast(&mut t,"b08_fang_slash");settle(&mut t,&outcomes);
            let buffs:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..} if buff_id==SOUL_BUFF=>
                Some((*duration,payload[StatKey::PreattackBonusDamage.as_str()].as_i64(),payload[StatKey::IncomingDamagePercentage.as_str()].as_i64())),_=>None}).collect();
            assert_eq!(buffs,vec![(Fixed64::from_i32(8),Some(15*1024),Some(-41))],"one +15 / −4% buff, refreshed to 8 s");
            let stacks:Vec<_>=outcomes.iter().filter_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id.starts_with(STACK_PREFIX)=>Some(payload.clone()),_=>None}).collect();
            assert_eq!(stacks.len(),expected,"rank 1 counts up to 4 stacks");
            assert!(stacks.iter().all(|p|p.as_object().is_some_and(|o|o.is_empty())),"stacks carry no stats");
        }
        let total=t.world.read_resource::<BuffStore>().sum_add(t.source,StatKey::PreattackBonusDamage);
        assert_eq!(total,Fixed64::from_i32(15),"the attack bonus never stacks");
    }

    #[test]
    fn items_trigger_except_the_source_exclusions() {
        let mut t=TestWorld::new();
        let learned=t.learn(&LivingSoulHandler,2);settle(&mut t,&learned);
        for (skill,fires) in [("item:moba_stride_charm",true),("item:flash_ring",true),("item:logging",false),("item:tpscroll",false),("item:samurai_cloth",false)] {
            let fired=owner_cast(&mut t,skill).iter().any(|o|matches!(o,Outcome::AddBuff {buff_id,..} if buff_id==SOUL_BUFF));
            assert_eq!(fired,fires,"{skill}");
        }
    }

    #[test]
    fn without_the_passive_casts_add_nothing() {
        let mut t=TestWorld::new();
        assert!(owner_cast(&mut t,"b08_fang_slash").is_empty());
        assert!(t.cast(&LivingSoulHandler,Target::None,1,"{}").is_err());
    }
}
