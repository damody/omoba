//! Match-owned Team 1 and Team 2 observer replicas.
use crate::game_proto::{TeamGameStart, TeamTickFrame};
use crate::runtime::{FrameApplyResult, SelectiveReplicaRuntime, SpecsDisclosedWorldStepper};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use prost::Message;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Instant;

pub const SUPPORTED_REPLICA_TEAMS: [u32; 2] = [1, 2];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoverageGap {
    pub team_id: u32,
    pub first_unverified_sequence: u64,
    pub observed_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserverMismatch {
    pub team_id: u32,
    pub frame_sequence: u64,
    pub replica_tick: u64,
    pub authority_revision: u64,
    pub first_divergent_tick: u64,
    pub expected_hash: [u8; 32],
    pub observed_hash: [u8; 32],
    pub post_repair_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ReplicaCheckpointKey {
    pub team_id: u32,
    pub replica_tick: u64,
    pub team_sequence: u64,
    pub authority_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObserverCheckpointReport {
    pub key: ReplicaCheckpointKey,
    pub expected_hash: Option<[u8; 32]>,
    pub observer_pre_repair_hash: [u8; 32],
    pub observer_post_repair_hash: [u8; 32],
    pub encoded_frame_hash: [u8; 32],
}

enum ValidationMessage {
    Bootstrap(Arc<[u8]>),
    Frame {
        sequence: u64,
        replica_tick: u64,
        encoded: Arc<[u8]>,
    },
    EndTeam,
    Shutdown,
}

#[derive(Default)]
pub struct TeamObserverMetrics {
    pub current_replica_tick: AtomicU64,
    pub queue_depth: AtomicUsize,
    pub lag_ticks: AtomicU64,
    pub coverage_gap_count: AtomicU64,
    pub verified_frame_count: AtomicU64,
    pub verified_through_sequence: AtomicU64,
    pub pre_repair_mismatch_count: AtomicU64,
    pub step_samples_ns: Mutex<Vec<u64>>,
    pub script_phase_samples_ns: Mutex<Vec<u64>>,
    pub rebootstrap_count: AtomicU64,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DurationPercentilesNs {
    pub p50: u64,
    pub p95: u64,
    pub p99: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationCoverageStatus {
    VerifiedThrough(u64),
    UnverifiedGap { first_sequence: u64 },
}

impl TeamObserverMetrics {
    pub fn step_percentiles(&self) -> DurationPercentilesNs {
        percentiles(&self.step_samples_ns.lock().expect("samples mutex"))
    }

    pub fn script_percentiles(&self) -> DurationPercentilesNs {
        percentiles(
            &self
                .script_phase_samples_ns
                .lock()
                .expect("script samples mutex"),
        )
    }
}

const MAX_DURATION_SAMPLES: usize = 4096;

fn record_duration(samples: &Mutex<Vec<u64>>, value: u64) {
    let mut samples = samples.lock().expect("duration samples mutex");
    if samples.len() == MAX_DURATION_SAMPLES {
        samples.drain(..MAX_DURATION_SAMPLES / 2);
    }
    samples.push(value);
}

fn percentiles(samples: &[u64]) -> DurationPercentilesNs {
    if samples.is_empty() {
        return DurationPercentilesNs::default();
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let at = |percent: usize| sorted[((sorted.len() - 1) * percent) / 100];
    DurationPercentilesNs {
        p50: at(50),
        p95: at(95),
        p99: at(99),
    }
}

#[derive(Default)]
pub struct ObserverValidationMetrics {
    pub teams: BTreeMap<u32, Arc<TeamObserverMetrics>>,
    pub queue_depth: AtomicUsize,
    pub audit_lag_ticks: AtomicU64,
    pub coverage_gap_count: AtomicU64,
    pub verified_frame_count: AtomicU64,
    pub verified_through_sequence: Mutex<BTreeMap<u32, u64>>,
}

#[derive(Clone)]
pub struct ObserverValidationTap {
    senders: Arc<BTreeMap<u32, Sender<ValidationMessage>>>,
    pub metrics: Arc<ObserverValidationMetrics>,
    gaps: Arc<Mutex<Vec<CoverageGap>>>,
}

impl ObserverValidationTap {
    pub fn try_bootstrap(&self, encoded: Arc<[u8]>) {
        if let Ok(start) = TeamGameStart::decode(encoded.as_ref()) {
            self.try_send(start.team_id, ValidationMessage::Bootstrap(encoded), None);
        }
    }
    pub fn try_frame(&self, team_id: u32, sequence: u64, replica_tick: u64, encoded: Arc<[u8]>) {
        self.try_send(
            team_id,
            ValidationMessage::Frame {
                sequence,
                replica_tick,
                encoded,
            },
            Some(sequence),
        );
    }
    pub fn end_team(&self, team_id: u32) {
        self.try_send(team_id, ValidationMessage::EndTeam, None);
    }
    fn try_send(&self, team_id: u32, message: ValidationMessage, sequence: Option<u64>) {
        let result = self.senders.get(&team_id).map(|tx| tx.try_send(message));
        match result {
            Some(Ok(())) => {
                self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed);
                if let Some(team) = self.metrics.teams.get(&team_id) {
                    team.queue_depth.fetch_add(1, Ordering::Relaxed);
                }
            }
            Some(Err(TrySendError::Full(_))) | Some(Err(TrySendError::Disconnected(_))) | None => {
                record_gap(
                    &self.gaps,
                    &self.metrics,
                    team_id,
                    sequence.unwrap_or_default(),
                )
            }
        }
    }
    pub fn coverage_gaps(&self) -> Vec<CoverageGap> {
        self.gaps.lock().expect("gap mutex").clone()
    }
    pub fn take_coverage_gaps(&self) -> Vec<CoverageGap> {
        std::mem::take(&mut *self.gaps.lock().expect("gap mutex"))
    }

    pub fn coverage_status(&self, team_id: u32) -> ValidationCoverageStatus {
        if let Some(first) = self
            .gaps
            .lock()
            .expect("gap mutex")
            .iter()
            .filter(|gap| gap.team_id == team_id)
            .map(|gap| gap.first_unverified_sequence)
            .min()
        {
            return ValidationCoverageStatus::UnverifiedGap {
                first_sequence: first,
            };
        }
        let verified = self.metrics.teams.get(&team_id).map_or(0, |team| {
            team.verified_through_sequence.load(Ordering::Relaxed)
        });
        ValidationCoverageStatus::VerifiedThrough(verified)
    }
}

struct TeamWorkerHandle {
    team_id: u32,
    tx: Sender<ValidationMessage>,
    handle: Option<JoinHandle<()>>,
}
pub struct ObserverValidationWorker {
    tap: ObserverValidationTap,
    mismatch_rx: Receiver<ObserverMismatch>,
    checkpoint_rx: Receiver<ObserverCheckpointReport>,
    workers: Vec<TeamWorkerHandle>,
}

impl ObserverValidationWorker {
    pub fn start(capacity: usize) -> Self {
        let (mismatch_tx, mismatch_rx) = bounded(capacity.max(1));
        let (checkpoint_tx, checkpoint_rx) = bounded(capacity.max(1));
        let gaps = Arc::new(Mutex::new(Vec::new()));
        let mut value = ObserverValidationMetrics::default();
        for id in SUPPORTED_REPLICA_TEAMS {
            value
                .teams
                .insert(id, Arc::new(TeamObserverMetrics::default()));
        }
        let metrics = Arc::new(value);
        let mut senders = BTreeMap::new();
        let mut workers = Vec::new();
        for team_id in SUPPORTED_REPLICA_TEAMS {
            let (tx, rx) = bounded(capacity.max(1));
            senders.insert(team_id, tx.clone());
            let (mm, cp, met, gp) = (
                mismatch_tx.clone(),
                checkpoint_tx.clone(),
                metrics.clone(),
                gaps.clone(),
            );
            let handle = thread::Builder::new()
                .name(format!("team-replica-{team_id}"))
                .spawn(move || run_team_worker(team_id, rx, mm, cp, met, gp))
                .expect("team worker spawn");
            workers.push(TeamWorkerHandle {
                team_id,
                tx,
                handle: Some(handle),
            });
        }
        Self {
            tap: ObserverValidationTap {
                senders: Arc::new(senders),
                metrics,
                gaps,
            },
            mismatch_rx,
            checkpoint_rx,
            workers,
        }
    }
    pub fn tap(&self) -> ObserverValidationTap {
        self.tap.clone()
    }
    pub fn try_recv_mismatch(&self) -> Option<ObserverMismatch> {
        self.mismatch_rx.try_recv().ok()
    }
    pub fn try_recv_checkpoint(&self) -> Option<ObserverCheckpointReport> {
        self.checkpoint_rx.try_recv().ok()
    }
    pub fn unverified_worker_teams(&self) -> Vec<u32> {
        self.workers
            .iter()
            .filter_map(|worker| {
                worker
                    .handle
                    .as_ref()
                    .is_some_and(JoinHandle::is_finished)
                    .then_some(worker.team_id)
            })
            .collect()
    }
}

impl Drop for ObserverValidationWorker {
    fn drop(&mut self) {
        for worker in &self.workers {
            let _ = worker.tx.send(ValidationMessage::Shutdown);
        }
        for worker in &mut self.workers {
            if let Some(handle) = worker.handle.take() {
                if handle.join().is_err() {
                    log::error!("team-replica-{} panicked", worker.team_id);
                }
            }
        }
    }
}

fn run_team_worker(
    team_id: u32,
    rx: Receiver<ValidationMessage>,
    mismatch_tx: Sender<ObserverMismatch>,
    checkpoint_tx: Sender<ObserverCheckpointReport>,
    metrics: Arc<ObserverValidationMetrics>,
    gaps: Arc<Mutex<Vec<CoverageGap>>>,
) {
    let mut runtime = None;
    let mut stepper: Option<SpecsDisclosedWorldStepper> = None;
    let mut latest_tick = 0;
    while let Ok(message) = rx.recv() {
        metrics
            .queue_depth
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_sub(1))
            })
            .ok();
        if let Some(team) = metrics.teams.get(&team_id) {
            team.queue_depth
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                    Some(v.saturating_sub(1))
                })
                .ok();
        }
        match message {
            ValidationMessage::Shutdown => break,
            ValidationMessage::EndTeam => {
                runtime = None;
                stepper = None;
            }
            ValidationMessage::Bootstrap(encoded) => {
                let Ok(start) = TeamGameStart::decode(encoded.as_ref()) else {
                    continue;
                };
                if start.team_id != team_id {
                    record_gap(&gaps, &metrics, team_id, 0);
                    continue;
                }
                let allow = crate::runtime::secure_replica_component_allowlist();
                let replacing_existing = runtime.is_some();
                if let Ok(next) = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                    &start,
                    allow.clone(),
                    crate::runtime::secure_replica_resource_allowlist(),
                ) {
                    // A session bootstrap is a complete lockstep boundary.
                    // Reusing a prior stepper would retain script/dispatcher
                    // state that a newly joined external runtime never had.
                    let mut created = SpecsDisclosedWorldStepper::from_start(
                        &start,
                        allow,
                        crate::runtime::secure_replica_resource_allowlist(),
                    );
                    let specs_result = created.bootstrap_membership(next.world()).map(|_| created);
                    let Ok(specs) = specs_result else {
                        record_gap(&gaps, &metrics, team_id, 0);
                        continue;
                    };
                    let bootstrap_hash = next.canonical_team_hash();
                    let _ = checkpoint_tx.try_send(ObserverCheckpointReport {
                        key: ReplicaCheckpointKey {
                            team_id,
                            replica_tick: start.replica_start_tick,
                            team_sequence: start.next_team_sequence.saturating_sub(1),
                            authority_revision: 0,
                        },
                        expected_hash: None,
                        observer_pre_repair_hash: bootstrap_hash,
                        observer_post_repair_hash: bootstrap_hash,
                        encoded_frame_hash: <sha2::Sha256 as sha2::Digest>::digest(
                            encoded.as_ref(),
                        )
                        .into(),
                    });
                    stepper = Some(specs);
                    runtime = Some(next);
                    if replacing_existing {
                        if let Some(team) = metrics.teams.get(&team_id) {
                            team.rebootstrap_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
            ValidationMessage::Frame {
                sequence,
                replica_tick,
                encoded,
            } => {
                latest_tick = latest_tick.max(replica_tick);
                let (Some(replica), Some(specs)) = (runtime.as_mut(), stepper.as_mut()) else {
                    record_gap(&gaps, &metrics, team_id, sequence);
                    continue;
                };
                let Ok(frame) = TeamTickFrame::decode(encoded.as_ref()) else {
                    runtime = None;
                    stepper = None;
                    record_gap(&gaps, &metrics, team_id, sequence);
                    continue;
                };
                if frame.team_id != team_id {
                    record_gap(&gaps, &metrics, team_id, sequence);
                    continue;
                }
                let expected = frame
                    .post_step
                    .as_ref()
                    .and_then(|p| p.hash_checkpoint.as_ref())
                    .and_then(|h| <[u8; 32]>::try_from(h.canonical_team_hash.as_slice()).ok());
                let revision = frame.authority_revision.as_ref().map_or(0, |r| r.value);
                let started = Instant::now();
                match replica.apply_frame(frame, specs) {
                    Ok(FrameApplyResult::Applied {
                        pre_repair_observed_hash,
                        post_repair_hash,
                        ..
                    }) => {
                        metrics.verified_frame_count.fetch_add(1, Ordering::Relaxed);
                        metrics
                            .verified_through_sequence
                            .lock()
                            .expect("verified mutex")
                            .insert(team_id, sequence);
                        metrics
                            .audit_lag_ticks
                            .store(latest_tick.saturating_sub(replica_tick), Ordering::Relaxed);
                        if let Some(team) = metrics.teams.get(&team_id) {
                            team.current_replica_tick
                                .store(replica_tick, Ordering::Relaxed);
                            team.verified_frame_count.fetch_add(1, Ordering::Relaxed);
                            team.verified_through_sequence
                                .store(sequence, Ordering::Relaxed);
                            team.lag_ticks
                                .store(latest_tick.saturating_sub(replica_tick), Ordering::Relaxed);
                            record_duration(
                                &team.step_samples_ns,
                                started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64,
                            );
                            record_duration(
                                &team.script_phase_samples_ns,
                                specs.last_script_phase_ns,
                            );
                        }
                        let _ = checkpoint_tx.try_send(ObserverCheckpointReport {
                            key: ReplicaCheckpointKey {
                                team_id,
                                replica_tick,
                                team_sequence: sequence,
                                authority_revision: revision,
                            },
                            expected_hash: expected,
                            observer_pre_repair_hash: pre_repair_observed_hash,
                            observer_post_repair_hash: post_repair_hash,
                            encoded_frame_hash: <sha2::Sha256 as sha2::Digest>::digest(
                                encoded.as_ref(),
                            )
                            .into(),
                        });
                        if let Some(expected_hash) = expected {
                            if expected_hash == post_repair_hash {
                                continue;
                            }
                            if let Some(team) = metrics.teams.get(&team_id) {
                                team.pre_repair_mismatch_count
                                    .fetch_add(1, Ordering::Relaxed);
                            }
                            let _ = mismatch_tx.try_send(ObserverMismatch {
                                team_id,
                                frame_sequence: sequence,
                                replica_tick,
                                authority_revision: revision,
                                first_divergent_tick: replica_tick,
                                expected_hash,
                                observed_hash: post_repair_hash,
                                post_repair_hash,
                            });
                        }
                    }
                    Ok(FrameApplyResult::Duplicate) => {
                        // A bootstrap and the concurrently queued live frame may
                        // cover the same committed sequence. This is benign and
                        // must not trigger authority recovery/rebootstrap.
                    }
                    other => {
                        log::warn!(
                            "team replica validation stopped team={} sequence={} result={:?}",
                            team_id,
                            sequence,
                            other
                        );
                        runtime = None;
                        stepper = None;
                        record_gap(&gaps, &metrics, team_id, sequence);
                    }
                }
            }
        }
    }
}

fn record_gap(
    gaps: &Mutex<Vec<CoverageGap>>,
    metrics: &ObserverValidationMetrics,
    team_id: u32,
    sequence: u64,
) {
    metrics.coverage_gap_count.fetch_add(1, Ordering::Relaxed);
    if let Some(team) = metrics.teams.get(&team_id) {
        team.coverage_gap_count.fetch_add(1, Ordering::Relaxed);
    }
    gaps.lock().expect("gap mutex").push(CoverageGap {
        team_id,
        first_unverified_sequence: sequence,
        observed_sequence: sequence,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::{ProjectionDependencyGraph, TeamProjectorConfig, TeamViewProjector};
    use std::collections::BTreeSet;
    use std::time::{Duration, Instant};

    fn wait_until(mut predicate: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(3);
        while !predicate() {
            assert!(Instant::now() < deadline, "observer fixture timed out");
            std::thread::yield_now();
        }
    }

    fn bootstrap_and_frame(team_id: u32, tick: u64) -> (Arc<[u8]>, Arc<[u8]>) {
        let mut projector = TeamViewProjector::new(team_id, TeamProjectorConfig::default());
        let start: Arc<[u8]> =
            Arc::from(projector.build_team_game_start(0, 120, 77).encode_to_vec());
        let frame: Arc<[u8]> = Arc::from(
            projector
                .build_frame(
                    tick,
                    tick,
                    &BTreeSet::new(),
                    Vec::new(),
                    &[],
                    &ProjectionDependencyGraph::default(),
                )
                .unwrap()
                .wire_bytes,
        );
        (start, frame)
    }

    #[test]
    fn one_team_rebootstrap_does_not_reset_other_team_progress() {
        let worker = ObserverValidationWorker::start(16);
        let tap = worker.tap();
        let (start1, frame1) = bootstrap_and_frame(1, 0);
        let (start2, frame2) = bootstrap_and_frame(2, 0);
        tap.try_bootstrap(start1.clone());
        tap.try_bootstrap(start2);
        tap.try_frame(1, 0, 0, frame1);
        tap.try_frame(2, 0, 0, frame2);
        wait_until(|| tap.metrics.verified_frame_count.load(Ordering::Relaxed) >= 2);
        let team2_before = tap.metrics.teams[&2]
            .verified_through_sequence
            .load(Ordering::Relaxed);
        tap.try_bootstrap(start1);
        wait_until(|| {
            tap.metrics.teams[&1]
                .rebootstrap_count
                .load(Ordering::Relaxed)
                == 1
        });
        assert_eq!(
            tap.metrics.teams[&2]
                .verified_through_sequence
                .load(Ordering::Relaxed),
            team2_before
        );
        assert_eq!(
            tap.metrics.teams[&2]
                .rebootstrap_count
                .load(Ordering::Relaxed),
            0
        );
    }

    #[test]
    fn corrupt_team_one_frame_marks_only_team_one_unverified() {
        let worker = ObserverValidationWorker::start(8);
        let tap = worker.tap();
        let (start1, _) = bootstrap_and_frame(1, 0);
        let (start2, frame2) = bootstrap_and_frame(2, 0);
        tap.try_bootstrap(start1);
        tap.try_bootstrap(start2);
        tap.try_frame(1, 0, 0, Arc::from(&b"corrupt"[..]));
        tap.try_frame(2, 0, 0, frame2);
        wait_until(|| !tap.coverage_gaps().is_empty());
        wait_until(|| {
            tap.metrics.teams[&2]
                .verified_frame_count
                .load(Ordering::Relaxed)
                == 1
        });
        assert!(matches!(
            tap.coverage_status(1),
            ValidationCoverageStatus::UnverifiedGap { .. }
        ));
        assert_eq!(
            tap.coverage_status(2),
            ValidationCoverageStatus::VerifiedThrough(0)
        );
    }

    #[test]
    fn full_team_queue_records_exact_unverified_sequence() {
        let (tx, _rx) = bounded(1);
        let mut senders = BTreeMap::new();
        senders.insert(1, tx);
        let mut metrics = ObserverValidationMetrics::default();
        metrics
            .teams
            .insert(1, Arc::new(TeamObserverMetrics::default()));
        let tap = ObserverValidationTap {
            senders: Arc::new(senders),
            metrics: Arc::new(metrics),
            gaps: Arc::new(Mutex::new(Vec::new())),
        };
        tap.try_frame(1, 10, 10, Arc::from(&b"first"[..]));
        tap.try_frame(1, 11, 11, Arc::from(&b"overflow"[..]));
        assert_eq!(
            tap.coverage_gaps(),
            vec![CoverageGap {
                team_id: 1,
                first_unverified_sequence: 11,
                observed_sequence: 11,
            }]
        );
        assert_eq!(
            tap.metrics.teams[&1]
                .verified_frame_count
                .load(Ordering::Relaxed),
            0
        );
    }
}
