use std::collections::{BTreeMap, BTreeSet};

use prost::Message;
use sha2::{Digest, Sha256};

use crate::game_proto::{
    AuthorityRevision, FilteredTeamSnapshot, SnapshotId, TeamGameStart, TeamTickFrame,
    TeamViewRebase, TeamViewRebaseChunk, ViewEpoch,
};

pub const FILTERED_SNAPSHOT_SCHEMA_VERSION: u32 = 1;
pub const REBASE_MANIFEST_VERSION: u32 = 1;
pub const REBASE_COMPRESSION_NONE: u32 = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalEntityKey {
    pub id: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReplicaEntityId(u64);

impl ReplicaEntityId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaIdAllocationError {
    Exhausted,
}

#[derive(Clone, Debug, Default)]
pub struct ReplicaEntityIdAllocator {
    next: u64,
}

impl ReplicaEntityIdAllocator {
    pub fn allocate(&mut self) -> Result<ReplicaEntityId, ReplicaIdAllocationError> {
        let value = if self.next == 0 { 1 } else { self.next };
        self.next = value
            .checked_add(1)
            .ok_or(ReplicaIdAllocationError::Exhausted)?;
        Ok(ReplicaEntityId(value))
    }

    pub fn retire(&mut self, _id: ReplicaEntityId) {
        // IDs are monotonically allocated and never reused. Keeping every
        // retired value would leak memory in long matches with repeated fog
        // Forget/Reveal transitions.
    }

    pub fn was_allocated(&self, id: ReplicaEntityId) -> bool {
        id.get() != 0 && id.get() < self.next
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MappingVisibility {
    Disclosed,
    Remembered,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamEntityMapping {
    pub replica_id: ReplicaEntityId,
    pub disclosure_epoch: u64,
    pub visibility: MappingVisibility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeamIdentityError {
    AllocationExhausted,
    UnknownCanonical,
    UnknownReplica,
    StaleDisclosureEpoch,
    RetiredReplica,
}

#[derive(Clone, Debug)]
pub struct TeamIdentityState {
    team_id: u32,
    allocator: ReplicaEntityIdAllocator,
    canonical_to_replica: BTreeMap<CanonicalEntityKey, TeamEntityMapping>,
    replica_to_canonical: BTreeMap<ReplicaEntityId, CanonicalEntityKey>,
}

impl TeamIdentityState {
    pub fn new(team_id: u32) -> Self {
        Self {
            team_id,
            allocator: ReplicaEntityIdAllocator::default(),
            canonical_to_replica: BTreeMap::new(),
            replica_to_canonical: BTreeMap::new(),
        }
    }

    pub fn team_id(&self) -> u32 {
        self.team_id
    }

    pub fn disclose(
        &mut self,
        canonical: CanonicalEntityKey,
    ) -> Result<TeamEntityMapping, TeamIdentityError> {
        if let Some(mapping) = self.canonical_to_replica.get_mut(&canonical) {
            mapping.disclosure_epoch = mapping.disclosure_epoch.saturating_add(1);
            mapping.visibility = MappingVisibility::Disclosed;
            return Ok(*mapping);
        }
        let replica_id = self
            .allocator
            .allocate()
            .map_err(|_| TeamIdentityError::AllocationExhausted)?;
        let mapping = TeamEntityMapping {
            replica_id,
            disclosure_epoch: 1,
            visibility: MappingVisibility::Disclosed,
        };
        self.canonical_to_replica.insert(canonical, mapping);
        self.replica_to_canonical.insert(replica_id, canonical);
        Ok(mapping)
    }

    pub fn remember(&mut self, canonical: CanonicalEntityKey) -> Result<(), TeamIdentityError> {
        let mapping = self
            .canonical_to_replica
            .get_mut(&canonical)
            .ok_or(TeamIdentityError::UnknownCanonical)?;
        mapping.visibility = MappingVisibility::Remembered;
        Ok(())
    }

    pub fn forget(
        &mut self,
        canonical: CanonicalEntityKey,
    ) -> Result<ReplicaEntityId, TeamIdentityError> {
        let mapping = self
            .canonical_to_replica
            .remove(&canonical)
            .ok_or(TeamIdentityError::UnknownCanonical)?;
        self.replica_to_canonical.remove(&mapping.replica_id);
        self.allocator.retire(mapping.replica_id);
        Ok(mapping.replica_id)
    }

    pub fn replica_for(&self, canonical: CanonicalEntityKey) -> Option<TeamEntityMapping> {
        self.canonical_to_replica.get(&canonical).copied()
    }

    /// Lookup for a client-visible replica id. Callers do not build `ReplicaEntityId`.
    /// Zero, unknown, retired, and stale epochs fail in the same guard as `canonical_for`.
    pub fn canonical_for_replica(
        &self,
        replica_id: u64,
        disclosure_epoch: u64,
    ) -> Result<CanonicalEntityKey, TeamIdentityError> {
        if replica_id == 0 {
            return Err(TeamIdentityError::UnknownReplica);
        }
        self.canonical_for(ReplicaEntityId(replica_id), disclosure_epoch)
    }

    pub fn canonical_for(
        &self,
        replica_id: ReplicaEntityId,
        disclosure_epoch: u64,
    ) -> Result<CanonicalEntityKey, TeamIdentityError> {
        let canonical = match self.replica_to_canonical.get(&replica_id) {
            Some(canonical) => *canonical,
            None if self.allocator.was_allocated(replica_id) => {
                return Err(TeamIdentityError::RetiredReplica);
            }
            None => return Err(TeamIdentityError::UnknownReplica),
        };
        let mapping = self.canonical_to_replica[&canonical];
        if disclosure_epoch != mapping.disclosure_epoch {
            return Err(TeamIdentityError::StaleDisclosureEpoch);
        }
        Ok(canonical)
    }

    /// Current disclosed identity only. Remembered, forgotten, stale-epoch, and
    /// generation-mismatched claims fail. This does not publish position, name,
    /// or status, and it does not authenticate a network session.
    pub fn disclosed_identity_matches(
        &self,
        replica_id: u64,
        disclosure_epoch: u64,
        canonical: CanonicalEntityKey,
    ) -> bool {
        let Ok(resolved) = self.canonical_for(ReplicaEntityId(replica_id), disclosure_epoch) else {
            return false;
        };
        if resolved != canonical {
            return false;
        }
        self.replica_for(resolved).is_some_and(|mapping| {
            mapping.visibility == MappingVisibility::Disclosed
                && mapping.disclosure_epoch == disclosure_epoch
                && mapping.replica_id.get() == replica_id
        })
    }

    pub fn disclosed_mappings(&self) -> Vec<(CanonicalEntityKey, TeamEntityMapping)> {
        self.canonical_to_replica
            .iter()
            .filter(|(_, mapping)| mapping.visibility == MappingVisibility::Disclosed)
            .map(|(canonical, mapping)| (*canonical, *mapping))
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisclosureClass {
    Public,
    TeamPrivate,
    VisibilityBound,
    ServerOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClassifiedComponentRecord {
    pub schema_id: u32,
    pub class: DisclosureClass,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CanonicalEntityRecord {
    pub canonical: CanonicalEntityKey,
    pub entity_kind: u32,
    pub components: Vec<ClassifiedComponentRecord>,
}

pub struct FilteredSnapshotBuildInput<'a> {
    pub team_id: u32,
    pub authoritative_tick: u64,
    pub view_epoch: u64,
    pub resolved_entities: &'a BTreeSet<CanonicalEntityKey>,
    pub component_allowlist: &'a BTreeSet<u32>,
    pub identity: &'a TeamIdentityState,
    pub entities: &'a [CanonicalEntityRecord],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilteredSnapshotBuildOutput {
    pub team_id: u32,
    pub authoritative_tick: u64,
    pub view_epoch: u64,
    pub disclosed_world: Vec<u8>,
    pub filtered_snapshot_hash: [u8; 32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FilteredSnapshotError {
    UnsupportedSchemaVersion,
    MissingReplicaMapping,
    ServerOnlyComponent,
    ComponentNotAllowlisted,
    LengthOverflow,
}

pub fn ensure_filtered_snapshot_schema(version: u32) -> Result<(), FilteredSnapshotError> {
    (version == FILTERED_SNAPSHOT_SCHEMA_VERSION)
        .then_some(())
        .ok_or(FilteredSnapshotError::UnsupportedSchemaVersion)
}

pub fn build_filtered_snapshot(
    input: FilteredSnapshotBuildInput<'_>,
) -> Result<FilteredSnapshotBuildOutput, FilteredSnapshotError> {
    let mut records: Vec<_> = input
        .entities
        .iter()
        .filter(|entity| input.resolved_entities.contains(&entity.canonical))
        .collect();
    records.sort_by_key(|entity| {
        input
            .identity
            .replica_for(entity.canonical)
            .map(|mapping| mapping.replica_id)
    });

    let mut disclosed_world = Vec::new();
    for entity in records {
        let mapping = input
            .identity
            .replica_for(entity.canonical)
            .ok_or(FilteredSnapshotError::MissingReplicaMapping)?;
        disclosed_world.extend_from_slice(&mapping.replica_id.get().to_be_bytes());
        disclosed_world.extend_from_slice(&mapping.disclosure_epoch.to_be_bytes());
        disclosed_world.extend_from_slice(&entity.entity_kind.to_be_bytes());
        let mut components = entity.components.clone();
        components.sort_by_key(|component| component.schema_id);
        let safe: Vec<_> = components
            .into_iter()
            .filter(|component| component.class != DisclosureClass::ServerOnly)
            .collect();
        disclosed_world.extend_from_slice(
            &u32::try_from(safe.len())
                .map_err(|_| FilteredSnapshotError::LengthOverflow)?
                .to_be_bytes(),
        );
        for component in safe {
            if !input.component_allowlist.contains(&component.schema_id) {
                return Err(FilteredSnapshotError::ComponentNotAllowlisted);
            }
            disclosed_world.extend_from_slice(&component.schema_id.to_be_bytes());
            disclosed_world.extend_from_slice(
                &u32::try_from(component.bytes.len())
                    .map_err(|_| FilteredSnapshotError::LengthOverflow)?
                    .to_be_bytes(),
            );
            disclosed_world.extend_from_slice(&component.bytes);
        }
    }
    let filtered_snapshot_hash: [u8; 32] = Sha256::digest(&disclosed_world).into();
    Ok(FilteredSnapshotBuildOutput {
        team_id: input.team_id,
        authoritative_tick: input.authoritative_tick,
        view_epoch: input.view_epoch,
        disclosed_world,
        filtered_snapshot_hash,
    })
}

#[derive(Clone, Debug)]
pub struct SnapshotIdAllocator {
    match_instance_id: [u8; 16],
    team_id: u32,
    next_ordinal: u64,
}

impl SnapshotIdAllocator {
    pub fn new(match_instance_id: [u8; 16], team_id: u32) -> Self {
        Self {
            match_instance_id,
            team_id,
            next_ordinal: 1,
        }
    }

    pub fn allocate(&mut self, view_epoch: u64, authoritative_tick: u64) -> Option<SnapshotId> {
        let ordinal = self.next_ordinal;
        self.next_ordinal = ordinal.checked_add(1)?;
        Some(SnapshotId {
            snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
            match_instance_id: self.match_instance_id.to_vec(),
            team_id: self.team_id,
            view_epoch: Some(ViewEpoch { value: view_epoch }),
            authoritative_tick,
            monotonic_snapshot_ordinal: ordinal,
        })
    }
}

fn canonical_snapshot_id_bytes(snapshot_id: &SnapshotId) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(48);
    bytes.extend_from_slice(&snapshot_id.snapshot_schema_version.to_be_bytes());
    bytes.extend_from_slice(&snapshot_id.match_instance_id);
    bytes.extend_from_slice(&snapshot_id.team_id.to_be_bytes());
    bytes.extend_from_slice(
        &snapshot_id
            .view_epoch
            .as_ref()
            .map_or(0, |epoch| epoch.value)
            .to_be_bytes(),
    );
    bytes.extend_from_slice(&snapshot_id.authoritative_tick.to_be_bytes());
    bytes.extend_from_slice(&snapshot_id.monotonic_snapshot_ordinal.to_be_bytes());
    bytes
}

pub fn chunk_hash(chunk: &TeamViewRebaseChunk) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(b"omoba-team-rebase-chunk-v1\0");
    if let Some(snapshot_id) = &chunk.snapshot_id {
        digest.update(canonical_snapshot_id_bytes(snapshot_id));
    }
    digest.update(chunk.chunk_index.to_be_bytes());
    digest.update(chunk.chunk_count.to_be_bytes());
    digest.update(chunk.uncompressed_offset.to_be_bytes());
    digest.update(chunk.uncompressed_len.to_be_bytes());
    digest.update(&chunk.payload);
    digest.finalize().into()
}

pub fn encode_snapshot_chunks(
    snapshot_id: &SnapshotId,
    snapshot_bytes: &[u8],
    chunk_size: usize,
) -> Result<Vec<TeamViewRebaseChunk>, FilteredSnapshotError> {
    if chunk_size == 0 {
        return Err(FilteredSnapshotError::LengthOverflow);
    }
    let count = snapshot_bytes.len().div_ceil(chunk_size).max(1);
    let count_u32 = u32::try_from(count).map_err(|_| FilteredSnapshotError::LengthOverflow)?;
    let mut chunks = Vec::with_capacity(count);
    for index in 0..count {
        let start = index * chunk_size;
        let end = (start + chunk_size).min(snapshot_bytes.len());
        let payload = snapshot_bytes[start..end].to_vec();
        let mut chunk = TeamViewRebaseChunk {
            protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
            snapshot_id: Some(snapshot_id.clone()),
            chunk_index: index as u32,
            chunk_count: count_u32,
            uncompressed_offset: start as u64,
            uncompressed_len: payload.len() as u32,
            compression_id: REBASE_COMPRESSION_NONE,
            payload,
            chunk_hash: Vec::new(),
        };
        chunk.chunk_hash = chunk_hash(&chunk).to_vec();
        chunks.push(chunk);
    }
    Ok(chunks)
}

pub fn manifest_hash(manifest: &TeamViewRebase) -> [u8; 32] {
    let mut digest = Sha256::new();
    digest.update(if manifest.manifest_version == 2 { b"omoba-team-rebase-manifest-v2\0" } else { b"omoba-team-rebase-manifest-v1\0" });
    digest.update(manifest.manifest_version.to_be_bytes());
    digest.update(manifest.protocol_version.to_be_bytes());
    digest.update(manifest.snapshot_schema_version.to_be_bytes());
    if let Some(snapshot_id) = &manifest.snapshot_id {
        digest.update(canonical_snapshot_id_bytes(snapshot_id));
    }
    digest.update(manifest.team_id.to_be_bytes());
    digest.update(
        manifest
            .view_epoch
            .as_ref()
            .map_or(0, |epoch| epoch.value)
            .to_be_bytes(),
    );
    digest.update(manifest.authoritative_tick.to_be_bytes());
    digest.update(manifest.resume_team_sequence.to_be_bytes());
    digest.update(
        manifest
            .authority_revision
            .as_ref()
            .map_or(0, |revision| revision.value)
            .to_be_bytes(),
    );
    digest.update(manifest.total_uncompressed_len.to_be_bytes());
    digest.update(manifest.compression_id.to_be_bytes());
    digest.update(manifest.chunk_count.to_be_bytes());
    digest.update((manifest.ordered_chunk_hashes.len() as u32).to_be_bytes());
    for hash in &manifest.ordered_chunk_hashes {
        digest.update(hash);
    }
    digest.update(&manifest.filtered_snapshot_hash);
    if manifest.manifest_version == 2 {
        if let Some(grid) = &manifest.fog_grid {
            let bytes = prost::Message::encode_to_vec(grid);
            digest.update((bytes.len() as u32).to_be_bytes());
            digest.update(bytes);
        }
    }
    // Empty stays outside the digest so v1 and v2 hashes of older manifests
    // do not change. Non-empty bytes are the owner-team projection codec.
    if !manifest.admitted_summoner_projection.is_empty() {
        digest.update(b"summoner-projection\0");
        digest.update((manifest.admitted_summoner_projection.len() as u32).to_be_bytes());
        digest.update(&manifest.admitted_summoner_projection);
    }
    // Empty owner-undo material stays outside the digest, same as an absent
    // summoner roster. Non-empty bytes are schema-1 hash material.
    if !manifest.retained_owner_undo_material.is_empty() {
        digest.update(b"owner-undo-material\0");
        digest.update((manifest.retained_owner_undo_material.len() as u32).to_be_bytes());
        digest.update(&manifest.retained_owner_undo_material);
    }
    if let Some(projection) = &manifest.combat_preferences {
        let bytes = prost::Message::encode_to_vec(projection);
        digest.update(b"combat-preferences\0");
        digest.update((bytes.len() as u32).to_be_bytes());
        digest.update(bytes);
    }
    digest.finalize().into()
}

pub fn build_snapshot_manifest(
    snapshot_id: SnapshotId,
    team_id: u32,
    view_epoch: u64,
    authoritative_tick: u64,
    resume_team_sequence: u64,
    authority_revision: u64,
    snapshot_bytes: &[u8],
    chunks: &[TeamViewRebaseChunk],
) -> TeamViewRebase {
    let mut manifest = TeamViewRebase {
        manifest_version: REBASE_MANIFEST_VERSION,
        protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
        snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
        snapshot_id: Some(snapshot_id),
        team_id,
        view_epoch: Some(ViewEpoch { value: view_epoch }),
        authoritative_tick,
        resume_team_sequence,
        authority_revision: Some(AuthorityRevision {
            value: authority_revision,
        }),
        total_uncompressed_len: snapshot_bytes.len() as u64,
        compression_id: REBASE_COMPRESSION_NONE,
        chunk_count: chunks.len() as u32,
        ordered_chunk_hashes: chunks
            .iter()
            .map(|chunk| chunk.chunk_hash.clone())
            .collect(),
        filtered_snapshot_hash: Sha256::digest(snapshot_bytes).to_vec(),
        manifest_hash: Vec::new(),
        fog_grid: None,
        admitted_summoner_projection: Vec::new(),
        retained_owner_undo_material: Vec::new(),
        combat_preferences: None,
    };
    manifest.manifest_hash = manifest_hash(&manifest).to_vec();
    manifest
}

pub fn verify_snapshot_manifest(manifest: &TeamViewRebase) -> bool {
    let valid_presentation = match manifest.manifest_version {
        1 => manifest.fog_grid.is_none(),
        2 => manifest.fog_grid.as_ref().is_some_and(|grid|
            crate::runtime::fog_grid::FogGridSnapshot::from_presentation(grid, manifest.team_id,
                manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value), manifest.authoritative_tick).is_ok()),
        _ => false,
    };
    let summoners_admitted = manifest.admitted_summoner_projection.is_empty()
        || crate::runtime::summoner_transport::decode_projected_summoners(
            &manifest.admitted_summoner_projection,
        )
        .is_some();
    let undo_retained = manifest.retained_owner_undo_material.is_empty()
        || crate::runtime::shop_undo_wire::canonical_retained_owner_undo_material(
            &manifest.retained_owner_undo_material,
        )
        .is_some();
    valid_presentation
        && crate::runtime::combat_transport::decode_projection(manifest.combat_preferences.as_ref(),
            manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value)).is_ok()
        && summoners_admitted
        && undo_retained
        && manifest.manifest_hash == manifest_hash(manifest).as_slice()
        && manifest.chunk_count as usize == manifest.ordered_chunk_hashes.len()
}

#[derive(Clone, Debug, Default)]
pub struct IncompleteSnapshotStaging {
    snapshot_id: Option<SnapshotId>,
    chunks: BTreeMap<u32, (Vec<u8>, Vec<u8>)>,
    expected_count: u32,
}

impl IncompleteSnapshotStaging {
    pub fn begin(&mut self, snapshot_id: SnapshotId, expected_count: u32) {
        self.discard();
        self.snapshot_id = Some(snapshot_id);
        self.expected_count = expected_count;
    }

    pub fn insert(&mut self, chunk: &TeamViewRebaseChunk) -> bool {
        if chunk.snapshot_id != self.snapshot_id
            || chunk.chunk_count != self.expected_count
            || chunk.chunk_hash != chunk_hash(chunk).as_slice()
        {
            return false;
        }
        self.chunks.insert(
            chunk.chunk_index,
            (chunk.payload.clone(), chunk.chunk_hash.clone()),
        );
        true
    }

    pub fn finish(&mut self, manifest: &TeamViewRebase) -> Option<Vec<u8>> {
        if !verify_snapshot_manifest(manifest)
            || manifest.snapshot_id != self.snapshot_id
            || self.chunks.len() != self.expected_count as usize
        {
            self.discard();
            return None;
        }
        let mut bytes = Vec::new();
        for index in 0..self.expected_count {
            let (payload, hash) = self.chunks.get(&index)?;
            if manifest.ordered_chunk_hashes.get(index as usize)? != hash {
                self.discard();
                return None;
            }
            bytes.extend_from_slice(payload);
        }
        if Sha256::digest(&bytes).as_slice() != manifest.filtered_snapshot_hash {
            self.discard();
            return None;
        }
        self.discard();
        Some(bytes)
    }

    pub fn discard(&mut self) {
        self.snapshot_id = None;
        self.chunks.clear();
        self.expected_count = 0;
    }
}

mod v2_payload_sealed {
    pub trait Sealed {}
}

#[cfg(test)]
mod summoner_manifest_tests {
    use super::*;
    use crate::runtime::summoner_transport::{
        encode_projected_summoners, ProjectedSummonerPlayer, ProjectedSummonerSlot,
    };

    fn bare_manifest() -> TeamViewRebase {
        build_snapshot_manifest(
            SnapshotId {
                snapshot_schema_version: FILTERED_SNAPSHOT_SCHEMA_VERSION,
                match_instance_id: vec![1; 16],
                team_id: 1,
                view_epoch: Some(ViewEpoch { value: 1 }),
                authoritative_tick: 1,
                monotonic_snapshot_ordinal: 1,
            },
            1,
            1,
            1,
            1,
            1,
            b"",
            &[],
        )
    }

    #[test]
    fn empty_summoner_bytes_preserve_the_manifest_hash_and_invalid_bytes_fail_verify() {
        let bare = bare_manifest();
        let mut manifest = bare.clone();
        manifest.admitted_summoner_projection = vec![1, 2, 3];
        assert_ne!(manifest_hash(&manifest).as_slice(), bare.manifest_hash.as_slice());
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert!(!verify_snapshot_manifest(&manifest));
        manifest.admitted_summoner_projection = encode_projected_summoners(&[ProjectedSummonerPlayer {
            player_id: 7,
            slots: [
                ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: 0 },
                ProjectedSummonerSlot { catalog_id: 2, cooldown_raw: 0 },
            ],
        }]);
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert!(verify_snapshot_manifest(&manifest));
        let blink_total = omoba_template_ids::summoner_spell_by_name("training_point_blink_v1")
            .unwrap()
            .cooldown
            .raw();
        for slots in [
            [
                ProjectedSummonerSlot { catalog_id: 0, cooldown_raw: 1 },
                ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: 0 },
            ],
            [
                ProjectedSummonerSlot { catalog_id: 1, cooldown_raw: blink_total + 1 },
                ProjectedSummonerSlot { catalog_id: 2, cooldown_raw: 0 },
            ],
        ] {
            manifest.admitted_summoner_projection = encode_projected_summoners(&[ProjectedSummonerPlayer {
                player_id: 7,
                slots,
            }]);
            manifest.manifest_hash = manifest_hash(&manifest).to_vec();
            assert!(!verify_snapshot_manifest(&manifest));
        }
        manifest.admitted_summoner_projection.clear();
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert_eq!(manifest.manifest_hash, bare.manifest_hash);
        assert!(verify_snapshot_manifest(&manifest));
    }

    #[test]
    fn empty_owner_undo_material_preserves_manifest_hash_and_bad_bytes_fail_verify() {
        use crate::game_proto::TeamPublicEvent;
        use crate::runtime::shop_undo_wire::{
            canonical_retained_owner_undo_material, owner_undo_hash_material, OwnerShopUndoProjection,
        };
        use crate::runtime::FactKind;
        let bare = bare_manifest();
        let mut manifest = bare.clone();
        assert!(manifest.retained_owner_undo_material.is_empty());
        assert_eq!(manifest_hash(&manifest).as_slice(), bare.manifest_hash.as_slice());
        manifest.retained_owner_undo_material = vec![1, 2, 3];
        assert_ne!(manifest_hash(&manifest).as_slice(), bare.manifest_hash.as_slice());
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert!(!verify_snapshot_manifest(&manifest));
        let projection = OwnerShopUndoProjection {
            phase: 1,
            player_id: 7,
            can_undo: true,
            reason: 0,
            newest_sequence: 4,
            commitment: [9; 32],
        };
        let material = owner_undo_hash_material(&[TeamPublicEvent {
            event_kind: FactKind::OwnerShopUndo as u32,
            stable_sub_index: 3,
            sanitized_payload: projection.encode(),
            ..Default::default()
        }]);
        assert!(canonical_retained_owner_undo_material(&material).is_some());
        manifest.retained_owner_undo_material = material;
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert!(verify_snapshot_manifest(&manifest));
        assert_ne!(manifest.manifest_hash, bare.manifest_hash);
        manifest.retained_owner_undo_material.push(0);
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert!(!verify_snapshot_manifest(&manifest));
        manifest.retained_owner_undo_material.clear();
        manifest.manifest_hash = manifest_hash(&manifest).to_vec();
        assert_eq!(manifest.manifest_hash, bare.manifest_hash);
        assert!(verify_snapshot_manifest(&manifest));
    }
}

pub trait V2PlayerPayload: Message + v2_payload_sealed::Sealed {}

macro_rules! impl_v2_payload {
    ($($type:ty),+ $(,)?) => {$(
        impl v2_payload_sealed::Sealed for $type {}
        impl V2PlayerPayload for $type {}
    )+};
}

impl_v2_payload!(
    TeamGameStart,
    TeamTickFrame,
    TeamViewRebaseChunk,
    TeamViewRebase,
    FilteredTeamSnapshot,
);

pub fn encode_v2_player_payload<T: V2PlayerPayload>(payload: &T) -> Vec<u8> {
    payload.encode_to_vec()
}
