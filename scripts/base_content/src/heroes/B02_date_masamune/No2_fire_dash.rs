//! 火焰衝擊：碰撞限制的瞬間衝刺，沿實際路徑各命中一次。

use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    types::{DamageKind, DamageProfile, EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, DamageType, EffectSpec, TargetSelector};
use omoba_template_ids::ABILITY_FIRE_DASH;

use crate::ability_builder::{build_ability_ffi, extra_at_id, extra_at_id_f32};

pub struct FireDashHandler;

impl AbilityScript for FireDashHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_FIRE_DASH.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let Some(def)=omoba_template_ids::active_ability_const(ABILITY_FIRE_DASH) else {return RErr("missing dash content".into());};
        if level==0 || usize::from(level)>def.levels.len() {return RErr("invalid dash rank".into());}
        let data:AbilityLevelData=match serde_json::from_str(level_data_json.as_str()) {
            Ok(data)=>data,Err(_)=>return RErr("invalid dash metadata".into()),
        };
        let Target::Point(dest)=target else {return RErr("dash requires a point".into());};
        let read=|key:&str,max:f64|->Result<Fixed64,&'static str> {
            match data.extra.get(key) {
                None=>Ok(extra_at_id(ABILITY_FIRE_DASH,key,level)),
                Some(value)=>value.as_f64().filter(|value|value.is_finite() && *value>0.0 && *value<=max)
                    .map(|value|Fixed64::from_raw((value*1024.0).round() as i64)).ok_or("invalid dash number"),
            }
        };
        if !data.range.is_finite() || data.range<=0.0 || data.range>10_000.0 {return RErr("invalid dash range".into());}
        let values=(read("damage_per_tick",100_000.0),read("dash_duration",60.0),read("dash_width",5_000.0));
        let (Ok(damage),Ok(duration),Ok(width))=values else {return RErr("invalid dash extras".into());};
        if duration<=Fixed64::ZERO || duration>Fixed64::from_i32(60)
            || damage<=Fixed64::ZERO || damage>Fixed64::from_i32(100_000) {
            return RErr("invalid dash damage budget".into());
        }
        // Ten authored damage quanta per second, with no rounded 0.1 divisor.
        let damage=damage*duration*Fixed64::from_i32(10);
        match crate::swept_dash::execute(world,caster,dest,crate::swept_dash::SweptDash {
            range:Fixed64::from_raw((f64::from(data.range)*1024.0).round() as i64),width,damage,
            damage_kind:DamageKind::Magical,damage_profile:DamageProfile::FIRE,duration,
        }) {Ok(_)=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn fire_dash_ffi() -> AbilityDefFFI {
    // EffectSpec 預覽值保持 f32（omoba_core 未遷移）。
    let dmg_lv1 = (extra_at_id(ABILITY_FIRE_DASH,"damage_per_tick",1)
        *extra_at_id(ABILITY_FIRE_DASH,"dash_duration",1)*Fixed64::from_i32(10)).to_f32_for_render();
    let width = extra_at_id_f32(ABILITY_FIRE_DASH, "dash_width", 1);
    let effects_preview = vec![EffectSpec::Damage {
        target: TargetSelector::Custom(format!("enemies_along_committed_dash(width={width})")),
        amount: dmg_lv1,
        damage_type: DamageType::Magical,
    }];
    build_ability_ffi(ABILITY_FIRE_DASH, FireDashHandler, effects_preview)
}
