//! 雨鐵炮（rain_iron_cannon）— 雜賀孫市的 R 大招：大範圍 AoE 傷害。
//!
//! 本版為「瞬發」實作：對目標點 `radius` 半徑內所有敵人立即套用全部傷害，
//! 並發一次爆炸特效。原始設計的 tick-based 持續傷害（每 0.2 秒一擊共 15 擊）
//! 需要 host 端的 ActiveEffectManager，待 Phase 3/4 接通後再改寫。

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RNone, ROk, RResult, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    types::{DamageKind, EntityHandle, Target, Vec2f},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, DamageType, EffectSpec, TargetSelector,
    TargetType,
};
use std::collections::HashMap;

pub const ABILITY_ID: &str = "rain_iron_cannon";

pub struct RainIronCannonHandler;

impl AbilityScript for RainIronCannonHandler {
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
        let damage = level_data
            .extra
            .get("damage")
            .and_then(|v| v.as_f64())
            .unwrap_or(80.0) as f32;
        let radius = level_data
            .extra
            .get("radius")
            .and_then(|v| v.as_f64())
            .unwrap_or(300.0) as f32;
        let vfx_duration = level_data
            .extra
            .get("duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(3.0) as f32;

        let center: Vec2f = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str(
                    "[rain_iron_cannon] missing target position — abort",
                ));
                return ROk(());
            }
        };

        world.emit_explosion(center, radius, vfx_duration);
        let enemies = world.query_enemies_in_range(center, radius, caster);
        for victim in enemies.iter().copied() {
            world.deal_damage(victim, damage, DamageKind::Magical, RNone);
        }
        ROk(())
    }
}

pub fn rain_iron_cannon_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "damage".into(),
            serde_json::json!(60.0 + 30.0 * lvl as f32),
        );
        extra.insert("radius".into(), serde_json::json!(250.0 + 25.0 * lvl as f32));
        extra.insert("duration".into(), serde_json::json!(3.0));
        extra.insert("tick_interval".into(), serde_json::json!(0.2));
        extra.insert("total_ticks".into(), serde_json::json!(15));
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 120.0 - 10.0 * lvl as f32,
                mana_cost: 200.0 + 50.0 * (lvl - 1) as f32,
                cast_time: 1.0,
                range: 1200.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "雨鐵炮".into(),
        description: "在目標區域降下鐵炮彈雨，對範圍內敵人造成持續傷害。".into(),
        ability_type: AbilityType::Ultimate,
        target_type: TargetType::Point,
        cast_type: CastType::Channeled,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::AreaEffect {
            radius: 300.0,
            duration: 3.0,
            tick_interval: 0.2,
            effects: vec![EffectSpec::Damage {
                target: TargetSelector::AllEnemies,
                amount: 5.3,
                damage_type: DamageType::Magical,
            }],
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn rain_iron_cannon_ffi() -> AbilityDefFFI {
    let def = rain_iron_cannon_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(RainIronCannonHandler, TD_Opaque),
    }
}
