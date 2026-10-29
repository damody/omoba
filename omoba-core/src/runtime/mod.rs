//! Shared deterministic runtime boundary for `omb` and native `omfx`.
//!
//! This module is intentionally part of `omoba-core` instead of a separate
//! runtime crate so frontend and backend use the same simulation contract
//! without adding another library boundary.

#[cfg(not(target_arch = "wasm32"))]
pub mod summoner_runtime;
#[cfg(not(target_arch = "wasm32"))]
pub mod combat_controls;
#[cfg(not(target_arch = "wasm32"))]
pub mod combat_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod input_scheme;
#[cfg(not(target_arch = "wasm32"))]
pub mod input_scheme_transport;
pub mod direction_transport;
pub mod summon_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod anti_probing;
#[cfg(not(target_arch = "wasm32"))]
pub mod authority_recovery;
#[cfg(not(target_arch = "wasm32"))]
pub mod filtered_specs;
#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub mod observer_validation;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod production_guards;
#[cfg(not(target_arch = "wasm32"))]
pub mod projection_policy;
#[cfg(not(target_arch = "wasm32"))]
pub mod presentation_cue;
#[cfg(not(target_arch = "wasm32"))]
pub mod buff_visual_state;
#[cfg(not(target_arch = "wasm32"))]
pub mod runtime_fact_state;
pub mod attack_visual_state;
pub mod channel_visual_state;
#[cfg(not(target_arch = "wasm32"))]
pub mod moba_map_layout;
#[cfg(not(target_arch = "wasm32"))]
pub mod secure_replica_contract;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod selective_contract_tests;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective_fixtures;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective_replica;
#[cfg(not(target_arch = "wasm32"))]
pub mod hero_attributes;
#[cfg(not(target_arch = "wasm32"))]
pub mod stable_fact;
#[cfg(not(target_arch = "wasm32"))]
pub mod communication;
#[cfg(not(target_arch = "wasm32"))]
pub mod communication_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod shop_receipt;
#[cfg(not(target_arch = "wasm32"))]
pub mod shop_undo_wire;
#[cfg(not(target_arch = "wasm32"))]
pub mod shop_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod recall_transport;
pub mod command_transport;
pub mod mana_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod summoner_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod ward_transport;
#[cfg(not(target_arch = "wasm32"))]
pub mod team_projector;
#[cfg(all(test, not(target_arch = "wasm32")))]
mod team_replica_contract_tests;
#[cfg(not(target_arch = "wasm32"))]
pub mod team_stream;
#[cfg(not(target_arch = "wasm32"))]
pub mod visibility;
#[cfg(not(target_arch = "wasm32"))]
pub mod fog_grid;
pub mod brush_vision;
#[cfg(not(target_arch = "wasm32"))]
pub mod authoritative_mode;

#[cfg(not(target_arch = "wasm32"))]
pub use anti_probing::*;
#[cfg(not(target_arch = "wasm32"))]
pub use authority_recovery::*;
#[cfg(not(target_arch = "wasm32"))]
pub use filtered_specs::*;
#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
pub use native::*;
#[cfg(not(target_arch = "wasm32"))]
pub use observer_validation::*;
#[cfg(not(target_arch = "wasm32"))]
pub use projection_policy::*;
#[cfg(not(target_arch = "wasm32"))]
pub use secure_replica_contract::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective_fixtures::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective_replica::*;
#[cfg(not(target_arch = "wasm32"))]
pub use stable_fact::*;
#[cfg(not(target_arch = "wasm32"))]
pub use team_projector::*;
#[cfg(not(target_arch = "wasm32"))]
pub use team_stream::*;
#[cfg(not(target_arch = "wasm32"))]
pub use visibility::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! Wasm currently uses protocol/rendering paths only. Native runtime
    //! execution is isolated behind target cfg, but the public module remains
    //! present so `omoba-core::runtime` is a mandatory contract.
}

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
pub use wasm::*;
