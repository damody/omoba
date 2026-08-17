//! omb 與 omfx 共用的 lockstep timing helpers。

/// 權威 lockstep tick rate。2-tick input lookahead 約為 16.7 ms。
pub const LOCKSTEP_TPS: u32 = 120;
pub const LOCKSTEP_TPS_U64: u64 = LOCKSTEP_TPS as u64;
pub const SUPPORTED_LOCKSTEP_FPS: [u32; 3] = [120, 90, 60];

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LockstepTiming {
    step_fps: u32,
}

impl LockstepTiming {
    pub const DEFAULT: Self = Self {
        step_fps: LOCKSTEP_TPS,
    };

    pub fn new(step_fps: u32) -> Result<Self, String> {
        if SUPPORTED_LOCKSTEP_FPS.contains(&step_fps) {
            Ok(Self { step_fps })
        } else {
            Err(format!(
                "unsupported STEP_FPS={step_fps}; expected one of 120, 90, 60"
            ))
        }
    }

    pub fn step_fps(self) -> u32 {
        self.step_fps
    }

    pub fn tick_period_us(self) -> u64 {
        1_000_000 / u64::from(self.step_fps)
    }

    pub fn dt_f64(self) -> f64 {
        1.0 / f64::from(self.step_fps)
    }

    pub fn dt_duration(self) -> std::time::Duration {
        std::time::Duration::from_secs_f64(self.dt_f64())
    }

    pub fn ticks_for_seconds(self, seconds: u64) -> u32 {
        self.step_fps
            .saturating_mul(seconds.min(u64::from(u32::MAX)) as u32)
    }

    pub fn ticks_for_seconds_u64(self, seconds: u64) -> u64 {
        u64::from(self.step_fps).saturating_mul(seconds)
    }

    pub fn ticks_to_seconds_f64(self, tick: u32) -> f64 {
        tick as f64 * self.dt_f64()
    }

    pub fn fixed_raw_for_tick(self, tick: u64) -> i64 {
        fixed_raw_for_tick_at_fps(tick, u64::from(self.step_fps))
    }
}

/// 回傳以 `tick` 結尾的 interval 對應的 Q10 Fixed64 raw delta。
///
/// 對 ticks `1..=LOCKSTEP_TPS`，回傳的 raw values 總和剛好為 1024，
/// 不使用 floats 也能在 120Hz 保留一秒 simulation time。
pub fn lockstep_dt_fixed_raw_for_tick(tick: u64) -> i64 {
    fixed_raw_for_tick_at_fps(tick, LOCKSTEP_TPS_U64)
}

/// Returns the deterministic Q10 delta for an arbitrary fixed-step profile.
///
/// This is public for headless simulation profiles. Production networking still
/// validates its rates through [`LockstepTiming`].
pub fn fixed_raw_for_tick_at_fps(tick: u64, tps: u64) -> i64 {
    if tick == 0 {
        return 0;
    }

    let scale = LOCKSTEP_FIXED_SCALE as u128;
    let tps = tps as u128;
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
    fn runtime_fixed_raw_dt_sums_to_one_second_at_supported_fps() {
        for fps in SUPPORTED_LOCKSTEP_FPS {
            let timing = LockstepTiming::new(fps).unwrap();
            let total: i64 = (1..=u64::from(fps))
                .map(|tick| timing.fixed_raw_for_tick(tick))
                .sum();

            assert_eq!(total, LOCKSTEP_FIXED_SCALE, "fps={fps}");
        }
    }

    #[test]
    fn runtime_timing_rejects_unsupported_fps() {
        assert!(LockstepTiming::new(120).is_ok());
        assert!(LockstepTiming::new(90).is_ok());
        assert!(LockstepTiming::new(60).is_ok());
        assert!(LockstepTiming::new(144).is_err());
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
