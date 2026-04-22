//! omb base_content — native scripts for base-game units.
//!
//! Exports one `Manifest` containing every unit this DLL provides.
//! omb host loads this via `abi_stable::library::RootModule::load_from_file`.

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RBox, RVec},
};
use omb_script_abi::{
    manifest::{Manifest, Manifest_Ref, UnitDef},
    script::UnitScript_TO,
};

mod towers;

#[export_root_module]
fn get_manifest() -> Manifest_Ref {
    Manifest { units }.leak_into_prefix()
}

#[sabi_extern_fn]
fn units() -> RVec<UnitDef> {
    let mut v: RVec<UnitDef> = RVec::new();

    v.push(UnitDef {
        unit_id: "tower_dart".into(),
        script: UnitScript_TO::from_value(towers::dart::DartTower, TD_Opaque),
    });

    v
}
