//! Helper：把 omoba_template_ids::AbilityConst（POD 常數）攤成完整
//! `omoba_core::ability_meta::AbilityDef`（含 HashMap）。所有 ability scripts
//! 共用一份，避免每個 `*_def()` 重抄 levels HashMap 構造邏輯。
//!
//! 數值來源是 templates.lua → omoba-template-ids/build.rs codegen，scripts
//! 端不再寫 hardcoded number。

use abi_stable::{sabi_trait::prelude::TD_Opaque, std_types::RBox};
use omb_script_abi::{
    ability::{AbilityDefFFI, AbilityScript, AbilityScript_TO},
    AbilityId,
};
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, EffectSpec, TargetType,
};
use omoba_template_ids::{
    active_ability_const, active_ability_display, AbilityConst, AbilityId as TemplateAbilityId,
    AbilityTypeC, CastTypeC, TargetTypeC,
};
use std::collections::HashMap;

fn ability_type_from(c: AbilityTypeC) -> AbilityType {
    match c {
        AbilityTypeC::Active => AbilityType::Active,
        AbilityTypeC::Toggle => AbilityType::Toggle,
        AbilityTypeC::Ultimate => AbilityType::Ultimate,
        AbilityTypeC::Passive => AbilityType::Passive,
    }
}

fn cast_type_from(c: CastTypeC) -> CastType {
    match c {
        CastTypeC::Instant => CastType::Instant,
        CastTypeC::Channeled => CastType::Channeled,
    }
}

fn target_type_from(c: TargetTypeC) -> TargetType {
    match c {
        TargetTypeC::None => TargetType::None,
        TargetTypeC::Point => TargetType::Point,
        TargetTypeC::Unit => TargetType::Unit,
    }
}

/// 從 const POD 攤成完整 `AbilityDef`。`effects_preview` 留給 caller 自己塞
/// （含遞迴 Vec / 不適合 const POD），無傳入時預設空 Vec。
pub fn build_ability_def_from_const(
    id: AbilityId,
    c: &AbilityConst,
    effects_preview: Vec<EffectSpec>,
) -> AbilityDef {
    let mut levels = HashMap::new();
    for (i, ld) in c.levels.iter().enumerate() {
        let lvl = (i + 1) as u8;
        let mut extra = HashMap::new();
        for (key, per_lvl) in c.extras.iter() {
            // 第 2 階段：omoba_core::ability_meta JSON 元資料仍為 f32；在邊界處轉換。
            // 第二階段 KCP 標籤返工中的重新設計。
            extra.insert(
                (*key).to_string(),
                serde_json::json!(per_lvl[i].to_f32_for_render()),
            );
        }
        levels.insert(
            lvl.to_string(),
            AbilityLevelData {
                cooldown: ld.cooldown.to_f32_for_render(),
                mana_cost: ld.mana_cost.to_f32_for_render(),
                cast_time: ld.cast_time.to_f32_for_render(),
                range: ld.range.to_f32_for_render(),
                extra,
            },
        );
    }
    AbilityDef {
        id: id.as_str().to_string(),
        name: active_ability_display(id).to_string(),
        description: c.description.to_string(),
        ability_type: ability_type_from(c.ability_type),
        target_type: target_type_from(c.target_type),
        cast_type: cast_type_from(c.cast_type),
        icon: (!c.icon.is_empty()).then(|| c.icon.to_string()),
        max_level: c.max_level,
        levels,
        effects_preview,
        conditions: Vec::new(),
        properties: HashMap::new(),
    }
}

/// 從 ability id 直接組 FFI 結構 — 各 script 的 `*_ffi()` 主要走這條捷徑。
pub fn build_ability_ffi<S: AbilityScript + 'static>(
    id: AbilityId,
    handler: S,
    effects_preview: Vec<EffectSpec>,
) -> AbilityDefFFI {
    let c = active_ability_const(id).unwrap_or_else(|| {
        panic!(
            "ability_const not found for id={} (raw={}); is templates.lua abilities[] complete?",
            id.as_str(),
            id.raw()
        )
    });
    let def = build_ability_def_from_const(id, c, effects_preview);
    let def_json = serde_json::to_string(&def).expect("AbilityDef serialize must succeed");
    AbilityDefFFI {
        def_json: def_json.into(),
        script: AbilityScript_TO::from_value(handler, TD_Opaque),
    }
}

/// 取單級 extras Fixed64 值；找不到 panic（caller 必須確認 templates.lua 有該 key）。
pub fn extra_at(c: &AbilityConst, key: &str, level: u8) -> omoba_sim::Fixed64 {
    let lvl_idx = (level.saturating_sub(1) as usize).min(c.max_level.saturating_sub(1) as usize);
    for (k, per_lvl) in c.extras.iter() {
        if *k == key {
            return per_lvl[lvl_idx];
        }
    }
    panic!(
        "extra '{}' not found in ability extras (max_level={})",
        key, c.max_level
    );
}

/// 取單級 extras 並轉 f32 — 給 EffectSpec / preview 等 omoba_core metadata 用。
/// 第 2 階段：omoba_core::ability_meta::EffectSpec 為 f32；這個助手保留到該層
/// 在第 2 階段 KCP 標籤返工中遷移。
pub fn extra_at_f32(c: &AbilityConst, key: &str, level: u8) -> f32 {
    extra_at(c, key, level).to_f32_for_render()
}

pub fn extra_at_id(id: TemplateAbilityId, key: &str, level: u8) -> omoba_sim::Fixed64 {
    let c = active_ability_const(id).unwrap_or_else(|| {
        panic!(
            "ability_const not found for id={} (raw={})",
            id.as_str(),
            id.raw()
        )
    });
    extra_at(c, key, level)
}

pub fn extra_at_id_f32(id: TemplateAbilityId, key: &str, level: u8) -> f32 {
    extra_at_id(id, key, level).to_f32_for_render()
}

/// 用 helper 從 const fetch 描述，不過 caller 直接 ability_description() 也行；
/// re-export 圖讓 ability scripts 不用 import omoba_template_ids 兩遍。
pub use omoba_template_ids::active_ability_description as description_of;
