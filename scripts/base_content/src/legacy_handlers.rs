//! M3' ④：產生的 Rust 取代的手寫 handler（取代 IR 之後的重播比對用；M5 換完一位就刪那位）。
//! 稻姬的 handler 在 hero_a02_impl（`legacy-handlers` feature），其餘還在本 crate 的 heroes/。
use abi_stable::{sabi_trait::TD_Opaque, std_types::RBox};
use omb_script_abi::{ability::AbilityScript_TO, script::UnitScript_TO};

/// The Rust handlers of the abilities that now run generated Rust.
pub(crate) fn handler_scripts() -> Vec<(&'static str, AbilityScript_TO<'static, RBox<()>>)> {
    use crate::heroes::B01_saika_magoichi as b01;
    use crate::heroes::B02_date_masamune as b04;
    use crate::heroes::C17_oichi as c17;
    let mut handlers = vec![
        ("b04_fire_blast", b04::b04_fire_blast_ffi().script),
        ("b04_flame_swiftness", b04::b04_flame_swiftness_ffi().script),
        ("b01_sniper_mode", b01::b01_sniper_mode_ffi().script),
        ("b01_saika_corps", b01::b01_saika_corps_ffi().script),
        ("b01_rain_iron_cannon", b01::b01_rain_iron_cannon_ffi().script),
        ("b01_three_stage", b01::b01_three_stage_ffi().script),
        ("c17_light_chain", c17::c17_light_chain_ffi().script),
        ("c17_victory_blessing", c17::c17_victory_blessing_ffi().script),
        ("c17_divine_strike", c17::c17_divine_strike_ffi().script),
        ("c17_aurora_guard", c17::c17_aurora_guard_ffi().script),
        ("c17_devotion", c17::c17_devotion_ffi().script),
    ];
    handlers.extend(hero_a02_impl::handler_scripts());
    handlers
}

/// The Rust hero UnitScripts the generated ones replaced.
pub(crate) fn handler_units() -> Vec<(&'static str, UnitScript_TO<'static, RBox<()>>)> {
    vec![("oichi", UnitScript_TO::from_value(crate::heroes::C17_oichi::hero_unit::hero_unit_script(), TD_Opaque))]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_replaced_handler_is_registered_by_its_generated_ability() {
        let abilities = (crate::get_manifest().abilities())();
        for (id, _) in handler_scripts() {
            assert!(abilities.iter().any(|a| a.script.ability_id().as_str() == id), "{id} registered");
        }
        let units = (crate::get_manifest().units())();
        for (hero, _) in handler_units() {
            assert!(units.iter().any(|u| u.unit_id.as_str() == hero), "{hero} unit registered");
        }
    }
}
