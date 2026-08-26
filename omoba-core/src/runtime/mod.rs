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
#[allow(unused_imports)]
pub use native::*;
#[cfg(not(target_arch = "wasm32"))]
pub use selective::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    //! Wasm currently uses protocol/rendering paths only. Native runtime
    //! execution is isolated behind target cfg, but the public module remains
    //! present so `omoba-core::runtime` is a mandatory contract.
}

#[cfg(target_arch = "wasm32")]
#[allow(unused_imports)]
pub use wasm::*;
