use std::collections::{BTreeMap, VecDeque};

use crate::game_proto::{
    AuthorityRevision, ComponentRepair, DisclosureEpoch, EntityReplace, ReplicaEntityId,
    TeamViewRebaseNotice, ViewEpoch,
};
use crate::runtime::{
    ObserverCheckpointReport, ObserverMismatch, ReplicaCheckpointKey, SafeMismatchMetadata,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientCheckpointReport {
    pub key: ReplicaCheckpointKey,
    pub pre_repair_hash: [u8; 32],
    pub post_repair_hash: [u8; 32],
    pub encoded_frame_hash: [u8; 32],
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThreeWayCheckpoint {
    pub expected_hash: Option<[u8; 32]>,
    pub observer_pre_repair_hash: Option<[u8; 32]>,
    pub observer_post_repair_hash: Option<[u8; 32]>,
    pub client_pre_repair_hash: Option<[u8; 32]>,
    pub client_post_repair_hash: Option<[u8; 32]>,
    pub pre_repair_parity: Option<bool>,
    pub observer_frame_hash: Option<[u8; 32]>,
    pub client_frame_hash: Option<[u8; 32]>,
    /// Blocking convergence verdict. This intentionally describes post-repair
    /// parity; pre-repair parity is retained separately as divergence evidence.
    pub parity: Option<bool>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointVerdict {
    Pass,
    Fail,
    Unverified,
}

impl ThreeWayCheckpoint {
    pub fn verdict(&self) -> CheckpointVerdict {
        match self.parity {
            Some(true) => CheckpointVerdict::Pass,
            Some(false) => CheckpointVerdict::Fail,
            None => CheckpointVerdict::Unverified,
        }
    }
}

fn update_three_way_parity(checkpoint: &mut ThreeWayCheckpoint) {
    checkpoint.pre_repair_parity = match (
        checkpoint.expected_hash,
        checkpoint.observer_pre_repair_hash,
        checkpoint.client_pre_repair_hash,
    ) {
        (Some(expected), Some(observer), Some(client)) => {
            Some(expected == observer && expected == client)
        }
        _ => None,
    };
    checkpoint.parity = match (
        checkpoint.expected_hash,
        checkpoint.observer_post_repair_hash,
        checkpoint.client_post_repair_hash,
    ) {
        (Some(expected), Some(observer), Some(client)) => {
            Some(expected == observer && expected == client)
        }
        _ => None,
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MismatchControl {
    Observer(ObserverMismatch),
    Client(ClientHashMismatch),
    CoverageGapRebootstrap {
        team_id: u32,
        first_missing_sequence: u64,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientHashMismatch {
    pub team_id: u32,
    pub frame_sequence: u64,
    pub replica_tick: u64,
    pub received_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RebaseFailureSignal {
    pub team_id: u32,
    pub last_safe_sequence: u64,
    pub replay_coverage_gap: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FirstDivergenceRecord {
    pub team_id: u32,
    pub frame_sequence: u64,
    pub replica_tick: u64,
    pub safe_component_path: Option<String>,
    pub metadata: SafeMismatchMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentDifference {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub component_schema_id: u32,
    pub safe_component_path: String,
    pub field_mask: Vec<u8>,
    pub replacement_fields: Vec<u8>,
    pub safe_entity_baseline: Vec<u8>,
}

#[derive(Clone, Debug)]
pub enum RecoveryAction {
    ComponentRepair {
        repair: ComponentRepair,
        reason: AuthorityCorrectionReason,
    },
    EntityReplace(EntityReplace),
    FilteredRebase {
        team_id: u32,
        resume_sequence: u64,
        view_epoch: u64,
    },
    SafeTerminate(SafeTerminationDiagnostic),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorityCorrectionReason {
    MismatchRepair,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeTerminationDiagnostic {
    pub team_id: u32,
    pub last_safe_sequence: u64,
    pub reason_class: String,
    pub safe_component_path: Option<String>,
    pub protocol_fallback_allowed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RebaseRetryDisposition {
    Retry,
    Terminate,
}

#[derive(Clone, Debug)]
struct TeamRecoveryState {
    next_revision: u64,
    consecutive_failures: u32,
    retry_count: u32,
    first_divergence: Option<FirstDivergenceRecord>,
    pending: VecDeque<RecoveryAction>,
    staging_active: bool,
}

impl Default for TeamRecoveryState {
    fn default() -> Self {
        Self {
            next_revision: 1,
            consecutive_failures: 0,
            retry_count: 0,
            first_divergence: None,
            pending: VecDeque::new(),
            staging_active: false,
        }
    }
}

#[derive(Default)]
pub struct AuthorityRepairCoordinator {
    teams: BTreeMap<u32, TeamRecoveryState>,
    pub repair_count: u64,
    pub rebase_count: u64,
    pub termination_count: u64,
    pub replace_threshold: usize,
    pub rebase_threshold: usize,
    pub max_retries: u32,
    pub max_consecutive_component_repairs: u32,
    pub component_repair_count_by_team: BTreeMap<u32, u64>,
    pub entity_replace_count_by_team: BTreeMap<u32, u64>,
    pub filtered_rebase_count_by_team: BTreeMap<u32, u64>,
    pub three_way_checkpoints: BTreeMap<ReplicaCheckpointKey, ThreeWayCheckpoint>,
}

impl AuthorityRepairCoordinator {
    pub fn checkpoint_verdicts(&self) -> BTreeMap<ReplicaCheckpointKey, CheckpointVerdict> {
        self.three_way_checkpoints
            .iter()
            .map(|(key, value)| (*key, value.verdict()))
            .collect()
    }
    pub fn configured() -> Self {
        Self {
            replace_threshold: 2,
            rebase_threshold: 8,
            max_retries: 3,
            max_consecutive_component_repairs: 3,
            component_repair_count_by_team: BTreeMap::new(),
            entity_replace_count_by_team: BTreeMap::new(),
            filtered_rebase_count_by_team: BTreeMap::new(),
            ..Self::default()
        }
    }

    pub fn report_observer_checkpoint(&mut self, report: ObserverCheckpointReport) {
        let checkpoint = self.three_way_checkpoints.entry(report.key).or_default();
        if let Some(expected_hash) = report.expected_hash {
            checkpoint.expected_hash = Some(expected_hash);
        }
        checkpoint.observer_pre_repair_hash = Some(report.observer_pre_repair_hash);
        checkpoint.observer_post_repair_hash = Some(report.observer_post_repair_hash);
        checkpoint.observer_frame_hash = Some(report.encoded_frame_hash);
        update_three_way_parity(checkpoint);
    }

    pub fn report_client_checkpoint(&mut self, report: ClientCheckpointReport) {
        let checkpoint = self.three_way_checkpoints.entry(report.key).or_default();
        checkpoint.client_pre_repair_hash = Some(report.pre_repair_hash);
        checkpoint.client_post_repair_hash = Some(report.post_repair_hash);
        checkpoint.client_frame_hash = Some(report.encoded_frame_hash);
        update_three_way_parity(checkpoint);
    }

    fn allocate_revision(state: &mut TeamRecoveryState) -> u64 {
        let revision = state.next_revision;
        state.next_revision = revision.saturating_add(1);
        revision
    }

    pub fn report_component_divergence(
        &mut self,
        team_id: u32,
        frame_sequence: u64,
        replica_tick: u64,
        metadata: SafeMismatchMetadata,
        differences: &[ComponentDifference],
    ) {
        let state = self.teams.entry(team_id).or_default();
        if state.first_divergence.is_none() {
            state.first_divergence = Some(FirstDivergenceRecord {
                team_id,
                frame_sequence,
                replica_tick,
                safe_component_path: differences
                    .first()
                    .map(|difference| difference.safe_component_path.clone()),
                metadata,
            });
        }
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures > self.max_consecutive_component_repairs.max(1) {
            state.pending.push_back(RecoveryAction::FilteredRebase {
                team_id,
                resume_sequence: frame_sequence.saturating_add(1),
                view_epoch: 1,
            });
            state.staging_active = true;
            self.rebase_count = self.rebase_count.saturating_add(1);
            *self
                .filtered_rebase_count_by_team
                .entry(team_id)
                .or_default() += 1;
            return;
        }
        if differences.len() >= self.rebase_threshold.max(1) {
            state.pending.push_back(RecoveryAction::FilteredRebase {
                team_id,
                resume_sequence: frame_sequence.saturating_add(1),
                view_epoch: 1,
            });
            state.staging_active = true;
            self.rebase_count = self.rebase_count.saturating_add(1);
            *self
                .filtered_rebase_count_by_team
                .entry(team_id)
                .or_default() += 1;
            return;
        }
        if differences.len() >= self.replace_threshold.max(1) {
            if let Some(difference) = differences.first() {
                let revision = Self::allocate_revision(state);
                state
                    .pending
                    .push_back(RecoveryAction::EntityReplace(EntityReplace {
                        replica_entity_id: Some(ReplicaEntityId {
                            value: difference.replica_id,
                        }),
                        disclosure_epoch: Some(DisclosureEpoch {
                            value: difference.disclosure_epoch,
                        }),
                        safe_baseline: difference.safe_entity_baseline.clone(),
                        authority_revision: Some(AuthorityRevision { value: revision }),
                        effective_tick: replica_tick.saturating_add(1),
                    }));
                *self
                    .entity_replace_count_by_team
                    .entry(team_id)
                    .or_default() += 1;
            }
        } else if let Some(difference) = differences.first() {
            let revision = Self::allocate_revision(state);
            state.pending.push_back(RecoveryAction::ComponentRepair {
                reason: AuthorityCorrectionReason::MismatchRepair,
                repair: ComponentRepair {
                    replica_entity_id: Some(ReplicaEntityId {
                        value: difference.replica_id,
                    }),
                    disclosure_epoch: Some(DisclosureEpoch {
                        value: difference.disclosure_epoch,
                    }),
                    component_schema_id: difference.component_schema_id,
                    field_mask: difference.field_mask.clone(),
                    replacement_fields: difference.replacement_fields.clone(),
                    authority_revision: Some(AuthorityRevision { value: revision }),
                    effective_tick: replica_tick.saturating_add(1),
                },
            });
            *self
                .component_repair_count_by_team
                .entry(team_id)
                .or_default() += 1;
        }
        self.repair_count = self.repair_count.saturating_add(1);
    }

    pub fn report_observer_mismatch(&mut self, mismatch: ObserverMismatch) {
        let state = self.teams.entry(mismatch.team_id).or_default();
        if state.staging_active && state.consecutive_failures >= self.max_retries.max(1) {
            state
                .pending
                .push_back(RecoveryAction::SafeTerminate(SafeTerminationDiagnostic {
                    team_id: mismatch.team_id,
                    last_safe_sequence: mismatch.frame_sequence.saturating_sub(1),
                    reason_class: "observer-mismatch-after-rebase".to_owned(),
                    safe_component_path: None,
                    protocol_fallback_allowed: false,
                }));
            self.termination_count = self.termination_count.saturating_add(1);
            return;
        }
        state.pending.push_back(RecoveryAction::FilteredRebase {
            team_id: mismatch.team_id,
            resume_sequence: mismatch.frame_sequence.saturating_add(1),
            view_epoch: 1,
        });
        state.staging_active = true;
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        self.rebase_count = self.rebase_count.saturating_add(1);
        *self
            .filtered_rebase_count_by_team
            .entry(mismatch.team_id)
            .or_default() += 1;
    }

    pub fn report_client_mismatch(&mut self, mismatch: ClientHashMismatch) {
        let state = self.teams.entry(mismatch.team_id).or_default();
        state.pending.push_back(RecoveryAction::FilteredRebase {
            team_id: mismatch.team_id,
            resume_sequence: mismatch.frame_sequence.saturating_add(1),
            view_epoch: 1,
        });
        state.staging_active = true;
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        self.rebase_count = self.rebase_count.saturating_add(1);
        *self
            .filtered_rebase_count_by_team
            .entry(mismatch.team_id)
            .or_default() += 1;
    }

    pub fn report_coverage_gap(&mut self, team_id: u32, first_missing_sequence: u64) {
        let state = self.teams.entry(team_id).or_default();
        state.pending.push_back(RecoveryAction::FilteredRebase {
            team_id,
            resume_sequence: first_missing_sequence,
            view_epoch: 1,
        });
        state.staging_active = true;
        self.rebase_count = self.rebase_count.saturating_add(1);
        *self
            .filtered_rebase_count_by_team
            .entry(team_id)
            .or_default() += 1;
    }

    pub fn discard_interrupted_rebase(&mut self, team_id: u32) {
        if let Some(state) = self.teams.get_mut(&team_id) {
            state.staging_active = false;
        }
    }

    pub fn manifest_verification_failed(
        &mut self,
        team_id: u32,
        last_safe_sequence: u64,
    ) -> RebaseRetryDisposition {
        let state = self.teams.entry(team_id).or_default();
        state.staging_active = false;
        state.retry_count = state.retry_count.saturating_add(1);
        if state.retry_count <= self.max_retries {
            state.pending.push_back(RecoveryAction::FilteredRebase {
                team_id,
                resume_sequence: last_safe_sequence.saturating_add(1),
                view_epoch: 1,
            });
            RebaseRetryDisposition::Retry
        } else {
            state
                .pending
                .push_back(RecoveryAction::SafeTerminate(SafeTerminationDiagnostic {
                    team_id,
                    last_safe_sequence,
                    reason_class: "authority-recovery-exhausted".to_owned(),
                    safe_component_path: state
                        .first_divergence
                        .as_ref()
                        .and_then(|record| record.safe_component_path.clone()),
                    protocol_fallback_allowed: false,
                }));
            self.termination_count = self.termination_count.saturating_add(1);
            RebaseRetryDisposition::Terminate
        }
    }

    pub fn drain_actions(&mut self, team_id: u32) -> Vec<RecoveryAction> {
        self.teams
            .get_mut(&team_id)
            .map(|state| state.pending.drain(..).collect())
            .unwrap_or_default()
    }

    pub fn first_divergence(&self, team_id: u32) -> Option<&FirstDivergenceRecord> {
        self.teams
            .get(&team_id)
            .and_then(|state| state.first_divergence.as_ref())
    }
}

pub fn rebase_notice(
    snapshot_id: crate::game_proto::SnapshotId,
    manifest_hash: Vec<u8>,
    resume_sequence: u64,
    view_epoch: u64,
    authority_revision: u64,
) -> TeamViewRebaseNotice {
    TeamViewRebaseNotice {
        snapshot_id: Some(snapshot_id),
        manifest_hash,
        resume_team_sequence: resume_sequence,
        view_epoch: Some(ViewEpoch { value: view_epoch }),
        authority_revision: Some(AuthorityRevision {
            value: authority_revision,
        }),
    }
}

#[cfg(test)]
mod checkpoint_tests {
    use super::*;

    fn key() -> ReplicaCheckpointKey {
        ReplicaCheckpointKey {
            team_id: 1,
            replica_tick: 120,
            team_sequence: 120,
            authority_revision: 239,
        }
    }

    #[test]
    fn preserves_three_way_pre_and_post_repair_evidence() {
        let expected = [7; 32];
        let divergent = [9; 32];
        let mut coordinator = AuthorityRepairCoordinator::configured();
        coordinator.report_observer_checkpoint(ObserverCheckpointReport {
            key: key(),
            expected_hash: Some(expected),
            observer_pre_repair_hash: expected,
            observer_post_repair_hash: expected,
            encoded_frame_hash: [4; 32],
        });
        coordinator.report_client_checkpoint(ClientCheckpointReport {
            key: key(),
            pre_repair_hash: divergent,
            post_repair_hash: expected,
            encoded_frame_hash: [4; 32],
        });

        let checkpoint = &coordinator.three_way_checkpoints[&key()];
        assert_eq!(checkpoint.observer_pre_repair_hash, Some(expected));
        assert_eq!(checkpoint.observer_post_repair_hash, Some(expected));
        assert_eq!(checkpoint.client_pre_repair_hash, Some(divergent));
        assert_eq!(checkpoint.client_post_repair_hash, Some(expected));
        assert_eq!(checkpoint.pre_repair_parity, Some(false));
        assert_eq!(checkpoint.verdict(), CheckpointVerdict::Pass);
    }

    #[test]
    fn incomplete_three_way_report_is_unverified() {
        let expected = [3; 32];
        let mut coordinator = AuthorityRepairCoordinator::configured();
        coordinator.report_client_checkpoint(ClientCheckpointReport {
            key: key(),
            pre_repair_hash: expected,
            post_repair_hash: expected,
            encoded_frame_hash: [4; 32],
        });

        let checkpoint = &coordinator.three_way_checkpoints[&key()];
        assert_eq!(checkpoint.pre_repair_parity, None);
        assert_eq!(checkpoint.verdict(), CheckpointVerdict::Unverified);
    }
}
