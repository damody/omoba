//! KCP 二進位協定 (P2) 的量化助手。
//!
//! 三種量化方案與「proto/game.proto」中的原型原語相符：
//!
//! - **Position16** (`scale = 0.25`) — 世界座標，±8191.75 範圍，
//!   0.25 px precision. Each軸 encoded as `sint32` (varint in proto3, ~2 B for
//!   typical values). Clamped to i16 range because map bounds are固定的.
//! - **固定** (`scale = 0.1`) — HP / 傷害 / 護甲 / move_speed，scale 0.1
//!   precision。Wire type 是 proto sint32 (varint)，**不再 clamp 到 i16**：
//!   typical values 1000s 仍只用 2 bytes，stress 場景的 10M HP 用 5 bytes —
//!   bandwidth 損失極小，但避免後端真實值被前端 clamp 成假數（10M → 3276.7）。
//!   名字保留 `Fixed16` 是 proto 的歷史命名；wire 是 sint32 全範圍。
//! - **Facing8**（跨 2π 256 個步長）— 面對角度，約 1.4° 精度。單身的
//! `uint32`（典型的 1 位元組 varint）。
//!
//! 將這些集中在這裡可以保持編碼器（omb）和解碼器（omfx via
//! `omoba-core`) 處於鎖步狀態 — 一個可以改變比例的地方。

pub const POSITION_SCALE: f32 = 0.25;
pub const FIXED_SCALE: f32 = 0.1;

/// 實值 x → 整數量化。 proto3 中的 `sint32` 適合 int16
/// 便宜地限制範圍（對於大多數遊戲世界值來說是 2 位元組 varint）。
pub fn pos_quant(v: f32) -> i32 {
    let scaled = (v / POSITION_SCALE).round();
    scaled.clamp(i16::MIN as f32, i16::MAX as f32) as i32
}

pub fn pos_dequant(q: i32) -> f32 {
    q as f32 * POSITION_SCALE
}

pub fn fixed_quant(v: f32) -> i32 {
    // 不 clamp 到 i16：HP 沒有自然上界（stress 場景 10M+ 也得能跑），
    // 前端應該無條件相信後端數字。typical 值仍走 2-byte varint，極端值 5 byte。
    // 用 i32 邊界當保險避免 overflow（f32→i32 saturating cast）。
    let scaled = (v / FIXED_SCALE).round();
    scaled.clamp(i32::MIN as f32, i32::MAX as f32) as i32
}

pub fn fixed_dequant(q: i32) -> f32 {
    q as f32 * FIXED_SCALE
}

/// 弧度 → u8-範圍（0..256 對應於 0..2π）。透過 `rem_euclid` 進行包裹。
/// 回傳 `u32` 因為 proto3 沒有 `u8`。
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
        // 10000 超出 i16 範圍（最大值 32767 × 標度 0.25 = 8191.75）
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
            // 公差：半格+小f32鬆弛。中點值如
            // -50.05 不明確且 f32 不精確，所以我們選擇
            // 上面的非中點樣本並慷慨地限制了鬆弛。
            let tol = FIXED_SCALE / 2.0 + 1e-3;
            assert!(
                (back - v).abs() <= tol,
                "fixed roundtrip v={} → q={} → back={} tol={}",
                v, q, back, tol
            );
        }
    }

    #[test]
    fn fixed_no_longer_clamps_at_i16() {
        // HP 大到超出舊的 ±3276.7 上限也要 round-trip。
        // 注意：f32 mantissa 24-bit，超過 ~16M 的 q 值會有 1 unit 精度損失，
        // 所以容忍度放寬到 FIXED_SCALE * 2。
        for v in [10_000.0f32, 100_000.0, 1_000_000.0, 10_000_000.0] {
            let q = fixed_quant(v);
            let back = fixed_dequant(q);
            let tol = (FIXED_SCALE * 2.0).max(v.abs() * 1e-6);
            assert!(
                (back - v).abs() <= tol,
                "fixed large-value roundtrip v={} → q={} → back={} tol={}",
                v, q, back, tol,
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
        // 跨象限採樣。
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
        // -π 應落在 128（中間）
        let q = facing_quant(-std::f32::consts::PI);
        assert_eq!(q, 128, "-π ≡ π mod 2π → bucket 128");
    }
}
