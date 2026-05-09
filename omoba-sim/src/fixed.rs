//! Fixed-point arithmetic (i64 backing, SCALE=1024 → Q53.10 effective).
//!
//! Type name is kept as `Fixed64` for backwards compat with existing code; the
//! "32" no longer reflects the storage width. Backing was widened from i32 to
//! i64 to fix `Vec2::length_squared` overflow when entity diffs exceed
//! ~1448 units (e.g. TD_1 has 2800-unit segments — 2800² = 7.84M, which can't
//! fit in the previous Q22.10 representation).
//!
//! Mul/Div/sqrt use i128 intermediates so the safe envelope of `|a| × |b|`
//! grows from ~2.1M (i32 result raw) to ~9e15 (i64 result raw) — well above
//! anything game math throws at it.

use serde::{Deserialize, Serialize};

pub const SCALE: i64 = 1024;
pub const SCALE_BITS: u32 = 10;
const _: () = assert!(SCALE == 1 << SCALE_BITS);

#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(transparent))]
#[derive(
    Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Fixed64(i64);

impl Fixed64 {
    pub const ZERO: Fixed64 = Fixed64(0);
    pub const ONE: Fixed64 = Fixed64(SCALE);

    pub const fn from_raw(raw: i64) -> Self {
        Fixed64(raw)
    }
    pub const fn from_i32(v: i32) -> Self {
        Fixed64((v as i64).wrapping_mul(SCALE))
    }
    pub const fn raw(self) -> i64 {
        self.0
    }
    /// Converts to f32 — RENDER-BRIDGE USE ONLY. Calling this from sim hot-path
    /// code introduces non-determinism (f32 arithmetic is platform-divergent in the
    /// last few ULPs). The omfx render layer converts at the sim→GPU boundary;
    /// nothing inside `omoba-sim` should call this.
    pub fn to_f32_for_render(self) -> f32 {
        self.0 as f32 / SCALE as f32
    }

    /// Integer square root via Newton's method, 32 iterations max (well past Q53.10 convergence).
    /// Returns ZERO for non-positive inputs (deterministic — never panics, never poisons).
    /// Uses i128 intermediates so that `self.0 << SCALE_BITS` cannot overflow when
    /// `self.0` is near i64::MAX.
    pub fn sqrt(self) -> Fixed64 {
        if self.0 <= 0 {
            return Fixed64::ZERO;
        }
        // In Q.10: sqrt(a*SCALE) gives a*sqrt(SCALE), so we widen first by SCALE_BITS to
        // recover the SCALE factor in the result. x = self.0 << SCALE_BITS keeps headroom in i128.
        let x: i128 = (self.0 as i128) << SCALE_BITS;
        let mut y: i128 = (self.0 as i128).max(SCALE as i128);
        for _ in 0..32 {
            let next = (y + x / y) >> 1;
            if next == y {
                break;
            }
            y = next;
        }
        Fixed64(y as i64)
    }
}

impl std::ops::Add for Fixed64 {
    type Output = Fixed64;
    fn add(self, rhs: Fixed64) -> Fixed64 {
        Fixed64(self.0.wrapping_add(rhs.0))
    }
}

impl std::ops::Sub for Fixed64 {
    type Output = Fixed64;
    fn sub(self, rhs: Fixed64) -> Fixed64 {
        Fixed64(self.0.wrapping_sub(rhs.0))
    }
}

impl std::ops::Mul for Fixed64 {
    type Output = Fixed64;
    /// Rounds toward -infinity (arithmetic right-shift). Asymmetric with Div which truncates toward zero.
    /// Safe envelope: |a| × |b| < ~9e15 real units (post-shift `as i64` truncates silently above this) —
    /// effectively unbounded for game math.
    fn mul(self, rhs: Fixed64) -> Fixed64 {
        let prod = (self.0 as i128) * (rhs.0 as i128);
        Fixed64((prod >> SCALE_BITS) as i64)
    }
}

impl std::ops::Mul<i32> for Fixed64 {
    type Output = Fixed64;
    /// Multiply Fixed64 by integer scalar. Equivalent to `self * Fixed64::from_i32(rhs)`
    /// but cheaper (no widening; just `raw.wrapping_mul(rhs as i64)`).
    fn mul(self, rhs: i32) -> Fixed64 {
        Fixed64(self.0.wrapping_mul(rhs as i64))
    }
}

