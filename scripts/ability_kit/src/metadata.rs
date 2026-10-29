//! The catalog metadata of an ability as the host registry reads it (`AbilityDefFFI::def_json`).
//! Ported from base_content `ability_builder.rs`: values come from the compiled content
//! (omoba-template-ids), never from the generated code.
use omoba_core::ability_meta::{
    AbilityDef, AbilityLevelData, AbilityType, CastType, TargetType,
};
use omoba_template_ids::{
    active_ability_const, active_ability_display,
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

/// The full `AbilityDef` of a catalog ability, or None when the catalog does not carry it.
pub fn ability_def(name: &str) -> Option<AbilityDef> {
    let id = omoba_template_ids::ability_by_name(name)?;
    let c = active_ability_const(id)?;
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
                cast_interrupt_on_damage: ld.cast_interrupt_on_damage,
                range: ld.range.to_f32_for_render(),
                extra,
            },
        );
    }
    Some(AbilityDef {
        // Content `channel` makes the host own the pulse lifecycle for any
        // handler; pulses call the handler's on_channel_pulse.
        host_execution:c.pulse_channel.map(|channel|omoba_core::ability_meta::HostAbilityExecution::PulseChannel {
            anchor:match channel.anchor {0=>omoba_content_model::ChannelAnchor::SelfOrigin,1=>omoba_content_model::ChannelAnchor::Point,
                _=>omoba_content_model::ChannelAnchor::Unit},
            interval_key:channel.interval_key.to_owned(),pulses_key:channel.pulses_key.to_owned(),
            radius_key:channel.radius_key.map(str::to_owned)}),
        id: id.as_str().to_string(),
        name: active_ability_display(id).to_string(),
        description: c.description.to_string(),
        ability_type: ability_type_from(c.ability_type),
        target_type: target_type_from(c.target_type),
        cast_type: cast_type_from(c.cast_type),
        icon: (!c.icon.is_empty()).then(|| c.icon.to_string()),
        max_level: c.max_level,
        levels,
        effects_preview: Vec::new(),
        conditions: Vec::new(),
        properties: HashMap::new(),
    })
}
