//! M3：改由 behavior IR 執行的技能（docs/plans/2026-10-28-history-nobu-full-port/m3-ir-design.md）。
//!
//! 技能的 metadata 照舊來自它的 `*_ffi()`；執行的腳本換成 omoba-core 的 `IrAbility`，讀 catalog 裡
//! 由 v3 文件編出的 IR。Rust handler 還留著，給重播比對（IR 版與 handler 版每 tick 狀態相同）與
//! M5 之前的回退。第二個欄位是 M3 的相容對照：v3 狀態 id（與 `@tether`）→ handler 用的 buff id
//! （`{caster}` 是施法者的 `id:gen`），`@learn` → 學會時的被動標記，讓兩版的 BuffStore 一致。
use abi_stable::{sabi_trait::TD_Opaque, std_types::{RBox, RVec}};
use omb_script_abi::ability::{AbilityDefFFI, AbilityScript_TO};
use omb_script_abi::{manifest::UnitDef, script::UnitScript_TO};
use omoba_core::runtime::native::ir::{IrAbility, IrHeroUnit};

pub(crate) const IR_ABILITIES: &[(&str, &[(&str, &str)])] = &[
    ("a02_venom_arrow", &[]),
    ("a02_wind_prayer", &[]),
    ("a02_wind_drift", &[]),
    ("a02_shadow_meld", &[]),
    ("b04_fire_blast", &[]),
    ("b04_flame_swiftness", &[]),
    ("b01_sniper_mode", &[]),
    ("b01_saika_corps", &[]),
    ("b01_rain_iron_cannon", &[]),
    ("b01_three_stage", &[]),
    ("c17_light_chain", &[("@tether", "c17_light_chain:{caster}")]),
    ("c17_victory_blessing", &[]),
    ("c17_divine_strike", &[]),
    ("c17_aurora_guard", &[]),
    ("c17_devotion", &[("@learn", "c17_devotion")]),
];

/// Heroes whose UnitScript only routed casts to an IR passive (`on_ability_executed`).
pub(crate) const IR_HERO_UNITS: &[(&str, &[&str])] = &[("oichi", &["c17_devotion"])];

fn rename(id: &str) -> &'static [(&'static str, &'static str)] {
    IR_ABILITIES.iter().find(|(name, _)| *name == id).map(|(_, r)| *r).unwrap_or(&[])
}

fn ability(id: &str) -> IrAbility {
    IrAbility::new(id, rename(id)).unwrap_or_else(|error| panic!("behavior IR: {error}"))
}

/// The IR script of `id`; panics when the catalog has no IR for it (a content build without
/// v3_ir.lua must not silently fall back to the handler).
pub(crate) fn script(id: &str) -> AbilityScript_TO<'static, RBox<()>> {
    AbilityScript_TO::from_value(ability(id), TD_Opaque)
}

/// Replace the scripts of the IR abilities in the registry list.
pub(crate) fn apply(abilities: &mut RVec<AbilityDefFFI>) {
    for entry in abilities.iter_mut() {
        let id = serde_json::from_str::<serde_json::Value>(entry.def_json.as_str()).ok()
            .and_then(|def| def["id"].as_str().map(str::to_owned));
        let Some(id) = id else { continue };
        if IR_ABILITIES.iter().any(|(name, _)| *name == id) { entry.script = script(&id); }
    }
}

/// The IR hero UnitScript of `hero`.
pub(crate) fn hero_unit(hero: &str) -> Option<UnitScript_TO<'static, RBox<()>>> {
    let (_, passives) = IR_HERO_UNITS.iter().find(|(name, _)| *name == hero)?;
    Some(UnitScript_TO::from_value(IrHeroUnit::new(hero, passives.iter().map(|id| ability(id)).collect()), TD_Opaque))
}

/// Replace the UnitScripts of the IR heroes in the registry list.
pub(crate) fn apply_units(units: &mut RVec<UnitDef>) {
    for unit in units.iter_mut() {
        if let Some(script) = hero_unit(unit.unit_id.as_str()) { unit.script = script; }
    }
}

/// The Rust handlers the IR replaced (M3 replay comparison; removed with them in M5).
#[cfg(test)]
pub(crate) fn handler_scripts() -> Vec<(&'static str, AbilityScript_TO<'static, RBox<()>>)> {
    use crate::heroes::A02_inahime as a02;
    use crate::heroes::B01_saika_magoichi as b01;
    use crate::heroes::B02_date_masamune as b04;
    use crate::heroes::C17_oichi as c17;
    vec![
        ("b04_fire_blast", b04::b04_fire_blast_ffi().script),
        ("b04_flame_swiftness", b04::b04_flame_swiftness_ffi().script),
        ("a02_venom_arrow", a02::a02_venom_arrow_ffi().script),
        ("a02_wind_prayer", a02::a02_wind_prayer_ffi().script),
        ("a02_wind_drift", a02::a02_wind_drift_ffi().script),
        ("a02_shadow_meld", a02::a02_shadow_meld_ffi().script),
        ("b01_sniper_mode", b01::b01_sniper_mode_ffi().script),
        ("b01_saika_corps", b01::b01_saika_corps_ffi().script),
        ("b01_rain_iron_cannon", b01::b01_rain_iron_cannon_ffi().script),
        ("b01_three_stage", b01::b01_three_stage_ffi().script),
        ("c17_light_chain", c17::c17_light_chain_ffi().script),
        ("c17_victory_blessing", c17::c17_victory_blessing_ffi().script),
        ("c17_divine_strike", c17::c17_divine_strike_ffi().script),
        ("c17_aurora_guard", c17::c17_aurora_guard_ffi().script),
        ("c17_devotion", c17::c17_devotion_ffi().script),
    ]
}

/// The Rust hero UnitScripts the IR replaced.
#[cfg(test)]
pub(crate) fn handler_units() -> Vec<(&'static str, UnitScript_TO<'static, RBox<()>>)> {
    vec![("oichi", UnitScript_TO::from_value(crate::heroes::C17_oichi::hero_unit::hero_unit_script(), TD_Opaque))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_ir_ability_loads_from_the_catalog_and_replaces_its_handler() {
        for (id, rename) in IR_ABILITIES {
            assert!(omoba_core::runtime::native::ir::has_ir(id), "{id}: no IR in the catalog");
            IrAbility::new(id, rename).unwrap();
        }
        let manifest = crate::get_manifest();
        let abilities = (manifest.abilities())();
        for (id, _) in IR_ABILITIES {
            assert!(abilities.iter().any(|a| a.def_json.as_str().contains(&format!("\"id\":\"{id}\""))), "{id} registered");
        }
        let units = (manifest.units())();
        for (hero, _) in IR_HERO_UNITS {
            assert!(units.iter().any(|u| u.unit_id.as_str() == *hero), "{hero} unit registered");
        }
    }
}
