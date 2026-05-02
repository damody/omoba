//! Fixed-point arithmetic (i32 + SCALE=1024). Implemented in Task 0.2-0.4.

use serde::{Serialize, Deserialize};

pub const SCALE: i32 = 1024;
pub const SCALE_BITS: u32 = 10;
const _: () = assert!(SCALE == 1 << SCALE_BITS);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed32(i32);

impl Fixed32 {
    pub const ZERO: Fixed32 = Fixed32(0);
    pub const ONE: Fixed32 = Fixed32(SCALE);

    pub const fn from_raw(raw: i32) -> Self { Fixed32(raw) }
    pub const fn from_i32(v: i32) -> Self { Fixed32(v.wrapping_mul(SCALE)) }
    pub const fn raw(self) -> i32 { self.0 }
    pub fn to_f32(self) -> f32 { self.0 as f32 / SCALE as f32 }
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
    fn mul(self, rhs: Fixed32) -> Fixed32 {
        // (a * SCALE) * (b * SCALE) / SCALE = a*b*SCALE
        let prod = (self.0 as i64) * (rhs.0 as i64);
        Fixed32((prod >> SCALE_BITS) as i32)
    }
}

impl std::ops::Div for Fixed32 {
    type Output = Fixed32;
    fn div(self, rhs: Fixed32) -> Fixed32 {
        let num = (self.0 as i64) << SCALE_BITS;
        Fixed32((num / rhs.0 as i64) as i32)
    }
}

impl std::ops::Neg for Fixed32 {
    type Output = Fixed32;
    fn neg(self) -> Fixed32 { Fixed32(-self.0) }
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
    fn mul_no_overflow_in_range() {
        // ±2000 unit (typical map size) × Fixed32 must not panic
        let pos = Fixed32::from_i32(2000);
        let scale = Fixed32::from_raw(2048);  // 2.0
        let _ = pos * scale;
    }
}
