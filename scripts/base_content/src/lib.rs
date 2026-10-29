//! omb base_content — 基礎遊戲單元的本機腳本。
//!
//! 匯出一個包含此 DLL 提供的每個單元和功能的「清單」。
//! omb 主機透過 `abi_stable::library::RootModule::load_from_file` 載入它。

#![allow(non_snake_case)]

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    sabi_trait::prelude::TD_Opaque,
    std_types::{RErr, ROk, RStr, RString, RVec},
};
use omb_script_abi::{
    ability::AbilityDefFFI,
    manifest::{
        Manifest, Manifest_Ref, RuntimeLuaReloadInfoFFI, RuntimeLuaReloadResultFFI, UnitDef,
    },
    prelude::{
        SUMMON_SAIKA_GUNNER, TOWER_ARTY, TOWER_BOMB, TOWER_BOOMERANG, TOWER_CAKE_SPLASH,
        TOWER_DART, TOWER_ICE, TOWER_TACK,
    },
    script::UnitScript_TO,
};

pub use ability_kit::ability_builder;
mod generic_effects;
mod swept_dash;
mod point_blast;
mod passive_cone;
mod self_transform;
#[cfg(test)]
mod legacy_handlers;
mod summon_formation;
mod owner_cast;
#[cfg(test)]
mod summon_formation_tests;
#[cfg(test)]
mod self_transform_tests;
#[cfg(test)]
mod passive_cone_tests;
#[cfg(test)]
mod point_blast_tests;
mod heroes;
mod summons;
mod towers;

#[cfg(test)]
mod headless_hero_cast_tests;

#[cfg(test)]
mod single_lane_match_tests;
#[cfg(test)]
mod mana_script_tests;
#[cfg(test)]
mod ward_world_tests;
#[cfg(test)]
mod ward_transport_tests;
#[cfg(test)]
mod channel_tests;

/// Module id in the manifest and in scripts-manifest.json.
pub const MODULE_ID: &str = "base_content";

#[export_root_module]
fn root_manifest() -> Manifest_Ref {
    Manifest {
        units,
        abilities,
        dev_reload_runtime_lua_content,
        module_id,
        content_marker,
    content_identity,
    }
    .leak_into_prefix()
}

#[cfg(not(test))]
#[allow(dead_code)]
fn get_manifest() -> Manifest_Ref {
    root_manifest()
}

/// Tests register what a launch loads: base_content, then every hero DLL of hero_dlls.lua
/// (linked in-process as rlibs), in that order; then the test_only v3 heroes
/// (scripts/v3_test_heroes.lua), which have no DLL of their own yet.
#[cfg(test)]
pub(crate) fn get_manifest() -> Manifest_Ref {
    #[sabi_extern_fn]
    fn all_units() -> RVec<UnitDef> {
        let mut v = units();
        v.extend(hero_a02_impl::units());
        v.extend(hero_b01_impl::units());
        v.extend(hero_c17_impl::units());
        v.extend(hero_a13_impl::units());
        v
    }
    #[sabi_extern_fn]
    fn all_abilities() -> RVec<AbilityDefFFI> {
        let mut v = abilities();
        v.extend(hero_a02_impl::abilities());
        v.extend(hero_b01_impl::abilities());
        v.extend(hero_c17_impl::abilities());
        v.extend(hero_a13_impl::abilities());
        v
    }
    Manifest {
        units: all_units,
        abilities: all_abilities,
        dev_reload_runtime_lua_content,
        module_id,
        content_marker,
    content_identity,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
fn module_id() -> RStr<'static> {
    RStr::from_str(MODULE_ID)
}

#[sabi_extern_fn]
fn content_marker() -> RStr<'static> {
    ability_kit::module::content_marker()
}

#[sabi_extern_fn]
fn content_identity() -> RStr<'static> {
    ability_kit::module::content_identity()
}

