//! omb base_content — native scripts for base-game units.
//!
//! Exports one `Manifest` containing every unit + ability this DLL provides.
//! omb host loads this via `abi_stable::library::RootModule::load_from_file`.

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RVec},
};
use omb_script_abi::{
    ability::AbilityDefFFI,
    manifest::{Manifest, Manifest_Ref, UnitDef},
    script::UnitScript_TO,
};

mod heroes;
mod towers;

#[export_root_module]
fn get_manifest() -> Manifest_Ref {
    Manifest { units, abilities }.leak_into_prefix()
}

#[sabi_extern_fn]
fn units() -> RVec<UnitDef> {
    let mut v: RVec<UnitDef> = RVec::new();

    v.push(UnitDef {
        unit_id: "tower_dart".into(),
        script: UnitScript_TO::from_value(towers::dart::DartTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: "tower_bomb".into(),
        script: UnitScript_TO::from_value(towers::bomb::BombTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: "tower_tack".into(),
        script: UnitScript_TO::from_value(towers::tack::TackTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: "tower_ice".into(),
        script: UnitScript_TO::from_value(towers::ice::IceTower, TD_Opaque),
    });

    v
}

#[sabi_extern_fn]
fn abilities() -> RVec<AbilityDefFFI> {
    let mut v: RVec<AbilityDefFFI> = RVec::new();

    // Saika Magoichi (B01)
    v.push(heroes::B01_saika_magoichi::sniper_mode_ffi());
    v.push(heroes::B01_saika_magoichi::saika_reinforcements_ffi());
    v.push(heroes::B01_saika_magoichi::rain_iron_cannon_ffi());
    v.push(heroes::B01_saika_magoichi::three_stage_ffi());

    // Date Masamune (B02)
    v.push(heroes::B02_date_masamune::flame_blade_ffi());
    v.push(heroes::B02_date_masamune::fire_dash_ffi());
    v.push(heroes::B02_date_masamune::flame_assault_ffi());
    v.push(heroes::B02_date_masamune::matchlock_gun_ffi());

    v
}
