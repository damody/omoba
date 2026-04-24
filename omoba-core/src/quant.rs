//! Quantization helpers for the KCP binary protocol (P2).
//!
//! Three quantization schemes match the proto primitives in `proto/game.proto`:
//!
//! - **Position16** (`scale = 0.25`) — world coordinates, ±8191.75 range,
//!   0.25 px precision. Each axis encoded as `sint32` (varint in proto3, ~2 B).
//! - **Fixed16** (`scale = 0.1`) — HP / damage / armor / move_speed, ±3276.7
//!   range, 0.1 precision.
//! - **Facing8** (256 steps across 2π) — facing angle, ~1.4° precision. Single
//!   `uint32` (1 byte varint typical).
//!
//! Centralizing these here keeps encoder (omb) and decoder (omfx via
//! `omoba-core`) in lock-step — a single place to change the scale.

pub const POSITION_SCALE: f32 = 0.25;
pub const FIXED_SCALE: f32 = 0.1;

/// Real-valued x → integer quantized. `sint32` in proto3 fits the int16
/// clamp range cheaply (2 byte varint for most game-world values).
pub fn pos_quant(v: f32) -> i32 {
    let scaled = (v / POSITION_SCALE).round();
    scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i32
}

pub fn pos_dequant(q: i32) -> f32 {
    q as f32 * POSITION_SCALE
}

pub fn fixed_quant(v: f32) -> i32 {
    let scaled = (v / FIXED_SCALE).round();
    scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i32
}

pub fn fixed_dequant(q: i32) -> f32 {
    q as f32 * FIXED_SCALE
}

/// Radian → u8-range (0..256 corresponds to 0..2π). Wraps via `rem_euclid`.
/// Returned `u32` because proto3 doesn't have `u8`.
pub fn facing_quant(rad: f32) -> u32 {
    let norm = rad.rem_euclid(std::f32::consts::TAU);
    let q = (norm / std::f32::consts::TAU * 256.0).round() as u32;
    q & 0xFF
}

pub fn facing_dequant(q: u32) -> f32 {
    (q & 0xFF) as f32 / 256.0 * std::f32::consts::TAU
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pos_roundtrip_within_precision() {
        for v in [-4000.0f32, -100.5, -0.25, 0.0, 0.5, 123.75, 4000.0] {
            let q = pos_quant(v);
            let back = pos_dequant(q);
            assert!(
                (back - v).abs() <= POSITION_SCALE / 2.0,
                "pos roundtrip v={} → q={} → back={}",
                v, q, back
            );
        }
    }

    #[test]
    fn pos_clamps_out_of_range() {
        // 10000 is outside i16 range (max 32767 × scale 0.25 = 8191.75)
        let q = pos_quant(10000.0);
        assert_eq!(q, i16::MAX as i32, "should clamp at positive bound");
        let q = pos_quant(-10000.0);
        assert_eq!(q, i16::MIN as i32, "should clamp at negative bound");
    }

    #[test]
    fn fixed_roundtrip_within_precision() {
        for v in [-1000.0f32, -50.07, -0.1, 0.0, 0.1, 123.4, 1000.0] {
            let q = fixed_quant(v);
            let back = fixed_dequant(q);
            // Tolerance: half-grid + small f32 slack. Midpoint values like
            // -50.05 are ambiguous and trip f32 imprecision, so we pick
            // non-midpoint samples above and bound the slack generously.
            let tol = FIXED_SCALE / 2.0 + 1e-3;
            assert!(
                (back - v).abs() <= tol,
                "fixed roundtrip v={} → q={} → back={} tol={}",
                v, q, back, tol
            );
        }
    }

    #[test]
    fn facing_wraps_at_tau() {
        let a = facing_quant(0.0);
        let b = facing_quant(std::f32::consts::TAU);
        let c = facing_quant(std::f32::consts::TAU * 2.0);
        assert_eq!(a, b, "τ should collapse to 0 bucket");
        assert_eq!(a, c, "2τ should collapse to 0 bucket");
    }

    #[test]
    fn facing_roundtrip_within_precision() {
        // Sample across quadrants.
        let samples = [
            0.0, std::f32::consts::PI / 4.0, std::f32::consts::PI / 2.0,
            std::f32::consts::PI, std::f32::consts::PI * 1.5,
        ];
        for &v in &samples {
            let q = facing_quant(v);
            let back = facing_dequant(q);
            let max_err = std::f32::consts::TAU / 256.0; // ~0.0245 rad ≈ 1.4°
            let err = (back - v).abs().min((std::f32::consts::TAU - (back - v).abs()).abs());
            assert!(err <= max_err, "facing v={} → q={} → back={} err={}", v, q, back, err);
        }
    }

    #[test]
    fn facing_negative_wraps() {
        // -π should land at 128 (halfway)
        let q = facing_quant(-std::f32::consts::PI);
        assert_eq!(q, 128, "-π ≡ π mod 2π → bucket 128");
    }
}
