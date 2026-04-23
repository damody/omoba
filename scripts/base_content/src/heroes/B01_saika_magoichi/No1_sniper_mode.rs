//! 狙擊模式（sniper_mode）— 雜賀孫市的 W 技能。
//!
//! Toggle：無冷卻、無法力消耗，切換開/關。開啟時施加 `sniper_mode` buff：
//! - 射程加成（range_bonus）
//! - 傷害加成（damage_bonus）
//! - 攻擊速度懲罰（attack_speed_multiplier）
//! - 移動速度懲罰（move_speed_multiplier）
//! - 命中率加成（accuracy_bonus）
//!
//! 數值以「buff 表驅動」：`effects_preview` 宣告 level 1 的基準值；host 端
//! 依 caster 當前 ability level 查 `AbilityDef.levels` 套實際數值（本次 PoC
//! 尚未串 stat 計算層，只驗證 buff add/remove 通路）。

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RErr, ROk, RResult, RStr, RString},
};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    types::{EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, EffectSpec, TargetSelector, TargetType,
};
use std::collections::HashMap;

pub const ABILITY_ID: &str = "sniper_mode";
const BUFF_ID: &str = "sniper_mode";

pub struct SniperModeHandler;

impl AbilityScript for SniperModeHandler {
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
        let buff = RStr::from_str(BUFF_ID);
        if world.has_buff(caster, buff) {
            world.remove_buff(caster, buff);
            world.log_info(RStr::from_str("[sniper_mode] toggled OFF"));
        } else {
            // f32::INFINITY 代表 toggle 無限期（直到再次施放 remove）
            world.add_buff(caster, buff, f32::INFINITY);
            world.log_info(RStr::from_str("[sniper_mode] toggled ON"));
        }
        ROk(())
    }
}

/// 本技能的 metadata 定義。`base_content/src/lib.rs::abilities()` 會
/// serialize 後放入 `AbilityDefFFI::def_json`。
pub fn sniper_mode_def() -> AbilityDef {
    let mut levels = HashMap::new();
    for lvl in 1u8..=4 {
        let range_bonus = 100.0 * lvl as f32;
        let damage_bonus = 0.15 + 0.10 * (lvl - 1) as f32;
        let mut extra = HashMap::new();
        extra.insert("range_bonus".into(), serde_json::json!(range_bonus));
        extra.insert("damage_bonus".into(), serde_json::json!(damage_bonus));
        extra.insert("attack_speed_penalty".into(), serde_json::json!(-0.30));
        extra.insert("move_speed_penalty".into(), serde_json::json!(-0.50));
        extra.insert("accuracy_bonus".into(), serde_json::json!(0.10));
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 0.0,
                mana_cost: 0.0,
                cast_time: 0.0,
                range: 0.0,
                extra,
            },
        );
    }

    let mut preview_mods = HashMap::new();
    preview_mods.insert("range_bonus".into(), 100.0);
    preview_mods.insert("damage_bonus".into(), 0.15);
    preview_mods.insert("attack_speed_multiplier".into(), 0.70);
    preview_mods.insert("move_speed_multiplier".into(), 0.50);
    preview_mods.insert("accuracy_bonus".into(), 0.10);

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "狙擊模式".into(),
        description: "切換到狙擊模式，增加射程和傷害，但降低攻擊速度和移動速度。".into(),
        ability_type: AbilityType::Toggle,
        target_type: TargetType::None,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 4,
        levels,
        effects_preview: vec![EffectSpec::Buff {
            target: TargetSelector::SelfUnit,
            duration: f32::INFINITY,
            modifiers: preview_mods,
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

/// 組 FFI 結構供 `base_content::abilities()` 回傳。
pub fn sniper_mode_ffi() -> AbilityDefFFI {
    let def = sniper_mode_def();
    let def_json = serde_json::to_string(&def)
        .expect("AbilityDef serialization must succeed for static definitions");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(SniperModeHandler, TD_Opaque),
    }
}
