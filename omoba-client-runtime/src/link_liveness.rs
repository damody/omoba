//! Authority link liveness from replica progress. Frames stop arriving when
//! the network path is down (KCP keeps the stream open and resumes in order),
//! so the runtime publishes STALLED after `LINK_STALL_AFTER` without progress,
//! READY on the first progress after a stall, and gives an explicit
//! SERVER_DISCONNECTED verdict once `LINK_LOST_AFTER` elapses; the authority
//! closes the silent session after the same tolerance.

use std::time::{Duration, Instant};

pub use omoba_core::lockstep_timing::{LINK_LOST_AFTER, LINK_STALL_AFTER};

/// An accepted input settles in a team frame near its target tick; the
/// authority refuses inputs later than its one-second late grace. Once the
/// authority has advanced this far past the target without settling it, the
/// input can never be accepted (typically sent into an outage) and the
/// renderer gets an explicit `INPUT_EXPIRED` instead of waiting forever.
pub const INPUT_SETTLE_DEADLINE_TICKS: u64 = 2 * omoba_core::lockstep_timing::LOCKSTEP_TPS_U64;

/// Plain forwarded inputs awaiting settlement, keyed by input id.
#[derive(Default)]
pub struct InputSettleDeadlines(std::collections::BTreeMap<u32, u64>);

impl InputSettleDeadlines {
    pub fn track(&mut self, input_id: u32, target_server_tick: u64) {
        self.0.insert(input_id, target_server_tick.saturating_add(INPUT_SETTLE_DEADLINE_TICKS));
    }

    /// Input ids whose deadline the authority's server tick has passed.
    pub fn expire(&mut self, latest_server_tick: u64) -> Vec<u32> {
        let expired: Vec<u32> = self.0.iter().filter(|(_, deadline)| latest_server_tick > **deadline).map(|(id, _)| *id).collect();
        for id in &expired { self.0.remove(id); }
        expired
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkState {
    Live,
    Stalled,
    Lost,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LinkTransition {
    Stalled { silent_for: Duration },
    Recovered { stalled_for: Duration },
    Lost { silent_for: Duration },
}

pub struct LinkLiveness {
    stall_after: Duration,
    lost_after: Duration,
    progress: u64,
    last_progress_at: Instant,
    state: LinkState,
}

impl LinkLiveness {
    pub fn new(now: Instant, progress: u64) -> Self {
        Self::with_limits(now, progress, LINK_STALL_AFTER, LINK_LOST_AFTER)
    }

    pub fn with_limits(now: Instant, progress: u64, stall_after: Duration, lost_after: Duration) -> Self {
        Self { stall_after, lost_after: lost_after.max(stall_after), progress, last_progress_at: now, state: LinkState::Live }
    }

    pub fn state(&self) -> LinkState {
        self.state
    }

    /// `progress` is any monotonic authority-driven counter (next replica tick).
    pub fn poll(&mut self, now: Instant, progress: u64) -> Option<LinkTransition> {
        if self.state == LinkState::Lost {
            return None;
        }
        if progress != self.progress {
            let stalled_for = now.saturating_duration_since(self.last_progress_at);
            self.progress = progress;
            self.last_progress_at = now;
            if self.state == LinkState::Stalled {
                self.state = LinkState::Live;
                return Some(LinkTransition::Recovered { stalled_for });
            }
            return None;
        }
        let silent_for = now.saturating_duration_since(self.last_progress_at);
        if silent_for >= self.lost_after {
            self.state = LinkState::Lost;
            return Some(LinkTransition::Lost { silent_for });
        }
        if silent_for >= self.stall_after && self.state == LinkState::Live {
            self.state = LinkState::Stalled;
            return Some(LinkTransition::Stalled { silent_for });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stalls_recovers_and_gives_one_lost_verdict() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut link = LinkLiveness::with_limits(start, 10, Duration::from_millis(2000), Duration::from_millis(30_000));
        assert_eq!(link.poll(at(1999), 10), None);
        assert_eq!(link.poll(at(2000), 10), Some(LinkTransition::Stalled { silent_for: Duration::from_millis(2000) }));
        assert_eq!(link.poll(at(5000), 10), None, "stall is published once");
        assert_eq!(link.poll(at(12_000), 11), Some(LinkTransition::Recovered { stalled_for: Duration::from_millis(12_000) }));
        assert_eq!(link.state(), LinkState::Live);
        assert_eq!(link.poll(at(12_500), 40), None, "steady progress stays live");
        assert_eq!(link.poll(at(14_500), 40), Some(LinkTransition::Stalled { silent_for: Duration::from_millis(2000) }));
        assert_eq!(link.poll(at(42_500), 40), Some(LinkTransition::Lost { silent_for: Duration::from_millis(30_000) }));
        assert_eq!(link.poll(at(43_000), 41), None, "lost is terminal");
        assert_eq!(link.state(), LinkState::Lost);
    }

    #[test]
    fn unsettled_inputs_expire_only_after_the_settle_deadline() {
        let mut deadlines = InputSettleDeadlines::default();
        deadlines.track(7, 1000);
        deadlines.track(8, 1100);
        assert!(deadlines.expire(1000 + INPUT_SETTLE_DEADLINE_TICKS).is_empty());
        assert_eq!(deadlines.expire(1001 + INPUT_SETTLE_DEADLINE_TICKS), vec![7]);
        assert!(deadlines.expire(1001 + INPUT_SETTLE_DEADLINE_TICKS).is_empty(), "expired once");
        assert_eq!(deadlines.expire(u64::MAX), vec![8]);
    }

    #[test]
    fn lost_never_precedes_stall_threshold() {
        let start = Instant::now();
        let mut link = LinkLiveness::with_limits(start, 0, Duration::from_millis(5000), Duration::from_millis(1000));
        assert_eq!(link.poll(start + Duration::from_millis(4999), 0), None);
        assert!(matches!(link.poll(start + Duration::from_millis(5000), 0), Some(LinkTransition::Lost { .. })));
    }
}
