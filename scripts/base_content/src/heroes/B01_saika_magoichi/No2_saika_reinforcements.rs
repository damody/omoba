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
            .unwrap_or(1)
            .max(1);
        let duration = level_data
            .extra
            .get("duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(45.0) as f32;
        // 陣形可被 level_data 覆寫；未指定用預設值。
        let spawn_distance = level_data
            .extra
            .get("spawn_distance")
            .and_then(|v| v.as_f64())
            .unwrap_or(150.0) as f32;
        let spacing = level_data
            .extra
            .get("spacing")
            .and_then(|v| v.as_f64())
            .unwrap_or(60.0) as f32;

        // 取 caster 位置 + forward 方向：
        // - 若 Target::Point 存在，forward 朝向目標點（玩家有點擊瞄準）
        // - 否則用 caster 的 Facing
        let caster_pos = world
            .get_pos(caster)
            .into_option()
            .unwrap_or(Vec2f::new(0.0, 0.0));
        let facing_rad = match target {
            Target::Point(p) => (p.y - caster_pos.y).atan2(p.x - caster_pos.x),
            _ => world.get_facing(caster),
        };

        let fwd_x = facing_rad.cos();
        let fwd_y = facing_rad.sin();
        let perp_x = -fwd_y;
        let perp_y = fwd_x;
        let base = Vec2f::new(
            caster_pos.x + fwd_x * spawn_distance,
            caster_pos.y + fwd_y * spawn_distance,
        );
        let offset_center = (count as f32 - 1.0) * 0.5;

        let unit_type = RStr::from_str("saika_gunner");
        for i in 0..count {
            let off = (i as f32 - offset_center) * spacing;
            let p = Vec2f::new(base.x + perp_x * off, base.y + perp_y * off);
            world.spawn_summoned_unit(p, unit_type, caster, duration);
        }

        world.log_info(
            RString::from(format!(
                "[saika_reinforcements] summoned {} gunners in front of caster at ({:.1},{:.1}) facing={:.2}rad dur={}",
                count, base.x, base.y, facing_rad, duration
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
