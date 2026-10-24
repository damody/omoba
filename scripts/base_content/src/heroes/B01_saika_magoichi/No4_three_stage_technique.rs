//! 三段擊（three_stage_technique）— 雜賀孫市的 R 大絕：變身 buff。
//!
//! 數值（duration / atk_bonus_pct / multi_shot_count）由 templates.lua
//! `abilities[three_stage_technique]` 提供。

use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{EffectSpec, TargetSelector};
use omoba_template_ids::ABILITY_THREE_STAGE_TECHNIQUE;
use std::collections::HashMap;

use crate::ability_builder::{build_ability_ffi, extra_at_id_f32};

const BUFF_ID: &str = "three_stage_transform";
use crate::self_transform::{Encoding,Field};
const THREE_STAGE_FIELDS: &[Field] = &[
    Field {key:"atk_bonus_pct",stat:StatKey::TotalDamageOutgoingPercentage,maximum:100,encoding:Encoding::Raw},
    Field {key:"multi_shot_count",stat:StatKey::MultiShotVisual,maximum:16,encoding:Encoding::Count},
];

pub struct ThreeStageHandler;

impl AbilityScript for ThreeStageHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_THREE_STAGE_TECHNIQUE.as_str())
    }

    fn execute(
        &self, caster: EntityHandle, target: Target, level: u8,
        json: RStr<'_>, world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let result=crate::self_transform::resolve(ABILITY_THREE_STAGE_TECHNIQUE,level,json.as_str(),THREE_STAGE_FIELDS,Some("three_stage_transform_red"))
            .and_then(|spec|crate::self_transform::execute(world,caster,target,BUFF_ID,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn three_stage_ffi() -> AbilityDefFFI {
    let dur_lv1 = extra_at_id_f32(ABILITY_THREE_STAGE_TECHNIQUE, "duration", 1);
    let atk_lv1 = extra_at_id_f32(ABILITY_THREE_STAGE_TECHNIQUE, "atk_bonus_pct", 1);
    let multi_lv1 = extra_at_id_f32(ABILITY_THREE_STAGE_TECHNIQUE, "multi_shot_count", 1);
    let mut preview_mods = HashMap::new();
    preview_mods.insert(
        StatKey::TotalDamageOutgoingPercentage.as_str().into(),
        atk_lv1,
    );
    preview_mods.insert(StatKey::MultiShotVisual.as_str().into(), multi_lv1);
    let effects_preview = vec![EffectSpec::Buff {
        target: TargetSelector::SelfUnit,
        duration: Some(dur_lv1),
        modifiers: preview_mods,
    }];
    build_ability_ffi(
        ABILITY_THREE_STAGE_TECHNIQUE,
        ThreeStageHandler,
        effects_preview,
    )
}
