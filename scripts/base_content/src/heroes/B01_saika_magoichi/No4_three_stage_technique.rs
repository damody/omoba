//! 三段擊（three_stage_technique）— 雜賀孫市的 R 大絕：變身 buff。
//!
//! 數值（duration / atk_bonus_pct / multi_shot_count）由 templates.json
//! `abilities[three_stage_technique]` 提供。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Fixed32, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, EffectSpec, TargetSelector};
use omoba_template_ids::{ABILITY_THREE_STAGE_TECHNIQUE, ABILITY_THREE_STAGE_TECHNIQUE_CONST};
use std::collections::HashMap;

use crate::ability_builder::{build_ability_ffi, extra_at, extra_at_f32};

const BUFF_ID: &str = "three_stage_transform";

pub struct ThreeStageHandler;

impl AbilityScript for ThreeStageHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_THREE_STAGE_TECHNIQUE.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        _target: Target,
        level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = serde_json::from_str(level_data_json.as_str())
            .unwrap_or_default();
        // omoba_core JSON extras still f32; convert to Fixed32 at boundary.
        // TODO Phase 1[d]: omoba_core::AbilityLevelData migrated → drop f64 hop.
        let get_fx = |k: &str, dft: Fixed32| -> Fixed32 {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| Fixed32::from_raw((v * 1024.0) as i32))
                .unwrap_or(dft)
        };
        let duration = get_fx("duration", extra_at(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "duration", level));
        let atk_bonus = get_fx("atk_bonus_pct", extra_at(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "atk_bonus_pct", level));
        let multi_shot = get_fx("multi_shot_count", extra_at(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "multi_shot_count", level));

        let mut modifiers = serde_json::Map::new();
        modifiers.insert(
            StatKey::TotalDamageOutgoingPercentage.as_str().into(),
            serde_json::json!(atk_bonus.to_f32_for_render()),
        );
        modifiers.insert(
            StatKey::MultiShotVisual.as_str().into(),
            serde_json::json!(multi_shot.to_f32_for_render()),
        );
        modifiers.insert(
            "visual_effect".into(),
            serde_json::json!("three_stage_transform_red"),
        );
        let mods_str = serde_json::Value::Object(modifiers).to_string();
        world.add_stat_buff(
            caster,
            RStr::from_str(BUFF_ID),
            duration,
            (&*mods_str).into(),
        );
        world.log_info(RStr::from_str("[three_stage_technique] transformed"));
        ROk(())
    }
}

pub fn three_stage_ffi() -> AbilityDefFFI {
    let dur_lv1 = extra_at_f32(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "duration", 1);
    let atk_lv1 = extra_at_f32(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "atk_bonus_pct", 1);
    let multi_lv1 = extra_at_f32(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "multi_shot_count", 1);
    let mut preview_mods = HashMap::new();
    preview_mods.insert(StatKey::TotalDamageOutgoingPercentage.as_str().into(), atk_lv1);
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
