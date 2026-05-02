//! 狙擊模式（sniper_mode）— 雜賀孫市的 W 技能。
//!
//! Toggle：無冷卻、無法力消耗，切換開/關。開啟時施加 `sniper_mode` buff，
//! 各等級數值（range_bonus / damage_bonus / attack_speed_penalty 等）由
//! templates.json `abilities[].extras` 透過 `omoba_template_ids::ABILITY_SNIPER_MODE_CONST`
//! 提供。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Fixed64, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityLevelData, EffectSpec, TargetSelector,
};
use omoba_template_ids::{ABILITY_SNIPER_MODE, ABILITY_SNIPER_MODE_CONST};
use std::collections::HashMap;

use crate::ability_builder::build_ability_ffi;

const BUFF_ID: &str = "sniper_mode";

pub struct SniperModeHandler;

impl AbilityScript for SniperModeHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_SNIPER_MODE.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        _target: Target,
        _level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let buff = RStr::from_str(BUFF_ID);
        if world.has_buff(caster, buff) {
            world.remove_buff(caster, buff);
            world.log_info(RStr::from_str("[sniper_mode] toggled OFF"));
        } else {
            let level_data: AbilityLevelData = serde_json::from_str(level_data_json.as_str())
                .unwrap_or_default();
            // Phase 1de.2: read from JSON extras as f64 then convert to raw Fixed64
            // for the wire payload (lockstep-correct integer encoding).
            let get_raw = |k: &str| -> i32 {
                let f = level_data
                    .extra
                    .get(k)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                Fixed64::from_raw((f * 1024.0) as i64).raw() as i32
            };
            let get_raw_scaled = |k: &str, mul: f64| -> i32 {
                let f = level_data
                    .extra
                    .get(k)
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0)
                    * mul;
                Fixed64::from_raw((f * 1024.0) as i64).raw() as i32
            };
            // Buff payload key 對齊 stat_keys.rs：
            // - ATTACK_RANGE_BONUS: 絕對加成（+100/200/...）
            // - BASEDAMAGEOUTGOING_PERCENTAGE: 0.15 表示 +15% 基礎傷
            // - ATTACKSPEED_BONUS_CONSTANT: Dota AS points（×100 對齊慣例）；-30 = 攻速降 30
            // - MOVESPEED_BONUS_PERCENTAGE: -0.5 會被聚合後套 (1 + sum) = 50% 移速
            // - ACCURACY_BONUS: 非 Dota 原生；game_processor 讀取
            let mut modifiers = serde_json::Map::new();
            modifiers.insert(StatKey::AttackRangeBonus.as_str().into(), serde_json::json!(get_raw("range_bonus")));
            modifiers.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(), serde_json::json!(get_raw("damage_bonus")));
            modifiers.insert(StatKey::AttackSpeedBonusConstant.as_str().into(), serde_json::json!(get_raw_scaled("attack_speed_penalty", 100.0)));
            modifiers.insert(StatKey::MoveSpeedBonusPercentage.as_str().into(), serde_json::json!(get_raw("move_speed_penalty")));
            modifiers.insert(StatKey::AccuracyBonus.as_str().into(), serde_json::json!(get_raw("accuracy_bonus")));
            let mods_str = serde_json::Value::Object(modifiers).to_string();
            // Toggle buff — duration is Fixed64 now; use a very large positive value as
            // "indefinite" sentinel (matches host BuffStore convention; toggle removes via has_buff/remove_buff).
            world.add_stat_buff(caster, buff, Fixed64::from_i32(i32::MAX / 1024), (&*mods_str).into());
            world.log_info(RStr::from_str("[sniper_mode] toggled ON"));
        }
        ROk(())
    }
}

pub fn sniper_mode_ffi() -> AbilityDefFFI {
    // preview 值對齊 Lv1 實際 buff payload，方便 UI 直接顯示
    let mut preview_mods = HashMap::new();
    preview_mods.insert(StatKey::AttackRangeBonus.as_str().into(), 100.0);
    preview_mods.insert(StatKey::BaseDamageOutgoingPercentage.as_str().into(), 0.15);
    preview_mods.insert(StatKey::AttackSpeedBonusConstant.as_str().into(), -30.0);
    preview_mods.insert(StatKey::MoveSpeedBonusPercentage.as_str().into(), -0.50);
    preview_mods.insert(StatKey::AccuracyBonus.as_str().into(), 0.10);
    let effects_preview = vec![EffectSpec::Buff {
        target: TargetSelector::SelfUnit,
        duration: None,
        modifiers: preview_mods,
    }];
    build_ability_ffi(ABILITY_SNIPER_MODE, SniperModeHandler, effects_preview)
}
