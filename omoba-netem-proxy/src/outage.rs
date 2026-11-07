//! Scheduled single-team outages: every datagram of the team's route is
//! dropped in both directions while a window is active, including datagrams
//! already queued for delayed release. Windows are relative to an anchor
//! (proxy readiness by default, or a control `anchor` message such as "all
//! clients admitted"); actual start/end are observed by the route loops.

use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

use serde::Serialize;

use crate::{delay::Direction, NetemError, Result};

pub const MAX_WINDOWS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct OutageWindow {
    pub team_id: u32,
    pub at_ms: u64,
    pub duration_ms: u64,
}

impl OutageWindow {
    /// `team=1,at_ms=120000,duration_ms=10000`
    pub fn parse(spec: &str) -> Result<Self> {
        let (mut team_id, mut at_ms, mut duration_ms) = (None, None, None);
        for part in spec.split(',') {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| NetemError::Config(format!("outage field must be key=value: {part}")))?;
            let number = || {
                value
                    .parse::<u64>()
                    .map_err(|_| NetemError::Config(format!("outage {key} must be an integer")))
            };
            match key {
                "team" => team_id = Some(number()?),
                "at_ms" => at_ms = Some(number()?),
                "duration_ms" => duration_ms = Some(number()?),
                _ => return Err(NetemError::Config(format!("unknown outage field {key}"))),
            }
        }
        let team_id = team_id
            .filter(|team| matches!(team, 1 | 2))
            .ok_or_else(|| NetemError::Config("outage team must be 1 or 2".into()))? as u32;
        let at_ms = at_ms.ok_or_else(|| NetemError::Config("outage at_ms is required".into()))?;
        let duration_ms = duration_ms
            .filter(|value| *value > 0)
            .ok_or_else(|| NetemError::Config("outage duration_ms must be positive".into()))?;
        at_ms
            .checked_add(duration_ms)
            .filter(|end| *end <= 24 * 3600 * 1000)
            .ok_or_else(|| NetemError::Config("outage window must end within 24 h".into()))?;
        Ok(Self { team_id, at_ms, duration_ms })
    }

    fn end_ms(&self) -> u64 {
        self.at_ms + self.duration_ms
    }
}