impl std::ops::Mul<Fixed64> for i32 {
    type Output = Fixed64;
    fn mul(self, rhs: Fixed64) -> Fixed64 {
        Fixed64((self as i64).wrapping_mul(rhs.0))
    }
}

impl std::ops::Div for Fixed64 {
    type Output = Fixed64;
    /// Rounds toward zero (Rust integer division semantics). Asymmetric with Mul which floors toward -infinity.
    /// Uses i128 intermediate so `self.0 << SCALE_BITS` cannot overflow when `self.0` is large.
    fn div(self, rhs: Fixed64) -> Fixed64 {
        let num: i128 = (self.0 as i128) << SCALE_BITS;
        Fixed64((num / rhs.0 as i128) as i64)
    }
}

impl std::ops::Neg for Fixed64 {
    type Output = Fixed64;
    fn neg(self) -> Fixed64 {
        Fixed64(self.0.wrapping_neg())
    }
}

impl std::ops::AddAssign for Fixed64 {
    fn add_assign(&mut self, rhs: Fixed64) {
        self.0 = self.0.wrapping_add(rhs.0);
    }
}

impl std::ops::SubAssign for Fixed64 {
    fn sub_assign(&mut self, rhs: Fixed64) {
        self.0 = self.0.wrapping_sub(rhs.0);
    }
}

impl std::ops::MulAssign for Fixed64 {
    fn mul_assign(&mut self, rhs: Fixed64) {
        *self = *self * rhs;
    }
}

