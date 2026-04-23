//! 火焰強襲（flame_assault）— 伊達政宗的 R 大招：範圍傷害 + 暈眩。

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

pub const ABILITY_ID: &str = "flame_assault";
/// 統一以 "stun" 作為暈眩狀態 buff id，host 端的 hero_tick / hero_move_tick /
/// creep_tick 會讀取此 id 做控制判定。
const STUN_BUFF_ID: &str = "stun";

pub struct FlameAssaultHandler;

impl AbilityScript for FlameAssaultHandler {
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
            .unwrap_or(200.0) as f32;
        let stun_duration = level_data
            .extra
            .get("stun_duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3) as f32;
        let radius = level_data
            .extra
            .get("radius")
            .and_then(|v| v.as_f64())
            .unwrap_or(500.0) as f32;

        let center = match target {
            Target::Point(p) => p,
            _ => {
                world.log_warn(RStr::from_str("[flame_assault] missing target point — abort"));
                return ROk(());
            }
        };

        world.emit_explosion(center, radius, 0.2);
        let enemies = world.query_enemies_in_range(center, radius, caster);
        let stun_buff = RStr::from_str(STUN_BUFF_ID);
        for victim in enemies.iter().copied() {
            world.deal_damage(victim, damage, DamageKind::Magical, RNone);
            world.add_buff(victim, stun_buff, stun_duration);
        }
        ROk(())
    }
}

pub fn flame_assault_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "damage".into(),
            serde_json::json!(100.0 + 100.0 * lvl as f32),
        );
        extra.insert(
            "stun_duration".into(),
            serde_json::json!(0.3 * lvl as f32),
        );
        extra.insert("radius".into(), serde_json::json!(500.0));
        extra.insert("screen_shake".into(), serde_json::json!(true));
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 14.0,
                mana_cost: 100.0 + 20.0 * lvl as f32,
                cast_time: 0.0,
                range: 700.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "火焰強襲".into(),
        description: "施展一道巨型火焰，對範圍內敵人造成傷害並暈眩。".into(),
        ability_type: AbilityType::Ultimate,
        target_type: TargetType::Point,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::AreaEffect {
            radius: 500.0,
            duration: 0.2,
            tick_interval: 0.0,
            effects: vec![EffectSpec::Damage {
                target: TargetSelector::AllEnemies,
                amount: 300.0,
                damage_type: DamageType::Magical,
            }],
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn flame_assault_ffi() -> AbilityDefFFI {
    let def = flame_assault_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(FlameAssaultHandler, TD_Opaque),
    }
}
