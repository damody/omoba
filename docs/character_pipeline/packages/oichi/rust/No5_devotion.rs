//! 自我奉獻（H04，來源 C17D current，C17.txt:321-389；C17.lua:new_C17D:40-85，D 欄被動，出生自動學會）：
//! - OnAbilityExecuted：每次成功使用技能或物品（切換技與 logging／tpscroll／flash_shoes／
//!   flash_ring／samurai_cloth 除外，49-54），自身與周圍 800 內友方英雄與小兵（不含建築，
//!   59-60）回復最大生命 5%（Heal）與最大法力 4%（74-77）。英雄 UnitScript on_skill_cast
//!   → host ally_area_restore 封包。
//! - tooltip 的「物理傷害 −20」：new_C17D_DMG（87-108）的條件 `caster.damagetype == 1` 永不成立，
//!   依腳本實際行為不做格擋（與 tooltip 不同）。
use abi_stable::std_types::{RErr,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{EntityHandle,Fixed64,Target},world::GameWorldDyn};
use omoba_core::ability_meta::EffectSpec;
use omoba_core::runtime::native::ally_area_restore;
use omoba_template_ids::{ABILITY_C17_DEVOTION,active_ability_const};
use crate::ability_builder::{build_ability_ffi,extra_at_id};
use crate::owner_cast;

const ID:omoba_template_ids::AbilityId=ABILITY_C17_DEVOTION;
/// Learned marker (payload `{"level":rank}`, no stats).
pub(crate) const PASSIVE_BUFF:&str="c17_devotion";
const HP:&str="heal_pct";
const MANA:&str="mana_pct";
const RADIUS:&str="shape_radius";

fn rank_ok(rank:u8)->bool {active_ability_const(ID).is_some_and(|c|rank>0 && rank<=c.max_level && usize::from(rank)<=c.levels.len())}

pub(crate) fn passive_payload(rank:u8)->Option<String> {
    rank_ok(rank).then(||serde_json::json!({"level":rank}).to_string())
}

/// One committed ability cast or item use by the owner while the passive is learned.
pub(crate) fn on_owner_cast(w:&mut GameWorldDyn<'_>,caster:EntityHandle,skill:&str) {
    if !owner_cast::counts(skill,owner_cast::C17D_EXCLUDED_ITEMS) || !w.has_buff(caster,RStr::from_str(PASSIVE_BUFF)) {return;}
    let Some(at)=w.get_pos(caster).into_option() else {return;};
    let fraction=|v:Fixed64|Fixed64::from_raw((v.raw()+50)/100);
    let radius=extra_at_id(ID,RADIUS,1);
    let Some(spec)=(ally_area_restore::Spec {exclude_buildings:true,..ally_area_restore::Spec::new(caster.id,caster.gen,at,radius,
        fraction(extra_at_id(ID,HP,1)),fraction(extra_at_id(ID,MANA,1)),radius)}).checked() else {return;};
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
    build_ability_ffi(ID,DevotionHandler,Vec::<EffectSpec>::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::heroes::C17_oichi::c17_test_support::*;
    use omoba_core::Outcome;
    use specs::WorldExt;

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
    fn learning_marks_without_a_block_and_casts_restore_allies_around_the_caster() {
        let mut t=TestWorld::new();
        assert!(owner_cast(&mut t,"c17_light_chain").is_empty(),"not learned yet");
        let learned=t.learn(&DevotionHandler,1);
        let payload=learned.iter().find_map(|o|match o {Outcome::AddBuff {buff_id,payload,..} if buff_id==PASSIVE_BUFF=>Some(payload.clone()),_=>None});
        assert_eq!(payload,Some(serde_json::json!({"level":1})),"no physical block (source condition never holds)");
        let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,PASSIVE_BUFF,Fixed64::from_raw(i64::MAX),serde_json::json!({}));
        let packet=owner_cast(&mut t,"c17_light_chain").iter().find_map(|o|match o {Outcome::AddBuff {buff_id,duration,payload,..}=>
            ally_area_restore::Spec::from_payload(buff_id,*duration,payload),_=>None}).expect("restore packet");
        assert_eq!((packet.center,packet.radius),(p(0,0),Fixed64::from_i32(800)));
        assert_eq!((packet.hp_fraction,packet.mana_fraction),(Fixed64::from_raw(51),Fixed64::from_raw(41)));
        assert!(packet.exclude_buildings && !packet.direct,"heroes and units only, through the healing policy");
        assert!(t.cast(&DevotionHandler,Target::None,1,"{}").is_err());
    }

    #[test]
    fn items_trigger_except_the_source_exclusions() {
        let mut t=TestWorld::new();let source=t.source;
        t.world.write_resource::<omoba_core::runtime::BuffStore>().add(source,PASSIVE_BUFF,Fixed64::from_raw(i64::MAX),serde_json::json!({}));
        for (skill,fires) in [("item:moba_stride_charm",true),("item:logging",false),("item:tpscroll",false),("item:flash_shoes",false),
            ("item:flash_ring",false),("item:samurai_cloth",false)] {
            assert_eq!(!owner_cast(&mut t,skill).is_empty(),fires,"{skill}");
        }
    }
}
