//! 雜賀眾（saika_reinforcements）— 雜賀孫市的 E 技能：召喚雜賀鐵炮兵。
//!
//! 召喚 API（`world.spawn_summoned_unit`）尚未存在，此 handler 暫時
//! 只發 log_info 佔位。等 summon API 接通後再補實作。

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, ROk, RResult, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    types::{EntityHandle, Target, Vec2f},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, EffectSpec, TargetSelector, TargetType,
};
use std::collections::HashMap;

pub const ABILITY_ID: &str = "saika_reinforcements";

pub struct SaikaReinforcementsHandler;

impl AbilityScript for SaikaReinforcementsHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_ID)
    }

    fn execute(
        &self,
        caster: EntityHandle,
        target: Target,
        _level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = match serde_json::from_str(level_data_json.as_str()) {
            Ok(d) => d,
            Err(_) => AbilityLevelData::default(),
        };
        let count = level_data
            .extra
            .get("summon_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(1);
        let formation_radius = level_data
            .extra
            .get("formation_radius")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0) as f32;

        let center: Vec2f = match target {
            Target::Point(p) => p,
            _ => world
                .get_pos(caster)
                .into_option()
                .unwrap_or(Vec2f::new(0.0, 0.0)),
        };

        // Summon API 尚未串接，先 log 佔位（Phase 3+ 會接 world.spawn_summoned_unit）
        world.log_info(
            RString::from(format!(
                "[saika_reinforcements] TODO summon {} gunners at ({:.1},{:.1}) r={}",
                count, center.x, center.y, formation_radius
            ))
            .as_rstr(),
        );
        ROk(())
    }
}

pub fn saika_reinforcements_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "summon_count".into(),
            serde_json::json!(1 + lvl as u32 / 2),
        );
        extra.insert("duration".into(), serde_json::json!(40.0 + 5.0 * lvl as f32));
        extra.insert("formation_radius".into(), serde_json::json!(100.0));
        extra.insert("gunner_hp".into(), serde_json::json!(300.0 + 50.0 * lvl as f32));
        extra.insert(
            "gunner_damage".into(),
            serde_json::json!(40.0 + 10.0 * lvl as f32),
        );
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 28.0 - 1.0 * lvl as f32,
                mana_cost: 90.0,
                cast_time: 0.5,
                range: 800.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "雜賀眾".into(),
        description: "在目標位置召喚雜賀鐵炮兵協助作戰。".into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::Point,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Summon {
            unit_type: "saika_gunner".into(),
            count: 2,
            duration: Some(50.0),
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn saika_reinforcements_ffi() -> AbilityDefFFI {
    let def = saika_reinforcements_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(SaikaReinforcementsHandler, TD_Opaque),
    }
}