pub fn validate(windows: &[OutageWindow]) -> Result<()> {
    if windows.len() > MAX_WINDOWS {
        return Err(NetemError::Config(format!("at most {MAX_WINDOWS} outage windows")));
    }
    for (index, left) in windows.iter().enumerate() {
        for right in &windows[index + 1..] {
            if left.team_id == right.team_id && left.at_ms < right.end_ms() && right.at_ms < left.end_ms() {
                return Err(NetemError::Config(format!("overlapping outage windows for team {}", left.team_id)));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Default, Serialize, PartialEq, Eq)]
pub struct OutageEvidence {
    pub team_id: u32,
    pub scheduled_at_ms: u64,
    pub scheduled_duration_ms: u64,
    /// Proxy-monotonic ms of the anchor the window was measured from.
    pub anchor_ms: Option<u64>,
    /// Proxy-monotonic ms when a route loop first observed the window active / ended.
    pub actual_start_ms: Option<u64>,
    pub actual_end_ms: Option<u64>,
    pub dropped_client_to_server: u64,
    pub dropped_server_to_client: u64,
}

struct State {
    anchor: Option<Instant>,
    rows: Vec<OutageEvidence>,
}

pub struct OutageSchedule {
    started: Instant,
    windows: Vec<OutageWindow>,
    state: Mutex<State>,
}

/// A window transition observed by a route loop, for its log line.
#[derive(Debug, PartialEq, Eq)]
pub enum Transition {
    Started { window: usize, at_ms: u64 },
    Ended { window: usize, at_ms: u64, dropped_client_to_server: u64, dropped_server_to_client: u64 },
}

impl OutageSchedule {
    pub fn new(started: Instant, windows: Vec<OutageWindow>) -> Self {
        let rows = windows
            .iter()
            .map(|window| OutageEvidence {
                team_id: window.team_id,
                scheduled_at_ms: window.at_ms,
                scheduled_duration_ms: window.duration_ms,
                ..Default::default()
            })
            .collect();
        Self { started, windows, state: Mutex::new(State { anchor: None, rows }) }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn ms(&self, at: Instant) -> u64 {
        at.saturating_duration_since(self.started).as_millis() as u64
    }

    /// Sets the anchor once windows can no longer be in flight. A re-anchor
    /// after any window started is refused so evidence stays unambiguous.
    pub fn anchor(&self, now: Instant) -> Result<()> {
        let mut state = self.lock();
        if state.rows.iter().any(|row| row.actual_start_ms.is_some()) {
            return Err(NetemError::Config("outage anchor cannot move after a window started".into()));
        }
        state.anchor = Some(now);
        let anchor_ms = self.ms(now);
        for row in &mut state.rows {
            row.anchor_ms = Some(anchor_ms);
        }
        Ok(())
    }

    /// Default anchor (proxy readiness) unless a control anchor already arrived.
    pub fn anchor_if_unset(&self, now: Instant) {
        if self.lock().anchor.is_none() {
            let _ = self.anchor(now);
        }
    }

    fn bounds(&self, anchor: Instant, window: &OutageWindow) -> (Instant, Instant) {
        (anchor + Duration::from_millis(window.at_ms), anchor + Duration::from_millis(window.end_ms()))
    }

    /// Index of the team's window active at `now`.
    pub fn active(&self, team_id: u32, now: Instant) -> Option<usize> {
        let anchor = self.lock().anchor?;
        self.windows.iter().position(|window| {
            let (start, end) = self.bounds(anchor, window);
            window.team_id == team_id && start <= now && now < end
        })
    }

    /// Next window boundary for the team after `now`, so the route wakes on time.
    pub fn next_boundary(&self, team_id: u32, now: Instant) -> Option<Instant> {
        let anchor = self.lock().anchor?;
        self.windows
            .iter()
            .filter(|window| window.team_id == team_id)
            .flat_map(|window| {
                let (start, end) = self.bounds(anchor, window);
                [start, end]
            })
            .filter(|boundary| *boundary > now)
            .min()
    }

    /// Records start/end transitions the team's route observes at `now`.
    pub fn observe(&self, team_id: u32, now: Instant) -> Vec<Transition> {
        let mut state = self.lock();
        let Some(anchor) = state.anchor else { return Vec::new() };
        let now_ms = self.ms(now);
        let mut out = Vec::new();
        for (index, window) in self.windows.iter().enumerate() {
            if window.team_id != team_id {
                continue;
            }
            let (start, end) = self.bounds(anchor, window);
            let row = &mut state.rows[index];
            if now >= start && row.actual_start_ms.is_none() {
                row.actual_start_ms = Some(now_ms);
                out.push(Transition::Started { window: index, at_ms: now_ms });
            }
            if now >= end && row.actual_start_ms.is_some() && row.actual_end_ms.is_none() {
                row.actual_end_ms = Some(now_ms);
                out.push(Transition::Ended {
                    window: index,
                    at_ms: now_ms,
                    dropped_client_to_server: row.dropped_client_to_server,
                    dropped_server_to_client: row.dropped_server_to_client,
                });
            }
        }
        out
    }

    pub fn record_drop(&self, window: usize, direction: Direction) {
        let mut state = self.lock();
        let row = &mut state.rows[window];
        match direction {
            Direction::ClientToServer => row.dropped_client_to_server += 1,
            Direction::ServerToClient => row.dropped_server_to_client += 1,
        }
    }

    pub fn evidence(&self) -> Vec<OutageEvidence> {
        self.lock().rows.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_rejects_malformed_windows() {
        assert_eq!(
            OutageWindow::parse("team=1,at_ms=120000,duration_ms=10000").unwrap(),
            OutageWindow { team_id: 1, at_ms: 120_000, duration_ms: 10_000 }
        );
        for bad in ["team=3,at_ms=1,duration_ms=1", "team=1,at_ms=1,duration_ms=0", "team=1,duration_ms=5",
            "team=1,at_ms=x,duration_ms=5", "team=1,at_ms=1,duration_ms=5,extra=1", "team=1;at_ms=1"] {
            assert!(OutageWindow::parse(bad).is_err(), "{bad}");
        }
        let a = OutageWindow { team_id: 1, at_ms: 0, duration_ms: 10 };
        assert!(validate(&[a, OutageWindow { team_id: 1, at_ms: 5, duration_ms: 10 }]).is_err());
        assert!(validate(&[a, OutageWindow { team_id: 1, at_ms: 10, duration_ms: 10 }]).is_ok());
        assert!(validate(&[a, OutageWindow { team_id: 2, at_ms: 5, duration_ms: 10 }]).is_ok());
    }

    #[test]
    fn windows_follow_the_anchor_and_record_actual_bounds_and_drops() {
        let started = Instant::now();
        let schedule = OutageSchedule::new(started, vec![
            OutageWindow { team_id: 1, at_ms: 100, duration_ms: 50 },
            OutageWindow { team_id: 1, at_ms: 300, duration_ms: 50 },
        ]);
        let at = |ms| started + Duration::from_millis(ms);
        assert_eq!(schedule.active(1, at(120)), None, "unanchored schedule never drops");
        schedule.anchor(at(1000)).unwrap();
        assert_eq!(schedule.next_boundary(1, at(1000)), Some(at(1100)));
        assert_eq!(schedule.next_boundary(2, at(1000)), None);
        assert_eq!(schedule.active(1, at(1099)), None);
        assert_eq!(schedule.active(1, at(1100)), Some(0));
        assert_eq!(schedule.active(2, at(1100)), None, "other team unaffected");
        assert_eq!(schedule.observe(1, at(1101)), vec![Transition::Started { window: 0, at_ms: 1101 }]);
        schedule.record_drop(0, Direction::ClientToServer);
        schedule.record_drop(0, Direction::ServerToClient);
        schedule.record_drop(0, Direction::ServerToClient);
        assert!(schedule.anchor(at(2000)).is_err(), "no re-anchor once a window started");
        assert_eq!(schedule.active(1, at(1150)), None, "end is exclusive");
        assert_eq!(schedule.observe(1, at(1150)), vec![Transition::Ended {
            window: 0, at_ms: 1150, dropped_client_to_server: 1, dropped_server_to_client: 2 }]);
        assert!(schedule.observe(1, at(1151)).is_empty());
        let rows = schedule.evidence();
        assert_eq!((rows[0].anchor_ms, rows[0].actual_start_ms, rows[0].actual_end_ms), (Some(1000), Some(1101), Some(1150)));
        assert_eq!((rows[1].actual_start_ms, rows[1].dropped_client_to_server), (None, 0));
    }

    #[test]
    fn default_anchor_does_not_override_a_control_anchor() {
        let started = Instant::now();
        let schedule = OutageSchedule::new(started, vec![OutageWindow { team_id: 2, at_ms: 10, duration_ms: 10 }]);
        schedule.anchor(started + Duration::from_millis(500)).unwrap();
        schedule.anchor_if_unset(started + Duration::from_millis(900));
        assert_eq!(schedule.evidence()[0].anchor_ms, Some(500));
    }
}
