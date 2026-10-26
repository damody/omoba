//! 自我奉獻（H04，來源 C17D current，D 欄被動，出生自動學會）：
//! - 受到的物理傷害 −20（physical_constant_block）。
//! - 每次成功施放技能，自身與周圍 800 內友軍回復最大生命 5% 與最大法力 4%
//!   （英雄 UnitScript on_skill_cast → host ally_area_restore 封包）。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},stat_keys::StatKey,types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{EffectSpec,TargetSelector};
use omoba_core::runtime::native::ally_area_restore;
use omoba_template_ids::{ABILITY_C17_DEVOTION,active_ability_const};
use std::collections::HashMap;
use crate::ability_builder::{build_ability_ffi,extra_at_id,extra_at_id_f32};

const ID:omoba_template_ids::AbilityId=ABILITY_C17_DEVOTION;
pub(crate) const PASSIVE_BUFF:&str="c17_devotion";
const BLOCK:&str="stat_bonus_permanent_physical_constant_block";
const HP:&str="heal_amount";
const MANA:&str="heal_mana_pct";
const RADIUS:&str="shape_radius";

fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    if !rank_ok(rank) {return None;}
    let block=extra_at_id(ID,BLOCK,rank);
    if block<=Fixed64::ZERO {return None;}
    Some(serde_json::json!({(StatKey::PhysicalConstantBlock.as_str()):block.raw(),"level":rank}).to_string())
}

/// One committed cast by the owner while the passive is learned.
pub(crate) fn on_owner_cast(w:&mut GameWorldDyn<'_>,caster:EntityHandle) {
    if !w.has_buff(caster,RStr::from_str(PASSIVE_BUFF)) {return;}
    let Some(at)=w.get_pos(caster).into_option() else {return;};
    let fraction=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let radius=extra_at_id(ID,RADIUS,1);
    let Some(spec)=(ally_area_restore::Spec {source_id:caster.id,source_generation:caster.gen,center:at,radius,
        hp_fraction:fraction(extra_at_id(ID,HP,1)),mana_fraction:fraction(extra_at_id(ID,MANA,1)),cast_range:radius}).checked() else {return;};
    w.add_stat_buff(caster,ally_area_restore::BUFF.into(),Fixed64::ONE,spec.payload().to_string().as_str().into());
}

pub struct DevotionHandler;
impl AbilityScript for DevotionHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ID.as_str())}
    fn execute(&self,_:EntityHandle,_:Target,_:u8,_:RStr<'_>,_:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        RErr("passive ability cannot be cast actively".into())
    }
    fn on_learn(&self,caster:EntityHandle,rank:u8,world:&mut GameWorldDyn<'_>) {
        let Some(payload)=passive_payload(rank) else {return;};
        world.add_stat_buff(caster,RStr::from_str(PASSIVE_BUFF),Fixed64::from_raw(i64::MAX),payload.as_str().into());
    }
}

pub fn c17_devotion_ffi()->AbilityDefFFI {
    let mut modifiers=HashMap::new();
    modifiers.insert(StatKey::PhysicalConstantBlock.as_str().into(),extra_at_id_f32(ID,BLOCK,1));
    build_ability_ffi(ID,DevotionHandler,vec![EffectSpec::Buff {target:TargetSelector::SelfUnit,duration:None,modifiers}])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

    fn owner_cast(t:&mut TestWorld)->Vec<Outcome> {
        use omoba_core::runtime::scripting::parallel_world_adapter::{ParallelAdapterCache,ParallelWorldAdapter};
        let cache=ParallelAdapterCache::new(&t.world,1);
        let mut adapter=ParallelWorldAdapter::new(&cache,t.source);
        let caster=ParallelWorldAdapter::entity_to_handle(t.source);
        let mut w=omb_script_abi::world::GameWorld_TO::from_ptr(abi_stable::RMut::new(&mut adapter),abi_stable::sabi_trait::prelude::TD_Opaque);
        on_owner_cast(&mut w,caster);
        drop(w);adapter.finish()
    }

    #[test]
    fn learning_blocks_twenty_physical_and_casts_restore_allies_around_the_caster() {
        let mut t=TestWorld::new();
        assert!(owner_cast(&mut t).is_empty(),"not learned yet");
        let learned=t.learn(&DevotionHandler,1);
        let block=learned.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==PASSIVE_BUFF=>
            payload[StatKey::PhysicalConstantBlock.as_str()].as_i64(),_=>None});
        assert_eq!(block,Some(20*1024));
        let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,PASSIVE_BUFF,Fixed64::from_raw(i64::MAX),serde_json::json!({}));
        let packet=owner_cast(&mut t).iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
            ally_area_restore::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("restore packet");
        assert_eq!((packet.center,packet.radius),(p(0,0),Fixed64::from_i32(800)));
        assert_eq!((packet.hp_fraction,packet.mana_fraction),(Fixed64::from_raw(51),Fixed64::from_raw(41)));
        assert!(t.cast(&DevotionHandler,Target::None,1,"{}").is_err());
    }
}
