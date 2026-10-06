//! Bounded formal performance windows. These counters are wall-clock observations
//! only: they are not simulation inputs and must never be folded into a state hash.
use std::time::Instant;

pub const PERF_WINDOW_SAMPLES: u64 = 60;
pub const SCOPE_STATE_TICK: &str = "state_tick_excluding_transport_send";
pub const SCOPE_REPLICA_STEP: &str = "apply_encoded_frame";
pub const SCOPE_LOCALHOST_IPC: &str = "localhost_tcp_length_prefixed_frame";
pub const SCOPE_GAME_FRAME_INTERVAL: &str = "game_thread_delta_seconds";
pub const SCOPE_PRESENTATION_WORK: &str = "actor_tick_fplatformtime";
pub const MEAN_DEFINITION: &str = "integer_floor_sum_over_samples";
pub const NOT_A_STATISTIC: &str = "p95";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PerfWindowClosed {
    pub samples: u64,
    pub sum: u128,
    pub max: u128,
    pub mean: u128,
    pub window_duration_ns: u128,
}

#[derive(Clone, Debug)]
pub struct PerfWindow {
    capacity: u64,
    samples: u64,
    sum: u128,
    max: u128,
    opened_at: Option<Instant>,
}

impl Default for PerfWindow {
    fn default() -> Self {
        Self::with_capacity(PERF_WINDOW_SAMPLES)
    }
}

impl PerfWindow {
    pub fn with_capacity(capacity: u64) -> Self {
        assert!(capacity >= 1, "performance window requires at least one sample");
        Self {
            capacity,
            samples: 0,
            sum: 0,
            max: 0,
            opened_at: None,
        }
    }

    pub fn samples(&self) -> u64 {
        self.samples
    }

    pub fn reset(&mut self) {
        let capacity = self.capacity;
        *self = Self::with_capacity(capacity);
    }

    pub fn record(&mut self, value: u128) -> Option<PerfWindowClosed> {
        self.record_at(value, Instant::now())
    }

