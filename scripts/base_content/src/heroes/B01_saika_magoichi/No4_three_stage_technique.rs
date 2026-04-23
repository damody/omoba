//! 三段擊（three_stage_technique）— 雜賀孫市的 T 技能：三次連續物理傷害。
//!
//! 原始設計帶 `attack_interval`（每擊隔 0.15 秒）。此版本簡化為三擊立即結算
//! 傷害（host 端沒有 delayed effect 排程）；等 Phase 3 有 EffectManager 時
//! 可改寫為 tick-based schedule。

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

pub const ABILITY_ID: &str = "three_stage_technique";
const ARMOR_DEBUFF_ID: &str = "three_stage_armor_reduction";

pub struct ThreeStageHandler;

impl AbilityScript for ThreeStageHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_ID)
    }

    fn execute(
        &self,
        _caster: EntityHandle,
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
            .get("damage_per_attack")
            .and_then(|v| v.as_f64())
            .unwrap_or(50.0) as f32;
        let count = level_data
            .extra
            .get("attacks_count")
            .and_then(|v| v.as_u64())
            .unwrap_or(3);
        let armor_debuff = level_data
            .extra
            .get("armor_reduction")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32);

        let victim = match target {
            Target::Entity(e) => e,
            _ => {
                world.log_warn(RStr::from_str(
                    "[three_stage_technique] no target entity — abort",
                ));
                return ROk(());
            }
        };

        for _ in 0..count {
            world.deal_damage(victim, damage, DamageKind::Physical, RNone);
        }
        if armor_debuff.is_some() {
            world.add_buff(victim, RStr::from_str(ARMOR_DEBUFF_ID), 3.0);
        }
        ROk(())
    }
}

pub fn three_stage_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let mut extra = HashMap::new();
        extra.insert(
            "damage_per_attack".into(),
            serde_json::json!(50.0 + 25.0 * (lvl - 1) as f32),
        );
        extra.insert("attacks_count".into(), serde_json::json!(3));
        extra.insert("attack_interval".into(), serde_json::json!(0.15));
        extra.insert(
            "armor_reduction".into(),
            serde_json::json!(1.0 + 0.5 * (lvl - 1) as f32),
        );
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 14.0 - 1.0 * lvl as f32,
                mana_cost: 60.0 + 10.0 * (lvl - 1) as f32,
                cast_time: 0.0,
                range: 0.0,
                extra,
            },
        );
    }

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "三段擊".into(),
        description: "快速進行三次連續攻擊，每擊造成額外傷害並削弱目標護甲。".into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::Unit,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Damage {
            target: TargetSelector::Target,
            amount: 50.0 * 3.0,
            damage_type: DamageType::Physical,
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
