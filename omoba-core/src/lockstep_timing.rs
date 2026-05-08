//! Shared lockstep timing constants used by omb and omfx.

/// Authoritative lockstep tick rate. A 2-tick input lookahead is ~16.7 ms.
pub const LOCKSTEP_TPS: u32 = 120;
pub const LOCKSTEP_TPS_U64: u64 = LOCKSTEP_TPS as u64;

/// Truncated microsecond period used by tokio intervals.
pub const LOCKSTEP_TICK_PERIOD_US: u64 = 1_000_000 / LOCKSTEP_TPS_U64;

pub const LOCKSTEP_DT_F32: f32 = 1.0 / LOCKSTEP_TPS as f32;
pub const LOCKSTEP_DT_F64: f64 = 1.0 / LOCKSTEP_TPS as f64;

pub const LOCKSTEP_ONE_SECOND_TICKS_U32: u32 = LOCKSTEP_TPS;
pub const LOCKSTEP_FIVE_SECONDS_TICKS_U32: u32 = LOCKSTEP_TPS * 5;
pub const LOCKSTEP_TEN_SECONDS_TICKS_U32: u32 = LOCKSTEP_TPS * 10;
pub const LOCKSTEP_TEN_SECONDS_TICKS_U64: u64 = LOCKSTEP_TPS_U64 * 10;
pub const LOCKSTEP_THIRTY_SECONDS_TICKS_U64: u64 = LOCKSTEP_TPS_U64 * 30;

pub fn ticks_to_seconds_f64(tick: u32) -> f64 {
    tick as f64 * LOCKSTEP_DT_F64
}
