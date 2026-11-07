//! What a script DLL reports about itself in its `Manifest` (M3' ①): its module id and the
//! content catalog it was built against. The host refuses a DLL whose catalog differs from its
//! own; launches read the same marker from the file (run_artifacts.lua `content_hash`).

use abi_stable::std_types::RStr;

/// `OM_CONTENT_CATALOG_DATA_HASH=<hash>;` of the content this DLL was compiled with. The
/// literal stays in the binary, so a launch can check the file without loading it (E125).
pub fn content_marker() -> RStr<'static> {
    RStr::from_str(std::hint::black_box(omoba_template_ids::CONTENT_CATALOG_MARKER))
}

/// The catalog identity hash (hero and ability ids, no values) this DLL was compiled with.
pub fn content_identity() -> RStr<'static> {
    RStr::from_str(omoba_template_ids::CONTENT_CATALOG_IDENTITY_HASH)
}

/// The catalog data hash inside a content marker, or None when it is not one.
pub fn marker_hash(marker: &str) -> Option<&str> {
    marker.strip_prefix("OM_CONTENT_CATALOG_DATA_HASH=")?.strip_suffix(';')
}

/// DEV-only `Manifest::dev_reload_runtime_lua_content`: every script DLL links its own copy of
/// omoba-template-ids, so each one reloads its own runtime Lua content snapshot.
#[cfg(feature = "runtime-lua-content")]
pub fn dev_reload_runtime_lua_content(
    module: &str,
    expected_hash: RStr<'_>,
) -> omb_script_abi::manifest::RuntimeLuaReloadResultFFI {
    use abi_stable::std_types::{RErr, ROk, RString};
    let expected = if expected_hash.is_empty() { None } else { Some(expected_hash.as_str()) };
    match omoba_template_ids::reload_runtime_lua_content_dev(expected) {
        Ok(Some(info)) => ROk(omb_script_abi::manifest::RuntimeLuaReloadInfoFFI {
            generation: info.generation,
            hash: info.hash.into(),
        }),
        Ok(None) => RErr(RString::from(format!("runtime Lua content is not active in {module}.dll"))),
        Err(err) => RErr(err.into()),
    }
}

#[cfg(not(feature = "runtime-lua-content"))]
pub fn dev_reload_runtime_lua_content(
    module: &str,
    _expected_hash: RStr<'_>,
) -> omb_script_abi::manifest::RuntimeLuaReloadResultFFI {
    abi_stable::std_types::RErr(format!("{module}.dll was built without runtime-lua-content").into())
}
