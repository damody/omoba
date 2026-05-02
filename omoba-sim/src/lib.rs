#![forbid(unsafe_code)]

//! omoba-sim: deterministic ECS simulation crate
//! Shared between server (omb) and client (omfx) for lockstep networking.

pub mod fixed;
pub mod vec2;
pub mod trig;
pub mod rng;
pub mod state_hash;
pub mod snapshot;

pub use crate::fixed::Fixed32;
pub use crate::vec2::Vec2;
pub use crate::trig::Angle;
pub use crate::rng::SimRng;
