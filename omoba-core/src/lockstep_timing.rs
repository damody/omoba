//! omb 與 omfx 共用的 lockstep timing constants。

/// 權威 lockstep tick rate。2-tick input lookahead 約為 16.7 ms。
pub const LOCKSTEP_TPS: u32 = 120;
pub const LOCKSTEP_TPS_U64: u64 = LOCKSTEP_TPS as u64;

/// tokio intervals 使用的截斷 microsecond period。
pub const LOCKSTEP_TICK_PERIOD_US: u64 = 1_000_000 / LOCKSTEP_TPS_U64;

pub const LOCKSTEP_DT_F32: f32 = 1.0 / LOCKSTEP_TPS as f32;
pub const LOCKSTEP_DT_F64: f64 = 1.0 / LOCKSTEP_TPS as f64;

/// Fixed64 在 omoba-sim 中使用 Q10 raw units。`1 / 120 * 1024` 不是整數，
/// 因此 per-tick fixed dt 必須分配成 deterministic 8/9 raw schedule，
/// 而不是每 tick 都截斷為 raw 8（`1/128s`）。
pub const LOCKSTEP_FIXED_SCALE: i64 = 1024;

pub const LOCKSTEP_ONE_SECOND_TICKS_U32: u32 = LOCKSTEP_TPS;
pub const LOCKSTEP_FIVE_SECONDS_TICKS_U32: u32 = LOCKSTEP_TPS * 5;
pub const LOCKSTEP_TEN_SECONDS_TICKS_U32: u32 = LOCKSTEP_TPS * 10;
pub const LOCKSTEP_TEN_SECONDS_TICKS_U64: u64 = LOCKSTEP_TPS_U64 * 10;
pub const LOCKSTEP_THIRTY_SECONDS_TICKS_U64: u64 = LOCKSTEP_TPS_U64 * 30;

pub fn ticks_to_seconds_f64(tick: u32) -> f64 {
    tick as f64 * LOCKSTEP_DT_F64
}

/// 回傳以 `tick` 結尾的 interval 對應的 Q10 Fixed64 raw delta。
///
/// 對 ticks `1..=LOCKSTEP_TPS`，回傳的 raw values 總和剛好為 1024，
/// 不使用 floats 也能在 120Hz 保留一秒 simulation time。
pub fn lockstep_dt_fixed_raw_for_tick(tick: u64) -> i64 {
    if tick == 0 {
        return 0;
    }

    let scale = LOCKSTEP_FIXED_SCALE as u128;
    let tps = LOCKSTEP_TPS_U64 as u128;
    let start = ((tick as u128 - 1) * scale) / tps;
    let end = (tick as u128 * scale) / tps;
    (end - start) as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_raw_dt_sums_to_one_second_at_lockstep_tps() {
        let total: i64 = (1..=LOCKSTEP_TPS_U64)
            .map(lockstep_dt_fixed_raw_for_tick)
            .sum();

        assert_eq!(total, LOCKSTEP_FIXED_SCALE);
    }

    #[test]
    fn fixed_raw_dt_uses_only_neighboring_raw_values() {
        let values: Vec<i64> = (1..=LOCKSTEP_TPS_U64)
            .map(lockstep_dt_fixed_raw_for_tick)
            .collect();

        assert!(values.iter().all(|&v| v == 8 || v == 9), "{values:?}");
        assert!(values.iter().any(|&v| v == 8));
        assert!(values.iter().any(|&v| v == 9));
    }
}
