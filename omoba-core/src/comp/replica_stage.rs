//! Optional wall-clock stages for one successful client replica apply.
//! Durations are not world resources and are not inputs to the canonical hash,
//! the wire format, the script ABI, or the content catalog.
//! `residual_ns` is unattributed wall time. It is not a CPU-cause claim.

use std::fmt::Write as _;
use std::time::Instant;

pub const REPLICA_STAGE_VERSION: u64 = 1;
pub const REPLICA_STAGE_SAMPLES: u64 = 60;
pub const REPLICA_STAGE_SCOPE: &str = "apply_encoded_frame";
pub const REPLICA_STAGE_PHASE_COUNT: usize = 7;

pub const REPLICA_STAGE_PHASE_KEYS: [&str; REPLICA_STAGE_PHASE_COUNT] = [
    "decode_ns",
    "staging_ns",
    "fixed_step_ns",
    "pre_repair_ns",
    "post_step_repair_ns",
    "post_repair_hash_ns",
    "host_finalize_ns",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ReplicaStagePhase {
    Decode = 0,
    Staging = 1,
    FixedStep = 2,
    PreRepair = 3,
    PostStepRepair = 4,
    PostRepairHash = 5,
    HostFinalize = 6,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReplicaStageDurations {
    pub phases_ns: [u128; REPLICA_STAGE_PHASE_COUNT],
}

impl ReplicaStageDurations {
    pub fn phase_sum_ns(&self) -> u128 {
        self.phases_ns
            .iter()
            .fold(0u128, |sum, phase| sum.saturating_add(*phase))
    }

    pub fn phase_ns(&self, phase: ReplicaStagePhase) -> u128 {
        self.phases_ns[phase as usize]
    }

    pub fn set_phase(&mut self, phase: ReplicaStagePhase, ns: u128) {
        self.phases_ns[phase as usize] = ns;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReplicaStageSample {
    pub replica_tick: u64,
    pub team_sequence: u64,
    pub phases_ns: [u128; REPLICA_STAGE_PHASE_COUNT],
    pub outer_ns: u128,
    pub residual_ns: u128,
}

impl ReplicaStageSample {
    pub fn from_durations(
        replica_tick: u64,
        team_sequence: u64,
        durations: &ReplicaStageDurations,
        outer_ns: u128,
    ) -> Result<Self, ReplicaStageReject> {
        let phase_sum_ns = durations.phase_sum_ns();
        if phase_sum_ns > outer_ns {
            return Err(ReplicaStageReject::InvalidPhaseSum {
                phase_sum_ns,
                residual_ns: 0,
                outer_ns,
            });
        }
        Ok(Self {
            replica_tick,
            team_sequence,
            phases_ns: durations.phases_ns,
            outer_ns,
            residual_ns: outer_ns - phase_sum_ns,
        })
    }

    pub fn phase_sum_ns(&self) -> u128 {
        self.phases_ns
            .iter()
            .fold(0u128, |sum, phase| sum.saturating_add(*phase))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaStageReject {
    InvalidPhaseSum {
        phase_sum_ns: u128,
        residual_ns: u128,
        outer_ns: u128,
    },
}

/// Fixed window. Only the first strict-maximum outer sample is retained.
/// Ties keep that first sample. Closing the window drops it; nothing else is stored.
#[derive(Clone, Debug)]
pub struct ReplicaStageWindow {
    player_id: u32,
    team_id: u32,
    capacity: u64,
    samples: u64,
    slowest: Option<ReplicaStageSample>,
}

impl ReplicaStageWindow {
    pub fn new(player_id: u32, team_id: u32) -> Self {
        Self::with_capacity(REPLICA_STAGE_SAMPLES, player_id, team_id)
    }

    pub fn with_capacity(capacity: u64, player_id: u32, team_id: u32) -> Self {
        assert!(capacity >= 1, "replica stage window requires at least one sample");
        Self {
            player_id,
            team_id,
            capacity,
            samples: 0,
            slowest: None,
        }
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }

    pub fn capacity(&self) -> u64 {
        self.capacity
    }

    pub fn slowest(&self) -> Option<&ReplicaStageSample> {
        self.slowest.as_ref()
    }

    pub fn reset(&mut self) {
        self.samples = 0;
        self.slowest = None;
    }

    pub fn record(&mut self, sample: ReplicaStageSample) -> Result<Option<String>, ReplicaStageReject> {
        let phase_sum_ns = sample.phase_sum_ns();
        match phase_sum_ns.checked_add(sample.residual_ns) {
            Some(total) if total == sample.outer_ns => {}
            _ => {
                return Err(ReplicaStageReject::InvalidPhaseSum {
                    phase_sum_ns,
                    residual_ns: sample.residual_ns,
                    outer_ns: sample.outer_ns,
                });
            }
        }
        let keep = match self.slowest {
            Some(current) => sample.outer_ns > current.outer_ns,
            None => true,
        };
        if keep {
            self.slowest = Some(sample);
        }
        self.samples = self.samples.saturating_add(1);
        if self.samples < self.capacity {
            return Ok(None);
        }
        let line = self.format_closed();
        self.reset();
        Ok(Some(line))
    }

    fn format_closed(&self) -> String {
        let sample = self
            .slowest
            .expect("a closed replica stage window has its slowest sample");
        let mut out = String::from("OM_REPLICA_STAGE {");
        push_num(&mut out, "v", u128::from(REPLICA_STAGE_VERSION));
        push_str(&mut out, "component", "client_runtime");
        push_str(&mut out, "metric", "replica_stage");
        push_str(&mut out, "unit", "ns");
        push_str(&mut out, "clock", "wall");
        push_str(&mut out, "scope", REPLICA_STAGE_SCOPE);
        push_num(&mut out, "samples", u128::from(self.samples));
        push_num(&mut out, "player_id", u128::from(self.player_id));
        push_num(&mut out, "team_id", u128::from(self.team_id));
        push_num(&mut out, "replica_tick", u128::from(sample.replica_tick));
        push_num(&mut out, "team_sequence", u128::from(sample.team_sequence));
        push_num(&mut out, "outer_ns", sample.outer_ns);
        push_num(&mut out, "residual_ns", sample.residual_ns);
        push_str(&mut out, "residual_meaning", "unattributed_wall_time");
        push_str(&mut out, "phases_are", "same_outer_slowest_sample");
        push_str(&mut out, "not", "independent_phase_maxima");
        for (index, key) in REPLICA_STAGE_PHASE_KEYS.iter().enumerate() {
            push_num(&mut out, key, sample.phases_ns[index]);
        }
        push_str(
            &mut out,
            "staging_covers",
            "preflight_pre_step_injection_baseline_staging",
        );
        push_str(&mut out, "pre_repair_covers", "restore_allowlist_pre_repair_hash");
        push_str(
            &mut out,
            "host_finalize_covers",
            "fog_sha_report_bookkeeping",
        );
        out.push('}');
        out
    }
}

fn push_str(out: &mut String, key: &str, value: &str) {
    out.push(',');
    out.push('"');
    out.push_str(key);
    out.push_str("\":\"");
    debug_assert!(!value.contains(['"', '\\']));
    out.push_str(value);
    out.push('"');
}

fn push_num(out: &mut String, key: &str, value: u128) {
    if !out.ends_with('{') {
        out.push(',');
    }
    out.push('"');
    out.push_str(key);
    out.push_str("\":");
    let _ = write!(out, "{value}");
}

pub trait ReplicaStageClock {
    fn enter(&mut self, phase: ReplicaStagePhase);
    fn leave(&mut self);
    fn measuring(&self) -> bool;
    fn durations(&self) -> ReplicaStageDurations;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct NoopReplicaStageClock;

impl ReplicaStageClock for NoopReplicaStageClock {
    fn enter(&mut self, _phase: ReplicaStagePhase) {}

    fn leave(&mut self) {}

    fn measuring(&self) -> bool {
        false
    }

    fn durations(&self) -> ReplicaStageDurations {
        ReplicaStageDurations::default()
    }
}

#[derive(Clone, Debug)]
pub struct MeasuringReplicaStageClock {
    open: Option<(ReplicaStagePhase, Instant)>,
    phases_ns: [u128; REPLICA_STAGE_PHASE_COUNT],
}

impl MeasuringReplicaStageClock {
    pub fn new() -> Self {
        Self {
            open: None,
            phases_ns: [0; REPLICA_STAGE_PHASE_COUNT],
        }
    }
}

impl Default for MeasuringReplicaStageClock {
    fn default() -> Self {
        Self::new()
    }
}

impl ReplicaStageClock for MeasuringReplicaStageClock {
    fn enter(&mut self, phase: ReplicaStagePhase) {
        self.leave();
        self.open = Some((phase, Instant::now()));
    }

    fn leave(&mut self) {
        if let Some((phase, started)) = self.open.take() {
            let elapsed = started.elapsed().as_nanos();
            self.phases_ns[phase as usize] = self.phases_ns[phase as usize].saturating_add(elapsed);
        }
    }

    fn measuring(&self) -> bool {
        true
    }

    fn durations(&self) -> ReplicaStageDurations {
        ReplicaStageDurations {
            phases_ns: self.phases_ns,
        }
    }
}

pub struct ReplicaPhaseGuard<'a, C: ReplicaStageClock> {
    clock: &'a mut C,
    open: bool,
}

impl<'a, C: ReplicaStageClock> ReplicaPhaseGuard<'a, C> {
    pub fn enter(clock: &'a mut C, phase: ReplicaStagePhase) -> Self {
        clock.enter(phase);
        Self { clock, open: true }
    }

    pub fn close(mut self) {
        self.leave();
    }

    fn leave(&mut self) {
        if self.open {
            self.clock.leave();
            self.open = false;
        }
    }
}

impl<'a, C: ReplicaStageClock> Drop for ReplicaPhaseGuard<'a, C> {
    fn drop(&mut self) {
        self.leave();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(tick: u64, sequence: u64, phases: [u128; 7], outer: u128) -> ReplicaStageSample {
        let sum = phases.iter().fold(0u128, |total, phase| total + *phase);
        assert!(sum <= outer);
        ReplicaStageSample {
            replica_tick: tick,
            team_sequence: sequence,
            phases_ns: phases,
            outer_ns: outer,
            residual_ns: outer - sum,
        }
    }

    fn parse(line: &str) -> serde_json::Value {
        let json = line.strip_prefix("OM_REPLICA_STAGE ").expect("stage marker");
        serde_json::from_str(json).expect("stage json")
    }

    #[test]
    fn replica_stage_window_is_bounded_to_one_slowest_sample() {
        let mut window = ReplicaStageWindow::with_capacity(3, 4, 9);
        assert_eq!(ReplicaStageWindow::new(4, 9).capacity(), REPLICA_STAGE_SAMPLES);
        assert_eq!(REPLICA_STAGE_SAMPLES, 60);
        assert!(window
            .record(sample(1, 1, [1, 0, 0, 0, 0, 0, 0], 10))
            .unwrap()
            .is_none());
        assert!(window
            .record(sample(2, 2, [2, 0, 0, 0, 0, 0, 0], 20))
            .unwrap()
            .is_none());
        assert_eq!(window.samples(), 2);
        assert_eq!(window.slowest().unwrap().replica_tick, 2);
        let line = window
            .record(sample(3, 3, [9, 0, 0, 0, 0, 0, 0], 15))
            .unwrap()
            .expect("third sample closes the window");
        assert_eq!(window.samples(), 0);
        assert!(window.slowest().is_none());
        let json = parse(&line);
        assert_eq!(json["samples"], 3);
        assert_eq!(json["replica_tick"], 2);
        assert!(window
            .record(sample(4, 4, [1, 0, 0, 0, 0, 0, 0], 5))
            .unwrap()
            .is_none());
        assert_eq!(window.samples(), 1);
        assert_eq!(window.slowest().unwrap().replica_tick, 4);
    }

    #[test]
    fn replica_stage_closed_line_uses_every_phase_of_the_same_outer_peak() {
        let mut window = ReplicaStageWindow::with_capacity(2, 8, 2);
        let peak = sample(41, 7, [1, 2, 3, 4, 5, 6, 7], 100);
        let other = sample(42, 8, [90, 1, 1, 1, 1, 1, 1], 99);
        assert!(window.record(peak).unwrap().is_none());
        let line = window.record(other).unwrap().expect("window closes");
        let json = parse(&line);
        assert_eq!(json["v"], 1);
        assert_eq!(json["player_id"], 8);
        assert_eq!(json["team_id"], 2);
        assert_eq!(json["unit"], "ns");
        assert_eq!(json["clock"], "wall");
        assert_eq!(json["scope"], REPLICA_STAGE_SCOPE);
        assert_eq!(json["samples"], 2);
        assert_eq!(json["replica_tick"], 41);
        assert_eq!(json["team_sequence"], 7);
        assert_eq!(json["outer_ns"], 100);
        assert_eq!(json["residual_ns"], 72);
        assert_eq!(json["residual_meaning"], "unattributed_wall_time");
        assert_eq!(json["phases_are"], "same_outer_slowest_sample");
        assert_eq!(json["not"], "independent_phase_maxima");
        assert_eq!(json["decode_ns"], 1);
        assert_eq!(json["staging_ns"], 2);
        assert_eq!(json["fixed_step_ns"], 3);
        assert_eq!(json["pre_repair_ns"], 4);
        assert_eq!(json["post_step_repair_ns"], 5);
        assert_eq!(json["post_repair_hash_ns"], 6);
        assert_eq!(json["host_finalize_ns"], 7);
        assert_ne!(json["decode_ns"], 90);
    }

    #[test]
    fn replica_stage_tie_keeps_the_first_sample_and_reset_drops_it() {
        let mut window = ReplicaStageWindow::with_capacity(2, 1, 1);
        let first = sample(5, 5, [4, 0, 0, 0, 0, 0, 0], 50);
        let tied = sample(6, 6, [40, 0, 0, 0, 0, 0, 0], 50);
        assert!(window.record(first).unwrap().is_none());
        let line = window.record(tied).unwrap().expect("tie still closes");
        let json = parse(&line);
        assert_eq!(json["replica_tick"], 5);
        assert_eq!(json["team_sequence"], 5);
        assert_eq!(json["decode_ns"], 4);
        assert_eq!(json["outer_ns"], 50);
        assert_eq!(window.samples(), 0);
        assert!(window.slowest().is_none());
        window.record(sample(9, 9, [1, 0, 0, 0, 0, 0, 0], 10)).unwrap();
        assert_eq!(window.samples(), 1);
        window.reset();
        assert_eq!(window.samples(), 0);
        assert!(window.slowest().is_none());
        assert!(window
            .record(sample(10, 10, [3, 0, 0, 0, 0, 0, 0], 8))
            .unwrap()
            .is_none());
        assert_eq!(window.slowest().unwrap().replica_tick, 10);
    }

    #[test]
    fn replica_stage_invalid_phase_sum_is_rejected_without_advancing() {
        let mut window = ReplicaStageWindow::with_capacity(2, 1, 1);
        let kept = sample(1, 1, [2, 0, 0, 0, 0, 0, 0], 10);
        assert!(window.record(kept).unwrap().is_none());
        let overflow = ReplicaStageSample {
            replica_tick: 2,
            team_sequence: 2,
            phases_ns: [8, 0, 0, 0, 0, 0, 0],
            outer_ns: 5,
            residual_ns: 0,
        };
        assert_eq!(
            window.record(overflow),
            Err(ReplicaStageReject::InvalidPhaseSum {
                phase_sum_ns: 8,
                residual_ns: 0,
                outer_ns: 5,
            })
        );
        let short_residual = ReplicaStageSample {
            replica_tick: 3,
            team_sequence: 3,
            phases_ns: [2, 0, 0, 0, 0, 0, 0],
            outer_ns: 10,
            residual_ns: 1,
        };
        assert!(window.record(short_residual).is_err());
        assert_eq!(window.samples(), 1);
        assert_eq!(window.slowest().unwrap().replica_tick, 1);
        let mut durations = ReplicaStageDurations::default();
        durations.set_phase(ReplicaStagePhase::Decode, 11);
        assert!(ReplicaStageSample::from_durations(1, 1, &durations, 10).is_err());
    }
}
