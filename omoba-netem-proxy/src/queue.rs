use std::{cmp::Ordering, collections::BinaryHeap, net::SocketAddr, time::Duration};

use tokio::time::Instant;

use crate::{
    config::DelayMode,
    delay::{Direction, RouteId},
    NetemError, Result,
};

#[derive(Debug)]
pub struct QueuedDatagram {
    pub route: RouteId,
    pub direction: Direction,
    pub ordinal: u64,
    pub deadline: Instant,
    pub scheduled_delay_ms: u32,
    pub rtt_ms: u32,
    pub bucket: usize,
    pub bytes: Vec<u8>,
    pub target: Option<SocketAddr>,
}

impl PartialEq for QueuedDatagram {
    fn eq(&self, other: &Self) -> bool {
        self.deadline == other.deadline && self.ordinal == other.ordinal
    }
}
impl Eq for QueuedDatagram {}
impl PartialOrd for QueuedDatagram {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for QueuedDatagram {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .deadline
            .cmp(&self.deadline)
            .then_with(|| other.ordinal.cmp(&self.ordinal))
    }
}

#[derive(Clone, Debug, Default)]
pub struct QueueMetrics {
    pub packets_high_watermark: usize,
    pub bytes_high_watermark: usize,
    pub reordered: u64,
    pub released: u64,
    pub scheduled_rtt_ms: Vec<u32>,
    pub scheduled_delay_ms: Vec<u32>,
    pub release_lateness_us: Vec<u64>,
    pub histogram: [u64; 20],
}

pub struct DelayQueue {
    route: RouteId,
    direction: Direction,
    mode: DelayMode,
    max_datagrams: usize,
    max_bytes: usize,
    watchdog: Duration,
    heap: BinaryHeap<QueuedDatagram>,
    queued_bytes: usize,
    next_ordinal: u64,
    last_scheduled_deadline: Option<Instant>,
    last_released_ordinal: Option<u64>,
    watchdog_deadline: Option<Instant>,
    pub metrics: QueueMetrics,
}

impl DelayQueue {
    pub fn new(
        route: RouteId,
        direction: Direction,
        mode: DelayMode,
        max_datagrams: usize,
        max_bytes: usize,
        watchdog: Duration,
    ) -> Self {
        Self {
            route,
            direction,
            mode,
            max_datagrams,
            max_bytes,
            watchdog,
            heap: BinaryHeap::new(),
            queued_bytes: 0,
            next_ordinal: 0,
            last_scheduled_deadline: None,
            last_released_ordinal: None,
            watchdog_deadline: None,
            metrics: QueueMetrics::default(),
        }
    }

