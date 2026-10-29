//! ability_kit: the layer every script DLL links statically (M3', docs/plans/
//! 2026-10-28-history-nobu-full-port/m3-rust-codegen-design.md §2.4). Hero DLLs and the code
//! v3_rust.lua generates call these helpers instead of the world directly. Nothing here keeps
//! state across calls: a hot reload swaps the DLL and keeps every state in the host (§4).

pub mod ability_builder;
pub mod metadata;
pub mod module;

// What the Rust generated from v3 documents (docs/character_pipeline/tools/v3_rust.lua) calls:
// `use ability_kit::*;` brings Cx, Ranked, StatusDef and the script-abi / abi_stable types.
mod codegen;
pub use codegen::*;

// Host components work line B adds (M5): more `impl Cx` methods, same glob export.
mod host_ext;
pub use host_ext::{DmgExtra, StatusFilter};
pub use omoba_core::runtime::native::hero_command::OrderKind;
/// Attribute names for `Cx::unit_num` / `unit_flag` / `unit_ref`.
pub use omb_script_abi::ext::unit_attr;
