//! 踏火無痕（fire_dash）— 伊達政宗的 E：往目標點衝刺，沿路對敵人造成持續傷害。
//!
//! 原設計是「dash 過程中每 0.1 秒 tick damage」。此版本簡化為：
//! - 即時位移到目標點（`world.set_pos`）
//! - 對目的地周圍 dash_width 內敵人造 `total_damage = damage_per_tick * dash_duration / tick_interval`

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RNone, ROk, RResult, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    types::{DamageKind, EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, DamageType, EffectSpec, TargetSelector,
    TargetType,
};
use std::collections::HashMap;

pub const ABILITY_ID: &str = "fire_dash";
const DASH_WIDTH: f32 = 150.0;

pub struct FireDashHandler;

impl AbilityScript for FireDashHandler {
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
        let damage_per_tick = level_data
            .extra
            .get("damage_per_tick")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0) as f32;
        let dash_duration = level_data
            .extra
            .get("dash_duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5) as f32;
        let tick_interval = 0.1f32;
        let total_damage = damage_per_tick * (dash_duration / tick_interval);

        let dest = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str("[fire_dash] missing target point — abort"));
                return ROk(());
            }
        };

        world.set_pos(caster, dest);
        let enemies = world.query_enemies_in_range(dest, DASH_WIDTH / 2.0, caster);
        for victim in enemies.iter().copied() {
            world.deal_damage(victim, total_damage, DamageKind::Magical, RNone);
        }
        ROk(())
    }
}

pub fn fire_dash_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "damage_per_tick".into(),
            serde_json::json!(40.0 + 10.0 * lvl as f32),
        );
        extra.insert("dash_duration".into(), serde_json::json!(0.5));
        extra.insert("speed_bonus".into(), serde_json::json!(2.0));
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 30.0,
                mana_cost: 60.0 + 20.0 * lvl as f32,
                cast_time: 0.0,
                range: 800.0 + 100.0 * lvl as f32,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "踏火無痕".into(),
        description: "往前衝刺，沿路對敵人造成火焰傷害。".into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::Point,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Damage {
            target: TargetSelector::InRadius {
                center: vek::Vec2::new(0.0, 0.0),
                radius: DASH_WIDTH / 2.0,
                enemy: true,
            },
            amount: 250.0,
            damage_type: DamageType::Magical,
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn fire_dash_ffi() -> AbilityDefFFI {
    let def = fire_dash_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(FireDashHandler, TD_Opaque),
    }
}
