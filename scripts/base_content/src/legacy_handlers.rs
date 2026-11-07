//! M3' ④：產生的 Rust 取代的手寫 handler（取代 IR 之後的重播比對用；M5 換完一位就刪那位）。
//! M5（2026-10-30）：所有英雄都改跑產生的 Rust，handler 已刪除；這張表現在是空的。
use abi_stable::std_types::RBox;
use omb_script_abi::{ability::AbilityScript_TO, script::UnitScript_TO};

/// The Rust handlers of the abilities that now run generated Rust (none left since M5).
pub(crate) fn handler_scripts() -> Vec<(&'static str, AbilityScript_TO<'static, RBox<()>>)> {
    Vec::new()
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
