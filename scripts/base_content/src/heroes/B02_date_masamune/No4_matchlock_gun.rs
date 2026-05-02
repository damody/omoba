//! 火繩銃（matchlock_gun）— 伊達政宗的 T 技能：變身火繩銃，45 秒增益。
//!
//! Buff-driven pattern：所有屬性變動（+射程/+傷害/暈眩機率）由 host 端的
//! buff 表根據 `matchlock_gun` buff id 決定。重複施放會 refresh duration。

use abi_stable::std_types::{ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Fixed32, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{AbilityLevelData, EffectSpec, TargetSelector};
use omoba_template_ids::{ABILITY_MATCHLOCK_GUN, ABILITY_MATCHLOCK_GUN_CONST};
use std::collections::HashMap;

use crate::ability_builder::{build_ability_ffi, extra_at_f32};

const BUFF_ID: &str = "matchlock_gun";

pub struct MatchlockGunHandler;

impl AbilityScript for MatchlockGunHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_MATCHLOCK_GUN.as_str())
    }

    fn execute(
        &self,
        caster: EntityHandle,
        _target: Target,
        _level: u8,
        level_data_json: RStr<'_>,
        world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let level_data: AbilityLevelData = serde_json::from_str(level_data_json.as_str())
            .unwrap_or_default();
        // JSON extras still f64-encoded.
        let get_f = |k: &str| {
            level_data
                .extra
                .get(k)
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
        };
        // Phase 1de.2: sum_add-aggregated stats emit raw Fixed32 i32 (lockstep-correct).
        // Helpers that read via raw `payload.get(...).as_f64()` (attack_stun_*) keep f64.
        let get_raw = |k: &str| -> i32 {
            Fixed32::from_raw((get_f(k) * 1024.0) as i32).raw()
        };
        // duration: f64 → Fixed32 at boundary for the FFI add_stat_buff call.
        let duration_f = get_f("duration");
        let duration = Fixed32::from_raw((duration_f * 1024.0) as i32);
        // damage_bonus 為絕對傷害點（90/130/170）→ BaseAttackBonusDamage；
        // attack_stun_* 不在 StatKey 聚合路徑（game_processor 直接 .as_f64() 讀）→ 保留 f64。
        let mut modifiers = serde_json::Map::new();
        modifiers.insert(StatKey::AttackRangeBonus.as_str().into(), serde_json::json!(get_raw("range_bonus")));
        modifiers.insert(StatKey::BaseAttackBonusDamage.as_str().into(), serde_json::json!(get_raw("damage_bonus")));
        modifiers.insert(StatKey::AttackStunChance.as_str().into(), serde_json::json!(get_f("stun_chance")));
        modifiers.insert(StatKey::AttackStunDuration.as_str().into(), serde_json::json!(get_f("stun_duration")));
        let mods_str = serde_json::Value::Object(modifiers).to_string();
        world.add_stat_buff(caster, RStr::from_str(BUFF_ID), duration, (&*mods_str).into());
        world.log_info(RStr::from_str("[matchlock_gun] transformed"));
        ROk(())
    }
}

pub fn matchlock_gun_ffi() -> AbilityDefFFI {
    let dur_lv1 = extra_at_f32(&ABILITY_MATCHLOCK_GUN_CONST, "duration", 1);
    let range_lv1 = extra_at_f32(&ABILITY_MATCHLOCK_GUN_CONST, "range_bonus", 1);
    let dmg_lv1 = extra_at_f32(&ABILITY_MATCHLOCK_GUN_CONST, "damage_bonus", 1);
    let stun_c_lv1 = extra_at_f32(&ABILITY_MATCHLOCK_GUN_CONST, "stun_chance", 1);
    let stun_d_lv1 = extra_at_f32(&ABILITY_MATCHLOCK_GUN_CONST, "stun_duration", 1);
    let mut preview_mods = HashMap::new();
    preview_mods.insert(StatKey::AttackRangeBonus.as_str().into(), range_lv1);
    preview_mods.insert(StatKey::BaseAttackBonusDamage.as_str().into(), dmg_lv1);
    preview_mods.insert(StatKey::AttackStunChance.as_str().into(), stun_c_lv1);
    preview_mods.insert(StatKey::AttackStunDuration.as_str().into(), stun_d_lv1);
    let effects_preview = vec![EffectSpec::Buff {
        target: TargetSelector::SelfUnit,
        duration: Some(dur_lv1),
        modifiers: preview_mods,
    }];
    build_ability_ffi(ABILITY_MATCHLOCK_GUN, MatchlockGunHandler, effects_preview)
}
