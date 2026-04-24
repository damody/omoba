//! 三段擊（three_stage_technique）— 雜賀孫市的 R 大絕：變身 buff。
//!
//! 變身 5 秒：
//! - 攻擊力 +200%（`TOTALDAMAGEOUTGOING_PERCENTAGE = 2.0` → `(base + bonus) × 3`）
//! - 普攻特效改為連發 3 發子彈但只判定 1 次傷害
//!   （`multi_shot_visual = 3.0`，由 `game_processor::handle_projectile` 讀取）
//! - 前端紅色變身特效標記 `visual_effect = "three_stage_transform_red"`

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, ROk, RResult, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    stat_keys::StatKey,
    types::{EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, EffectSpec, TargetSelector, TargetType,
};
use std::collections::HashMap;

pub const ABILITY_ID: &str = "three_stage_technique";
const BUFF_ID: &str = "three_stage_transform";

/// 變身持續秒數
const DURATION_S: f32 = 5.0;
/// 攻擊力 +200% → 套在 TOTALDAMAGEOUTGOING_PERCENTAGE（value = 2.0）
const ATK_BONUS_PCT: f32 = 2.0;
/// 普攻連發視覺彈數（其中 1 發真實判傷，其餘為無傷害視覺彈）
const MULTI_SHOT_COUNT: f32 = 3.0;

pub struct ThreeStageHandler;

impl AbilityScript for ThreeStageHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_ID)
    }

    fn execute(
        &self,
        caster: EntityHandle,
        _target: Target,
        _level: u8,
        _level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let mut modifiers = serde_json::Map::new();
        modifiers.insert(
            StatKey::TotalDamageOutgoingPercentage.as_str().into(),
            serde_json::json!(ATK_BONUS_PCT),
        );
        modifiers.insert(
            "multi_shot_visual".into(),
            serde_json::json!(MULTI_SHOT_COUNT),
        );
        modifiers.insert(
            "visual_effect".into(),
            serde_json::json!("three_stage_transform_red"),
        );
        let mods_str = serde_json::Value::Object(modifiers).to_string();
        world.add_stat_buff(
            caster,
            RStr::from_str(BUFF_ID),
            DURATION_S,
            (&*mods_str).into(),
        );
        world.log_info(RStr::from_str("[three_stage_technique] transformed 5s"));
        ROk(())
    }
}

pub fn three_stage_def() -> AbilityDef {
    let mut levels = HashMap::new();
    // Lv1-4：冷卻 15/13/11/9 秒，mana 90/100/110/120
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert("duration".into(), serde_json::json!(DURATION_S));
        extra.insert("atk_bonus_pct".into(), serde_json::json!(ATK_BONUS_PCT));
        extra.insert("multi_shot_count".into(), serde_json::json!(MULTI_SHOT_COUNT));
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 17.0 - 2.0 * lvl as f32,
                mana_cost: 80.0 + 10.0 * (lvl - 1) as f32,
                cast_time: 0.0,
                range: 0.0,
                extra,
            },
        );
    }

    let mut preview_mods = HashMap::new();
    preview_mods.insert(StatKey::TotalDamageOutgoingPercentage.as_str().into(), ATK_BONUS_PCT);
    preview_mods.insert("multi_shot_visual".into(), MULTI_SHOT_COUNT);

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "三段擊".into(),
        description: "變身 5 秒：攻擊力 +200%，普攻特效改為連發 3 發子彈（只判定 1 次傷害），身上有紅色變身特效。".into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::None,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Buff {
            target: TargetSelector::SelfUnit,
            duration: Some(DURATION_S),
            modifiers: preview_mods,
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn three_stage_ffi() -> AbilityDefFFI {
    let def = three_stage_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(ThreeStageHandler, TD_Opaque),
    }
}
