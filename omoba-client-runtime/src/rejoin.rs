//! Rejoin after a lost link (E060 follow-up). When the authority link is lost
//! the runtime does not exit: it reports RECONNECTING and retries the same match
//! with the resume token its last TeamGameStart issued, with exponential backoff,
//! until it is back in, the player gives up (shutdown), or the reconnect window
//! closes (the match most likely ended; the server is gone).
use std::time::{Duration, Instant};

/// First retry delay, doubling per attempt up to `MAX_DELAY`.
pub const FIRST_DELAY: Duration = Duration::from_secs(1);
pub const MAX_DELAY: Duration = Duration::from_secs(10);
/// One connect-and-join attempt may take at most this long.
pub const ATTEMPT_TIMEOUT: Duration = Duration::from_secs(10);
/// Total time spent reconnecting before giving up.
pub const RECONNECT_WINDOW: Duration = Duration::from_secs(15 * 60);

/// Backoff schedule of one reconnect episode.
#[derive(Debug, Clone)]
pub struct RejoinBackoff {
    started: Instant,
    window: Duration,
    attempts: u32,
}

/// What the runtime does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejoinStep {
    /// Wait this long, then make attempt number `attempt` (1-based).
    Retry { attempt: u32, after: Duration },
    /// The reconnect window closed; stop with a final state.
    GiveUp { attempts: u32 },
}

impl RejoinBackoff {
    pub fn new(started: Instant) -> Self { Self::with_window(started, RECONNECT_WINDOW) }

    pub fn with_window(started: Instant, window: Duration) -> Self {
        Self { started, window, attempts: 0 }
    }

    pub fn attempts(&self) -> u32 { self.attempts }

    /// Delay before the next attempt: 0 for the first (the link is already 30 s
    /// silent), then 1, 2, 4, 8, 10, 10, ... seconds. Never schedules past the window.
    pub fn next(&mut self, now: Instant) -> RejoinStep {
        let elapsed = now.saturating_duration_since(self.started);
        let delay = match self.attempts {
            0 => Duration::ZERO,
            n => FIRST_DELAY.saturating_mul(1u32 << (n - 1).min(16)).min(MAX_DELAY),
        };
        if elapsed.saturating_add(delay) >= self.window {
            return RejoinStep::GiveUp { attempts: self.attempts };
        }
        self.attempts += 1;
        RejoinStep::Retry { attempt: self.attempts, after: delay }
    }
}

/// Safe reason shown with the RECONNECTING session state.
pub fn reconnecting_reason(attempt: u32) -> String {
    if attempt == 0 { "link_lost".to_owned() } else { format!("rejoin_attempt:{attempt}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_starts_immediately_doubles_and_caps() {
        let start = Instant::now();
        let mut backoff = RejoinBackoff::new(start);
        let delays: Vec<u64> = (0..8).map(|_| match backoff.next(start) {
            RejoinStep::Retry { after, .. } => after.as_secs(),
            RejoinStep::GiveUp { .. } => panic!("window not reached"),
        }).collect();
        assert_eq!(delays, vec![0, 1, 2, 4, 8, 10, 10, 10]);
        assert_eq!(backoff.attempts(), 8);
    }

    #[test]
    fn backoff_gives_up_when_the_window_closes() {
        let start = Instant::now();
        let mut backoff = RejoinBackoff::with_window(start, Duration::from_secs(5));
        assert_eq!(backoff.next(start), RejoinStep::Retry { attempt: 1, after: Duration::ZERO });
        assert_eq!(backoff.next(start + Duration::from_secs(1)), RejoinStep::Retry { attempt: 2, after: Duration::from_secs(1) });
        // 4 s elapsed + a 2 s delay would end past the 5 s window.
        assert_eq!(backoff.next(start + Duration::from_secs(4)), RejoinStep::GiveUp { attempts: 2 });
        assert_eq!(backoff.next(start + Duration::from_secs(60)), RejoinStep::GiveUp { attempts: 2 });
    }

    #[test]
    fn reasons_name_the_attempt() {
        assert_eq!(reconnecting_reason(0), "link_lost");
        assert_eq!(reconnecting_reason(3), "rejoin_attempt:3");
    }
}
