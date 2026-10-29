//! 心眼（真田幸村，來源 B06R；history_nobu B06.txt、eventfororder.lua:134）：被動。
//! - 閃避 evasion（EVASION_CONSTANT），學習／升級以同一 buff id 取代；
//! - 真田心眼術：被敵方指定技能時 guard_duration 秒內魔抗 +magic_resist，之後 guard_cooldown 秒才能再次發動。
//! 數值見 v3.lua。
//!   以 host spell_guard 項目宣告（同一個被動 buff 的 payload），由 host 在敵方指定技能
//!   施放成立時觸發；重生時 BuffStore 清空、on_learn 重新放回，所以冷卻重置。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,
    types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::spell_guard;
use omoba_template_ids::{ABILITY_B06_MIND_EYE,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_B06_MIND_EYE;
pub(crate) const PASSIVE_BUFF:&str="b06_mind_eye";

fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let evasion=extra_at_id(ID,"evasion",rank);
    let (resist,duration,cooldown)=(extra_at_id(ID,"magic_resist",rank),extra_at_id(ID,"guard_duration",rank),
        extra_at_id(ID,"guard_cooldown",rank));
    if evasion<=Fixed64::ZERO || evasion>Fixed64::from_i32(100) || resist<=Fixed64::ZERO || duration<=Fixed64::ZERO || cooldown<Fixed64::ZERO {return None;}
    let mut row=spell_guard::payload(resist,duration,cooldown);
    // Authored percent becomes a fraction of one, rounded (§5.3).
    row[StatKey::EvasionConstant.as_str()]=((evasion.raw()+50)/100).into();
    row["level"]=rank.into();
    Some(row.to_string())
}

pub struct MindEyeHandler;
impl AbilityScript for MindEyeHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn b06_mind_eye_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::EvasionConstant.as_str().into(),extra_at_id_f32(ID,"evasion",1)/100.0);
    build_ability_ffi(ID,MindEyeHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::B06_sanada_yukimura::b06_test_support::*;
    use omoba_core::Outcome;
    use omoba_core::runtime::{BuffStore,UnitStats};
    use specs::WorldExt;

    #[test]
    fn learning_grants_rank_evasion_and_the_guard_row() {
        for (rank,evasion,cooldown) in [(1u8,92,23),(2,143,17),(3,195,12),(4,246,7)] {
            let mut t=TestWorld::new();
            let outcomes=t.learn(&MindEyeHandler,rank);
            let rows=buffs(&outcomes,PASSIVE_BUFF);
            assert_eq!(rows.len(),1);
            assert_eq!(rows[0].1[StatKey::EvasionConstant.as_str()],evasion,"rank {rank}");
            assert_eq!(rows[0].1[spell_guard::MAGIC_RESIST],75*1024);
            assert_eq!(rows[0].1[spell_guard::DURATION],3*1024);
            assert_eq!(rows[0].1[spell_guard::COOLDOWN].as_i64(),Some(cooldown*1024));
            t.apply(outcomes);
            let b=t.world.read_resource::<BuffStore>();
            assert_eq!(UnitStats::from_refs(&b,false).evasion_chance(t.source),Fixed64::from_raw(evasion));
        }
    }

    /// The host trips the guard when an enemy targets the hero with an ability.
    #[test]
    fn an_enemy_targeted_cast_trips_the_guard_through_the_host() {
        let mut t=TestWorld::new();let enemy=t.body(p(100,0),2);
        let learned=t.learn(&MindEyeHandler,1);t.apply(learned);
        let b=t.world.read_resource::<BuffStore>();
        let factions=t.world.read_storage::<omoba_core::Faction>();
        let rows=spell_guard::trip(&b,factions.get(enemy),factions.get(t.source),t.source).unwrap();
        assert_eq!(rows[0].1,Fixed64::from_i32(3));assert_eq!(rows[1].1,Fixed64::from_i32(26));
    }

    #[test]
    fn passive_cannot_be_cast_and_bad_ranks_grant_nothing() {
        let mut t=TestWorld::new();
        assert!(t.cast(&MindEyeHandler,Target::None,1,&rank_json(ID,1)).is_err());
        assert!(t.learn(&MindEyeHandler,0).iter().all(|o|!matches!(o,Outcome::AddBuff {..})));
        assert!(t.learn(&MindEyeHandler,5).iter().all(|o|!matches!(o,Outcome::AddBuff {..})));
    }
}
