//! 火繩銃（matchlock_gun）— 伊達政宗的 R 技能：變身火繩銃，45 秒增益。
//!
//! Buff-driven pattern：所有屬性變動（+射程/+傷害/暈眩機率）由 host 端的
//! buff 表根據 `matchlock_gun` buff id 決定。重複施放會 refresh duration。

use abi_stable::std_types::{RErr, ROk, RResult, RStr, RString};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript},
    stat_keys::StatKey,
    types::{EntityHandle, Target},
    world::GameWorldDyn,
};
use omoba_core::ability_meta::{EffectSpec, TargetSelector};
use omoba_template_ids::ABILITY_MATCHLOCK_GUN;
use std::collections::HashMap;

use crate::ability_builder::{build_ability_ffi, extra_at_id_f32};

const BUFF_ID: &str = "matchlock_gun";
use crate::self_transform::{Encoding,Field};
const MATCHLOCK_FIELDS: &[Field] = &[
    Field {key:"range_bonus",stat:StatKey::AttackRangeBonus,maximum:10_000,encoding:Encoding::Raw},
    Field {key:"damage_bonus",stat:StatKey::BaseAttackBonusDamage,maximum:1_000_000,encoding:Encoding::Raw},
    Field {key:"lifesteal",stat:StatKey::Lifesteal,maximum:1,encoding:Encoding::Raw},
    Field {key:"stun_chance",stat:StatKey::AttackStunChance,maximum:1,encoding:Encoding::Normalized},
    Field {key:"stun_duration",stat:StatKey::AttackStunDuration,maximum:60,encoding:Encoding::Normalized},
];

pub struct MatchlockGunHandler;

impl AbilityScript for MatchlockGunHandler {
    fn ability_id(&self) -> RStr<'_> {
        RStr::from_str(ABILITY_MATCHLOCK_GUN.as_str())
    }

    fn execute(
        &self, caster: EntityHandle, target: Target, level: u8,
        json: RStr<'_>, world: &mut GameWorldDyn<'_>,
    ) -> RResult<(), RString> {
        let result=crate::self_transform::resolve(ABILITY_MATCHLOCK_GUN,level,json.as_str(),MATCHLOCK_FIELDS,None)
            .and_then(|spec|crate::self_transform::execute(world,caster,target,BUFF_ID,spec));
        match result {Ok(())=>ROk(()),Err(error)=>RErr(error.into())}
    }
}

pub fn matchlock_gun_ffi() -> AbilityDefFFI {
    let dur_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "duration", 1);
    let range_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "range_bonus", 1);
    let dmg_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "damage_bonus", 1);
    let leech_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "lifesteal", 1);
    let stun_c_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "stun_chance", 1);
    let stun_d_lv1 = extra_at_id_f32(ABILITY_MATCHLOCK_GUN, "stun_duration", 1);
    let mut preview_mods = HashMap::new();
    preview_mods.insert(StatKey::AttackRangeBonus.as_str().into(), range_lv1);
    preview_mods.insert(StatKey::BaseAttackBonusDamage.as_str().into(), dmg_lv1);
    preview_mods.insert(StatKey::Lifesteal.as_str().into(), leech_lv1);
    preview_mods.insert(StatKey::AttackStunChance.as_str().into(), stun_c_lv1);
    preview_mods.insert(StatKey::AttackStunDuration.as_str().into(), stun_d_lv1);
    let effects_preview = vec![EffectSpec::Buff {
        target: TargetSelector::SelfUnit,
        duration: Some(dur_lv1),
        modifiers: preview_mods,
    }];
    build_ability_ffi(ABILITY_MATCHLOCK_GUN, MatchlockGunHandler, effects_preview)
}
