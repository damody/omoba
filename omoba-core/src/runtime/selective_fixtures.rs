use std::collections::BTreeSet;

use prost::Message;
use sha2::{Digest, Sha256};

use crate::game_proto::{
    transition, AuthorityRevision, ComponentRepair, DisclosureEpoch, EntityReplace,
    FilteredTeamSnapshot, HideEntity, PostStep, PreStep, ReplicaEntityId as WireReplicaEntityId,
    RevealEntity, SnapshotId, TeamTickFrame, TeamViewRebase, TeamViewRebaseChunk, Transition,
    ViewEpoch,
};

use super::{
    build_snapshot_manifest, encode_snapshot_chunks, FilteredSnapshotError,
    SelectiveReplicaRuntime, FILTERED_SNAPSHOT_SCHEMA_VERSION,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyntheticReplicaKind {
    Client,
    ServerObserver,
}

pub struct SyntheticReplicaFixture {
    pub kind: SyntheticReplicaKind,
    pub encoded_frame: Vec<u8>,
    pub runtime: SelectiveReplicaRuntime,
}

fn fixture_from_encoded(
    kind: SyntheticReplicaKind,
    encoded_frame: Vec<u8>,
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
) -> Result<SyntheticReplicaFixture, prost::DecodeError> {
    let frame = TeamTickFrame::decode(encoded_frame.as_slice())?;
    let runtime = SelectiveReplicaRuntime::new(
        frame.team_id,
        frame.replica_tick,
        frame.team_sequence,
        frame.view_epoch.as_ref().map_or(0, |epoch| epoch.value),
        component_allowlist,
        resource_allowlist,
    );
    Ok(SyntheticReplicaFixture {
        kind,
        encoded_frame,
        runtime,
    })
}

pub fn synthetic_client_from_encoded(
    encoded_frame: Vec<u8>,
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
) -> Result<SyntheticReplicaFixture, prost::DecodeError> {
    fixture_from_encoded(
        SyntheticReplicaKind::Client,
        encoded_frame,
        component_allowlist,
        resource_allowlist,
    )
}

pub fn synthetic_observer_from_encoded(
    encoded_frame: Vec<u8>,
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
) -> Result<SyntheticReplicaFixture, prost::DecodeError> {
    fixture_from_encoded(
        SyntheticReplicaKind::ServerObserver,
        encoded_frame,
        component_allowlist,
        resource_allowlist,
    )
}

pub fn encode_component_baseline(components: &[(u32, &[u8])]) -> Vec<u8> {
    let mut sorted = components.to_vec();
    sorted.sort_by_key(|(schema_id, _)| *schema_id);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(sorted.len() as u32).to_be_bytes());
    for (schema_id, value) in sorted {
        bytes.extend_from_slice(&schema_id.to_be_bytes());
        bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    bytes
}

fn base_frame(team_id: u32, tick: u64, sequence: u64, revision: u64) -> TeamTickFrame {
    TeamTickFrame {
        protocol_version: 2,
        frame_schema_version: 1,
        content_schema_version: 1,
        team_id,
        server_tick: tick,
        replica_tick: tick,
        team_sequence: sequence,
        view_epoch: Some(ViewEpoch { value: 1 }),
        authority_revision: Some(AuthorityRevision { value: revision }),
        pre_step: Some(PreStep::default()),
        step: Some(crate::game_proto::Step::default()),
        post_step: Some(PostStep::default()),
        padding: Vec::new(),
    }
}

pub fn single_reveal_frame_fixture(
    team_id: u32,
    tick: u64,
    sequence: u64,
    replica_id: u64,
    component_schema_id: u32,
) -> Vec<u8> {
    let mut frame = base_frame(team_id, tick, sequence, 1);
    frame
        .pre_step
        .as_mut()
        .unwrap()
        .transitions
        .push(Transition {
            transition: Some(transition::Transition::Reveal(RevealEntity {
                replica_entity_id: Some(WireReplicaEntityId { value: replica_id }),
                disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
                effective_tick: tick,
                entity_kind: 1,
                safe_baseline: encode_component_baseline(&[(component_schema_id, b"baseline")]),
                disclosed_dependencies: Vec::new(),
                stable_sub_index: 0,
            })),
        });
    frame.encode_to_vec()
}

pub fn single_hide_frame_fixture(
    team_id: u32,
    tick: u64,
    sequence: u64,
    replica_id: u64,
) -> Vec<u8> {
    let mut frame = base_frame(team_id, tick, sequence, 2);
    frame
        .pre_step
        .as_mut()
        .unwrap()
        .transitions
        .push(Transition {
            transition: Some(transition::Transition::Hide(HideEntity {
                replica_entity_id: Some(WireReplicaEntityId { value: replica_id }),
                disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
                effective_tick: tick,
                remember_policy: 1,
                sanitized_remembered_presentation: b"last-known".to_vec(),
                stable_sub_index: 0,
            })),
        });
    frame.encode_to_vec()
}

pub fn single_component_repair_frame_fixture(
    team_id: u32,
    tick: u64,
    sequence: u64,
    replica_id: u64,
    component_schema_id: u32,
) -> Vec<u8> {
    let mut frame = base_frame(team_id, tick, sequence, 2);
    frame
        .post_step
        .as_mut()
        .unwrap()
        .component_repairs
        .push(ComponentRepair {
            replica_entity_id: Some(WireReplicaEntityId { value: replica_id }),
            disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
            component_schema_id,
            field_mask: vec![0xff],
            replacement_fields: b"server-repair".to_vec(),
            authority_revision: Some(AuthorityRevision { value: 2 }),
            effective_tick: tick,
        });
    frame.encode_to_vec()
}

pub fn single_entity_replace_frame_fixture(
    team_id: u32,
    tick: u64,
    sequence: u64,
    replica_id: u64,
    component_schema_id: u32,
) -> Vec<u8> {
    let mut frame = base_frame(team_id, tick, sequence, 3);
    frame
        .post_step
        .as_mut()
        .unwrap()
        .entity_replaces
        .push(EntityReplace {
            replica_entity_id: Some(WireReplicaEntityId { value: replica_id }),
            disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
            safe_baseline: encode_component_baseline(&[(component_schema_id, b"replacement")]),
            authority_revision: Some(AuthorityRevision { value: 3 }),
            effective_tick: tick,
        });
    frame.encode_to_vec()
}

pub struct SyntheticRebaseFixture {
    pub filtered_snapshot: FilteredTeamSnapshot,
    pub chunks: Vec<TeamViewRebaseChunk>,
    pub manifest: TeamViewRebase,
}

pub fn single_rebase_fixture(
    match_instance_id: [u8; 16],
    team_id: u32,
    tick: u64,
    resume_sequence: u64,
    disclosed_world: Vec<u8>,
) -> Result<SyntheticRebaseFixture, FilteredSnapshotError> {
    let snapshot_id = SnapshotId {
        snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
        match_instance_id: match_instance_id.to_vec(),
        team_id,
        view_epoch: Some(ViewEpoch { value: 2 }),
        authoritative_tick: tick,
        monotonic_snapshot_ordinal: 1,
    };
    let chunks = encode_snapshot_chunks(&snapshot_id, &disclosed_world, 1024)?;
    let manifest = build_snapshot_manifest(
        snapshot_id.clone(),
        team_id,
        2,
        tick,
        resume_sequence,
        4,
        &disclosed_world,
        &chunks,
    );
    let filtered_snapshot = FilteredTeamSnapshot {
        snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
        snapshot_id: Some(snapshot_id),
        team_id,
        view_epoch: Some(ViewEpoch { value: 2 }),
        authoritative_tick: tick,
        disclosed_world: disclosed_world.clone(),
        public_metadata: Vec::new(),
        team_private_metadata: Vec::new(),
        filtered_snapshot_hash: Sha256::digest(disclosed_world).to_vec(),
    };
    Ok(SyntheticRebaseFixture {
        filtered_snapshot,
        chunks,
        manifest,
    })
}
