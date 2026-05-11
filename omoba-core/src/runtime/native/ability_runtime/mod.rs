//! Ability runtime framework.
//!
//! - `registry`: `AbilityRegistry` stores ability metadata collected from DLL scripts.
//! - `buff_store`: `BuffStore` stores and ticks unified buff state.

pub mod buff_store;
pub mod registry;
pub mod unit_stats;

pub use buff_store::{BuffEntry, BuffStore};
pub use registry::AbilityRegistry;
pub use unit_stats::{armor_to_mult, UnitStats};
