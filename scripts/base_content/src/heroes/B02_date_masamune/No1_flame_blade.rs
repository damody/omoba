//! 火焰刀（flame_blade）— 伊達政宗的 W：對單體目標或目標點範圍造成火焰傷害。

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

pub const ABILITY_ID: &str = "flame_blade";
const SWIPE_RADIUS: f32 = 100.0;

pub struct FlameBladeHandler;

impl AbilityScript for FlameBladeHandler {
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

        match target {
            Target::Entity(victim) => {
                world.deal_damage(victim, damage, DamageKind::Magical, RNone);
            }
            Target::Point(p) => {
                world.emit_explosion(p, SWIPE_RADIUS, 0.2);
                let enemies = world.query_enemies_in_range(p, SWIPE_RADIUS, caster);
                for victim in enemies.iter().copied() {
                    world.deal_damage(victim, damage, DamageKind::Magical, RNone);
                }
            }
            Target::None => {
                world.log_warn(RStr::from_str("[flame_blade] no target — abort"));
            }
        }
        ROk(())
    }
}

pub fn flame_blade_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "damage".into(),
            serde_json::json!(150.0 + 50.0 * lvl as f32),
        );
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 10.0,
                mana_cost: 80.0 + 20.0 * lvl as f32,
                cast_time: 0.0,
                range: 1100.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "火焰刀".into(),
        description: "往前方揮出一刀，對單體或範圍目標造成大量火焰傷害。".into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::Point,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Damage {
            target: TargetSelector::Target,
            amount: 200.0,
            damage_type: DamageType::Magical,
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn flame_blade_ffi() -> AbilityDefFFI {
    let def = flame_blade_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(FlameBladeHandler, TD_Opaque),
    }
}
