#![allow(ambiguous_glob_reexports, dead_code, unused_variables)]

#[path = "../../../../../omb/src/tick/attack_phase.rs"]
pub mod attack_phase;
#[path = "../../../../../omb/src/tick/buff_tick.rs"]
pub mod buff_tick;
#[path = "../../../../../omb/src/tick/creep_tick.rs"]
pub mod creep_tick;
#[path = "../../../../../omb/src/tick/creep_wave.rs"]
pub mod creep_wave;
#[path = "../../../../../omb/src/tick/damage_tick.rs"]
pub mod damage_tick;
#[path = "../../../../../omb/src/tick/death_tick.rs"]
pub mod death_tick;
#[path = "../../../../../omb/src/tick/hero_move_tick.rs"]
pub mod hero_move_tick;
#[path = "../../../../../omb/src/tick/hero_tick.rs"]
pub mod hero_tick;
#[path = "../../../../../omb/src/tick/item_tick.rs"]
pub mod item_tick;
#[path = "../../../../../omb/src/tick/nearby_tick.rs"]
pub mod nearby_tick;
#[path = "../../../../../omb/src/tick/player_input_tick.rs"]
pub mod player_input_tick;
#[path = "../../../../../omb/src/tick/player_tick.rs"]
pub mod player_tick;
#[path = "../../../../../omb/src/tick/projectile_tick.rs"]
pub mod projectile_tick;
#[path = "../../../../../omb/src/tick/regen_tick.rs"]
pub mod regen_tick;
#[path = "../../../../../omb/src/tick/summon_tick.rs"]
pub mod summon_tick;
#[path = "../../../../../omb/src/tick/tower_tick.rs"]
pub mod tower_tick;

pub use self::{
    creep_tick::*, creep_wave::*, damage_tick::*, death_tick::*, hero_move_tick::*, hero_tick::*,
    item_tick::*, nearby_tick::*, player_tick::*, projectile_tick::*, tower_tick::*,
};
