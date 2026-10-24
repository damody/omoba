//! 火焰刀：指定地點的瞬間圓形火焰傷害。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_template_ids::ABILITY_FLAME_BLADE;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

pub struct FlameBladeHandler;
impl AbilityScript for FlameBladeHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ABILITY_FLAME_BLADE.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::point_blast::resolve(ABILITY_FLAME_BLADE,level,json.as_str(),"damage","swipe_radius",None,
            DamageKind::Magical,DamageProfile::FIRE)
            .and_then(|spec|crate::point_blast::execute(world,caster,target,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}
pub fn flame_blade_ffi()->AbilityDefFFI {
    let effects=vec![EffectSpec::AreaEffect {
        radius:extra_at_id_f32(ABILITY_FLAME_BLADE,"swipe_radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,
            amount:extra_at_id_f32(ABILITY_FLAME_BLADE,"damage",1),damage_type:DamageType::Magical}],
    }];
    build_ability_ffi(ABILITY_FLAME_BLADE,FlameBladeHandler,effects)
}