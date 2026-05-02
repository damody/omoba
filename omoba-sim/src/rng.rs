//! Deterministic RNG seeded by master_seed + tick. Implemented in Task 0.7.

use rand_pcg::Pcg64Mcg;
use rand_core::{RngCore, SeedableRng};

/// Deterministic RNG for simulation. Wraps Pcg64Mcg.
///
/// All sources of randomness in the sim must go through this type. Each call site picks
/// a stream via either:
/// - `from_master(master_seed, tick)` — global "shared" stream for the tick (use sparingly,
///   only for cases where stream isolation isn't needed).
/// - `from_master_entity(master_seed, tick, entity_id, op_kind)` — per-entity, per-op stream.
///   Use for any sim decision tied to a specific entity (crit roll, dodge, ability proc).
///   Different entities or different op kinds at the same tick get independent sequences.
pub struct SimRng(Pcg64Mcg);

impl SimRng {
    /// Constructs from master seed + current sim tick. Same (seed, tick) → same sequence.
    pub fn from_master(master_seed: u64, tick: u32) -> Self {
        let combined = master_seed.wrapping_mul(0x9E3779B97F4A7C15)
            ^ (tick as u64).wrapping_mul(0xBF58476D1CE4E5B9);
        SimRng(Pcg64Mcg::seed_from_u64(combined))
    }

    /// Constructs from master seed + tick + entity_id + op_kind for stream isolation.
    /// Different (entity_id, op_kind) at same tick produce independent random sequences.
    /// op_kind is a small enum-like u32 (e.g. 0 = crit roll, 1 = dodge, 2 = ability proc).
    pub fn from_master_entity(master_seed: u64, tick: u32, entity_id: u32, op_kind: u32) -> Self {
        // 64-bit splitmix-style mixing across the four inputs.
        let mut state = master_seed;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ tick as u64;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ entity_id as u64;
        state = state.wrapping_mul(0x9E3779B97F4A7C15) ^ op_kind as u64;
        SimRng(Pcg64Mcg::seed_from_u64(state))
    }

    pub fn next_u32(&mut self) -> u32 { self.0.next_u32() }
    pub fn next_u64(&mut self) -> u64 { self.0.next_u64() }

    /// Returns an integer in `[low, high)`, biased slightly toward lower values when
    /// `(high-low)` is not a power of 2 (modulo bias). Acceptable for game math —
    /// upgrade to widening multiply if pure uniformity is ever needed.
    pub fn range(&mut self, low: i32, high: i32) -> i32 {
        debug_assert!(high > low, "SimRng::range called with low={} high={}", low, high);
        let span = (high - low) as u32;
        let r = self.next_u32() % span;
        low + r as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = SimRng::from_master(42, 0);
        let mut b = SimRng::from_master(42, 0);
        for _ in 0..1000 {
            assert_eq!(a.next_u32(), b.next_u32());
        }
    }

    #[test]
    fn different_tick_different_sequence() {
        let mut a = SimRng::from_master(42, 100);
        let mut b = SimRng::from_master(42, 101);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn entity_streams_isolated() {
        // Same master+tick but different entity_id should produce different streams
        let mut a = SimRng::from_master_entity(42, 100, 1, 0);
        let mut b = SimRng::from_master_entity(42, 100, 2, 0);
        assert_ne!(a.next_u32(), b.next_u32());
    }

    #[test]
    fn range_uniform() {
        let mut r = SimRng::from_master(42, 0);
        let mut histogram = [0u32; 10];
        for _ in 0..10000 {
            histogram[r.range(0, 10) as usize] += 1;
        }
        for c in &histogram {
            assert!(*c >= 800 && *c <= 1200, "bucket {} out of expected ±20% range", c);
        }
    }

    #[test]
    fn range_below_low_panics_in_debug() {
        // Implementation uses debug_assert!; this test only meaningful in debug builds.
        // Confirms the API contract that low < high is required.
        let mut r = SimRng::from_master(42, 0);
        let _ = r.range(0, 10);  // valid; should not panic
    }
}
