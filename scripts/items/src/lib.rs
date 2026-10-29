//! items：商店道具的腳本，一個 DLL（M7，m3-rust-codegen-design.md §2.1）。
//! 道具程式由 build.rs 在每次建置前以 v3_rust.lua --items 產生到 $OUT_DIR/generated.rs；本檔手寫。
//! 還不能產生的道具不在 `abilities()` 裡，商店標「尚未開放」。
#![deny(warnings)]

/// Every item document's generated scripts (v3_rust.lua --items --single).
pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}

use ability_kit::*;

pub use ability_kit::Manifest_Ref;

/// Module id in the manifest and in scripts-manifest.json.
pub const MODULE_ID: &str = "items";

/// The root module `dll/` exports as items.dll.
pub fn get_manifest() -> Manifest_Ref {
    Manifest { units, abilities, dev_reload_runtime_lua_content, module_id, content_marker, content_identity }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn units() -> RVec<UnitDef> { generated::registry::units() }

#[sabi_extern_fn]
pub fn abilities() -> RVec<AbilityDefFFI> { generated::registry::abilities() }

#[sabi_extern_fn]
fn dev_reload_runtime_lua_content(expected_hash: RStr<'_>) -> RuntimeLuaReloadResultFFI {
    ability_kit::module::dev_reload_runtime_lua_content(MODULE_ID, expected_hash)
}

#[sabi_extern_fn]
fn module_id() -> RStr<'static> { RStr::from_str(MODULE_ID) }

#[sabi_extern_fn]
fn content_marker() -> RStr<'static> { ability_kit::module::content_marker() }

#[sabi_extern_fn]
fn content_identity() -> RStr<'static> { ability_kit::module::content_identity() }

#[cfg(test)]
mod tests {
    use omoba_core::ability_meta::{AbilityDef, AbilityKind, TargetType};
    use omoba_template_ids::MobaItemTargetShapeConst as Shape;

    /// items.dll (M7): each generated item is an item script (never a hero ability) whose target
    /// and rank-1 costs are the shop catalog's.
    #[test]
    fn every_generated_item_registers_as_its_catalog_item() {
        let all = super::generated::registry::abilities();
        assert!(!all.is_empty(), "some item generates");
        for ffi in all.iter() {
            let def: AbilityDef = serde_json::from_str(ffi.def_json.as_str()).expect("def json");
            assert_eq!(def.kind, AbilityKind::Item, "{}", def.id);
            let item = omoba_template_ids::MOBA_ITEM_CATALOG.iter()
                .find(|item| item.source_ref.is_some_and(|r| r.ends_with(&format!(":{}", def.id)))).expect("catalog item");
            let target = match item.target.shape { Shape::Self_ => TargetType::None, Shape::Point => TargetType::Point, Shape::Unit => TargetType::Unit };
            assert_eq!(def.target_type, target, "{}", def.id);
            let rank = def.get_level_data(1).expect("rank 1");
            assert_eq!(rank.cooldown, item.cooldown.to_f32_for_render(), "{}", def.id);
        }
    }
}
