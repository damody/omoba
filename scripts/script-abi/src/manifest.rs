//! Manifest — 每個腳本 DLL 導出的根模組。
//!
//! 主機呼叫“Manifest_Ref::load_from_file(dll)”，然後迭代
//! 提供函數指標來收集`UnitDef`條目（遺留）和
//! `AbilityDefFFI` 條目（新）。

use crate::ability::AbilityDefFFI;
use crate::script::UnitScript_TO;
use abi_stable::{
    library::RootModule,
    package_version_strings,
    sabi_types::VersionStrings,
    std_types::{RBox, RString, RVec},
    StableAbi,
};

#[repr(C)]
#[derive(StableAbi)]
pub struct UnitDef {
    pub unit_id: RString,
    pub script: UnitScript_TO<'static, RBox<()>>,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = Manifest_Ref, prefix_fields = Manifest_Prefix)))]
#[sabi(missing_field(panic))]
pub struct Manifest {
    /// 傳回此 DLL 提供的每個單元。
    pub units: extern "C" fn() -> RVec<UnitDef>,

    /// 傳回此 DLL 提供的所有功能。未定义的 DLL
    /// 能力仍然需要導出這個函數傳回一個空
    /// `RVec`（`missing_field(panic)` 策略）。
    #[sabi(last_prefix_field)]
    pub abilities: extern "C" fn() -> RVec<AbilityDefFFI>,
}

impl RootModule for Manifest_Ref {
    abi_stable::declare_root_module_statics! { Manifest_Ref }
    const BASE_NAME: &'static str = "omb_script";
    const NAME: &'static str = "omb_script";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}
