//! Deterministic sin/cos via 4096-tick LUT. Implemented in Task 0.6.

use crate::fixed::{Fixed32, SCALE};
use serde::{Serialize, Deserialize};

/// Number of discrete angle ticks in a full turn (2π). Each tick = ~0.0879°.
pub const TAU_TICKS: i32 = 4096;

/// Angle in fixed ticks, modulo TAU_TICKS. 0 = 0°, TAU_TICKS/4 = 90°, etc.
#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(transparent))]
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

/// 4096-entry lookup table for `atan(t)` where `t = y/x` in real units, t ∈ [0, 1].
/// Generated lazily from f64::atan() with cross-platform-stable rounding to ticks.
/// Output is in TAU_TICKS units (0..=TAU_TICKS/8 == [0, π/4]).
static ATAN_LUT: once_cell::sync::Lazy<[i32; 1024]> = once_cell::sync::Lazy::new(|| {
    let mut lut = [0i32; 1024];
    for i in 0..1024 {
        let t = i as f64 / 1024.0;  // t in [0, 1)
        // atan(t) in radians; convert to ticks: ticks = (atan / 2pi) * TAU_TICKS
        let radians = t.atan();
        let ticks = (radians / std::f64::consts::TAU * TAU_TICKS as f64).round() as i32;
        lut[i] = ticks;
    }
    lut
});

/// Two-argument arctangent. Returns `Angle` in `[0, TAU_TICKS)` with 0 at +x axis,
/// rotation counter-clockwise (so π/2 = +y axis).
///
/// Uses octant decomposition: reduces to atan(t) for t ∈ [0, 1] in the first
/// octant (where y ≤ x, both ≥ 0), then maps via symmetry to the correct octant.
/// Lookup is via a 1024-entry LUT keyed on `(t * 1024) as i32`. Determinism comes
/// from i32 arithmetic + spec-defined integer division throughout.
///
/// Special cases:
/// - `atan2(0, 0) = 0` (no panic, no NaN-equivalent — lockstep cannot tolerate either)
/// - Cardinal axes hit table boundaries exactly (atan2(0, +1) = 0, atan2(+1, 0) = π/2)
pub fn atan2(y: Fixed32, x: Fixed32) -> Angle {
    let yr = y.raw();
    let xr = x.raw();
    if yr == 0 && xr == 0 {
        return Angle::ZERO;
    }

    // Compute |y|, |x| as i64 to avoid overflow when one is i32::MIN
    let ay = (yr as i64).unsigned_abs() as i64;
    let ax = (xr as i64).unsigned_abs() as i64;

    // Reduce to first octant: t = min/max in [0, 1]
    let (t_num, t_den, swap_xy) = if ay <= ax {
        (ay, ax, false)  // |y| ≤ |x| — already in first octant of upper-right
    } else {
        (ax, ay, true)   // |y| > |x| — swap so we look up atan(x/y) then complement
    };

    // LUT index in [0, 1024): (t_num * 1024) / t_den. t_den != 0 here (else
    // both raw zero, handled above; or one is zero and that case has |·| ≤ |·|
    // making t_num = 0 → idx = 0 → atan(0) = 0).
    let idx = ((t_num * 1024) / t_den).min(1023) as usize;
    let mut ticks = ATAN_LUT[idx];

    if swap_xy {
        // atan(x/y) instead of atan(y/x): result is π/2 - atan(y/x), so ticks = TAU/4 - ticks
        ticks = TAU_TICKS / 4 - ticks;
    }

    // Map to correct octant using sign of x and y. Reference: standard atan2 quadrant fix.
    let final_ticks = match (xr >= 0, yr >= 0) {
        (true, true)   => ticks,                            // Q1 (+x, +y)
        (false, true)  => TAU_TICKS / 2 - ticks,            // Q2 (-x, +y)
        (false, false) => TAU_TICKS / 2 + ticks,            // Q3 (-x, -y)
        (true, false)  => TAU_TICKS - ticks,                // Q4 (+x, -y)
    };

    Angle::from_ticks(final_ticks)
}

