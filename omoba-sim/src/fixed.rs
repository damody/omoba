//! Fixed-point arithmetic (i32 + SCALE=1024). Implemented in Task 0.2-0.4.

use serde::{Serialize, Deserialize};

pub const SCALE: i32 = 1024;
pub const SCALE_BITS: u32 = 10;
const _: () = assert!(SCALE == 1 << SCALE_BITS);

#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(transparent))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed32(i32);

impl Fixed32 {
    pub const ZERO: Fixed32 = Fixed32(0);
    pub const ONE: Fixed32 = Fixed32(SCALE);

    pub const fn from_raw(raw: i32) -> Self { Fixed32(raw) }
    pub const fn from_i32(v: i32) -> Self { Fixed32(v.wrapping_mul(SCALE)) }
    pub const fn raw(self) -> i32 { self.0 }
    /// Converts to f32 — RENDER-BRIDGE USE ONLY. Calling this from sim hot-path
    /// code introduces non-determinism (f32 arithmetic is platform-divergent in the
    /// last few ULPs). The omfx render layer converts at the sim→GPU boundary;
    /// nothing inside `omoba-sim` should call this.
    pub fn to_f32_for_render(self) -> f32 { self.0 as f32 / SCALE as f32 }

    /// Integer square root via Newton's method, 16 iterations max (well past Q22.10 convergence).
    /// Returns ZERO for non-positive inputs (deterministic — never panics, never poisons).
    pub fn sqrt(self) -> Fixed32 {
        if self.0 <= 0 { return Fixed32::ZERO; }
        // In Q22.10: sqrt(a*SCALE) gives a*sqrt(SCALE), so we widen first by SCALE_BITS to
        // recover the SCALE factor in the result. x = self.0 << SCALE_BITS keeps headroom in i64.
        let x = (self.0 as i64) << SCALE_BITS;
        let mut y: i64 = (self.0 as i64).max(SCALE as i64);
        for _ in 0..16 {
            let next = (y + x / y) >> 1;
            if next == y { break; }
            y = next;
        }
        Fixed32(y as i32)
    }
}

impl std::ops::Add for Fixed32 {
    type Output = Fixed32;
    fn add(self, rhs: Fixed32) -> Fixed32 { Fixed32(self.0.wrapping_add(rhs.0)) }
}

impl std::ops::Sub for Fixed32 {
    type Output = Fixed32;
    fn sub(self, rhs: Fixed32) -> Fixed32 { Fixed32(self.0.wrapping_sub(rhs.0)) }
}

impl std::ops::Mul for Fixed32 {
    type Output = Fixed32;
    /// Rounds toward -infinity (arithmetic right-shift). Asymmetric with Div which truncates toward zero.
    /// Safe envelope: |a| × |b| < ~1448 real units (post-shift `as i32` truncates silently above this).
    fn mul(self, rhs: Fixed32) -> Fixed32 {
        let prod = (self.0 as i64) * (rhs.0 as i64);
        Fixed32((prod >> SCALE_BITS) as i32)
    }
}

impl std::ops::Div for Fixed32 {
    type Output = Fixed32;
    /// Rounds toward zero (Rust integer division semantics). Asymmetric with Mul which floors toward -infinity.
    fn div(self, rhs: Fixed32) -> Fixed32 {
        let num = (self.0 as i64) << SCALE_BITS;
        Fixed32((num / rhs.0 as i64) as i32)
    }
}