    pub fn record_at(&mut self, value: u128, now: Instant) -> Option<PerfWindowClosed> {
        if self.samples == 0 {
            self.opened_at = Some(now);
            self.sum = 0;
            self.max = 0;
        }
        self.samples += 1;
        self.sum = self.sum.saturating_add(value);
        self.max = self.max.max(value);
        if self.samples < self.capacity {
            return None;
        }
        let opened = self.opened_at.unwrap_or(now);
        let closed = PerfWindowClosed {
            samples: self.samples,
            sum: self.sum,
            max: self.max,
            mean: self.sum / u128::from(self.samples),
            window_duration_ns: now.saturating_duration_since(opened).as_nanos(),
        };
        self.reset();
        Some(closed)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PerfEmit {
    pub enable: Option<String>,
    pub summary: Option<String>,
}

pub struct ReplicaStepWindow {
    player_id: u32,
    team_id: u32,
    window: PerfWindow,
}

impl ReplicaStepWindow {
    pub fn new(player_id: u32, team_id: u32) -> Self {
        Self {
            player_id,
            team_id,
            window: PerfWindow::default(),
        }
    }

    pub fn samples(&self) -> u64 {
        self.window.samples()
    }

    pub fn reset(&mut self) {
        self.window.reset();
    }

    pub fn enable_line(&self) -> String {
        format!(
            "OM_PERF_ENABLE {}",
            json_object([
                num("v", 1u64),
                string("component", "client_runtime"),
                string("metric", "replica_step"),
                string("scope", SCOPE_REPLICA_STEP),
                string("level", "info"),
                num("window_samples", PERF_WINDOW_SAMPLES),
                string("excludes", "evidence_io,presentation,kcp"),
                num("player_id", self.player_id),
                num("team_id", self.team_id),
            ])
        )
    }

    /// `Ok(None)` is an unapplied frame. `Err` is a failed apply. Neither is a sample.
    pub fn accept_apply_result(&mut self, result: Result<Option<u128>, ()>) -> Option<String> {
        match result {
            Ok(Some(step_ns)) => self.observe_completed(step_ns),
            Ok(None) | Err(()) => None,
        }
    }

    pub fn observe_completed(&mut self, step_ns: u128) -> Option<String> {
        self.window.record(step_ns).map(|closed| {
            format!(
                "OM_PERF {}",
                time_summary(
                    "client_runtime",
                    "replica_step",
                    SCOPE_REPLICA_STEP,
                    &closed,
                    Some(self.player_id),
                    Some(self.team_id),
                )
            )
        })
    }
}

pub struct PresentationIpcMeter {
    player_id: u32,
    team_id: u32,
    connection: u64,
    send: PerfWindow,
    receive: PerfWindow,
}

impl PresentationIpcMeter {
    pub fn new(player_id: u32, team_id: u32, connection: u64) -> Self {
        Self::with_capacity(PERF_WINDOW_SAMPLES, player_id, team_id, connection)
    }

    pub fn with_capacity(capacity: u64, player_id: u32, team_id: u32, connection: u64) -> Self {
        Self {
            player_id,
            team_id,
            connection,
            send: PerfWindow::with_capacity(capacity),
            receive: PerfWindow::with_capacity(capacity),
        }
    }

    pub fn send_samples(&self) -> u64 {
        self.send.samples()
    }

    pub fn receive_samples(&self) -> u64 {
        self.receive.samples()
    }

    pub fn reset(&mut self) {
        self.send.reset();
        self.receive.reset();
    }

    pub fn enable_line(&self) -> String {
        format!(
            "OM_PERF_ENABLE {}",
            json_object([
                num("v", 1u64),
                string("component", "presentation_ipc"),
                string("metric", "wire_bytes"),
                string("scope", SCOPE_LOCALHOST_IPC),
                string("level", "info"),
                num("window_samples", PERF_WINDOW_SAMPLES),
                string("transport", "localhost_tcp"),
                string("not_transport", "kcp"),
                num("player_id", self.player_id),
                num("team_id", self.team_id),
                num("connection", self.connection),
            ])
        )
    }

    pub fn observe_send_result<E>(&mut self, result: Result<u64, E>) -> Option<String> {
        match result {
            Ok(bytes) => self.observe_send(bytes),
            Err(_) => None,
        }
    }

    pub fn observe_receive_result<E>(&mut self, result: Result<u64, E>) -> Option<String> {
        match result {
            Ok(bytes) => self.observe_receive(bytes),
            Err(_) => None,
        }
    }

    pub fn observe_send(&mut self, wire_bytes: u64) -> Option<String> {
        self.observe_send_at(wire_bytes, Instant::now())
    }

    pub fn observe_receive(&mut self, wire_bytes: u64) -> Option<String> {
        self.observe_receive_at(wire_bytes, Instant::now())
    }

    pub fn observe_send_at(&mut self, wire_bytes: u64, now: Instant) -> Option<String> {
        self.observe_at("send", true, wire_bytes, now)
    }

    pub fn observe_receive_at(&mut self, wire_bytes: u64, now: Instant) -> Option<String> {
        self.observe_at("receive", false, wire_bytes, now)
    }

    fn observe_at(
        &mut self,
        direction: &'static str,
        send: bool,
        wire_bytes: u64,
        now: Instant,
    ) -> Option<String> {
        // A zero-length write is not a length-prefixed frame.
        if wire_bytes == 0 {
            return None;
        }
        let player_id = self.player_id;
        let team_id = self.team_id;
        let connection = self.connection;
        let window = if send { &mut self.send } else { &mut self.receive };
        window.record_at(u128::from(wire_bytes), now).map(|closed| {
            let rate = (closed.window_duration_ns > 0).then(|| {
                closed.sum.saturating_mul(1_000_000_000) / closed.window_duration_ns
            });
            format!(
                "OM_PERF {}",
                json_object([
                    num("v", 1u64),
                    string("component", "presentation_ipc"),
                    string("metric", "wire_bytes"),
                    string("scope", SCOPE_LOCALHOST_IPC),
                    string("unit", "byte"),
                    string("direction", direction),
                    num("samples", closed.samples),
                    num("count", closed.samples),
                    num("messages", closed.samples),
                    num("sum", closed.sum),
                    num("bytes", closed.sum),
                    num("max", closed.max),
                    num("mean", closed.mean),
                    string("mean_definition", MEAN_DEFINITION),
                    string("not_a_statistic", NOT_A_STATISTIC),
                    num("window_duration_ns", closed.window_duration_ns),
                    match rate {
                        Some(value) => num("rate_bytes_per_s", value),
                        None => null("rate_bytes_per_s"),
                    },
                    string("rate_definition", "integer_floor_bytes_per_window_second"),
                    string("rate_is_not", "frame_interval"),
                    num("player_id", player_id),
                    num("team_id", team_id),
                    num("connection", connection),
                ])
            )
        })
    }
}

pub fn format_tick_compute_enable() -> String {
    format!(
        "OM_PERF_ENABLE {}",
        json_object([
            num("v", 1u64),
            string("component", "server"),
            string("metric", "tick_compute"),
            string("scope", SCOPE_STATE_TICK),
            string("level", "info"),
            num("window_samples", PERF_WINDOW_SAMPLES),
            string(
                "excludes",
                "scheduler_sleep,reliable_send_timeout,warmup,paused,finished",
            ),
            bool_value("hash_input", false),
        ])
    )
}

pub fn format_tick_compute(closed: &PerfWindowClosed) -> String {
    format!(
        "OM_PERF {}",
        time_summary("server", "tick_compute", SCOPE_STATE_TICK, closed, None, None)
    )
}

fn time_summary(
    component: &'static str,
    metric: &'static str,
    scope: &'static str,
    closed: &PerfWindowClosed,
    player_id: Option<u32>,
    team_id: Option<u32>,
) -> String {
    let mut fields = vec![
        num("v", 1u64),
        string("component", component),
        string("metric", metric),
        string("scope", scope),
        string("unit", "ns"),
        num("samples", closed.samples),
        num("count", closed.samples),
        num("sum", closed.sum),
        num("max", closed.max),
        num("mean", closed.mean),
        string("mean_definition", MEAN_DEFINITION),
        string("not_a_statistic", NOT_A_STATISTIC),
        num("window_duration_ns", closed.window_duration_ns),
    ];
    if let Some(player_id) = player_id {
        fields.push(num("player_id", player_id));
    }
    if let Some(team_id) = team_id {
        fields.push(num("team_id", team_id));
    }
    json_object(fields)
}

enum JsonField {
    Number(&'static str, u128),
    Text(&'static str, &'static str),
    Bool(&'static str, bool),
    Null(&'static str),
}

fn num(key: &'static str, value: impl Into<u128>) -> JsonField {
    JsonField::Number(key, value.into())
}

fn string(key: &'static str, value: &'static str) -> JsonField {
    JsonField::Text(key, value)
}

fn bool_value(key: &'static str, value: bool) -> JsonField {
    JsonField::Bool(key, value)
}

fn null(key: &'static str) -> JsonField {
    JsonField::Null(key)
}

fn json_object(fields: impl IntoIterator<Item = JsonField>) -> String {
    let mut out = String::from("{");
    for (index, field) in fields.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        match field {
            JsonField::Number(key, value) => {
                out.push('"');
                out.push_str(key);
                out.push_str("\":");
                out.push_str(&value.to_string());
            }
            JsonField::Text(key, value) => {
                debug_assert!(!value.contains(['"', '\\']));
                out.push('"');
                out.push_str(key);
                out.push_str("\":\"");
                out.push_str(value);
                out.push('"');
            }
            JsonField::Bool(key, value) => {
                out.push('"');
                out.push_str(key);
                out.push_str(if value { "\":true" } else { "\":false" });
            }
            JsonField::Null(key) => {
                out.push('"');
                out.push_str(key);
                out.push_str("\":null");
            }
        }
    }
    out.push('}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn parse_om(line: &str) -> serde_json::Value {
        let json = line
            .strip_prefix("OM_PERF ")
            .or_else(|| line.strip_prefix("OM_PERF_ENABLE "))
            .expect("perf marker");
        serde_json::from_str(json).expect("perf json")
    }

    #[test]
    fn formal_perf_window_stats_reset_and_integer_mean() {
        let mut window = PerfWindow::with_capacity(3);
        let opened = Instant::now();
        assert!(window.record_at(10, opened).is_none());
        assert_eq!(window.samples(), 1);
        assert!(window
            .record_at(31, opened + Duration::from_millis(4))
            .is_none());
        let closed = window
            .record_at(5, opened + Duration::from_millis(9))
            .expect("third sample closes the window");
        assert_eq!(closed.samples, 3);
        assert_eq!(closed.sum, 46);
        assert_eq!(closed.max, 31);
        assert_eq!(closed.mean, 15);
        assert_eq!(closed.window_duration_ns, 9_000_000);
        assert_eq!(window.samples(), 0);
        assert!(window
            .record_at(7, opened + Duration::from_millis(20))
            .is_none());
        assert_eq!(window.samples(), 1);
        window.reset();
        assert_eq!(window.samples(), 0);
        assert!(window.record_at(4, opened).is_none());
        assert_eq!(window.samples(), 1);
    }

    #[test]
    fn formal_perf_replica_step_ignores_unapplied_and_failed_frames() {
        let mut steps = ReplicaStepWindow::new(7, 2);
        let enable = parse_om(&steps.enable_line());
        assert_eq!(enable["component"], "client_runtime");
        assert_eq!(enable["scope"], SCOPE_REPLICA_STEP);
        assert_eq!(enable["excludes"], "evidence_io,presentation,kcp");
        assert_eq!(enable["player_id"], 7);
        assert!(steps.accept_apply_result(Ok(None)).is_none());
        assert!(steps.accept_apply_result(Err(())).is_none());
        assert_eq!(steps.samples(), 0);
        assert!(steps.accept_apply_result(Ok(Some(25))).is_none());
        assert_eq!(steps.samples(), 1);
        steps.reset();
        assert_eq!(steps.samples(), 0);
    }

    #[test]
    fn formal_perf_ipc_rejects_incomplete_and_isolates_connections() {
        let opened = Instant::now();
        let mut server = PresentationIpcMeter::with_capacity(2, 1, 1, 11);
        let mut client = PresentationIpcMeter::with_capacity(2, 1, 1, 12);
        assert!(server.observe_send_result(Err::<u64, &str>("broken")).is_none());
        assert!(server.observe_receive_result(Err::<u64, &str>("broken")).is_none());
        assert!(server.observe_send_at(0, opened).is_none());
        assert_eq!(server.send_samples(), 0);
        assert_eq!(server.receive_samples(), 0);
        assert!(client.observe_send_at(40, opened).is_none());
        assert_eq!(server.send_samples(), 0, "client write must not enter the server meter");
        assert_eq!(client.send_samples(), 1);
        assert!(server.observe_receive_at(10, opened).is_none());
        let line = server
            .observe_receive_at(30, opened + Duration::from_secs(2))
            .expect("receive window");
        let json = parse_om(&line);
        assert_eq!(json["direction"], "receive");
        assert_eq!(json["scope"], SCOPE_LOCALHOST_IPC);
        assert_eq!(json["unit"], "byte");
        assert_eq!(json["samples"], 2);
        assert_eq!(json["messages"], 2);
        assert_eq!(json["bytes"], 40);
        assert_eq!(json["sum"], 40);
        assert_eq!(json["max"], 30);
        assert_eq!(json["mean"], 20);
        assert_eq!(json["mean_definition"], MEAN_DEFINITION);
        assert_eq!(json["not_a_statistic"], NOT_A_STATISTIC);
        assert_eq!(json["window_duration_ns"], 2_000_000_000u64);
        assert_eq!(json["rate_bytes_per_s"], 20);
        assert_eq!(json["rate_is_not"], "frame_interval");
        assert_eq!(json["connection"], 11);
        assert!(json.get("interval_is").is_none());
        assert_eq!(server.send_samples(), 0);
        assert_eq!(client.receive_samples(), 0);
        let zero_duration = PresentationIpcMeter::with_capacity(1, 3, 1, 4)
            .observe_send_at(8, opened)
            .expect("single sample");
        assert!(parse_om(&zero_duration)["rate_bytes_per_s"].is_null());
        server.reset();
        assert_eq!(server.receive_samples(), 0);
    }
}
