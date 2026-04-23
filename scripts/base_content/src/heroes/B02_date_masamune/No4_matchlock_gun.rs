//! 火繩銃（matchlock_gun）— 伊達政宗的 T 技能：變身火繩銃，45 秒增益。
//!
//! Buff-driven pattern：所有屬性變動（+射程/+傷害/暈眩機率）由 host 端的
//! buff 表根據 `matchlock_gun` buff id 決定。重複施放會 refresh duration
//! （host 的 `add_buff` 實作預期會覆蓋 / 取 max duration）。

use abi_stable::{
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, ROk, RResult, RStr, RString},
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

pub const ABILITY_ID: &str = "matchlock_gun";
const BUFF_ID: &str = "matchlock_gun";

pub struct MatchlockGunHandler;

impl AbilityScript for MatchlockGunHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_ID)
    }

    fn execute(
        &self,
        caster: EntityHandle,
        _target: Target,
        _level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = match serde_json::from_str(level_data_json.as_str()) {
            Ok(d) => d,
            Err(_) => AbilityLevelData::default(),
        };
        let duration = level_data
            .extra
            .get("duration")
            .and_then(|v| v.as_f64())
            .unwrap_or(45.0) as f32;

        world.add_buff(caster, RStr::from_str(BUFF_ID), duration);
        world.log_info(RStr::from_str("[matchlock_gun] transformed for 45s"));
        ROk(())
    }
}

pub fn matchlock_gun_def() -> AbilityDef {
    let mut levels = HashMap::new();
    // 原設計有 3 級 (1~3)，保留既有資料
    let per_level: [(f32, f32, f32, f32); 3] = [
        (700.0, 90.0, 0.1, 260.0),  // lvl 1: range_bonus / damage_bonus / stun_duration / mana_cost
        (800.0, 130.0, 0.2, 310.0),
        (900.0, 170.0, 0.3, 360.0),
    ];
    for (i, (range_bonus, damage_bonus, stun_duration, mana_cost)) in per_level.iter().enumerate() {
        let lvl = (i + 1) as u8;
        let mut extra = HashMap::new();
        extra.insert("duration".into(), serde_json::json!(45.0));
        extra.insert("range_bonus".into(), serde_json::json!(*range_bonus));
        extra.insert("damage_bonus".into(), serde_json::json!(*damage_bonus));
        extra.insert("stun_chance".into(), serde_json::json!(0.87));
        extra.insert("stun_duration".into(), serde_json::json!(*stun_duration));
        extra.insert(
            "transformation_effect".into(),
            serde_json::json!("matchlock_transform"),
        );
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: 110.0,
                mana_cost: *mana_cost,
                cast_time: 0.0,
                range: 0.0,
                extra,
            },
        );
    }

    let mut preview_mods = HashMap::new();
    preview_mods.insert("attack_range_bonus".into(), 700.0);
    preview_mods.insert("base_damage_bonus".into(), 90.0);
    preview_mods.insert("attack_stun_chance".into(), 0.87);
    preview_mods.insert("attack_stun_duration".into(), 0.1);

    AbilityDef {
        id: ABILITY_ID.into(),
        name: "火繩銃".into(),
        description: "將武器變身火繩銃，增加攻擊距離與傷害，攻擊時有機率暈眩敵人。持續 45 秒。"
            .into(),
        ability_type: AbilityType::Active,
        target_type: TargetType::None,
        cast_type: CastType::Instant,
        icon: None,
        max_level: 3,
        levels,
        effects_preview: vec![EffectSpec::Buff {
            target: TargetSelector::SelfUnit,
            duration: 45.0,
            modifiers: preview_mods,
        }],
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

pub fn matchlock_gun_ffi() -> AbilityDefFFI {
    let def = matchlock_gun_def();
    let def_json = serde_json::to_string(&def).expect("serialize");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(MatchlockGunHandler, TD_Opaque),
    }
}
