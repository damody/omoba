//! Shared deterministic runtime boundary for `omb` and native `omfx`.
//!
//! This module is intentionally part of `omoba-core` instead of a separate
//! runtime crate so frontend and backend use the same simulation contract
//! without adding another library boundary.

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective_fixtures;
#[cfg(not(target_arch = "wasm32"))]
pub mod selective_replica;
#[cfg(not(target_arch = "wasm32"))]
pub mod projection_policy;
#[cfg(not(target_arch = "wasm32"))]
pub mod stable_fact;
#[cfg(not(target_arch = "wasm32"))]
pub mod visibility;
#[cfg(not(target_arch = "wasm32"))]
pub mod team_projector;
#[cfg(not(target_arch = "wasm32"))]
pub mod team_stream;
#[cfg(not(target_arch = "wasm32"))]
pub mod observer_validation;

#[cfg(not(target_arch = "wasm32"))]
#[allow(unused_imports)]
pub use native::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective_fixtures::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective_replica::*;
#[cfg(not(target_arch = "wasm32"))]
pub use projection_policy::*;
#[cfg(not(target_arch = "wasm32"))]
pub use stable_fact::*;
#[cfg(not(target_arch = "wasm32"))]
pub use visibility::*;
#[cfg(not(target_arch = "wasm32"))]
pub use team_projector::*;
#[cfg(not(target_arch = "wasm32"))]
pub use team_stream::*;
#[cfg(not(target_arch = "wasm32"))]
pub use observer_validation::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! Wasm currently uses protocol/rendering paths only. Native runtime
    //! execution is isolated behind target cfg, but the public module remains
    //! present so `omoba-core::runtime` is a mandatory contract.
}

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
pub use wasm::*;
