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

pub mod ability_builder;
mod generic_effects;
mod heroes;
mod summons;
mod towers;

#[cfg(test)]
mod headless_hero_cast_tests;

#[cfg(test)]
mod single_lane_match_tests;

#[export_root_module]
fn get_manifest() -> Manifest_Ref {
    Manifest {
        units,
        abilities,
        dev_reload_runtime_lua_content,
    }
    .leak_into_prefix()
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

    v
}

#[sabi_extern_fn]
fn abilities() -> RVec<AbilityDefFFI> {
    let mut v: RVec<AbilityDefFFI> = RVec::new();
    include!(concat!(env!("OUT_DIR"), "/hero_ability_registry.rs"));
    v
}

#[sabi_extern_fn]
fn dev_reload_runtime_lua_content(expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    dev_reload_runtime_lua_content_impl(expected_hash)
}

#[cfg(feature = "runtime-lua-content")]
fn dev_reload_runtime_lua_content_impl(expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    let expected = if expected_hash.is_empty() {
        None
    } else {
        Some(expected_hash.as_str())
    };
    match omoba_template_ids::reload_runtime_lua_content_dev(expected) {
        Ok(Some(info)) => ROk(RuntimeLuaReloadInfoFFI {
            generation: info.generation,
            hash: info.hash.into(),
        }),
        Ok(None) => RErr(RString::from(
            "runtime Lua content is not active in base_content.dll",
        )),
        Err(err) => RErr(err.into()),
    }
}

#[cfg(not(feature = "runtime-lua-content"))]
fn dev_reload_runtime_lua_content_impl(_expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    RErr(RString::from(
        "base_content.dll was built without runtime-lua-content",
    ))
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
            assert_eq!(metadata["effects_preview"].as_array().unwrap().len(), 1);
        }
    }
}