#[sabi_extern_fn]
fn units() -> RVec<UnitDef> {
    let mut v: RVec<UnitDef> = RVec::new();

    v.push(UnitDef {
        unit_id: TOWER_DART.as_str().into(),
        script: UnitScript_TO::from_value(towers::dart::DartTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_BOMB.as_str().into(),
        script: UnitScript_TO::from_value(towers::bomb::BombTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_TACK.as_str().into(),
        script: UnitScript_TO::from_value(towers::tack::TackTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_ICE.as_str().into(),
        script: UnitScript_TO::from_value(towers::ice::IceTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_BOOMERANG.as_str().into(),
        script: UnitScript_TO::from_value(towers::boomerang::BoomerangTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_ARTY.as_str().into(),
        script: UnitScript_TO::from_value(towers::arty::ArtyTower, TD_Opaque),
    });
    v.push(UnitDef {
        unit_id: TOWER_CAKE_SPLASH.as_str().into(),
        script: UnitScript_TO::from_value(towers::cake_splash::CakeSplashTower, TD_Opaque),
    });

    // 召喚物：由英雄技能（如 saika_reinforcements）透過 spawn_summoned_unit
    // 呼叫時附加 ScriptUnitTag 綁定到此 id，讓 dispatch on_tick 驅動 AI。
    v.push(UnitDef {
        unit_id: SUMMON_SAIKA_GUNNER.as_str().into(),
        script: UnitScript_TO::from_value(summons::SaikaGunner, TD_Opaque),
    });
    // Heroes whose content sets `unit_script = true` (generated from templates).
    include!(concat!(env!("OUT_DIR"), "/hero_unit_registry.rs"));
    // Mobile hunting summons (generic flying chase AI, summons::chaser).
    for chaser in heroes::C15_tamako::No4_hundred_arrows::chasers() {
        v.push(UnitDef { unit_id: chaser.unit_id.clone(), script: UnitScript_TO::from_value(chaser, TD_Opaque) });
    }
    // G-3: every immobile template without a dedicated script gets the generic
    // emplacement AI, so fixed summons are pure data.
    // Tombstones leave holes in the ID space; scan a bounded range, once at load.
    for raw in 1..=4096u16 {
        let Some(name) = omoba_template_ids::try_summon_id_str(omoba_template_ids::SummonId(raw)) else { continue; };
        let immobile = omoba_template_ids::active_summon_stats(omoba_template_ids::SummonId(raw)).is_some_and(|stats| stats.immobile);
        if immobile && !v.iter().any(|def| def.unit_id.as_str() == name) {
            v.push(UnitDef {
                unit_id: name.into(),
                script: UnitScript_TO::from_value(summons::emplacement::Emplacement { unit_id: name.into() }, TD_Opaque),
            });
        }
    }

    v
}

#[sabi_extern_fn]
fn abilities() -> RVec<AbilityDefFFI> {
    let mut v: RVec<AbilityDefFFI> = RVec::new();
    include!(concat!(env!("OUT_DIR"), "/hero_ability_registry.rs"));
    // M3' ④: B04's fire blast and flame swiftness run its generated Rust (heroes/date_masamune);
    // its other abilities stay hand-written here until B04 gets its own DLL (M5).
    for entry in v.iter_mut() {
        let generated = match entry.script.ability_id().as_str() {
            "b04_fire_blast" => hero_b04_impl::generated::b04_fire_blast::ffi(),
            "b04_flame_swiftness" => hero_b04_impl::generated::b04_flame_swiftness::ffi(),
            _ => continue,
        };
        // The metadata stays the catalog one the handler used; only the script changes.
        entry.script = generated.script;
    }
    v
}

#[sabi_extern_fn]
fn dev_reload_runtime_lua_content(expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    ability_kit::module::dev_reload_runtime_lua_content(MODULE_ID, expected_hash)
}

#[cfg(test)]
mod generated_registry_tests {
    use super::*;

    const EXPECTED_IDS: &[&str] = include!(concat!(env!("OUT_DIR"), "/hero_ability_ids.rs"));

    #[test]
    fn lua_generated_registry_matches_exported_script_ids() {
        let definitions = abilities();
        let actual = definitions
            .iter()
            .map(|definition| definition.script.ability_id().as_str().to_owned())
            .collect::<Vec<_>>();
        assert_eq!(actual, EXPECTED_IDS);
        for (definition, expected_id) in definitions.iter().zip(EXPECTED_IDS) {
            let metadata: serde_json::Value =
                serde_json::from_str(definition.def_json.as_str()).expect("ability metadata JSON");
            assert_eq!(metadata["id"], *expected_id);
        }
        assert_eq!(actual.iter().collect::<std::collections::BTreeSet<_>>().len(),actual.len(),"one registration per ability ID");
        for id in ["lumen_bolt", "lumen_touch", "lumen_lance", "lumen_mend",
            "apprentice_bolt", "apprentice_touch", "apprentice_lance", "apprentice_mend"] {
            let definition = definitions
                .iter()
                .find(|definition| definition.script.ability_id().as_str() == id)
                .expect("Lua-declared ability is registered");
            let metadata: serde_json::Value =
                serde_json::from_str(definition.def_json.as_str()).unwrap();
            assert_eq!(metadata["effects_preview"].as_array().unwrap().len()
                +usize::from(!metadata["host_execution"].is_null()),1,
                "one preview or one explicit host execution declaration per registered generic skill");
        }
    }
}

#[cfg(test)]
mod hot_swap_tests {
    use abi_stable::prefix_type::PrefixTypeTrait;
    use omb_script_abi::manifest::Manifest;
    use omoba_core::scripting::ScriptRegistry;

    fn hero_manifest() -> omb_script_abi::manifest::Manifest_Ref {
        hero_a02_impl::get_manifest()
    }

    #[test]
    fn a_module_is_replaced_in_place_and_only_with_the_same_registrations() {
        let mut registry = ScriptRegistry::new();
        registry.insert_manifest(super::root_manifest());
        registry.insert_manifest(hero_manifest());
        let order: Vec<String> = registry.iter_abilities().map(|(id, _, _)| id.to_owned()).collect();
        let modules: Vec<String> = registry.modules().iter().map(|m| m.module.clone()).collect();
        assert_eq!(modules, ["base_content", "hero_a02"]);
        assert!(registry.modules()[1].abilities.iter().any(|id| id == "a02_wind_prayer"));

        let swapped = registry.replace_module(hero_manifest()).expect("same registrations swap");
        assert_eq!(swapped.module, "hero_a02");
        let after: Vec<String> = registry.iter_abilities().map(|(id, _, _)| id.to_owned()).collect();
        assert_eq!(after, order, "registration order stays");

        // A build that registers other abilities, or a module that is not loaded, is refused.
        #[abi_stable::sabi_extern_fn]
        fn no_abilities() -> abi_stable::std_types::RVec<omb_script_abi::ability::AbilityDefFFI> {
            abi_stable::std_types::RVec::new()
        }
        let fewer = Manifest {
            units: hero_a02_impl::units,
            abilities: no_abilities,
            dev_reload_runtime_lua_content: super::dev_reload_runtime_lua_content,
            module_id: super::module_id,
            content_marker: super::content_marker,
            content_identity: super::content_identity,
        }
        .leak_into_prefix();
        assert!(registry.replace_module(fewer).unwrap_err().contains("other units or abilities"));
        assert_eq!(registry.iter_abilities().count(), order.len());
    }
}
