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
}