/// Rotate `current` toward `target` by at most `max_step_ticks` ticks.
/// Picks the shorter direction (clockwise vs counter-clockwise).
///
/// Examples:
/// - current=10, target=20, max=5 → returns 15 (3-tick step toward target along shorter arc)
/// - current=20, target=10, max=5 → returns 15 (similar, opposite direction)
/// - current=0, target=TAU/2, max=TAU/4 → returns TAU/4 (steps half-way; tie-broken by sign of signed_diff)
/// - current=0, target=TAU/4, max=10000 → returns TAU/4 (overshoot clamped)
///
/// Deterministic across platforms (i32 arithmetic + rem_euclid).
pub fn angle_rotate_toward(current: Angle, target: Angle, max_step_ticks: i32) -> Angle {
    let diff = (target.ticks() - current.ticks()).rem_euclid(TAU_TICKS);
    // Convert to signed distance in (-TAU/2, TAU/2]
    let signed_diff = if diff > TAU_TICKS / 2 { diff - TAU_TICKS } else { diff };
    let clamped = signed_diff.clamp(-max_step_ticks.abs(), max_step_ticks.abs());
    Angle::from_ticks(current.ticks() + clamped)
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

    #[test]
    fn atan2_zero_zero() {
        assert_eq!(atan2(Fixed32::ZERO, Fixed32::ZERO), Angle::ZERO);
    }

    #[test]
    fn atan2_cardinal_axes() {
        // +x axis: atan2(0, 1) = 0
        assert_eq!(atan2(Fixed32::ZERO, Fixed32::ONE), Angle::ZERO);
        // +y axis: atan2(1, 0) = π/2 = TAU_TICKS/4
        assert_eq!(atan2(Fixed32::ONE, Fixed32::ZERO).ticks(), TAU_TICKS / 4);
        // -x axis: atan2(0, -1) = π = TAU_TICKS/2
        assert_eq!(atan2(Fixed32::ZERO, -Fixed32::ONE).ticks(), TAU_TICKS / 2);
        // -y axis: atan2(-1, 0) = 3π/2 = 3*TAU_TICKS/4
        assert_eq!(atan2(-Fixed32::ONE, Fixed32::ZERO).ticks(), 3 * TAU_TICKS / 4);
    }

    #[test]
    fn atan2_45deg_diagonals() {
        // First-quadrant diagonal: atan2(1, 1) = π/4 = TAU_TICKS/8 = 512
        let a = atan2(Fixed32::ONE, Fixed32::ONE);
        assert!((a.ticks() - TAU_TICKS / 8).abs() <= 1, "Q1 diagonal: got {}, expected ~512", a.ticks());

        // Q2 diagonal: atan2(1, -1) = 3π/4 = 3*TAU_TICKS/8 = 1536
        let b = atan2(Fixed32::ONE, -Fixed32::ONE);
        assert!((b.ticks() - 3 * TAU_TICKS / 8).abs() <= 1, "Q2 diagonal: got {}, expected ~1536", b.ticks());

        // Q3 diagonal: atan2(-1, -1) = 5π/4 = 5*TAU_TICKS/8 = 2560
        let c = atan2(-Fixed32::ONE, -Fixed32::ONE);
        assert!((c.ticks() - 5 * TAU_TICKS / 8).abs() <= 1, "Q3 diagonal: got {}, expected ~2560", c.ticks());

        // Q4 diagonal: atan2(-1, 1) = 7π/4 = 7*TAU_TICKS/8 = 3584
        let d = atan2(-Fixed32::ONE, Fixed32::ONE);
        assert!((d.ticks() - 7 * TAU_TICKS / 8).abs() <= 1, "Q4 diagonal: got {}, expected ~3584", d.ticks());
    }

    #[test]
    fn atan2_30deg() {
        // atan2(1, sqrt(3)) ≈ 30° = TAU_TICKS * 30/360 = ~341 ticks
        // Approximate sqrt(3) as Fixed32(1773) (raw, ≈ 1.732)
        let y = Fixed32::ONE;
        let x = Fixed32::from_raw(1773);
        let a = atan2(y, x);
        let expected = Angle::from_degrees_i32(30).ticks();
        let diff = (a.ticks() - expected).abs();
        assert!(diff <= 4, "atan2(1, √3) ticks={} expected={} diff={}", a.ticks(), expected, diff);
    }

    #[test]
    fn angle_rotate_toward_overshoot_clamped_to_target() {
        // 0 → TAU/8, with max=TAU (huge), should reach exactly TAU/8
        let r = angle_rotate_toward(Angle::ZERO, Angle::from_ticks(TAU_TICKS / 8), TAU_TICKS);
        assert_eq!(r.ticks(), TAU_TICKS / 8);
    }

    #[test]
    fn angle_rotate_toward_step_partial_forward() {
        let r = angle_rotate_toward(Angle::from_ticks(10), Angle::from_ticks(20), 3);
        assert_eq!(r.ticks(), 13);
    }

    #[test]
    fn angle_rotate_toward_step_partial_backward() {
        let r = angle_rotate_toward(Angle::from_ticks(20), Angle::from_ticks(10), 3);
        assert_eq!(r.ticks(), 17);
    }

    #[test]
    fn angle_rotate_toward_shortest_path_wraparound() {
        // Going from near-end-of-cycle to near-start: should wrap forward, not backward
        // current = TAU - 5, target = 5, max = 20 → should reach target = 5 directly via short arc
        let r = angle_rotate_toward(
            Angle::from_ticks(TAU_TICKS - 5),
            Angle::from_ticks(5),
            20,
        );
        assert_eq!(r.ticks(), 5);
    }

    #[test]
    fn angle_rotate_toward_equal_unchanged() {
        let r = angle_rotate_toward(Angle::from_ticks(123), Angle::from_ticks(123), 10);
        assert_eq!(r.ticks(), 123);
    }
}
