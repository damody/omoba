//! Fixed-point arithmetic (i32 + SCALE=1024). Implemented in Task 0.2-0.4.

use serde::{Serialize, Deserialize};

pub const SCALE: i32 = 1024;
pub const SCALE_BITS: u32 = 10;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Fixed32(i32);

impl Fixed32 {
    pub const ZERO: Fixed32 = Fixed32(0);
    pub const ONE: Fixed32 = Fixed32(SCALE);

    pub const fn from_raw(raw: i32) -> Self { Fixed32(raw) }
    pub const fn from_i32(v: i32) -> Self { Fixed32(v * SCALE) }
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
}
