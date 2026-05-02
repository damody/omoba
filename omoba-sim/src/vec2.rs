//! 2D vector of Fixed64 — Phase 1 ECS Position / Velocity / Direction substrate.
//!
//! Pure deterministic arithmetic. `length_squared` uses Fixed64 mul (rounds toward -inf);
//! `length` uses Fixed64::sqrt (returns ZERO for zero/negative). All ops are platform-invariant
//! by virtue of Fixed64's underlying contract.

use crate::fixed::Fixed64;
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(C))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Fixed64,
    pub y: Fixed64,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: Fixed64::ZERO, y: Fixed64::ZERO };

    pub const fn new(x: Fixed64, y: Fixed64) -> Self { Vec2 { x, y } }

    /// Dot product: x1*x2 + y1*y2. Useful for projection / facing checks.
    pub fn dot(self, other: Vec2) -> Fixed64 {
        self.x * other.x + self.y * other.y
    }

    /// Squared length. Cheaper than `length()` — prefer for distance comparisons.
    pub fn length_squared(self) -> Fixed64 {
        self.x * self.x + self.y * self.y
    }

    /// Euclidean length via Fixed64::sqrt. Returns ZERO for zero vector.
    pub fn length(self) -> Fixed64 {
        self.length_squared().sqrt()
    }

    /// Squared distance to `other`. Cheaper than `distance()` for comparisons —
    /// avoids the sqrt. Use for "is X within range Y" checks: compare against
    /// `range * range`.
    pub fn distance_squared(self, other: Vec2) -> Fixed64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Euclidean distance to `other`. Uses `Fixed64::sqrt`.
    pub fn distance(self, other: Vec2) -> Fixed64 {
        self.distance_squared(other).sqrt()
    }

    /// Returns this vector divided by its length (unit vector pointing same direction).
    /// Returns `Vec2::ZERO` if length is zero — deterministic, no NaN, matches the
    /// `Fixed64::sqrt(non-positive) → ZERO` contract used elsewhere in the sim.
    pub fn normalized(self) -> Vec2 {
        let len = self.length();
        if len == Fixed64::ZERO { return Vec2::ZERO; }
        Vec2 { x: self.x / len, y: self.y / len }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Vec2;
    fn add(self, rhs: Vec2) -> Vec2 { Vec2 { x: self.x + rhs.x, y: self.y + rhs.y } }
}

impl std::ops::Sub for Vec2 {
    type Output = Vec2;
    fn sub(self, rhs: Vec2) -> Vec2 { Vec2 { x: self.x - rhs.x, y: self.y - rhs.y } }
}

impl std::ops::Neg for Vec2 {
    type Output = Vec2;
    fn neg(self) -> Vec2 { Vec2 { x: -self.x, y: -self.y } }
}

impl std::ops::Mul<Fixed64> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: Fixed64) -> Vec2 { Vec2 { x: self.x * scalar, y: self.y * scalar } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        let b = Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(2));
        assert_eq!(a + b, Vec2::new(Fixed64::from_i32(4), Fixed64::from_i32(6)));
        assert_eq!(a - b, Vec2::new(Fixed64::from_i32(2), Fixed64::from_i32(2)));
    }

    #[test]
    fn dot_product() {
        let a = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        let b = Vec2::new(Fixed64::from_i32(2), Fixed64::from_i32(1));
        // 3*2 + 4*1 = 10
        assert_eq!(a.dot(b), Fixed64::from_i32(10));
    }

    #[test]
    fn length_3_4_5() {
        let v = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        assert_eq!(v.length_squared(), Fixed64::from_i32(25));
        // sqrt(25) = 5
        assert_eq!(v.length(), Fixed64::from_i32(5));
    }

    #[test]
    fn length_zero() {
        assert_eq!(Vec2::ZERO.length(), Fixed64::ZERO);
        assert_eq!(Vec2::ZERO.length_squared(), Fixed64::ZERO);
    }

    #[test]
    fn scalar_mul() {
        let v = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        let r = v * Fixed64::from_i32(2);
        assert_eq!(r, Vec2::new(Fixed64::from_i32(6), Fixed64::from_i32(8)));
    }

    #[test]
    fn normalized_unit_x() {
        let v = Vec2::new(Fixed64::from_i32(3), Fixed64::ZERO);
        let n = v.normalized();
        assert_eq!(n, Vec2::new(Fixed64::ONE, Fixed64::ZERO));
    }

    #[test]
    fn normalized_3_4_5() {
        // (3, 4) has length 5, so normalized = (0.6, 0.8) = raw (614, 819) ± 2 LSB
        let v = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        let n = v.normalized();
        assert!((n.x.raw() - 614).abs() <= 2, "n.x.raw() = {}", n.x.raw());
        assert!((n.y.raw() - 819).abs() <= 2, "n.y.raw() = {}", n.y.raw());
    }

    #[test]
    fn normalized_zero_returns_zero() {
        assert_eq!(Vec2::ZERO.normalized(), Vec2::ZERO);
    }

    #[test]
    fn distance_squared_basic() {
        let a = Vec2::new(Fixed64::from_i32(1), Fixed64::from_i32(2));
        let b = Vec2::new(Fixed64::from_i32(4), Fixed64::from_i32(6));
        // (4-1)^2 + (6-2)^2 = 9 + 16 = 25
        assert_eq!(a.distance_squared(b), Fixed64::from_i32(25));
    }

    #[test]
    fn distance_3_4() {
        let a = Vec2::ZERO;
        let b = Vec2::new(Fixed64::from_i32(3), Fixed64::from_i32(4));
        // sqrt(9 + 16) = 5
        assert_eq!(a.distance(b), Fixed64::from_i32(5));
    }

    #[test]
    fn distance_squared_td1_full_segment() {
        // Regression for the creep-walks-straight-to-endpoint bug:
        // TD_1's td_spawn → td_cp1 horizontal segment is 2800 units.
        // Old i32-backed Fixed64 wrapped 2800² to negative, making the
        // arrived-at-waypoint check pass on tick 1 and pidx skip every
        // checkpoint until CreepLeaked. Must round-trip cleanly now.
        let a = Vec2::new(Fixed64::from_i32(-1400), Fixed64::from_i32(-800));
        let b = Vec2::new(Fixed64::from_i32( 1400), Fixed64::from_i32(-800));
        let d2 = a.distance_squared(b);
        assert_eq!(d2, Fixed64::from_i32(2800 * 2800),
            "distance_squared overflow: raw={}", d2.raw());
        let d = a.distance(b);
        let diff = (d.raw() - Fixed64::from_i32(2800).raw()).abs();
        assert!(diff <= 2, "distance: raw={} expected~={}, diff={}",
            d.raw(), Fixed64::from_i32(2800).raw(), diff);
    }

    #[test]
    fn normalized_long_vector_unit_length() {
        // Old i32 backing made length() return ~0 for long vectors (length_squared
        // overflowed negative → sqrt returns ZERO), so normalized() returned ZERO
        // and downstream `diff.normalized() * step` produced no movement.
        // With i64 backing the unit vector for (2800, 0) must be (1, 0).
        let v = Vec2::new(Fixed64::from_i32(2800), Fixed64::ZERO);
        let n = v.normalized();
        assert_eq!(n.x, Fixed64::ONE, "n.x raw={}", n.x.raw());
        assert_eq!(n.y, Fixed64::ZERO, "n.y raw={}", n.y.raw());
    }
}
