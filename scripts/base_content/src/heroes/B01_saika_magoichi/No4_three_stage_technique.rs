//! 三段擊（three_stage_technique）— 雜賀孫市的 R 大絕：變身 buff。
//!
//! 數值（duration / atk_bonus_pct / multi_shot_count）由 templates.lua
//! `abilities[three_stage_technique]` 提供。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Fixed64, Target},
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
        let level_data: AbilityLevelData =
            serde_json::from_str(level_data_json.as_str()).unwrap_or_default();
        // omoba_core JSON 額外內容仍為 f32；在邊界處轉換為固定64。
        // 第 2 階段：omoba_core::AbilityLevelData 仍為 f32；第二階段 KCP 標籤返工中的重新設計。
        let get_fx = |k: &str, dft: Fixed64| -> Fixed64 {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .map(|v| Fixed64::from_raw((v * 1024.0) as i64))
                .unwrap_or(dft)
        };
        let duration = get_fx(
            "duration",
            extra_at(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "duration", level),
        );
        let atk_bonus = get_fx(
            "atk_bonus_pct",
            extra_at(&ABILITY_THREE_STAGE_TECHNIQUE_CONST, "atk_bonus_pct", level),
        );
        let multi_shot = get_fx(
            "multi_shot_count",
            extra_at(
                &ABILITY_THREE_STAGE_TECHNIQUE_CONST,
                "multi_shot_count",
                level,
            ),
        );

        let mut modifiers = serde_json::Map::new();
        // 階段 1de.2：發出原始的 Fix64 i32 — 主機 BuffStore::read_fixed_from_payload
        // 與傳統的 f64 量化路徑相比，偏好整數（鎖步正確）。
        modifiers.insert(
            StatKey::TotalDamageOutgoingPercentage.as_str().into(),
            serde_json::json!(atk_bonus.raw()),
        );
        modifiers.insert(
            StatKey::MultiShotVisual.as_str().into(),
            serde_json::json!(multi_shot.raw()),
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
