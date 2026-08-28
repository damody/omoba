use std::collections::BTreeMap;

use omoba_core::{
    game_proto::{FilteredTeamSnapshot, TeamGameStart, TeamViewRebase, TeamViewRebaseChunk},
    runtime::{
        secure_replica_component_allowlist, secure_replica_resource_allowlist,
        FilteredRenderSnapshot, FrameApplyResult, IncompleteSnapshotStaging,
        SelectiveReplicaRuntime, SpecsDisclosedWorldStepper,
    },
};

use crate::ClientRuntimeError;

#[derive(Clone, Debug)]
pub struct ReplicaApplyReport {
    pub replica_tick: u64,
    pub team_sequence: u64,
    pub authority_revision: u64,
    pub pre_repair_hash: [u8; 32],
    pub post_repair_hash: [u8; 32],
}

pub struct ReplicaHost {
    team_id: u32,
    runtime: SelectiveReplicaRuntime,
    stepper: SpecsDisclosedWorldStepper,
    pre_repair_reports: BTreeMap<(u64, u64, u64), ReplicaApplyReport>,
    staging: IncompleteSnapshotStaging,
    expected_team_sequence: u64,
}

impl ReplicaHost {
    pub fn bootstrap(start: &TeamGameStart) -> Result<Self, ClientRuntimeError> {
        let components = secure_replica_component_allowlist();
        let resources = secure_replica_resource_allowlist();
        let runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            start,
            components.clone(),
            resources.clone(),
        )
        .map_err(|error| ClientRuntimeError::Replica(format!("bootstrap: {error:?}")))?;
        let mut stepper = SpecsDisclosedWorldStepper::from_start(start, components, resources);
        stepper
            .bootstrap_membership(runtime.world())
            .map_err(|error| ClientRuntimeError::Replica(format!("Specs bootstrap: {error:?}")))?;
        Ok(Self {
            team_id: start.team_id,
            runtime,
            stepper,
            pre_repair_reports: BTreeMap::new(),
            staging: IncompleteSnapshotStaging::default(),
            expected_team_sequence: start.next_team_sequence,
        })
    }

    pub fn apply_encoded_frame(
        &mut self,
        encoded: &[u8],
        authority_revision: u64,
    ) -> Result<Option<ReplicaApplyReport>, ClientRuntimeError> {
        match self.runtime.apply_encoded_frame(encoded, &mut self.stepper) {
            Ok(FrameApplyResult::Applied {
                replica_tick,
                team_sequence,
                pre_repair_observed_hash,
                post_repair_hash,
                ..
            }) => {
                let report = ReplicaApplyReport {
                    replica_tick,
                    team_sequence,
                    authority_revision,
                    pre_repair_hash: pre_repair_observed_hash,
                    post_repair_hash,
                };
                self.pre_repair_reports.insert(
                    (replica_tick, team_sequence, authority_revision),
                    report.clone(),
                );
                self.expected_team_sequence = team_sequence.saturating_add(1);
                Ok(Some(report))
            }
            Ok(FrameApplyResult::Duplicate) => Ok(None),
            Ok(FrameApplyResult::Stalled(state)) => Err(ClientRuntimeError::Replica(format!(
                "frame barrier stalled: {state:?}"
            ))),
            Err(error) => Err(ClientRuntimeError::Replica(format!(
                "frame rejected: {error:?}"
            ))),
        }
    }

    pub fn extract_presentation_source(&mut self) -> FilteredRenderSnapshot {
        self.runtime.extract_filtered_render_snapshot()
    }

    pub fn team_id(&self) -> u32 {
        self.team_id
    }

    pub fn global_seed(&self) -> u64 {
        self.runtime.global_seed()
    }

    pub fn view_epoch(&self) -> u64 {
        self.runtime.view_epoch()
    }

    pub fn next_replica_tick(&self) -> u64 {
        self.runtime.next_replica_tick()
    }

    pub fn expected_team_sequence(&self) -> u64 {
        self.expected_team_sequence
    }

    pub fn receive_rebase_chunk(&mut self, chunk: &TeamViewRebaseChunk) -> bool {
        if chunk.chunk_index == 0 {
            if let Some(snapshot_id) = chunk.snapshot_id.clone() {
                self.staging.begin(snapshot_id, chunk.chunk_count);
            }
        }
        self.staging.insert(chunk)
    }

    pub fn receive_rebase_manifest(
        &mut self,
        manifest: &TeamViewRebase,
    ) -> Result<(), ClientRuntimeError> {
        if manifest.team_id != self.team_id {
            return Err(ClientRuntimeError::Replica("wrong team rebase".into()));
        }
        let bytes = self
            .staging
            .finish(manifest)
            .ok_or_else(|| ClientRuntimeError::Replica("unverified rebase manifest".into()))?;
        let snapshot = FilteredTeamSnapshot {
            snapshot_schema_version: manifest.snapshot_schema_version,
            snapshot_id: manifest.snapshot_id.clone(),
            team_id: manifest.team_id,
            view_epoch: manifest.view_epoch.clone(),
            authoritative_tick: manifest.authoritative_tick,
            disclosed_world: bytes.clone(),
            public_metadata: Vec::new(),
            team_private_metadata: Vec::new(),
            filtered_snapshot_hash: manifest.filtered_snapshot_hash.clone(),
        };
        self.runtime
            .apply_verified_rebase(&snapshot, manifest, &bytes)
            .map_err(|error| ClientRuntimeError::Replica(format!("rebase: {error:?}")))?;
        self.stepper
            .rebootstrap(
                &TeamGameStart {
                    team_id: self.team_id,
                    replica_start_tick: snapshot.authoritative_tick,
                    global_seed: self.runtime.global_seed(),
                    tick_rate_hz: 120,
                    ..TeamGameStart::default()
                },
                secure_replica_component_allowlist(),
                secure_replica_resource_allowlist(),
                self.runtime.world(),
            )
            .map_err(|error| ClientRuntimeError::Replica(format!("Specs rebase: {error:?}")))?;
        self.expected_team_sequence = manifest.resume_team_sequence;
        Ok(())
    }

    pub fn contains_render_id(&self, render_id: u64, disclosure_epoch: u64) -> bool {
        self.runtime
            .world()
            .entities
            .get(&render_id)
            .is_some_and(|entity| entity.disclosure_epoch == disclosure_epoch)
    }

    pub fn secure_reference(
        &self,
        render_id: u64,
    ) -> Option<omoba_core::game_proto::SecureReplicaTarget> {
        let entity = self.runtime.world().entities.get(&render_id)?;
        Some(omoba_core::game_proto::SecureReplicaTarget {
            replica_entity_id: Some(omoba_core::game_proto::ReplicaEntityId { value: render_id }),
            view_epoch: Some(omoba_core::game_proto::ViewEpoch {
                value: self.view_epoch(),
            }),
            disclosure_epoch: Some(omoba_core::game_proto::DisclosureEpoch {
                value: entity.disclosure_epoch,
            }),
        })
    }

    pub fn owned_hero_reference(
        &self,
        player_id: u32,
    ) -> Option<omoba_core::game_proto::SecureReplicaTarget> {
        let render_id = self
            .runtime
            .world()
            .entities
            .iter()
            .find_map(|(id, entity)| {
                entity
                    .components
                    .get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                    .and_then(|bytes| omoba_core::runtime::decode_demo_render_state(bytes))
                    .filter(|render| {
                        render.team_id == self.team_id
                            && render.owner_player_id == player_id
                            && render.kind == 1
                    })
                    .map(|_| *id)
            })?;
        self.secure_reference(render_id)
    }

    pub fn owns_hero(&self, player_id: u32) -> bool {
        self.runtime.world().entities.values().any(|entity| {
            entity
                .components
                .get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| omoba_core::runtime::decode_demo_render_state(bytes))
                .is_some_and(|render| {
                    render.team_id == self.team_id
                        && render.owner_player_id == player_id
                        && render.kind == 1
                })
        })
    }

    pub fn owned_hero_position(&self, player_id: u32) -> Option<(i64, i64)> {
        self.runtime.world().entities.values().find_map(|entity| {
            entity
                .components
                .get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                .and_then(|bytes| omoba_core::runtime::decode_demo_render_state(bytes))
                .filter(|render| {
                    render.team_id == self.team_id
                        && render.owner_player_id == player_id
                        && render.kind == 1
                })
                .map(|render| (render.x_raw, render.y_raw))
        })
    }
    pub fn inject_test_only_fault(&mut self) -> bool {
        self.stepper.inject_test_only_position_fault()
    }
}
