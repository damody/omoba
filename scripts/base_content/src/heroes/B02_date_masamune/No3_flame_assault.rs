//! 炎襲：瞬間圓形火焰／暈眩，接共用施法者生命綁定地面脈衝。
use abi_stable::std_types::{RErr,ROk,RResult,RStr,RString};
use omb_script_abi::{ability::{AbilityDefFFI,AbilityScript},types::{DamageKind,DamageProfile,EntityHandle,Target},world::GameWorldDyn};
use omoba_core::ability_meta::{DamageType,EffectSpec,TargetSelector};
use omoba_content_model::ControlEffectKind;
use omoba_template_ids::ABILITY_FLAME_ASSAULT;
use crate::ability_builder::{build_ability_ffi,extra_at_id_f32};

pub struct FlameAssaultHandler;
impl AbilityScript for FlameAssaultHandler {
    fn ability_id(&self)->RStr<'_> {RStr::from_str(ABILITY_FLAME_ASSAULT.as_str())}
    fn execute(&self,caster:EntityHandle,target:Target,level:u8,json:RStr<'_>,world:&mut GameWorldDyn<'_>)->RResult<(),RString> {
        let result=crate::point_blast::resolve(ABILITY_FLAME_ASSAULT,level,json.as_str(),"damage","radius",
            Some((ControlEffectKind::Stun,"stun_duration")),DamageKind::Magical,DamageProfile::FIRE)
            .and_then(|spec|crate::point_blast::with_zone(ABILITY_FLAME_ASSAULT,level,json.as_str(),spec,"zone_damage","zone_interval","zone_duration"))
            .and_then(|spec|crate::point_blast::execute(world,caster,target,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}
pub fn flame_assault_ffi()->AbilityDefFFI {
    let effects=vec![EffectSpec::AreaEffect {
        radius:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"radius",1),duration:0.2,tick_interval:0.0,
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,
            amount:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"damage",1),damage_type:DamageType::Magical},
            EffectSpec::StatusModifier {target:TargetSelector::AllEnemies,modifier_type:"stun".into(),value:1.0,
                duration:Some(extra_at_id_f32(ABILITY_FLAME_ASSAULT,"stun_duration",1))}],
    },EffectSpec::AreaEffect {
        radius:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"radius",1),
        duration:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"zone_duration",1),
        tick_interval:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"zone_interval",1),
        effects:vec![EffectSpec::Damage {target:TargetSelector::AllEnemies,
            amount:extra_at_id_f32(ABILITY_FLAME_ASSAULT,"zone_damage",1),damage_type:DamageType::Magical}],
    }];
    build_ability_ffi(ABILITY_FLAME_ASSAULT,FlameAssaultHandler,effects)
}
