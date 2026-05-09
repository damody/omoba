#![forbid(unsafe_code)]

//! omoba-sim: deterministic ECS simulation crate
//! Shared between server (omb) and client (omfx) for lockstep networking.

pub mod fixed;
pub mod rng;
pub mod snapshot;
pub mod state_hash;
pub mod trig;
pub mod vec2;

pub use crate::fixed::Fixed64;
pub use crate::rng::SimRng;
pub use crate::trig::Angle;
pub use crate::vec2::Vec2;
