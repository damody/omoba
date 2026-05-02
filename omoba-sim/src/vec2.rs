//! 2D vector of Fixed32 — Phase 1 ECS Position / Velocity / Direction substrate.
//!
//! Pure deterministic arithmetic. `length_squared` uses Fixed32 mul (rounds toward -inf);
//! `length` uses Fixed32::sqrt (returns ZERO for zero/negative). All ops are platform-invariant
//! by virtue of Fixed32's underlying contract.

use crate::fixed::Fixed32;
use serde::{Serialize, Deserialize};

#[cfg_attr(feature = "abi-stable", derive(abi_stable::StableAbi))]
#[cfg_attr(feature = "abi-stable", repr(C))]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: Fixed32,
    pub y: Fixed32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: Fixed32::ZERO, y: Fixed32::ZERO };

    pub const fn new(x: Fixed32, y: Fixed32) -> Self { Vec2 { x, y } }

    /// Dot product: x1*x2 + y1*y2. Useful for projection / facing checks.
    pub fn dot(self, other: Vec2) -> Fixed32 {
        self.x * other.x + self.y * other.y
    }

    /// Squared length. Cheaper than `length()` — prefer for distance comparisons.
    pub fn length_squared(self) -> Fixed32 {
        self.x * self.x + self.y * self.y
    }

    /// Euclidean length via Fixed32::sqrt. Returns ZERO for zero vector.
    pub fn length(self) -> Fixed32 {
        self.length_squared().sqrt()
    }

    /// Squared distance to `other`. Cheaper than `distance()` for comparisons —
    /// avoids the sqrt. Use for "is X within range Y" checks: compare against
    /// `range * range`.
    pub fn distance_squared(self, other: Vec2) -> Fixed32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    /// Euclidean distance to `other`. Uses `Fixed32::sqrt`.
    pub fn distance(self, other: Vec2) -> Fixed32 {
        self.distance_squared(other).sqrt()
    }

    /// Returns this vector divided by its length (unit vector pointing same direction).
    /// Returns `Vec2::ZERO` if length is zero — deterministic, no NaN, matches the
    /// `Fixed32::sqrt(non-positive) → ZERO` contract used elsewhere in the sim.
    pub fn normalized(self) -> Vec2 {
        let len = self.length();
        if len == Fixed32::ZERO { return Vec2::ZERO; }
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

impl std::ops::Mul<Fixed32> for Vec2 {
    type Output = Vec2;
    fn mul(self, scalar: Fixed32) -> Vec2 { Vec2 { x: self.x * scalar, y: self.y * scalar } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_sub() {
        let a = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
        let b = Vec2::new(Fixed32::from_i32(1), Fixed32::from_i32(2));
        assert_eq!(a + b, Vec2::new(Fixed32::from_i32(4), Fixed32::from_i32(6)));
        assert_eq!(a - b, Vec2::new(Fixed32::from_i32(2), Fixed32::from_i32(2)));
    }

    #[test]
    fn dot_product() {
        let a = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
        let b = Vec2::new(Fixed32::from_i32(2), Fixed32::from_i32(1));
        // 3*2 + 4*1 = 10
        assert_eq!(a.dot(b), Fixed32::from_i32(10));
    }

    #[test]
    fn length_3_4_5() {
        let v = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
        assert_eq!(v.length_squared(), Fixed32::from_i32(25));
        // sqrt(25) = 5
        assert_eq!(v.length(), Fixed32::from_i32(5));
    }

    #[test]
    fn length_zero() {
        assert_eq!(Vec2::ZERO.length(), Fixed32::ZERO);
        assert_eq!(Vec2::ZERO.length_squared(), Fixed32::ZERO);
    }

    #[test]
    fn scalar_mul() {
        let v = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
        let r = v * Fixed32::from_i32(2);
        assert_eq!(r, Vec2::new(Fixed32::from_i32(6), Fixed32::from_i32(8)));
    }

    #[test]
    fn normalized_unit_x() {
        let v = Vec2::new(Fixed32::from_i32(3), Fixed32::ZERO);
        let n = v.normalized();
        assert_eq!(n, Vec2::new(Fixed32::ONE, Fixed32::ZERO));
    }

    #[test]
    fn normalized_3_4_5() {
        // (3, 4) has length 5, so normalized = (0.6, 0.8) = raw (614, 819) ± 2 LSB
        let v = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
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
        let a = Vec2::new(Fixed32::from_i32(1), Fixed32::from_i32(2));
        let b = Vec2::new(Fixed32::from_i32(4), Fixed32::from_i32(6));
        // (4-1)^2 + (6-2)^2 = 9 + 16 = 25
        assert_eq!(a.distance_squared(b), Fixed32::from_i32(25));
    }

    #[test]
    fn distance_3_4() {
        let a = Vec2::ZERO;
        let b = Vec2::new(Fixed32::from_i32(3), Fixed32::from_i32(4));
        // sqrt(9 + 16) = 5
        assert_eq!(a.distance(b), Fixed32::from_i32(5));
    }
}
