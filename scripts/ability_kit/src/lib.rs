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
pub use host_ext::{DmgExtra, PickOrder, PickedItem, SourceFilter, StatusFilter};
// v3 `chance { mode = "prd" }` (E505).
mod prd;
// v3 stat `when` (E508).
mod when;
pub use when::{Attr, When};
// v3 `chain` (E528).
mod chain;
pub use chain::ChainOrder;
// v3 `passive.orb_slot` / `orb_slot_free` (E534).
mod orb_slot;
/// v3 `armor_multiplier(armor)`: the share of physical damage that lands through `armor` (the
/// rule layer's one armor formula).
pub use omoba_core::runtime::native::comp::damage::armor_multiplier;
pub use omoba_core::runtime::native::hero_command::OrderKind;
// v3 create_illusion's copy options (Cx::create_illusion)
pub use omoba_core::runtime::native::illusion::{Copy as IllusionCopy, Scale as IllusionScale, StatusCopy as IllusionStatusCopy};
/// Attribute names for `Cx::unit_num` / `unit_flag` / `unit_ref`.
pub use omb_script_abi::ext::unit_attr;
/// Attribute names for `Cx::game_num`.
pub use omb_script_abi::ext::game_attr;