    pub fn enqueue(
        &mut self,
        now: Instant,
        delay_ms: u32,
        rtt_ms: u32,
        bucket: usize,
        bytes: Vec<u8>,
        target: Option<SocketAddr>,
    ) -> Result<()> {
        if self.heap.len() >= self.max_datagrams {
            return Err(self.overflow("datagram"));
        }
        let next_bytes = self
            .queued_bytes
            .checked_add(bytes.len())
            .ok_or_else(|| self.overflow("byte arithmetic"))?;
        if next_bytes > self.max_bytes {
            return Err(self.overflow("bytes"));
        }
        let raw_deadline = now + Duration::from_millis(u64::from(delay_ms));
        let deadline = if self.mode == DelayMode::OrderedDelay {
            self.last_scheduled_deadline
                .map_or(raw_deadline, |last| raw_deadline.max(last))
        } else {
            raw_deadline
        };
        if self.mode == DelayMode::NaturalReorder
            && self
                .last_scheduled_deadline
                .is_some_and(|last| deadline < last)
        {
            self.metrics.reordered += 1;
        }
        self.last_scheduled_deadline = Some(deadline);
        let ordinal = self.next_ordinal;
        self.next_ordinal = self.next_ordinal.saturating_add(1);
        self.heap.push(QueuedDatagram {
            route: self.route,
            direction: self.direction,
            ordinal,
            deadline,
            scheduled_delay_ms: delay_ms,
            rtt_ms,
            bucket,
            bytes,
            target,
        });
        self.queued_bytes = next_bytes;
        self.metrics.packets_high_watermark =
            self.metrics.packets_high_watermark.max(self.heap.len());
        self.metrics.bytes_high_watermark =
            self.metrics.bytes_high_watermark.max(self.queued_bytes);
        self.metrics.scheduled_rtt_ms.push(rtt_ms);
        self.metrics.scheduled_delay_ms.push(delay_ms);
        self.metrics.histogram[bucket] += 1;
        self.watchdog_deadline.get_or_insert(now + self.watchdog);
        Ok(())
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.heap.peek().map(|value| value.deadline)
    }
    pub fn pop_due(&mut self, now: Instant) -> Option<QueuedDatagram> {
        if self.heap.peek().is_none_or(|value| value.deadline > now) {
            return None;
        }
        let value = self.heap.pop()?;
        self.queued_bytes -= value.bytes.len();
        if self
            .last_released_ordinal
            .is_some_and(|last| value.ordinal < last)
        {
            self.metrics.reordered += 1;
        }
        self.last_released_ordinal = Some(value.ordinal);
        self.metrics.released += 1;
        self.metrics
            .release_lateness_us
            .push(now.saturating_duration_since(value.deadline).as_micros() as u64);
        self.watchdog_deadline = (!self.heap.is_empty()).then_some(now + self.watchdog);
        Some(value)
    }
    pub fn check_watchdog(&self, now: Instant) -> Result<()> {
        if self
            .watchdog_deadline
            .is_some_and(|deadline| now >= deadline)
        {
            return Err(NetemError::Watchdog(format!(
                "team={} direction={:?}",
                self.route.team_id(),
                self.direction
            )));
        }
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
    fn overflow(&self, kind: &str) -> NetemError {
        NetemError::Queue(format!(
            "{kind} budget team={} direction={:?} packets={} bytes={}",
            self.route.team_id(),
            self.direction,
            self.heap.len(),
            self.queued_bytes
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn q(mode: DelayMode, packets: usize, bytes: usize) -> DelayQueue {
        DelayQueue::new(
            RouteId::Team1,
            Direction::ClientToServer,
            mode,
            packets,
            bytes,
            Duration::from_millis(20),
        )
    }
    #[test]
    fn ordered_deadlines_never_regress() {
        let now = Instant::now();
        let mut q = q(DelayMode::OrderedDelay, 10, 100);
        q.enqueue(now, 50, 100, 19, vec![1], None).unwrap();
        q.enqueue(now, 10, 20, 0, vec![2], None).unwrap();
        assert_eq!(q.metrics.reordered, 0);
        assert!(q.pop_due(now + Duration::from_millis(49)).is_none());
        assert_eq!(
            q.pop_due(now + Duration::from_millis(50)).unwrap().ordinal,
            0
        );
        assert_eq!(
            q.pop_due(now + Duration::from_millis(50)).unwrap().ordinal,
            1
        )
    }
    #[test]
    fn natural_mode_records_overtake() {
        let now = Instant::now();
        let mut q = q(DelayMode::NaturalReorder, 10, 100);
        q.enqueue(now, 50, 100, 19, vec![1], None).unwrap();
        q.enqueue(now, 10, 20, 0, vec![2], None).unwrap();
        assert!(q.metrics.reordered > 0);
        assert_eq!(
            q.pop_due(now + Duration::from_millis(10)).unwrap().ordinal,
            1
        )
    }
    #[test]
    fn equal_deadline_uses_ordinal() {
        let now = Instant::now();
        let mut q = q(DelayMode::NaturalReorder, 10, 100);
        q.enqueue(now, 10, 20, 0, vec![1], None).unwrap();
        q.enqueue(now, 10, 20, 0, vec![2], None).unwrap();
        assert_eq!(
            q.pop_due(now + Duration::from_millis(10)).unwrap().ordinal,
            0
        )
    }
    #[test]
    fn budgets_and_watchdog_fail_closed() {
        let now = Instant::now();
        let mut packet_queue = q(DelayMode::OrderedDelay, 1, 1);
        packet_queue
            .enqueue(now, 100, 100, 19, vec![1], None)
            .unwrap();
        assert!(packet_queue
            .enqueue(now, 100, 100, 19, vec![1], None)
            .is_err());
        assert!(packet_queue
            .check_watchdog(now + Duration::from_millis(21))
            .is_err());
        let mut byte_queue = q(DelayMode::OrderedDelay, 2, 1);
        assert!(byte_queue.enqueue(now, 1, 20, 0, vec![1, 2], None).is_err())
    }
}