impl std::ops::DivAssign for Fixed64 {
    fn div_assign(&mut self, rhs: Fixed64) {
        *self = *self / rhs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_basic() {
        let a = Fixed64::from_i32(3);
        let b = Fixed64::from_i32(2);
        assert_eq!(a + b, Fixed64::from_i32(5));
        assert_eq!(a - b, Fixed64::from_i32(1));
    }

    #[test]
    fn add_fractional() {
        let a = Fixed64::from_raw(512); // 0.5
        let b = Fixed64::from_raw(256); // 0.25
        assert_eq!((a + b).raw(), 768); // 0.75
    }

    #[test]
    fn mul_basic() {
        let a = Fixed64::from_i32(3);
        let b = Fixed64::from_i32(4);
        assert_eq!(a * b, Fixed64::from_i32(12));
    }

    #[test]
    fn mul_fractional() {
        let a = Fixed64::from_raw(512); // 0.5
        let b = Fixed64::from_raw(512); // 0.5
        assert_eq!((a * b).raw(), 256); // 0.25
    }

    #[test]
    fn div_basic() {
        let a = Fixed64::from_i32(10);
        let b = Fixed64::from_i32(4);
        assert_eq!((a / b).raw(), 2560); // 2.5
    }

    #[test]
    fn mul_typical_map_scale() {
        // ±2000 unit (typical map size) × scale value; both well within Mul's safe envelope.
        let pos = Fixed64::from_i32(2000);
        let scale = Fixed64::from_raw(2048); // 2.0
        let _ = pos * scale;
    }

    #[test]
    fn mul_large_position_squared() {
        // Regression: previous i32 backing overflowed when squaring values > ~1448, causing
        // Vec2::length_squared to wrap to negative for diffs across the TD_1 map (2800 units),
        // making creeps insta-arrive at every waypoint and walk straight to the endpoint.
        // With i64 backing + i128 intermediate, 2800² = 7,840,000 must round-trip cleanly.
        let v = Fixed64::from_i32(2800);
        let sq = v * v;
        assert_eq!(sq, Fixed64::from_i32(2800 * 2800), "got raw={}", sq.raw());
        assert!(
            sq.raw() > 0,
            "squared value went negative — overflow regression"
        );
    }

    #[test]
    fn neg_basic() {
        assert_eq!(-Fixed64::from_i32(3), Fixed64::from_i32(-3));
        assert_eq!(-Fixed64::from_raw(512), Fixed64::from_raw(-512));
    }

    #[test]
    fn neg_min_wraps() {
        // Documents the wrap contract (matches Add/Sub/from_i32). Without wrapping_neg this would
        // panic in debug builds — a dev/release divergence that breaks lockstep determinism.
        let m = Fixed64::from_raw(i64::MIN);
        assert_eq!((-m).raw(), i64::MIN);
    }

    #[test]
    fn mul_negative() {
        assert_eq!(
            Fixed64::from_i32(-3) * Fixed64::from_i32(4),
            Fixed64::from_i32(-12)
        );
        assert_eq!(
            Fixed64::from_i32(-3) * Fixed64::from_i32(-4),
            Fixed64::from_i32(12)
        );
    }

    #[test]
    fn mul_floor_on_tiny_negative() {
        // Locks the Mul-floors-toward-neg-inf contract.
        // Fixed64(raw=-1) * Fixed64(raw=1): real product ≈ -1/2^20, post-shift floors to -1.
        // Div would yield 0 here — this test ensures Mul stays asymmetric.
        let a = Fixed64::from_raw(-1);
        let b = Fixed64::from_raw(1);
        assert_eq!((a * b).raw(), -1);
    }

    #[test]
    fn div_negative() {
        let a = Fixed64::from_i32(-10);
        let b = Fixed64::from_i32(4);
        assert_eq!((a / b).raw(), -2560);
        let c = Fixed64::from_i32(10);
        let d = Fixed64::from_i32(-4);
        assert_eq!((c / d).raw(), -2560);
    }

    #[test]
    fn sqrt_perfect() {
        assert_eq!(Fixed64::from_i32(4).sqrt(), Fixed64::from_i32(2));
        assert_eq!(Fixed64::from_i32(9).sqrt(), Fixed64::from_i32(3));
        assert_eq!(Fixed64::from_i32(100).sqrt(), Fixed64::from_i32(10));
    }

    #[test]
    fn sqrt_imperfect_within_1lsb() {
        // sqrt(2) ≈ 1.41421
        let r = Fixed64::from_i32(2).sqrt();
        let expected = Fixed64::from_raw(1448); // ≈ 1.41406
        let diff = (r.raw() - expected.raw()).abs();
        assert!(
            diff <= 2,
            "sqrt(2) raw={} expected~={}, diff={}",
            r.raw(),
            expected.raw(),
            diff
        );
    }

    #[test]
    fn sqrt_zero() {
        assert_eq!(Fixed64::ZERO.sqrt(), Fixed64::ZERO);
    }

    #[test]
    fn sqrt_large_value() {
        // 2800² = 7,840,000; sqrt should recover 2800.
        let big = Fixed64::from_i32(2800 * 2800);
        let r = big.sqrt();
        let diff = (r.raw() - Fixed64::from_i32(2800).raw()).abs();
        assert!(
            diff <= 2,
            "sqrt(7,840,000) raw={} expected~={}, diff={}",
            r.raw(),
            Fixed64::from_i32(2800).raw(),
            diff
        );
    }

    #[test]
    fn add_assign_basic() {
        let mut a = Fixed64::from_i32(3);
        a += Fixed64::from_i32(2);
        assert_eq!(a, Fixed64::from_i32(5));
    }

    #[test]
    fn sub_assign_basic() {
        let mut a = Fixed64::from_i32(5);
        a -= Fixed64::from_i32(2);
        assert_eq!(a, Fixed64::from_i32(3));
    }

    #[test]
    fn sqrt_negative_returns_zero() {
        // Determinism guarantee: sqrt of negative returns ZERO, not panic, not NaN-equivalent.
        // Lockstep cannot tolerate either dev/release divergence or sentinel poisoning.
        assert_eq!(Fixed64::from_i32(-5).sqrt(), Fixed64::ZERO);
        assert_eq!(Fixed64::from_raw(i64::MIN).sqrt(), Fixed64::ZERO);
    }

    #[test]
    fn mul_by_i32_basic() {
        assert_eq!(Fixed64::from_raw(512) * 3, Fixed64::from_raw(1536));
        assert_eq!(2 * Fixed64::from_raw(512), Fixed64::from_raw(1024));
    }

    #[test]
    fn mul_assign_basic() {
        let mut a = Fixed64::from_i32(3);
        a *= Fixed64::from_i32(4);
        assert_eq!(a, Fixed64::from_i32(12));
    }

    #[test]
    fn div_assign_basic() {
        let mut a = Fixed64::from_i32(12);
        a /= Fixed64::from_i32(4);
        assert_eq!(a, Fixed64::from_i32(3));
    }
}
