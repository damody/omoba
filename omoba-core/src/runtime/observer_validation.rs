//! Non-blocking in-process observer replicas. This module intentionally has no
//! Specs `World` or canonical identity type in its API.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use prost::Message;

use crate::game_proto::{TeamGameStart, TeamTickFrame};
use crate::runtime::{FrameApplyResult, NoopDisclosedWorldStepper, SelectiveReplicaRuntime};

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
    pub expected_hash: [u8; 32],
    pub observed_hash: [u8; 32],
}

enum ValidationMessage {
    Bootstrap { encoded: Arc<[u8]> },
    Frame { team_id: u32, sequence: u64, replica_tick: u64, encoded: Arc<[u8]> },
    EndTeam { team_id: u32 },
    Shutdown,
}

#[derive(Default)]
pub struct ObserverValidationMetrics {
    pub queue_depth: AtomicUsize,
    pub audit_lag_ticks: AtomicU64,
    pub coverage_gap_count: AtomicU64,
    pub verified_frame_count: AtomicU64,
    pub verified_through_sequence: Mutex<BTreeMap<u32, u64>>,
}

#[derive(Clone)]
pub struct ObserverValidationTap {
    tx: Sender<ValidationMessage>,
    pub metrics: Arc<ObserverValidationMetrics>,
    gaps: Arc<Mutex<Vec<CoverageGap>>>,
}

impl ObserverValidationTap {
    pub fn try_bootstrap(&self, encoded: Arc<[u8]>) {
        self.try_send(ValidationMessage::Bootstrap { encoded }, None);
    }

    pub fn try_frame(&self, team_id: u32, sequence: u64, replica_tick: u64, encoded: Arc<[u8]>) {
        self.try_send(
            ValidationMessage::Frame { team_id, sequence, replica_tick, encoded },
            Some((team_id, sequence)),
        );
    }

    pub fn end_team(&self, team_id: u32) { self.try_send(ValidationMessage::EndTeam { team_id }, None); }

    fn try_send(&self, message: ValidationMessage, frame: Option<(u32, u64)>) {
        match self.tx.try_send(message) {
            Ok(()) => { self.metrics.queue_depth.fetch_add(1, Ordering::Relaxed); }
            Err(TrySendError::Full(_)) => {
                self.metrics.coverage_gap_count.fetch_add(1, Ordering::Relaxed);
                if let Some((team_id, sequence)) = frame {
                    self.gaps.lock().expect("coverage gap mutex poisoned").push(CoverageGap {
                        team_id,
                        first_unverified_sequence: sequence,
                        observed_sequence: sequence,
                    });
                }
            }
            Err(TrySendError::Disconnected(_)) => {}
        }
    }

    pub fn coverage_gaps(&self) -> Vec<CoverageGap> {
        self.gaps.lock().expect("coverage gap mutex poisoned").clone()
    }

    pub fn take_coverage_gaps(&self) -> Vec<CoverageGap> {
        std::mem::take(&mut *self.gaps.lock().expect("coverage gap mutex poisoned"))
    }
}

pub struct ObserverValidationWorker {
    tap: ObserverValidationTap,
    mismatch_rx: Receiver<ObserverMismatch>,
    handle: Option<JoinHandle<()>>,
}

impl ObserverValidationWorker {
    pub fn start(capacity: usize) -> Self {
        let (tx, rx) = bounded(capacity.max(1));
        let (mismatch_tx, mismatch_rx) = bounded(capacity.max(1));
        let metrics = Arc::new(ObserverValidationMetrics::default());
        let gaps = Arc::new(Mutex::new(Vec::new()));
        let tap = ObserverValidationTap { tx, metrics: Arc::clone(&metrics), gaps: Arc::clone(&gaps) };
        let handle = thread::Builder::new().name("selective-observer-validation".into())
            .spawn(move || run_worker(rx, mismatch_tx, metrics, gaps))
            .expect("observer validation worker spawn");
        Self { tap, mismatch_rx, handle: Some(handle) }
    }

    pub fn tap(&self) -> ObserverValidationTap { self.tap.clone() }
    pub fn try_recv_mismatch(&self) -> Option<ObserverMismatch> { self.mismatch_rx.try_recv().ok() }
}

impl Drop for ObserverValidationWorker {
    fn drop(&mut self) {
        let _ = self.tap.tx.send(ValidationMessage::Shutdown);
        if let Some(handle) = self.handle.take() { let _ = handle.join(); }
    }
}

fn run_worker(
    rx: Receiver<ValidationMessage>,
    mismatch_tx: Sender<ObserverMismatch>,
    metrics: Arc<ObserverValidationMetrics>,
    gaps: Arc<Mutex<Vec<CoverageGap>>>,
) {
    let mut observers: BTreeMap<u32, SelectiveReplicaRuntime> = BTreeMap::new();
    let mut latest_seen_tick = 0u64;
    while let Ok(message) = rx.recv() {
        let _ = metrics.queue_depth.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |depth| {
            Some(depth.saturating_sub(1))
        });
        match message {
            ValidationMessage::Shutdown => break,
            ValidationMessage::EndTeam { team_id } => { observers.remove(&team_id); }
            ValidationMessage::Bootstrap { encoded } => {
                let Ok(start) = TeamGameStart::decode(encoded.as_ref()) else { continue; };
                if let Ok(runtime) = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
                    &start,
                    BTreeSet::new(),
                    BTreeSet::new(),
                ) {
                    observers.insert(start.team_id, runtime);
                }
            }
            ValidationMessage::Frame { team_id, sequence, replica_tick, encoded } => {
                latest_seen_tick = latest_seen_tick.max(replica_tick);
                let Some(observer) = observers.get_mut(&team_id) else {
                    record_gap(&gaps, &metrics, team_id, sequence, sequence);
                    continue;
                };
                let Ok(frame) = TeamTickFrame::decode(encoded.as_ref()) else {
                    observers.remove(&team_id);
                    record_gap(&gaps, &metrics, team_id, sequence, sequence);
                    continue;
                };
                let expected = frame.post_step.as_ref().and_then(|post| post.hash_checkpoint.as_ref())
                    .and_then(|checkpoint| <[u8; 32]>::try_from(checkpoint.canonical_team_hash.as_slice()).ok());
                let mut stepper = NoopDisclosedWorldStepper;
                match observer.apply_frame(frame, &mut stepper) {
                    Ok(FrameApplyResult::Applied { team_hash, .. }) => {
                        metrics.verified_frame_count.fetch_add(1, Ordering::Relaxed);
                        metrics.verified_through_sequence.lock().expect("coverage mutex poisoned")
                            .insert(team_id, sequence);
                        metrics.audit_lag_ticks.store(latest_seen_tick.saturating_sub(replica_tick), Ordering::Relaxed);
                        if let Some(expected_hash) = expected.filter(|hash| hash != &team_hash) {
                            let _ = mismatch_tx.try_send(ObserverMismatch {
                                team_id, frame_sequence: sequence, replica_tick,
                                expected_hash, observed_hash: team_hash,
                            });
                        }
                    }
                    _ => {
                        observers.remove(&team_id);
                        record_gap(&gaps, &metrics, team_id, sequence, sequence);
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
    first: u64,
    observed: u64,
) {
    metrics.coverage_gap_count.fetch_add(1, Ordering::Relaxed);
    gaps.lock().expect("coverage gap mutex poisoned").push(CoverageGap {
        team_id, first_unverified_sequence: first, observed_sequence: observed,
    });
}
