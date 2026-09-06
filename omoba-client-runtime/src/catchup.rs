use std::collections::BTreeMap;
use std::time::{Duration, Instant};

pub const MAX_CATCH_UP_FRAMES: u32 = 32;
pub const MAX_CATCH_UP_DURATION: Duration = Duration::from_millis(4);
/// 1～3 幀的 queue jitter 是 120Hz 常態，不進入追趕、每幀仍送 snapshot。
pub const MIN_CATCH_UP_DEPTH: usize = 4;

/// Buffer an already-decoded frame if it is the next expected sequence or a
/// future one. Older duplicates are ignored.
pub fn ingest_pending_frame<T>(
    pending: &mut BTreeMap<u64, T>,
    expected: u64,
    sequence: u64,
    frame: T,
) -> bool {
    if sequence >= expected {
        pending.entry(sequence).or_insert(frame);
        true
    } else {
        false
    }
}

/// Pull every currently queued inbound item into `pending` without waiting.
pub fn drain_available_inbound<T>(
    pending: &mut BTreeMap<u64, T>,
    expected: u64,
    mut next: impl FnMut() -> Option<(u64, T)>,
) {
    while let Some((sequence, frame)) = next() {
        ingest_pending_frame(pending, expected, sequence, frame);
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatchUpPlan {
    /// 普通 snapshot 可在追趕途中合併；critical lifecycle/input 不受影響。
    pub publish_latest_snapshot: bool,
    /// 套用本 frame 後讓出 Tokio executor，避免餓死輸入與控制訊息。
    pub yield_after_frame: bool,
}

pub struct ReplicaLagTracker {
    latest_received_tick: u64,
    last_applied_tick: u64,
    batch_started: Instant,
    batch_frames: u32,
    last_summary: Instant,
}

impl ReplicaLagTracker {
    pub fn new(initial_tick: u64) -> Self {
        let now = Instant::now();
        Self {
            latest_received_tick: initial_tick,
            last_applied_tick: initial_tick,
            batch_started: now,
            batch_frames: 0,
            last_summary: now,
        }
    }

    pub fn observe_received(&mut self, tick: u64) {
        self.latest_received_tick = self.latest_received_tick.max(tick);
    }

    pub fn lag_ticks(&self) -> u64 {
        self.latest_received_tick
            .saturating_sub(self.last_applied_tick)
    }

    pub fn is_catching_up(&self, inbound_depth: usize) -> bool {
        inbound_depth >= MIN_CATCH_UP_DEPTH || self.lag_ticks() >= MIN_CATCH_UP_DEPTH as u64
    }

    /// Yield 只是讓出 executor；沒有待處理 input 時不該結束整批追趕，
    /// 否則會回到 select 空等下一幀，把 300ms 的落後釘死。
    pub fn should_pause_for_input(yielded: bool, input_pending: bool) -> bool {
        yielded && input_pending
    }

    pub fn plan_next_frame(&self, inbound_depth: usize) -> CatchUpPlan {
        let catching_up = self.is_catching_up(inbound_depth);
        let reaches_frame_limit = self.batch_frames.saturating_add(1) >= MAX_CATCH_UP_FRAMES;
        let reaches_time_limit = self.batch_started.elapsed() >= MAX_CATCH_UP_DURATION;
        let yield_after_frame = catching_up && (reaches_frame_limit || reaches_time_limit);
        CatchUpPlan {
            publish_latest_snapshot: !catching_up || yield_after_frame,
            yield_after_frame,
        }
    }

    pub fn observe_applied(
        &mut self,
        tick: u64,
        inbound_depth: usize,
        checkpoint_depth: usize,
        yielded: bool,
    ) {
        self.last_applied_tick = self.last_applied_tick.max(tick);
        self.batch_frames = self.batch_frames.saturating_add(1);
        let now = Instant::now();
        if inbound_depth < MIN_CATCH_UP_DEPTH || yielded {
            if inbound_depth >= MIN_CATCH_UP_DEPTH {
                log::debug!(
                    "replica catch-up batch frames={} elapsed_us={} inbound_depth={} checkpoint_depth={}",
                    self.batch_frames,
                    self.batch_started.elapsed().as_micros(),
                    inbound_depth,
                    checkpoint_depth,
                );
            }
            self.batch_frames = 0;
            self.batch_started = now;
        }
        if inbound_depth >= MIN_CATCH_UP_DEPTH
            && now.duration_since(self.last_summary) >= Duration::from_secs(1)
        {
            log::warn!(
                "replica lag summary latest_received_tick={} last_applied_tick={} lag_ticks={} inbound_depth={} checkpoint_depth={}",
                self.latest_received_tick,
                self.last_applied_tick,
                self.latest_received_tick.saturating_sub(self.last_applied_tick),
                inbound_depth,
                checkpoint_depth,
            );
            self.last_summary = now;
        }
    }

    #[cfg(test)]
    pub(crate) fn force_batch_age(&mut self, age: Duration) {
        self.batch_started = Instant::now() - age;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_limit_ends_a_catch_up_batch() {
        let mut tracker = ReplicaLagTracker::new(0);
        for tick in 1..MAX_CATCH_UP_FRAMES {
            let plan = tracker.plan_next_frame(10);
            assert!(!plan.yield_after_frame);
            tracker.observe_applied(u64::from(tick), 10, 0, false);
        }
        let plan = tracker.plan_next_frame(10);
        assert!(plan.yield_after_frame);
        assert!(plan.publish_latest_snapshot);
    }

    #[test]
    fn time_limit_ends_a_catch_up_batch() {
        let mut tracker = ReplicaLagTracker::new(0);
        tracker.force_batch_age(MAX_CATCH_UP_DURATION);
        let plan = tracker.plan_next_frame(MIN_CATCH_UP_DEPTH);
        assert!(plan.yield_after_frame);
    }

    #[test]
    fn one_or_three_queued_frames_stay_live_and_publish() {
        let tracker = ReplicaLagTracker::new(0);
        for depth in [1_usize, 3] {
            assert_eq!(
                tracker.plan_next_frame(depth),
                CatchUpPlan {
                    publish_latest_snapshot: true,
                    yield_after_frame: false
                }
            );
        }
    }

    #[test]
    fn four_queued_frames_enter_catch_up() {
        let tracker = ReplicaLagTracker::new(0);
        let plan = tracker.plan_next_frame(MIN_CATCH_UP_DEPTH);
        assert!(!plan.publish_latest_snapshot);
        assert!(!plan.yield_after_frame);
    }

    #[test]
    fn catch_up_keeps_going_after_yield_when_no_input_is_waiting() {
        assert!(!ReplicaLagTracker::should_pause_for_input(true, false));
        assert!(ReplicaLagTracker::should_pause_for_input(true, true));
        assert!(!ReplicaLagTracker::should_pause_for_input(false, true));
    }

    #[test]
    fn no_backlog_always_publishes_latest_snapshot() {
        let tracker = ReplicaLagTracker::new(0);
        let plan = tracker.plan_next_frame(0);
        assert_eq!(
            plan,
            CatchUpPlan {
                publish_latest_snapshot: true,
                yield_after_frame: false
            }
        );
    }

    fn simulate_backlog(frame_count: u32) -> (u32, u32) {
        let mut tracker = ReplicaLagTracker::new(0);
        let mut applied = 0;
        let mut yields = 0;
        for tick in 1..=frame_count {
            let remaining = (frame_count - tick) as usize;
            let plan = tracker.plan_next_frame(remaining);
            applied += 1;
            yields += u32::from(plan.yield_after_frame);
            tracker.observe_applied(u64::from(tick), remaining, 0, plan.yield_after_frame);
        }
        (applied, yields)
    }

    #[test]
    fn catches_up_ten_frames_without_loss() {
        assert_eq!(simulate_backlog(10), (10, 0));
    }

    #[test]
    fn catches_up_seventy_two_frames_in_bounded_batches() {
        assert_eq!(simulate_backlog(72), (72, 2));
    }

    #[test]
    fn catches_up_one_hundred_twenty_frames_in_bounded_batches() {
        assert_eq!(simulate_backlog(120), (120, 3));
    }

    #[test]
    fn draining_available_inbound_applies_a_full_catch_up_batch() {
        let mut available: std::collections::VecDeque<u64> = (1..=72).collect();
        let mut pending = BTreeMap::new();
        let mut tracker = ReplicaLagTracker::new(0);
        let mut expected = 1_u64;
        let first = available.pop_front().unwrap();
        ingest_pending_frame(&mut pending, expected, first, first);
        drain_available_inbound(&mut pending, expected, || available.pop_front().map(|seq| (seq, seq)));
        assert_eq!(pending.len(), 72);

        let mut applied = 0_u32;
        while let Some(_frame) = pending.remove(&expected) {
            let remaining = pending.len();
            let plan = tracker.plan_next_frame(remaining);
            applied += 1;
            expected += 1;
            tracker.observe_applied(expected - 1, remaining, 0, plan.yield_after_frame);
            if plan.yield_after_frame {
                break;
            }
        }
        assert_eq!(applied, MAX_CATCH_UP_FRAMES);
        assert_eq!(pending.len(), 40);
        assert_eq!(expected, u64::from(MAX_CATCH_UP_FRAMES) + 1);
    }
}
