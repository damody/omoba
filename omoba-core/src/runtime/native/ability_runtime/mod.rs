//! Ability runtime framework.
//!
//! - `registry`: `AbilityRegistry` stores ability metadata collected from DLL scripts.
//! - `buff_store`: `BuffStore` stores and ticks unified buff state.

pub mod buff_store;
pub mod mana_pool;
pub mod mana_cost;
pub mod mana_projection;
pub mod registry;
pub mod unit_stats;

pub use buff_store::{BuffEntry, BuffStore, ItemTimedModifier};
pub use mana_pool::{ManaPool, ManaPoolError};
pub use mana_cost::checked_mana_cost;
pub use mana_projection::CommittedManaState;
pub use registry::AbilityRegistry;
pub use unit_stats::{armor_to_mult, UnitStats};
