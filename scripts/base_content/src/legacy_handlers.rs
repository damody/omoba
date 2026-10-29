//! M3' ④：產生的 Rust 取代的手寫 handler（取代 IR 之後的重播比對用；M5 換完一位就刪那位）。
//! M5（2026-10-30）：A02、B01、C17 已全部改跑產生的 Rust，它們的 handler 已刪除；剩 B04 的兩招。
use abi_stable::std_types::RBox;
use omb_script_abi::{ability::AbilityScript_TO, script::UnitScript_TO};

/// The Rust handlers of the abilities that now run generated Rust. M5 (2026-10-30): A02, B01 and
/// C17 run generated Rust everywhere and their handlers are gone; B04's two stay until all of B04
/// generates.
pub(crate) fn handler_scripts() -> Vec<(&'static str, AbilityScript_TO<'static, RBox<()>>)> {
    use crate::heroes::B02_date_masamune as b04;
    vec![
        ("b04_fire_blast", b04::b04_fire_blast_ffi().script),
        ("b04_flame_swiftness", b04::b04_flame_swiftness_ffi().script),
    ]
}

/// The Rust hero UnitScripts the generated ones replaced (none left since M5).
pub(crate) fn handler_units() -> Vec<(&'static str, UnitScript_TO<'static, RBox<()>>)> {
    Vec::new()
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
