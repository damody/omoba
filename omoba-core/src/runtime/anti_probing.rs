use std::collections::{BTreeMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SecureTargetReference {
    pub replica_id: u64,
    pub view_epoch: u64,
    pub disclosure_epoch: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GeneralizedInputRejection {
    InvalidTarget,
}

#[derive(Clone, Copy, Debug)]
pub struct TargetValidationFacts {
    pub session_team_matches: bool,
    pub view_epoch_matches: bool,
    pub disclosure_epoch_matches: bool,
    pub visible_at_input_tick: bool,
    pub actor_owned_by_session: bool,
    pub replica_mapping_exists: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplicaValidationRecord {
    pub canonical_id: u64,
    pub disclosure_epoch: u64,
    pub owner_team: Option<u32>,
}

#[derive(Clone, Debug, Default)]
pub struct TeamInputValidationView {
    pub view_epoch: u64,
    pub replicas: BTreeMap<u64, ReplicaValidationRecord>,
    pub visible_by_tick: BTreeMap<u64, std::collections::BTreeSet<u64>>,
}

#[derive(Clone, Debug, Default)]
pub struct SecureInputValidationSnapshot {
    pub teams: BTreeMap<u32, TeamInputValidationView>,
}

pub type SharedSecureInputValidationSnapshot = Arc<Mutex<SecureInputValidationSnapshot>>;

impl SecureInputValidationSnapshot {
    pub fn validate(
        &self,
        authenticated_team: u32,
        input_tick: u64,
        actor: SecureTargetReference,
        target: SecureTargetReference,
    ) -> Result<(), GeneralizedInputRejection> {
        let team = self.teams.get(&authenticated_team);
        let actor_record = team.and_then(|view| view.replicas.get(&actor.replica_id));
        let target_record = team.and_then(|view| view.replicas.get(&target.replica_id));
        let visible = |record: Option<&ReplicaValidationRecord>, tick: u64| {
            team.and_then(|view| {
                view.visible_by_tick
                    .range(..=tick)
                    .next_back()
                    .map(|(_, set)| set)
            })
            .zip(record)
            .is_some_and(|(set, record)| set.contains(&record.canonical_id))
        };
        validate_secure_target(TargetValidationFacts {
            session_team_matches: team.is_some(),
            view_epoch_matches: team.is_some_and(|view| {
                actor.view_epoch == view.view_epoch && target.view_epoch == view.view_epoch
            }),
            disclosure_epoch_matches: actor_record
                .is_some_and(|record| record.disclosure_epoch == actor.disclosure_epoch)
                && target_record
                    .is_some_and(|record| record.disclosure_epoch == target.disclosure_epoch),
            visible_at_input_tick: visible(actor_record, input_tick)
                && visible(target_record, input_tick),
            actor_owned_by_session: actor_record
                .is_some_and(|record| record.owner_team == Some(authenticated_team)),
            replica_mapping_exists: actor_record.is_some() && target_record.is_some(),
        })
    }
}

pub fn validate_secure_target(
    facts: TargetValidationFacts,
) -> Result<(), GeneralizedInputRejection> {
    (facts.session_team_matches
        && facts.view_epoch_matches
        && facts.disclosure_epoch_matches
        && facts.visible_at_input_tick
        && facts.actor_owned_by_session
        && facts.replica_mapping_exists)
        .then_some(())
        .ok_or(GeneralizedInputRejection::InvalidTarget)
}

pub const INVALID_TARGET_TIMING_BUCKET: Duration = Duration::from_millis(8);

#[derive(Clone, Debug)]
pub struct InvalidReferenceRateLimiter {
    max_events: usize,
    window_ticks: u64,
    events: BTreeMap<String, VecDeque<u64>>,
}

impl InvalidReferenceRateLimiter {
    pub fn new(max_events: usize, window_ticks: u64) -> Self {
        Self {
            max_events: max_events.max(1),
            window_ticks: window_ticks.max(1),
            events: BTreeMap::new(),
        }
    }
    pub fn admit(&mut self, session_id: &str, tick: u64) -> bool {
        let events = self.events.entry(session_id.to_owned()).or_default();
        while events
            .front()
            .is_some_and(|old| tick.saturating_sub(*old) >= self.window_ticks)
        {
            events.pop_front();
        }
        if events.len() >= self.max_events {
            return false;
        }
        events.push_back(tick);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlayerSinkKind {
    Log,
    Replay,
    CrashBundle,
    Trace,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiagnosticRecord {
    pub fields: BTreeMap<String, String>,
}

pub const PLAYER_SAFE_DIAGNOSTIC_FIELDS: &[&str] = &[
    "team_id",
    "frame_sequence",
    "replica_tick",
    "reason_class",
    "safe_component_path",
    "queue_depth",
    "audit_lag_ticks",
];

pub fn redact_player_diagnostic(
    _sink: PlayerSinkKind,
    record: DiagnosticRecord,
    metrics: &SelectiveSecurityMetrics,
) -> DiagnosticRecord {
    let mut redacted = DiagnosticRecord::default();
    for (key, value) in record.fields {
        if PLAYER_SAFE_DIAGNOSTIC_FIELDS.contains(&key.as_str()) {
            redacted.fields.insert(key, value);
        } else {
            metrics
                .redaction_violation_count
                .fetch_add(1, Ordering::Relaxed);
        }
    }
    redacted
}

/// The only player-facing diagnostic fan-out. Every sink passes through the
/// same allowlist so adding a replay/crash/trace exporter cannot bypass the
/// fog boundary accidentally.
#[derive(Default)]
pub struct PlayerDiagnosticSinks {
    pub log: Vec<DiagnosticRecord>,
    pub replay: Vec<DiagnosticRecord>,
    pub crash_bundle: Vec<DiagnosticRecord>,
    pub trace: Vec<DiagnosticRecord>,
}

impl PlayerDiagnosticSinks {
    pub fn emit(
        &mut self,
        sink: PlayerSinkKind,
        record: DiagnosticRecord,
        metrics: &SelectiveSecurityMetrics,
    ) {
        let record = redact_player_diagnostic(sink, record, metrics);
        match sink {
            PlayerSinkKind::Log => self.log.push(record),
            PlayerSinkKind::Replay => self.replay.push(record),
            PlayerSinkKind::CrashBundle => self.crash_bundle.push(record),
            PlayerSinkKind::Trace => self.trace.push(record),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServerAdminDiagnosticCapability([u8; 32]);

impl ServerAdminDiagnosticCapability {
    pub fn from_server_secret(secret: [u8; 32]) -> Self {
        Self(secret)
    }
}

pub struct AdminDiagnosticTransport {
    capability: ServerAdminDiagnosticCapability,
}

/// Deliberately distinct from `PlayerDiagnosticSinks`: full records can only
/// be emitted after possession of a server-created capability is proven.
#[derive(Default)]
pub struct FullAdminDiagnosticSink {
    records: Vec<DiagnosticRecord>,
}

impl FullAdminDiagnosticSink {
    pub fn emit(
        &mut self,
        transport: &AdminDiagnosticTransport,
        presented: &ServerAdminDiagnosticCapability,
        record: DiagnosticRecord,
    ) -> bool {
        if !transport.authorize(presented) {
            return false;
        }
        self.records.push(record);
        true
    }

    pub fn records(
        &self,
        transport: &AdminDiagnosticTransport,
        presented: &ServerAdminDiagnosticCapability,
    ) -> Option<&[DiagnosticRecord]> {
        transport
            .authorize(presented)
            .then_some(self.records.as_slice())
    }
}

impl AdminDiagnosticTransport {
    pub fn new(capability: ServerAdminDiagnosticCapability) -> Self {
        Self { capability }
    }
    pub fn authorize(&self, presented: &ServerAdminDiagnosticCapability) -> bool {
        constant_time_eq(&self.capability.0, &presented.0)
    }
}

fn constant_time_eq(left: &[u8; 32], right: &[u8; 32]) -> bool {
    left.iter()
        .zip(right)
        .fold(0u8, |diff, (a, b)| diff | (a ^ b))
        == 0
}

#[derive(Default)]
pub struct SelectiveSecurityMetrics {
    pub visibility_transition_count: AtomicU64,
    pub steady_state_padding_bytes: AtomicU64,
    pub encoded_frame_bytes: AtomicU64,
    pub outbound_queue_depth: AtomicU64,
    pub observer_audit_lag_ticks: AtomicU64,
    pub coverage_gap_count: AtomicU64,
    pub authority_repair_count: AtomicU64,
    pub authority_rebase_count: AtomicU64,
    pub redaction_violation_count: AtomicU64,
    pub reveal_burst_bytes: AtomicU64,
    pub rebase_burst_bytes: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectiveSecurityMetricsSnapshot {
    pub visibility_transition_count: u64,
    pub steady_state_padding_bytes: u64,
    pub encoded_frame_bytes: u64,
    pub outbound_queue_depth: u64,
    pub observer_audit_lag_ticks: u64,
    pub coverage_gap_count: u64,
    pub authority_repair_count: u64,
    pub authority_rebase_count: u64,
    pub redaction_violation_count: u64,
    pub reveal_burst_bytes: u64,
    pub rebase_burst_bytes: u64,
}

impl SelectiveSecurityMetrics {
    pub fn snapshot(&self) -> SelectiveSecurityMetricsSnapshot {
        let load = |value: &AtomicU64| value.load(Ordering::Relaxed);
        SelectiveSecurityMetricsSnapshot {
            visibility_transition_count: load(&self.visibility_transition_count),
            steady_state_padding_bytes: load(&self.steady_state_padding_bytes),
            encoded_frame_bytes: load(&self.encoded_frame_bytes),
            outbound_queue_depth: load(&self.outbound_queue_depth),
            observer_audit_lag_ticks: load(&self.observer_audit_lag_ticks),
            coverage_gap_count: load(&self.coverage_gap_count),
            authority_repair_count: load(&self.authority_repair_count),
            authority_rebase_count: load(&self.authority_rebase_count),
            redaction_violation_count: load(&self.redaction_violation_count),
            reveal_burst_bytes: load(&self.reveal_burst_bytes),
            rebase_burst_bytes: load(&self.rebase_burst_bytes),
        }
    }
}
