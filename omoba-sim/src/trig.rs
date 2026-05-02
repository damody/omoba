//! Deterministic sin/cos via 4096-tick LUT. Implemented in Task 0.6.

use crate::fixed::{Fixed32, SCALE};
use serde::{Serialize, Deserialize};

/// Number of discrete angle ticks in a full turn (2π). Each tick = ~0.0879°.
pub const TAU_TICKS: i32 = 4096;

/// Angle in fixed ticks, modulo TAU_TICKS. 0 = 0°, TAU_TICKS/4 = 90°, etc.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Angle(i32);

impl Angle {
    pub const ZERO: Angle = Angle(0);
    pub const QUARTER_TURN: Angle = Angle(TAU_TICKS / 4);
    pub const HALF_TURN: Angle = Angle(TAU_TICKS / 2);
    pub const THREE_QUARTER_TURN: Angle = Angle(3 * TAU_TICKS / 4);

    /// Constructs an angle from raw ticks. Reduces modulo TAU_TICKS to canonical [0, TAU_TICKS).
    pub fn from_ticks(t: i32) -> Self { Angle(t.rem_euclid(TAU_TICKS)) }

    /// Constructs from integer degrees. 360° → TAU_TICKS via i64 intermediate to avoid overflow.
    pub fn from_degrees_i32(d: i32) -> Self {
        Angle::from_ticks(((d as i64 * TAU_TICKS as i64) / 360) as i32)
    }

    /// Raw tick count, in [0, TAU_TICKS).
    pub fn ticks(self) -> i32 { self.0 }
}

// LUT generated at first call. Uses f64 with `round()` for cross-platform reproducibility:
// IEEE-754 f64 sin() agrees bit-for-bit on all major platforms (the variation is in trig
// intrinsics' last bits which `round()` discards).
static SIN_LUT: once_cell::sync::Lazy<[i32; TAU_TICKS as usize]> = once_cell::sync::Lazy::new(|| {
    let mut lut = [0i32; TAU_TICKS as usize];
    for i in 0..TAU_TICKS {
        let rad = (i as f64) * std::f64::consts::TAU / TAU_TICKS as f64;
        lut[i as usize] = (rad.sin() * SCALE as f64).round() as i32;
    }
    lut
});

pub fn sin(a: Angle) -> Fixed32 {
    Fixed32::from_raw(SIN_LUT[a.0 as usize])
}

pub fn cos(a: Angle) -> Fixed32 {
    let i = (a.0 + TAU_TICKS / 4).rem_euclid(TAU_TICKS);
    Fixed32::from_raw(SIN_LUT[i as usize])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed::Fixed32;

    #[test]
    fn sin_known_values() {
        assert_eq!(sin(Angle::ZERO), Fixed32::ZERO);
        assert_eq!(sin(Angle::QUARTER_TURN), Fixed32::ONE);
        assert_eq!(sin(Angle::HALF_TURN), Fixed32::ZERO);
    }

    #[test]
    fn cos_known_values() {
        assert_eq!(cos(Angle::ZERO), Fixed32::ONE);
        assert_eq!(cos(Angle::QUARTER_TURN), Fixed32::ZERO);
        assert_eq!(cos(Angle::HALF_TURN), -Fixed32::ONE);
    }

    #[test]
    fn sin_30deg_within_2lsb() {
        // 30° = π/6, sin = 0.5
        let a = Angle::from_degrees_i32(30);
        let s = sin(a);
        let expected = Fixed32::from_raw(512);  // 0.5
        assert!((s.raw() - expected.raw()).abs() <= 2);
    }
}
