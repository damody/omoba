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
    std_types::RVec,
};
use omb_script_abi::{
    ability::AbilityDefFFI,
    manifest::{Manifest, Manifest_Ref, UnitDef},
    prelude::{
        SUMMON_SAIKA_GUNNER, TOWER_BOMB, TOWER_CAKE_SPLASH, TOWER_DART, TOWER_ICE, TOWER_TACK,
    },
    script::UnitScript_TO,
};

pub mod ability_builder;
mod heroes;
mod summons;
mod towers;

#[export_root_module]
fn get_manifest() -> Manifest_Ref {
    Manifest { units, abilities }.leak_into_prefix()
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

    // 齋香孫市 (B01)
    v.push(heroes::B01_saika_magoichi::sniper_mode_ffi());
    v.push(heroes::B01_saika_magoichi::saika_reinforcements_ffi());
    v.push(heroes::B01_saika_magoichi::rain_iron_cannon_ffi());
    v.push(heroes::B01_saika_magoichi::three_stage_ffi());

    // 伊達政宗 (B02)
    v.push(heroes::B02_date_masamune::flame_blade_ffi());
    v.push(heroes::B02_date_masamune::fire_dash_ffi());
    v.push(heroes::B02_date_masamune::flame_assault_ffi());
    v.push(heroes::B02_date_masamune::matchlock_gun_ffi());

    v
}
