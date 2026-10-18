//! Authoritative placed trinket. This is not a hero, creep, tower, or unit.
//! Nearby search merges every `Unit` into the creep index, and the death
//! system pays bounties for `Unit` rows. A ward stays out of both.

use specs::{Component, VecStorage};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct PlacedWard {
    pub owner_player_id: u32,
    pub catalog_id: u16,
    pub placement_tick: u64,
    pub order: u64,
    pub expires_at_raw: i64,
}

/// Filtered reconstruction of a normally disclosed ward body.
/// Lifetime, charges, and authoritative coordinates are not stored here.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
#[storage(VecStorage)]
pub struct DisclosedWardBody {
    pub catalog_id: u16,
    pub generation: u32,
    pub direct_attack: u8,
    pub automatic_acquisition: u8,
}