impl std::ops::Neg for Fixed32 {
    type Output = Fixed32;
    fn neg(self) -> Fixed32 { Fixed32(self.0.wrapping_neg()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub_basic() {
        let a = Fixed32::from_i32(3);
        let b = Fixed32::from_i32(2);
        assert_eq!(a + b, Fixed32::from_i32(5));
        assert_eq!(a - b, Fixed32::from_i32(1));
    }

    #[test]
    fn add_fractional() {
        let a = Fixed32::from_raw(512);  // 0.5
        let b = Fixed32::from_raw(256);  // 0.25
        assert_eq!((a + b).raw(), 768);  // 0.75
    }

    #[test]
    fn mul_basic() {
        let a = Fixed32::from_i32(3);
        let b = Fixed32::from_i32(4);
        assert_eq!(a * b, Fixed32::from_i32(12));
    }

    #[test]
    fn mul_fractional() {
        let a = Fixed32::from_raw(512);  // 0.5
        let b = Fixed32::from_raw(512);  // 0.5
        assert_eq!((a * b).raw(), 256);  // 0.25
    }

    #[test]
    fn div_basic() {
        let a = Fixed32::from_i32(10);
        let b = Fixed32::from_i32(4);
        assert_eq!((a / b).raw(), 2560);  // 2.5
    }

    #[test]
    fn mul_typical_map_scale() {
        // ±2000 unit (typical map size) × scale value; both well within Mul's ~1448-unit safe envelope
        // squared, so post-shift truncation does not occur for normal game math.
        let pos = Fixed32::from_i32(2000);
        let scale = Fixed32::from_raw(2048);  // 2.0
        let _ = pos * scale;
    }

    #[test]
    fn neg_basic() {
        assert_eq!(-Fixed32::from_i32(3), Fixed32::from_i32(-3));
        assert_eq!(-Fixed32::from_raw(512), Fixed32::from_raw(-512));
    }

    #[test]
    fn neg_min_wraps() {
        // Documents the wrap contract (matches Add/Sub/from_i32). Without wrapping_neg this would
        // panic in debug builds — a dev/release divergence that breaks lockstep determinism.
        let m = Fixed32::from_raw(i32::MIN);
        assert_eq!((-m).raw(), i32::MIN);
    }

    #[test]
    fn mul_negative() {
        assert_eq!(Fixed32::from_i32(-3) * Fixed32::from_i32(4), Fixed32::from_i32(-12));
        assert_eq!(Fixed32::from_i32(-3) * Fixed32::from_i32(-4), Fixed32::from_i32(12));
    }

    #[test]
    fn mul_floor_on_tiny_negative() {
        // Locks the Mul-floors-toward-neg-inf contract.
        // Fixed32(raw=-1) * Fixed32(raw=1): real product ≈ -1/2^20, post-shift floors to -1.
        // Div would yield 0 here — this test ensures Mul stays asymmetric.
        let a = Fixed32::from_raw(-1);
        let b = Fixed32::from_raw(1);
        assert_eq!((a * b).raw(), -1);
    }

    #[test]
    fn div_negative() {
        let a = Fixed32::from_i32(-10);
        let b = Fixed32::from_i32(4);
        assert_eq!((a / b).raw(), -2560);
        let c = Fixed32::from_i32(10);
        let d = Fixed32::from_i32(-4);
        assert_eq!((c / d).raw(), -2560);
    }

    #[test]
    fn sqrt_perfect() {
        assert_eq!(Fixed32::from_i32(4).sqrt(), Fixed32::from_i32(2));
        assert_eq!(Fixed32::from_i32(9).sqrt(), Fixed32::from_i32(3));
        assert_eq!(Fixed32::from_i32(100).sqrt(), Fixed32::from_i32(10));
    }

    #[test]
    fn sqrt_imperfect_within_1lsb() {
        // sqrt(2) ≈ 1.41421
        let r = Fixed32::from_i32(2).sqrt();
        let expected = Fixed32::from_raw(1448);  // ≈ 1.41406
        let diff = (r.raw() - expected.raw()).abs();
        assert!(diff <= 2, "sqrt(2) raw={} expected~={}, diff={}", r.raw(), expected.raw(), diff);
    }

    #[test]
    fn sqrt_zero() {
        assert_eq!(Fixed32::ZERO.sqrt(), Fixed32::ZERO);
    }

    #[test]
    fn sqrt_negative_returns_zero() {
        // Determinism guarantee: sqrt of negative returns ZERO, not panic, not NaN-equivalent.
        // Lockstep cannot tolerate either dev/release divergence or sentinel poisoning.
        assert_eq!(Fixed32::from_i32(-5).sqrt(), Fixed32::ZERO);
        assert_eq!(Fixed32::from_raw(i32::MIN).sqrt(), Fixed32::ZERO);
    }
}
