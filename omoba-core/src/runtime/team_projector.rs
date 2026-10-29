//! Safe per-team projection from committed facts/state into protocol-v2 frames.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use prost::Message;
use sha2::{Digest, Sha256};

use crate::game_proto::{
    transition, AuthoritativeModePresentation, AuthorityRevision, ComponentRepair,
    DeterministicMetadata, DisclosureEpoch,
    EntityReplace, FilteredTeamSnapshot, ForgetEntity, HideEntity, PostStep, PreStep,
    ReplicaEntityId as ProtoReplicaEntityId, RevealEntity, SanitizedExternalEffect, SnapshotId,
    Step, TeamGameStart, TeamHashCheckpoint, TeamPublicEvent, TeamTickFrame, TeamViewRebase,
    TeamViewRebaseChunk, TeamViewRebaseNotice, Transition, ViewEpoch,
};
use crate::runtime::{
    CanonicalEntityKey, ClassifiedComponentRecord, DisclosureClass, FactAudience,
    MappingVisibility, ObservableFact, OrderedFact, RememberDisposition, TeamIdentityError,
    TeamIdentityState, VisibilityTransition,
};
use specs::{SystemData, World, WorldExt, Write};

pub const TEAM_FRAME_SCHEMA_VERSION: u32 = 1;
pub const CONTENT_SCHEMA_VERSION: u32 = 1;
/// Bounded production padding policy for full 5v5 maps and authored wave bursts.
/// This is a projector capacity, independent of the KCP stream's framing length.
pub const MAX_PADDED_TEAM_FRAME_BYTES: usize = 64 * 1024;

pub(crate) fn next_tick_after_committed(committed_tick: u64) -> u64 {
    committed_tick.saturating_add(1)
}

pub(crate) fn resume_sequence_after_committed(
    requested_resume: u64,
    committed_frame_sequence: u64,
) -> u64 {
    requested_resume.max(committed_frame_sequence.saturating_add(1))
}

fn evidence_sentinel_metadata(team_id: u32) -> Vec<DeterministicMetadata> {
    let name = format!("OMOBA_TEAM_{team_id}_SENTINEL_HEX");
    let Some(value) = std::env::var(name).ok().and_then(|hex| decode_hex_16(&hex)) else {
        return Vec::new();
    };
    vec![
        DeterministicMetadata {
            namespace: "fog-evidence".into(),
            key: "team-sentinel".into(),
            schema_version: 1,
            value: value.to_vec(),
        },
        DeterministicMetadata {
            namespace: "fog-evidence".into(),
            key: "property-pattern".into(),
            schema_version: 1,
            value: Sha256::digest([value.as_slice(), b"property"].concat()).to_vec(),
        },
        DeterministicMetadata {
            namespace: "fog-evidence".into(),
            key: "position-pattern".into(),
            schema_version: 1,
            value: Sha256::digest([value.as_slice(), b"position"].concat()).to_vec(),
        },
    ]
}

fn decode_hex_16(hex: &str) -> Option<[u8; 16]> {
    if hex.len() != 32 {
        return None;
    }
    let mut output = [0; 16];
    for (index, byte) in output.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&hex[index * 2..index * 2 + 2], 16).ok()?;
    }
    Some(output)
}

#[derive(Clone, Debug, Default)]
pub struct ProjectionDependencyGraph {
    pub edges: BTreeMap<u64, BTreeSet<u64>>,
    pub server_only: BTreeSet<u64>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProjectionError {
    Identity(TeamIdentityError),
    ServerOnlyDependency(u64),
    HiddenDependency(u64),
    ServerOnlyComponent(u32),
    EmptyPaddingBuckets,
    FrameTooLarge,
    MalformedDisclosedState,
    MissingProjectionPolicy(String),
    InvalidFog(&'static str),
}

impl From<TeamIdentityError> for ProjectionError {
    fn from(value: TeamIdentityError) -> Self {
        Self::Identity(value)
    }
}

pub fn redact_component_fields(
    components: &[ClassifiedComponentRecord],
    allowlist: &BTreeSet<u32>,
) -> Result<Vec<ClassifiedComponentRecord>, ProjectionError> {
    let mut safe = Vec::new();
    for component in components {
        if component.class == DisclosureClass::ServerOnly {
            return Err(ProjectionError::ServerOnlyComponent(component.schema_id));
        }
        if allowlist.contains(&component.schema_id) {
            safe.push(component.clone());
        }
    }
    safe.sort_by_key(|component| component.schema_id);
    Ok(safe)
}

pub fn disclosed_dependency_closure(
    root: u64,
    visible: &BTreeSet<u64>,
    graph: &ProjectionDependencyGraph,
) -> Result<Vec<u64>, ProjectionError> {
    let mut pending = vec![root];
    let mut visited = BTreeSet::new();
    while let Some(entity) = pending.pop() {
        if !visited.insert(entity) {
            continue;
        }
        if graph.server_only.contains(&entity) {
            return Err(ProjectionError::ServerOnlyDependency(entity));
        }
        if entity != root && !visible.contains(&entity) {
            return Err(ProjectionError::HiddenDependency(entity));
        }
        if let Some(dependencies) = graph.edges.get(&entity) {
            pending.extend(dependencies.iter().rev().copied());
        }
    }
    visited.remove(&root);
    Ok(visited.into_iter().collect())
}

#[derive(Clone, Debug)]
pub struct TeamProjectorConfig {
    pub component_allowlist: BTreeSet<u32>,
    pub size_buckets: Vec<usize>,
    pub mass_reveal_chunk_entities: usize,
    pub rebase_chunks_per_tick: usize,
    pub hash_checkpoint_interval_ticks: u64,
    pub fog_sample_interval_ticks: u64,
}

impl Default for TeamProjectorConfig {
    fn default() -> Self {
        Self {
            component_allowlist: crate::runtime::secure_replica_component_allowlist(),
            size_buckets: (8..=MAX_PADDED_TEAM_FRAME_BYTES.ilog2()).map(|power|1usize<<power).collect(),
            mass_reveal_chunk_entities: 64,
            rebase_chunks_per_tick: 2,
            hash_checkpoint_interval_ticks: 120,
            fog_sample_interval_ticks: 6,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaddedTeamFrame {
    pub frame: TeamTickFrame,
    pub canonical_bytes: Vec<u8>,
    pub wire_bytes: Vec<u8>,
    pub padding_len: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SafeMismatchMetadata {
    pub team_id: u32,
    pub replica_tick: u64,
    pub expected_hash_prefix: [u8; 8],
    pub received_hash_prefix: [u8; 8],
    pub disclosed_entity_count: u32,
}

pub struct TeamViewProjector {
    team_id: u32,
    sequence: u64,
    view_epoch: u64,
    authority_revision: u64,
    identity: TeamIdentityState,
    config: TeamProjectorConfig,
    pending_reveals: VecDeque<VisibilityTransition>,
    pending_rebase_chunks: VecDeque<Vec<u8>>,
    hash_entities: BTreeMap<u64, (u64, u32, BTreeMap<u32, Vec<u8>>)>,
    pending_component_repairs: VecDeque<ComponentRepair>,
    pending_entity_replaces: VecDeque<EntityReplace>,
    pending_rebase_notice: Option<TeamViewRebaseNotice>,
    next_snapshot_ordinal: u64,
    latest_external_property_by_replica: BTreeMap<u64, Vec<u8>>,
    fog_grid: Option<crate::runtime::fog_grid::AuthorityFogGrid>,
    pending_fog_event: Option<TeamPublicEvent>,
    prepared_mode: Option<AuthoritativeModePresentation>,
    combat_preferences: Option<crate::game_proto::CombatPreferencesProjection>,
    /// Last step's owner-undo hash material. Empty before the first frame and
    /// whenever that step had no owner-undo fact. The next bootstrap and rebase
    /// carry this vector; they do not recompute it from private ledger state.
    retained_owner_undo_material: Vec<u8>,
    /// Recent checkpoint views as cheap per-component digests, so an observer
    /// mismatch can name the diverging (entity, schema) instead of a bare hash.
    expected_digests: std::collections::VecDeque<(u64, Vec<ComponentDigestRow>)>,
    /// Full expected components of the last few checkpoints, for mismatch diagnostics.
    expected_snapshots: std::collections::VecDeque<(u64, ComponentBytes)>,
}

#[derive(Default)]
pub struct TeamProjectionRuntime {
    projectors: BTreeMap<u32, TeamViewProjector>,
    pub latest_frames: BTreeMap<u32, PaddedTeamFrame>,
    pub dependency_graph: ProjectionDependencyGraph,
    pub pending_filtered_rebases: BTreeMap<u32, (u64, u64)>,
    pub safe_terminations: Vec<crate::runtime::SafeTerminationDiagnostic>,
    pub latest_rebases: BTreeMap<u32, RecoveryRebaseBundle>,
    pending_rebase_outbound: BTreeMap<u32, PendingRebaseOutbound>,
    pub pending_accepted_inputs: Vec<CanonicalAcceptedInput>,
    /// Wall-clock ns of the latest projection per team (diagnostics only).
    pub last_projection_ns: BTreeMap<u32, u128>,
}

impl TeamProjectionRuntime {
    /// Current team disclosure and view epoch. The caller must not keep this
    /// borrow across a later mutable world fetch.
    pub fn communication_disclosure(&self, team_id: u32) -> Option<(&TeamIdentityState, u64)> {
        self.projectors.get(&team_id).map(|projector| (&projector.identity, projector.view_epoch))
    }

    /// Gameplay input fog rule shared by authority admission and projection:
    /// an input may name only an entity its team's replica can resolve. A team
    /// without a projector (non-selective modes) keeps every target.
    pub fn target_disclosed(&self, team_id: u32, canonical: u64) -> bool {
        self.projectors.get(&team_id).map_or(true, |projector| replica_proto(&projector.identity, canonical).is_some())
    }

    /// Full admission fog rule: the target must be disclosed to the team's
    /// replica and actually sighted at the latest wave B (not merely kept by
    /// a previous tick's input grace).
    pub fn input_target_admissible(&self, visibility: &crate::runtime::TeamVisibilityRuntime, team_id: u32, canonical: u64) -> bool {
        self.target_disclosed(team_id, canonical)
            && visibility.teams.get(&team_id).map_or(true, |state| state.sighted.contains(&canonical))
    }

    /// Owned copies so the authority drain can release this borrow before admission.
    pub fn communication_disclosures(&self) -> Vec<(u32, u64, TeamIdentityState)> {
        self.projectors.iter().map(|(team_id, projector)| (*team_id, projector.view_epoch, projector.identity.clone())).collect()
    }
}

/// Server input acceptance shared by omb and replica parity harnesses: the
/// action kind and named target of one input. The target and other host-local
/// identities are stripped from the payload replicas receive. `None` refuses
/// the input (an unsealed client payload).
pub fn sanitize_accepted_action(
    action: &mut crate::game_proto::player_input::Action,
) -> Option<(u32, Option<u32>, Option<u32>)> {
    use crate::game_proto::player_input::Action;
    Some(match action {
        Action::SummonCommand(_) => return None, // unsealed client payload
        Action::OwnedSummonOrder(value) => {
            let (_,summon)=crate::runtime::summon_transport::validate_authority(value).ok()?;
            let id=summon as u32;let generation=(summon>>32)as u32;
            crate::runtime::summon_transport::redact(value).ok()?;
            (crate::runtime::summon_transport::ACTION_KIND,Some(id),Some(generation))
        }
        Action::Stop(_) => (21, None, None),
        Action::HoldPosition(_) => (20, None, None),
        Action::Recall(_) => (19, None, None),
        Action::NoOp(_) => (1, None, None),
        Action::MoveTo(_) => (2, None, None),
        Action::AttackTarget(value) => {
            let id = value.target_id;
            value.target_id = 0;
            (3, Some(id), None)
        }
        Action::CastAbility(value) => {
            let id = value.target_entity.take();
            (4, id, None)
        }
        Action::TowerPlace(_) => (5, None, None),
        Action::TowerUpgrade(value) => {
            let id = value.tower_entity_id;
            value.tower_entity_id = 0;
            (6, Some(id), None)
        }
        Action::TowerSell(value) => {
            let id = value.tower_entity_id;
            value.tower_entity_id = 0;
            (7, Some(id), None)
        }
        Action::ItemUse(value) => {
            let id = value.target_entity.take();
            let generation = value.target_generation.take();
            value.owner_entity = None;
            value.owner_generation = None;
            (8, id, generation)
        }
        Action::StartRound(_) => (9, None, None),
        Action::ItemBuy(_) => (17, None, None),
        Action::ItemSell(_) => (18, None, None),
        Action::ShopUndo(_) => (crate::runtime::shop_undo_wire::SHOP_UNDO_ACTION_KIND, None, None),
        Action::UpgradeAbility(_) => (10, None, None),
        Action::AttackMove(_) => (11, None, None),
        Action::SetTowerTargetPriority(value) => {
            let id = value.tower_entity_id;
            value.tower_entity_id = 0;
            (12, Some(id), None)
        }
        Action::TogglePause(_) => (13, None, None),
        Action::ToggleGameSpeed(_) => (14, None, None),
        Action::DebugSpawnCreep(_) => (15, None, None),
        Action::TowerAbilityCast(value) => {
            let id = value.tower_entity_id;
            value.tower_entity_id = 0;
            (16, Some(id), None)
        }
        Action::SummonerCast(_) => (22, None, None),
        Action::WardPlacement(_) => (25, None, None),
        Action::CombatPreferences(_) => (26, None, None),
        Action::InputScheme(_) => (27, None, None),
        Action::Direction(value) => {
            value.source=None;value.sealed_source=0;
            (crate::runtime::direction_transport::ACTION_KIND,None,None)
        }
    })
}

/// Canonical id of an accepted input's named target. An explicit generation is
/// the exact life (a dead life is not replaced by the recycled index); without
/// one the current index is used. The outer `None` refuses the whole input.
pub fn resolve_accepted_target(world: &specs::World, index: Option<u32>, generation: Option<u32>) -> Option<Option<u64>> {
    use specs::WorldExt;
    let entities = world.entities();
    let target = match (index, generation) {
        (Some(id), Some(generation)) => i32::try_from(generation).ok().filter(|gen| *gen > 0).and_then(|gen| {
            let entity = specs::world::Entity::new(id, specs::world::Generation::new(gen));
            entities.is_alive(entity).then(|| crate::runtime::canonical_entity_id(entity))
        }),
        (Some(id), None) => {
            let entity = entities.entity(id);
            entities.is_alive(entity).then(|| crate::runtime::canonical_entity_id(entity))
        }
        (None, _) => None,
    };
    if (index.is_some() || generation.is_some()) && target.is_none() { return None; }
    Some(target)
}

#[derive(Clone, Debug)]
pub struct CanonicalAcceptedInput {
    pub team_id: u32,
    pub player_id: u32,
    pub input_id: u64,
    pub action_kind: u32,
    pub actor_canonical_id: u64,
    pub target_canonical_id: Option<u64>,
    pub sanitized_payload: Vec<u8>,
}

impl CanonicalAcceptedInput {
    pub fn from_authoritative_acceptance(
        team_id: u32,
        player_id: u32,
        stable_order: u64,
        action_kind: u32,
        actor_canonical_id: u64,
        target_canonical_id: Option<u64>,
        sanitized_payload: Vec<u8>,
    ) -> Self {
        Self {
            team_id,
            player_id,
            input_id: stable_order,
            action_kind,
            actor_canonical_id,
            target_canonical_id,
            sanitized_payload,
        }
    }
}

#[derive(Clone, Debug)]
pub struct RecoveryRebaseBundle {
    pub chunks: Vec<TeamViewRebaseChunk>,
    pub manifest: TeamViewRebase,
}

#[derive(Clone, Debug)]
struct PendingRebaseOutbound {
    chunks: VecDeque<Vec<u8>>,
    manifest: Option<Vec<u8>>,
}

impl TeamProjectionRuntime {
    pub fn take_rate_limited_rebase_outbound(
        &mut self,
    ) -> Vec<(u32, Vec<Vec<u8>>, Option<Vec<u8>>)> {
        let teams: Vec<_> = self.pending_rebase_outbound.keys().copied().collect();
        let mut result = Vec::new();
        for team_id in teams {
            let limit = self.projectors.get(&team_id).map_or(1, |projector| {
                projector.config.rebase_chunks_per_tick.max(1)
            });
            let pending = self
                .pending_rebase_outbound
                .get_mut(&team_id)
                .expect("known pending rebase");
            let chunks = (0..limit)
                .filter_map(|_| pending.chunks.pop_front())
                .collect::<Vec<_>>();
            let manifest = pending
                .chunks
                .is_empty()
                .then(|| pending.manifest.take())
                .flatten();
            let finished = pending.chunks.is_empty() && pending.manifest.is_none();
            result.push((team_id, chunks, manifest));
            if finished {
                self.pending_rebase_outbound.remove(&team_id);
            }
        }
        result
    }
    pub fn apply_recovery_actions(
        &mut self,
        coordinator: &mut crate::runtime::AuthorityRepairCoordinator,
    ) {
        let teams: Vec<_> = self.projectors.keys().copied().collect();
        for team in teams {
            for action in coordinator.drain_actions(team) {
                match action {
                    crate::runtime::RecoveryAction::ComponentRepair { repair, .. } => {
                        if let Some(projector) = self.projectors.get_mut(&team) {
                            projector.enqueue_component_repair(repair);
                        }
                    }
                    crate::runtime::RecoveryAction::EntityReplace(replace) => {
                        if let Some(projector) = self.projectors.get_mut(&team) {
                            projector.enqueue_entity_replace(replace);
                        }
                    }
                    crate::runtime::RecoveryAction::FilteredRebase {
                        resume_sequence,
                        view_epoch,
                        ..
                    } => {
                        self.pending_filtered_rebases
                            .insert(team, (resume_sequence, view_epoch));
                    }
                    crate::runtime::RecoveryAction::SafeTerminate(diagnostic) => {
                        self.safe_terminations.push(diagnostic)
                    }
                }
            }
        }
    }

    pub fn build_team_bootstraps(
        &mut self,
        server_tick: u64,
        tick_rate_hz: u32,
        global_seed: u64,
    ) -> BTreeMap<u32, TeamGameStart> {
        self.projectors
            .iter_mut()
            .map(|(team, projector)| {
                (
                    *team,
                    projector.build_team_game_start(server_tick, tick_rate_hz, global_seed),
                )
            })
            .collect()
    }

    pub fn build_input_validation_snapshot(
        &self,
        visibility: &crate::runtime::TeamVisibilityRuntime,
    ) -> crate::runtime::SecureInputValidationSnapshot {
        let mut snapshot = crate::runtime::SecureInputValidationSnapshot::default();
        for (team_id, projector) in &self.projectors {
            let Some(team_visibility) = visibility.teams.get(team_id) else {
                continue;
            };
            let replicas = projector
                .identity
                .disclosed_mappings()
                .into_iter()
                .map(|(canonical, mapping)| {
                    let canonical_id =
                        ((canonical.generation as u64) << 32) | u64::from(canonical.id);
                    (
                        mapping.replica_id.get(),
                        crate::runtime::ReplicaValidationRecord {
                            canonical_id,
                            disclosure_epoch: mapping.disclosure_epoch,
                            owner_team: visibility
                                .latest_owner_by_canonical
                                .get(&canonical_id)
                                .copied()
                                .flatten(),
                        },
                    )
                })
                .collect();
            snapshot.teams.insert(
                *team_id,
                crate::runtime::TeamInputValidationView {
                    view_epoch: projector.view_epoch,
                    replicas,
                    visible_by_tick: team_visibility.history.snapshot(),
                },
            );
        }
        snapshot
    }
}

pub fn run_team_projection_after_wave_b(
    world: &mut World,
    server_tick: u64,
) -> Result<(), ProjectionError> {
    let visibility = world
        .read_resource::<crate::runtime::TeamVisibilityRuntime>()
        .clone();
    let committed = world
        .read_resource::<crate::runtime::CommittedProjectionBatch>()
        .clone();
    if !committed.barrier_reached {
        return Ok(());
    }
    let (settlements, undo_settlements) = {
        let mut queue = world.write_resource::<crate::runtime::PendingItemUseQueue>();
        (
            std::mem::take(&mut queue.settlements),
            std::mem::take(&mut queue.undo_settlements),
        )
    };
    // Read the authority roster before the mutable projection borrow. An empty
    // decode, or no MOBA match, leaves the manifest field empty.
    let authoritative_game_mode = world
        .try_fetch::<crate::runtime::native::comp::GameMode>()
        .map(|mode| *mode);
    let combat_projections: BTreeMap<_, _> = world.try_fetch::<crate::runtime::MobaMatch>().map(|match_state| {
        visibility.teams.iter().filter_map(|(team, visibility_state)| {
            crate::runtime::combat_transport::project(&match_state, *team, &visibility_state.index.current, 1)
                .map(|projection| (*team, projection))
        }).collect()
    }).unwrap_or_default();
    let admitted_summoners = world.try_fetch::<crate::runtime::MobaMatch>().map(|state| {
        let mut rows = BTreeMap::new();
        for team in state.config.teams {
            let bytes = crate::runtime::project_team_summoners(&state, team);
            if crate::runtime::summoner_transport::decode_projected_summoners(&bytes)
                .is_some_and(|decoded| !decoded.is_empty())
            {
                rows.insert(team, bytes);
            }
        }
        rows
    }).unwrap_or_default();
    let (mut runtime, mut coordinator) = <(
        Write<TeamProjectionRuntime>,
        Write<crate::runtime::AuthorityRepairCoordinator>,
    )>::fetch(world);
    runtime.apply_recovery_actions(&mut coordinator);
    let graph = runtime.dependency_graph.clone();
    let mut frames = BTreeMap::new();
    let pending_rebases = std::mem::take(&mut runtime.pending_filtered_rebases);
    let pending_inputs = std::mem::take(&mut runtime.pending_accepted_inputs);
    let mut rebases = BTreeMap::new();
    for (team, state) in &visibility.teams {
        let team_started = std::time::Instant::now();
        let transitions = visibility
            .last_transitions
            .get(team)
            .cloned()
            .unwrap_or_default();
        let projector = runtime
            .projectors
            .entry(*team)
            .or_insert_with(|| TeamViewProjector::new(*team, TeamProjectorConfig::default()));
        let combat_projection = combat_projections.get(team).cloned().map(|mut projection| {
            projection.view_epoch = projector.view_epoch;
            projection
        });
        projector.prepare_combat_preferences(combat_projection)?;
        projector.refresh_expected_visible_components(
            committed.tick,
            &state.index.current,
            &visibility.latest_disclosed_baseline_by_canonical,
        );
        if let (Some(geometry), Some(view)) = (visibility.fog_geometry, &visibility.latest_read_view) {
            if view.tick != committed.tick { return Err(ProjectionError::InvalidFog("fog barrier tick mismatch")); }
            projector.sample_authority_fog(geometry, view)?;
        }
        if let Some((resume_sequence, view_epoch)) = pending_rebases.get(team) {
            let view_epoch = (*view_epoch).max(projector.view_epoch);
            // Recovery requests may refer to the frame where divergence was
            // detected.  A rebase snapshot represents the current world, so
            // replay must resume at the projector's next sequence, never at
            // an older sequence whose replica tick predates the snapshot.
            // The rebase baseline already contains the committed result for
            // the frame about to be emitted at projector.sequence. Resume at
            // the following sequence; replaying the current frame would also
            // replay its replica tick from the completed baseline.
            let resume_sequence =
                resume_sequence_after_committed(*resume_sequence, projector.sequence);
            // hash_entities is the committed result of replica tick T. A
            // rebase consumer must resume by executing T+1, not execute T a
            // second time from an already-completed baseline.
            let mut bundle = projector.build_filtered_rebase(
                next_tick_after_committed(committed.tick),
                resume_sequence,
                view_epoch,
            )?;
            if let Some(bytes) = admitted_summoners.get(team) {
                bundle.manifest.admitted_summoner_projection = bytes.clone();
                bundle.manifest.manifest_hash =
                    crate::runtime::manifest_hash(&bundle.manifest).to_vec();
            }
            projector.enqueue_rebase_notice(crate::runtime::rebase_notice(
                bundle
                    .manifest
                    .snapshot_id
                    .clone()
                    .expect("rebase snapshot id"),
                bundle.manifest.manifest_hash.clone(),
                resume_sequence,
                view_epoch,
                bundle
                    .manifest
                    .authority_revision
                    .as_ref()
                    .map_or(0, |revision| revision.value),
            ));
            rebases.insert(*team, bundle);
        }
        projector.prepare_authoritative_mode(authoritative_game_mode);
        let frame = projector.build_frame_with_undo_settlements(
            server_tick,
            committed.tick,
            &state.index.current,
            transitions,
            &committed.facts,
            &graph,
            pending_inputs
                .iter()
                .filter(|input| input.team_id == *team)
                .cloned()
                .collect(),
            &settlements,
            &undo_settlements,
        )?;
        if committed.tick % projector.config.hash_checkpoint_interval_ticks.max(1) == 0 {
            record_expected_component_evidence(
                *team,
                committed.tick,
                &projector.expected_component_digests(),
            );
        }
        frames.insert(*team, frame);
        runtime.last_projection_ns.insert(*team, team_started.elapsed().as_nanos());
    }
    runtime.latest_frames = frames;
    for (team_id, bundle) in &rebases {
        runtime.pending_rebase_outbound.insert(
            *team_id,
            PendingRebaseOutbound {
                chunks: bundle.chunks.iter().map(Message::encode_to_vec).collect(),
                manifest: Some(bundle.manifest.encode_to_vec()),
            },
        );
    }
    runtime.latest_rebases = rebases;
    Ok(())
}

fn record_expected_component_evidence(
    team_id: u32,
    replica_tick: u64,
    rows: &[crate::runtime::DisclosedComponentDigest],
) {
    let Ok(root) = std::env::var("OMOBA_FOG_EVIDENCE_DIR") else {
        return;
    };
    let dir = std::path::Path::new(&root)
        .join("server")
        .join(format!("team-{team_id}"));
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let path = dir.join(format!("expected-components-{replica_tick}.json"));
    if let Ok(bytes) = serde_json::to_vec_pretty(rows) {
        let _ = std::fs::write(path, bytes);
    }
}

fn presentation_trace_enabled() -> bool {
    std::env::var("OMOBA_PRESENTATION_TRACE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
}

impl TeamViewProjector {
    pub fn expected_component_digests(&self) -> Vec<crate::runtime::DisclosedComponentDigest> {
        let mut rows = Vec::new();
        for (replica_id, (epoch, entity_kind, components)) in &self.hash_entities {
            for (schema_id, bytes) in components {
                rows.push(crate::runtime::DisclosedComponentDigest {
                    replica_id: *replica_id,
                    disclosure_epoch: *epoch,
                    entity_kind: *entity_kind,
                    component_schema_id: *schema_id,
                    byte_len: bytes.len(),
                    sha256: {
                        let digest = Sha256::digest(bytes);
                        digest.iter().map(|byte| format!("{byte:02x}")).collect()
                    },
                    disclosed_value_hex: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
                });
            }
        }
        rows
    }
    pub fn new(team_id: u32, config: TeamProjectorConfig) -> Self {
        Self {
            team_id,
            sequence: 1,
            view_epoch: 1,
            authority_revision: 1,
            identity: TeamIdentityState::new(team_id),
            config,
            pending_reveals: VecDeque::new(),
            pending_rebase_chunks: VecDeque::new(),
            hash_entities: BTreeMap::new(),
            pending_component_repairs: VecDeque::new(),
            pending_entity_replaces: VecDeque::new(),
            pending_rebase_notice: None,
            next_snapshot_ordinal: 1,
            latest_external_property_by_replica: BTreeMap::new(),
            fog_grid: None,
            pending_fog_event: None,
            prepared_mode: None,
            combat_preferences: None,
            retained_owner_undo_material: Vec::new(),
            expected_digests: std::collections::VecDeque::new(),
            expected_snapshots: std::collections::VecDeque::new(),
        }
    }

    pub fn prepare_authoritative_mode(
        &mut self,
        mode: Option<crate::runtime::native::comp::GameMode>,
    ) {
        self.prepared_mode = mode.and_then(|mode| {
            crate::runtime::authoritative_mode::presentation(mode, self.view_epoch)
        });
    }

    pub fn prepare_combat_preferences(&mut self, projection: Option<crate::game_proto::CombatPreferencesProjection>)
        -> Result<(), ProjectionError> {
        crate::runtime::combat_transport::decode_projection(projection.as_ref(), self.view_epoch)
            .map_err(|_| ProjectionError::MalformedDisclosedState)?;
        self.combat_preferences = projection;
        Ok(())
    }

    pub fn sample_authority_fog(
        &mut self,
        geometry: crate::runtime::fog_grid::FogGridGeometry,
        view: &crate::runtime::visibility::WaveBReadView,
    ) -> Result<(), ProjectionError> {
        use crate::runtime::fog_grid::{AuthorityFogGrid, FOG_GRID_EVENT_KIND};
        if let Some(latest) = self.fog_grid.as_ref().and_then(|grid| grid.latest()) {
            if latest.view_epoch != self.view_epoch {
                self.fog_grid = None;
            } else if latest.geometry != geometry {
                return Err(ProjectionError::InvalidFog("fog geometry changed within view"));
            }
        }
        if self.fog_grid.is_none() {
            self.fog_grid = Some(AuthorityFogGrid::new(self.team_id, self.view_epoch, geometry)
                .map_err(ProjectionError::InvalidFog)?);
        }
        let grid = self.fog_grid.as_mut().unwrap();
        if let Some(latest) = grid.latest() {
            if view.tick < latest.tick { return Err(ProjectionError::InvalidFog("stale fog view")); }
            if view.tick - latest.tick < self.config.fog_sample_interval_ticks.max(1) { return Ok(()); }
        }
        let snapshot = grid.sample(view).map_err(ProjectionError::InvalidFog)?;
        self.pending_fog_event = Some(TeamPublicEvent {
            event_kind: FOG_GRID_EVENT_KIND,
            subject: None,
            sanitized_payload: snapshot.encode().map_err(ProjectionError::InvalidFog)?,
            stable_sub_index: 0,
        });
        Ok(())
    }

    pub fn enqueue_rebase_chunks(&mut self, chunks: impl IntoIterator<Item = Vec<u8>>) {
        self.pending_rebase_chunks.extend(chunks);
    }

    pub fn take_rate_limited_rebase_chunks(&mut self) -> Vec<Vec<u8>> {
        (0..self.config.rebase_chunks_per_tick.max(1))
            .filter_map(|_| self.pending_rebase_chunks.pop_front())
            .collect()
    }

    pub fn enqueue_component_repair(&mut self, repair: ComponentRepair) {
        self.pending_component_repairs.push_back(repair);
    }

    pub fn refresh_expected_visible_components(
        &mut self,
        _effective_tick: u64,
        visible: &BTreeSet<u64>,
        baselines: &BTreeMap<u64, Vec<u8>>,
    ) {
        for canonical_id in visible {
            let Some(mapping) = self.identity.replica_for(unpack_canonical(*canonical_id)) else {
                continue;
            };
            let Some(baseline) = baselines.get(canonical_id) else {
                continue;
            };
            if let Some((epoch, kind, components)) =
                self.hash_entities.get_mut(&mapping.replica_id.get())
            {
                let next_components = team_safe_components(baseline, self.team_id);
                // This is the server-side expected view used by checkpoint
                // validation, not a per-tick state replication path. Visible
                // entities continue through player-view lockstep; only an
                // explicit recovery decision may enqueue ComponentRepair.
                *epoch = mapping.disclosure_epoch;
                *kind = 0;
                *components = next_components;
                if let Some(property) =
                    components.get(&crate::runtime::DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)
                {
                    self.latest_external_property_by_replica
                        .insert(mapping.replica_id.get(), property.clone());
                }
            }
        }
    }

    pub fn enqueue_visible_demo_repairs(
        &mut self,
        effective_tick: u64,
        visible: &BTreeSet<u64>,
        components: &BTreeMap<u64, Vec<u8>>,
    ) {
        for canonical_id in visible {
            let Some(mapping) = self.identity.replica_for(unpack_canonical(*canonical_id)) else {
                continue;
            };
            let Some(value) = components.get(canonical_id) else {
                continue;
            };
            let current =
                self.hash_entities
                    .get(&mapping.replica_id.get())
                    .and_then(|(_, _, values)| {
                        values.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                    });
            if current == Some(value) {
                continue;
            }
            self.pending_component_repairs.push_back(ComponentRepair {
                replica_entity_id: Some(ProtoReplicaEntityId {
                    value: mapping.replica_id.get(),
                }),
                disclosure_epoch: Some(DisclosureEpoch {
                    value: mapping.disclosure_epoch,
                }),
                component_schema_id: crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,
                field_mask: vec![0xff],
                replacement_fields: value.clone(),
                authority_revision: Some(AuthorityRevision {
                    value: self.authority_revision,
                }),
                effective_tick,
            });
            if let Some((_, _, stored)) = self.hash_entities.get_mut(&mapping.replica_id.get()) {
                stored.insert(
                    crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,
                    value.clone(),
                );
            }
        }
    }

    pub fn enqueue_entity_replace(&mut self, replace: EntityReplace) {
        self.pending_entity_replaces.push_back(replace);
    }

    pub fn enqueue_rebase_notice(&mut self, notice: TeamViewRebaseNotice) {
        self.pending_rebase_notice = Some(notice);
    }

    pub fn build_filtered_rebase(
        &mut self,
        authoritative_tick: u64,
        resume_sequence: u64,
        view_epoch: u64,
    ) -> Result<RecoveryRebaseBundle, ProjectionError> {
        if view_epoch == 0 || view_epoch < self.view_epoch { return Err(ProjectionError::InvalidFog("stale projector rebase epoch")); }
        let mut recovered_fog = self.fog_grid.clone();
        if let Some(grid) = &mut recovered_fog { grid.rebase_epoch(view_epoch).map_err(ProjectionError::InvalidFog)?; }
        let disclosed_world = self.encode_disclosed_world();
        let snapshot_id = SnapshotId {
            snapshot_schema_version: 1,
            match_instance_id: vec![0; 16],
            team_id: self.team_id,
            view_epoch: Some(ViewEpoch { value: view_epoch }),
            authoritative_tick,
            monotonic_snapshot_ordinal: self.next_snapshot_ordinal,
        };
        self.next_snapshot_ordinal = self.next_snapshot_ordinal.saturating_add(1);
        let chunks =
            crate::runtime::encode_snapshot_chunks(&snapshot_id, &disclosed_world, 16 * 1024)
                .map_err(|_| ProjectionError::FrameTooLarge)?;
        let mut manifest = crate::runtime::build_snapshot_manifest(
            snapshot_id,
            self.team_id,
            view_epoch,
            authoritative_tick,
            resume_sequence,
            self.authority_revision,
            &disclosed_world,
            &chunks,
        );
        manifest.combat_preferences = self.combat_preferences.clone().map(|mut projection| {
            projection.view_epoch = view_epoch;
            projection
        });
        if !self.retained_owner_undo_material.is_empty() {
            manifest.retained_owner_undo_material = self.retained_owner_undo_material.clone();
        }
        if let Some(grid) = recovered_fog.as_ref().and_then(|grid| grid.latest()) {
            if grid.tick > authoritative_tick { return Err(ProjectionError::InvalidFog("future fog rebase")); }
            manifest.manifest_version = 2;
            manifest.fog_grid = Some(grid.to_presentation());
        }
        if manifest.fog_grid.is_some() || !manifest.retained_owner_undo_material.is_empty() || manifest.combat_preferences.is_some() {
            manifest.manifest_hash = crate::runtime::selective::manifest_hash(&manifest).to_vec();
        }
        if view_epoch != self.view_epoch {
            self.pending_fog_event = recovered_fog.as_ref().and_then(|grid| grid.latest())
                .map(|grid| grid.presentation_event()).transpose().map_err(ProjectionError::InvalidFog)?;
        }
        self.view_epoch = view_epoch;
        self.combat_preferences = manifest.combat_preferences.clone();
        self.fog_grid = recovered_fog;
        Ok(RecoveryRebaseBundle { chunks, manifest })
    }

    pub fn build_team_game_start(
        &mut self,
        server_tick: u64,
        tick_rate_hz: u32,
        global_seed: u64,
    ) -> TeamGameStart {
        let disclosed_world = self.encode_disclosed_world();
        let filtered_snapshot_hash: [u8; 32] = Sha256::digest(&disclosed_world).into();
        let snapshot_id = SnapshotId {
            snapshot_schema_version: 1,
            match_instance_id: vec![0; 16],
            team_id: self.team_id,
            view_epoch: Some(ViewEpoch {
                value: self.view_epoch,
            }),
            authoritative_tick: server_tick,
            monotonic_snapshot_ordinal: self.next_snapshot_ordinal,
        };
        self.next_snapshot_ordinal = self.next_snapshot_ordinal.saturating_add(1);
        let mut team_private_metadata = evidence_sentinel_metadata(self.team_id);
        if let Some(snapshot) = self.fog_grid.as_ref().and_then(|grid| grid.latest()) {
            team_private_metadata.push(DeterministicMetadata {
                namespace: crate::runtime::fog_grid::FOG_GRID_NAMESPACE.into(),
                key: crate::runtime::fog_grid::FOG_GRID_KEY.into(),
                schema_version: 1,
                value: snapshot.encode().expect("validated authority fog snapshot"),
            });
        }
        if let Some(entry) = crate::runtime::shop_undo_wire::retained_owner_undo_metadata(
            &self.retained_owner_undo_material,
        ) {
            team_private_metadata.push(entry);
        }
        let snapshot = FilteredTeamSnapshot {
            snapshot_schema_version: 1,
            snapshot_id: Some(snapshot_id.clone()),
            team_id: self.team_id,
            view_epoch: Some(ViewEpoch {
                value: self.view_epoch,
            }),
            authoritative_tick: server_tick,
            disclosed_world,
            public_metadata: Vec::new(),
            team_private_metadata: team_private_metadata.clone(),
            filtered_snapshot_hash: filtered_snapshot_hash.to_vec(),
        };
        TeamGameStart {
            protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            snapshot_schema_version: 1,
            content_schema_version: CONTENT_SCHEMA_VERSION,
            player_id: 0,
            team_id: self.team_id,
            server_tick,
            replica_start_tick: server_tick,
            tick_rate_hz,
            visibility_commit_delay_ticks: 1,
            replica_buffer_ticks: 2,
            view_epoch: Some(ViewEpoch {
                value: self.view_epoch,
            }),
            next_team_sequence: self.sequence,
            snapshot_id: Some(snapshot_id),
            snapshot_manifest_hash: filtered_snapshot_hash.to_vec(),
            filtered_snapshot: Some(snapshot),
            public_metadata: Vec::new(),
            team_private_metadata,
            global_seed,
            shop_catalog_version: 0,
            shop_catalog_hash: String::new(),
            input_allocator_version: 0,
            last_seen_input_id: 0,
            shop_protocol_version: 0,
            shop_rules_hash: String::new(),
            recall_protocol_version: 0,
            recall_rules_hash: String::new(),
            mana_protocol_version: 0,
            mana_rules_hash: String::new(),
            command_protocol_version: 0,
            command_rules_hash: String::new(),
            summoner_protocol_version: 0,
            summoner_rules_hash: String::new(),
            ward_protocol_version: 0,
            ward_rules_hash: String::new(),
            owner_ward_projection: Vec::new(),
            owner_ward_envelope: None,
            combat_preferences: self.combat_preferences.clone(),
            // Per-player rejoin secret; stamped by the transport on each unicast join.
            resume_token: Vec::new(),
        }
    }

    fn encode_disclosed_world(&self) -> Vec<u8> {
        let mut disclosed_world = Vec::new();
        for (replica_id, (epoch, entity_kind, components)) in &self.hash_entities {
            disclosed_world.extend_from_slice(&replica_id.to_be_bytes());
            disclosed_world.extend_from_slice(&epoch.to_be_bytes());
            disclosed_world.extend_from_slice(&entity_kind.to_be_bytes());
            disclosed_world.extend_from_slice(&(components.len() as u32).to_be_bytes());
            for (schema_id, bytes) in components {
                disclosed_world.extend_from_slice(&schema_id.to_be_bytes());
                disclosed_world.extend_from_slice(&(bytes.len() as u32).to_be_bytes());
                disclosed_world.extend_from_slice(bytes);
            }
        }
        disclosed_world
    }

    pub fn build_frame(
        &mut self,
        server_tick: u64,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        transitions: Vec<VisibilityTransition>,
        facts: &[OrderedFact],
        graph: &ProjectionDependencyGraph,
    ) -> Result<PaddedTeamFrame, ProjectionError> {
        self.build_frame_with_inputs(
            server_tick,
            replica_tick,
            visible,
            transitions,
            facts,
            graph,
            Vec::new(),
        )
    }

    pub fn build_frame_with_inputs(
        &mut self,
        server_tick: u64,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        transitions: Vec<VisibilityTransition>,
        facts: &[OrderedFact],
        graph: &ProjectionDependencyGraph,
        accepted_inputs: Vec<CanonicalAcceptedInput>,
    ) -> Result<PaddedTeamFrame, ProjectionError> {
        self.build_frame_with_settlements(server_tick, replica_tick, visible,
            transitions, facts, graph, accepted_inputs, &[])
    }

    pub fn build_frame_with_settlements(
        &mut self,
        server_tick: u64,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        transitions: Vec<VisibilityTransition>,
        facts: &[OrderedFact],
        graph: &ProjectionDependencyGraph,
        accepted_inputs: Vec<CanonicalAcceptedInput>,
        settlements: &[crate::runtime::shop::ShopSettlement],
    ) -> Result<PaddedTeamFrame, ProjectionError> {
        self.build_frame_with_undo_settlements(
            server_tick, replica_tick, visible, transitions, facts, graph,
            accepted_inputs, settlements, &[],
        )
    }

    pub fn build_frame_with_undo_settlements(
        &mut self,
        server_tick: u64,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        transitions: Vec<VisibilityTransition>,
        facts: &[OrderedFact],
        graph: &ProjectionDependencyGraph,
        accepted_inputs: Vec<CanonicalAcceptedInput>,
        settlements: &[crate::runtime::shop::ShopSettlement],
        undo_settlements: &[crate::runtime::shop::ShopUndoSettlement],
    ) -> Result<PaddedTeamFrame, ProjectionError> {
        self.pending_reveals.extend(transitions);
        let pre_step = self.build_pre_step(replica_tick, visible, graph)?;
        let step = self.build_step(replica_tick, visible, facts, accepted_inputs, settlements, undo_settlements)?;
        // Recovery actions are queued before visibility transitions are
        // projected.  A unit can leave vision in that same frame; never emit a
        // post-step repair/replace for an entity pre-step just removed.
        self.pending_component_repairs.retain(|repair| {
            repair
                .replica_entity_id
                .as_ref()
                .is_some_and(|id| self.hash_entities.contains_key(&id.value))
        });
        self.pending_entity_replaces.retain(|replace| {
            replace
                .replica_entity_id
                .as_ref()
                .is_some_and(|id| self.hash_entities.contains_key(&id.value))
        });
        let undo_material = crate::runtime::shop_undo_wire::owner_undo_hash_material(&step.public_events);
        if crate::runtime::shop_undo_wire::canonical_retained_owner_undo_material(&undo_material).is_none() {
            return Err(ProjectionError::MalformedDisclosedState);
        }
        self.retained_owner_undo_material = undo_material.clone();
        let hash = expected_team_hash(
            self.team_id,
            replica_tick.saturating_add(1),
            &self.hash_entities,
            &undo_material,
            self.combat_preferences.as_ref(),
        );
        if replica_tick % self.config.hash_checkpoint_interval_ticks.max(1) == 0 {
            if self.expected_digests.len() >= EXPECTED_DIGEST_TICKS { self.expected_digests.pop_front(); }
            self.expected_digests.push_back((replica_tick, component_digest_rows(&self.hash_entities)));
            if self.expected_snapshots.len() >= EXPECTED_SNAPSHOT_TICKS { self.expected_snapshots.pop_front(); }
            self.expected_snapshots.push_back((replica_tick, component_bytes(&self.hash_entities)));
        }
        let post_step = PostStep {
            component_repairs: self.pending_component_repairs.drain(..).collect(),
            entity_replaces: self.pending_entity_replaces.drain(..).collect(),
            hash_checkpoint: (replica_tick % self.config.hash_checkpoint_interval_ticks.max(1)
                == 0)
                .then_some(TeamHashCheckpoint {
                    replica_tick,
                    canonical_team_hash: hash.to_vec(),
                    authority_revision: Some(AuthorityRevision {
                        value: self.authority_revision,
                    }),
                }),
            rebase_notice: self.pending_rebase_notice.take(),
        };
        let frame = TeamTickFrame {
            protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            frame_schema_version: TEAM_FRAME_SCHEMA_VERSION,
            content_schema_version: CONTENT_SCHEMA_VERSION,
            team_id: self.team_id,
            server_tick,
            replica_tick,
            team_sequence: self.sequence,
            view_epoch: Some(ViewEpoch {
                value: self.view_epoch,
            }),
            authority_revision: Some(AuthorityRevision {
                value: self.authority_revision,
            }),
            pre_step: Some(pre_step),
            step: Some(step),
            post_step: Some(post_step),
            padding: Vec::new(),
            authoritative_mode: self.prepared_mode.take(),
            combat_preferences: self.combat_preferences.clone(),
        };
        self.sequence = self.sequence.saturating_add(1);
        // Reserve the following revision for post-step authoritative repairs.
        self.authority_revision = self.authority_revision.saturating_add(2);
        pad_frame(frame, &self.config.size_buckets)
    }

    fn build_pre_step(
        &mut self,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        graph: &ProjectionDependencyGraph,
    ) -> Result<PreStep, ProjectionError> {
        let mut transitions = Vec::new();
        let budget = self.config.mass_reveal_chunk_entities.max(1);
        let mut reveal_count = 0usize;
        let mut deferred = VecDeque::new();
        while let Some(item) = self.pending_reveals.pop_front() {
            if matches!(item, VisibilityTransition::Reveal { .. }) && reveal_count >= budget {
                deferred.push_back(item);
                continue;
            }
            match item {
                VisibilityTransition::Reveal {
                    canonical_id,
                    effective_tick,
                    baseline,
                } => {
                    let baseline = if decode_safe_components_for_hash(&baseline)
                        .contains_key(&crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID)
                    {
                        crate::runtime::visibility::encode_disclosed_baseline(
                            &team_safe_components(&baseline, self.team_id).into_iter().collect::<Vec<_>>())
                    } else { baseline };
                    let effective_tick = effective_tick.max(replica_tick);
                    let key = unpack_canonical(canonical_id);
                    let mapping = self.identity.disclose(key)?;
                    if presentation_trace_enabled() {
                        let components = decode_safe_components_for_hash(&baseline);
                        if let Some(state) = components
                            .get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                            .and_then(|payload| crate::runtime::decode_demo_render_state(payload))
                            .filter(|state| state.kind == 1)
                        {
                            log::warn!(
                                "presentation_trace stage=server_projector team={} transition=Reveal canonical_id={} replica_id={} disclosure_epoch={} effective_tick={} player={} entity_team={} pos=({}, {})",
                                self.team_id,
                                canonical_id,
                                mapping.replica_id.get(),
                                mapping.disclosure_epoch,
                                effective_tick,
                                state.owner_player_id,
                                state.team_id,
                                state.x_raw,
                                state.y_raw,
                            );
                        }
                    }
                    let dependencies = disclosed_dependency_closure(canonical_id, visible, graph)?
                        .into_iter()
                        .filter_map(|dependency| {
                            self.identity
                                .replica_for(unpack_canonical(dependency))
                                .map(|mapping| ProtoReplicaEntityId {
                                    value: mapping.replica_id.get(),
                                })
                        })
                        .collect();
                    transitions.push(Transition {
                        transition: Some(transition::Transition::Reveal(RevealEntity {
                            replica_entity_id: Some(ProtoReplicaEntityId {
                                value: mapping.replica_id.get(),
                            }),
                            disclosure_epoch: Some(DisclosureEpoch {
                                value: mapping.disclosure_epoch,
                            }),
                            effective_tick,
                            entity_kind: 0,
                            safe_baseline: baseline.clone(),
                            disclosed_dependencies: dependencies,
                            stable_sub_index: reveal_count as u32,
                        })),
                    });
                    self.hash_entities.insert(
                        mapping.replica_id.get(),
                        (
                            mapping.disclosure_epoch,
                            0,
                            decode_safe_components_for_hash(&baseline),
                        ),
                    );
                    reveal_count += 1;
                }
                VisibilityTransition::Hide {
                    canonical_id,
                    effective_tick,
                    disposition,
                } => {
                    let effective_tick = effective_tick.max(replica_tick);
                    let key = unpack_canonical(canonical_id);
                    let mapping = self
                        .identity
                        .replica_for(key)
                        .ok_or(TeamIdentityError::UnknownCanonical)?;
                    if presentation_trace_enabled() {
                        if let Some(state) = self
                            .hash_entities
                            .get(&mapping.replica_id.get())
                            .and_then(|(_, _, components)| {
                                components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                            })
                            .and_then(|payload| crate::runtime::decode_demo_render_state(payload))
                            .filter(|state| state.kind == 1)
                        {
                            log::warn!(
                                "presentation_trace stage=server_projector team={} transition={:?} canonical_id={} replica_id={} disclosure_epoch={} effective_tick={} player={} entity_team={} pos=({}, {})",
                                self.team_id,
                                disposition,
                                canonical_id,
                                mapping.replica_id.get(),
                                mapping.disclosure_epoch,
                                effective_tick,
                                state.owner_player_id,
                                state.team_id,
                                state.x_raw,
                                state.y_raw,
                            );
                        }
                    }
                    let transition = match disposition {
                        RememberDisposition::Forget => {
                            self.identity.forget(key)?;
                            transition::Transition::Forget(ForgetEntity {
                                replica_entity_id: Some(ProtoReplicaEntityId {
                                    value: mapping.replica_id.get(),
                                }),
                                disclosure_epoch: Some(DisclosureEpoch {
                                    value: mapping.disclosure_epoch,
                                }),
                                effective_tick,
                                retire_reason: 1,
                                stable_sub_index: transitions.len() as u32,
                            })
                        }
                        RememberDisposition::LastKnown | RememberDisposition::Silhouette => {
                            self.identity.remember(key)?;
                            let sanitized = self
                                .hash_entities
                                .get(&mapping.replica_id.get())
                                .and_then(|(_, _, components)| {
                                    components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
                                })
                                .cloned()
                                .unwrap_or_default();
                            transition::Transition::Hide(HideEntity {
                                replica_entity_id: Some(ProtoReplicaEntityId {
                                    value: mapping.replica_id.get(),
                                }),
                                disclosure_epoch: Some(DisclosureEpoch {
                                    value: mapping.disclosure_epoch,
                                }),
                                effective_tick,
                                remember_policy: if disposition == RememberDisposition::LastKnown {
                                    1
                                } else {
                                    2
                                },
                                sanitized_remembered_presentation: sanitized,
                                stable_sub_index: transitions.len() as u32,
                            })
                        }
                    };
                    self.hash_entities.remove(&mapping.replica_id.get());
                    transitions.push(Transition {
                        transition: Some(transition),
                    });
                }
            }
        }
        self.pending_reveals = deferred;
        transitions.sort_by_key(transition_sort_key);
        Ok(PreStep { transitions })
    }

    fn build_step(
        &mut self,
        replica_tick: u64,
        visible: &BTreeSet<u64>,
        facts: &[OrderedFact],
        accepted: Vec<CanonicalAcceptedInput>,
        settlements: &[crate::runtime::shop::ShopSettlement],
        undo_settlements: &[crate::runtime::shop::ShopUndoSettlement],
    ) -> Result<Step, ProjectionError> {
        let mut public_events = crate::runtime::shop_receipt::project_shop_receipts(
            self.team_id, replica_tick, &accepted, settlements,
        )?;
        public_events.extend(crate::runtime::shop_undo_wire::project_shop_undo_receipts(
            self.team_id, replica_tick, &accepted, undo_settlements,
        )?);
        let mut external_effects = Vec::new();
        let policies = crate::runtime::ProjectionPolicyRegistry::secure_defaults();
        if let Some(id) = facts.iter().find_map(|fact| match &fact.audience {
            FactAudience::VisibilityPolicy(id) if !policies.contains(id) => Some(id.clone()),
            _ => None,
        }) {
            return Err(ProjectionError::MissingProjectionPolicy(id));
        }
        let mut ordered: Vec<_> = facts
            .iter()
            .filter(|fact| audience_allows(&fact.audience, self.team_id))
            .collect();
        ordered.sort_by_key(|fact| {
            (
                fact.key.fact_kind,
                fact_subject_replica(fact, &self.identity),
                fact.key.local_ordinal,
            )
        });
        for fact in ordered {
            // Control rows exist only for this team's own heroes.
            if matches!(fact.fact, ObservableFact::OwnerControl { .. })
                && !matches!(fact.audience, FactAudience::Team(value) if value == self.team_id) {
                continue;
            }
            if let ObservableFact::OwnerEconomy { team, .. } | ObservableFact::OwnerShopUndo { team, .. } = &fact.fact {
                if *team != self.team_id || !matches!(fact.audience, FactAudience::Team(value) if value == self.team_id) {
                    continue;
                }
            }
            if let ObservableFact::CommittedEconomy { source, .. } = &fact.fact {
                let owns_economy = self.identity.replica_for(unpack_canonical(*source))
                    .and_then(|mapping| self.hash_entities.get(&mapping.replica_id.get()))
                    .is_some_and(|(_, _, components)| components.contains_key(&crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID));
                if !owns_economy || !matches!(fact.audience, FactAudience::Team(team) if team == self.team_id) {
                    continue;
                }
            }
            // Our team's cooldowns remain deterministic gameplay from its
            // accepted inputs. Only disclosed opponents need settled state.
            if let ObservableFact::CommittedCooldown { source, .. } | ObservableFact::CommittedCharge { source, .. } = &fact.fact {
                if self.identity.replica_for(unpack_canonical(*source))
                    .and_then(|mapping| self.hash_entities.get(&mapping.replica_id.get()))
                    .and_then(|(_, _, components)| components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID))
                    .and_then(|bytes| crate::runtime::decode_demo_render_state(bytes))
                    .is_some_and(|render| render.team_id == self.team_id) {
                    continue;
                }
            }
            project_fact(
                fact,
                visible,
                &self.identity,
                &mut public_events,
                &mut external_effects,
            );
        }
        if let Some(event) = self.pending_fog_event.take() { public_events.push(event); }
        for (&id, (_, _, components)) in &self.hash_entities {
            if let Some(range)=components.get(&crate::runtime::attack_visual_state::RANGE_SCHEMA_ID){
                public_events.push(TeamPublicEvent {event_kind:crate::runtime::FactKind::DisclosedAttackRange as u32,subject:Some(ProtoReplicaEntityId {value:id}),sanitized_payload:range.clone(),stable_sub_index:0});
            }
            if let Some(state)=components.get(&crate::runtime::DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID) {
                public_events.push(TeamPublicEvent {event_kind:crate::runtime::FactKind::DisclosedCombatRelation as u32,
                    subject:Some(ProtoReplicaEntityId {value:id}),sanitized_payload:state.clone(),stable_sub_index:0});
            }
            if let Some(state) = components.get(&crate::runtime::channel_visual_state::SCHEMA_ID) {
                public_events.push(TeamPublicEvent { event_kind: crate::runtime::FactKind::ChannelVisual as u32,
                    subject: Some(ProtoReplicaEntityId { value: id }), sanitized_payload: state.clone(), stable_sub_index: 0 });
            }
            if let Some(state) = components.get(&crate::runtime::attack_visual_state::SCHEMA_ID) {
                public_events.push(TeamPublicEvent { event_kind: crate::runtime::FactKind::AttackVisual as u32,
                    subject: Some(ProtoReplicaEntityId { value: id }), sanitized_payload: state.clone(), stable_sub_index: 0 });
            }
            if let Some(state)=components.get(&crate::runtime::buff_visual_state::SCHEMA_ID) {
                public_events.push(TeamPublicEvent {event_kind:crate::runtime::FactKind::OwnerBuffVisual as u32,
                    subject:Some(ProtoReplicaEntityId {value:id}),sanitized_payload:state.clone(),stable_sub_index:0});
            }
        }
        public_events.sort_by_key(|event| {
            (
                event.event_kind,
                event.subject.as_ref().map_or(0, |id| id.value),
                event.stable_sub_index,
            )
        });
        external_effects.sort_by(|left, right| {
            (
                left.effect_kind,
                left.visible_target.as_ref().map_or(0, |id| id.value),
                left.stable_sub_index,
                &left.sanitized_payload,
            )
                .cmp(&(
                    right.effect_kind,
                    right.visible_target.as_ref().map_or(0, |id| id.value),
                    right.stable_sub_index,
                    &right.sanitized_payload,
                ))
        });
        // local_ordinal is only unique per producer. Assign a team-safe ordinal
        // after sorting sanitized facts, without exposing the hidden producer.
        assign_presentation_ordinals(&mut public_events, &mut external_effects);
        for effect in &mut external_effects {
            let replica_id = effect.visible_target.as_ref().map_or(0, |id| id.value);
            if let Some(property) = self.latest_external_property_by_replica.get(&replica_id) {
                effect.sanitized_payload.extend_from_slice(b"PROP");
                effect.sanitized_payload.extend_from_slice(property);
            }
        }
        let mut accepted_inputs = accepted
            .into_iter()
            .filter(|input| input.team_id == self.team_id)
            .filter(|input| !super::input_scheme_transport::is_owner_setting_action(input.action_kind) ||
                super::input_scheme_transport::owner_setting_available(self.combat_preferences.as_ref(),self.view_epoch,input.action_kind,input.player_id))
            .filter_map(|input| {
                // New summon acknowledgements are notifications with redacted lives.
                if input.action_kind==super::direction_transport::ACTION_KIND
                    && !super::direction_transport::available(self.combat_preferences.as_ref(),self.view_epoch,input.player_id) {return None;}
                let actor = if super::input_scheme_transport::is_owner_setting_action(input.action_kind) && input.actor_canonical_id == 0 {
                    None
                } else {
                    Some(replica_proto(&self.identity, input.actor_canonical_id)?)
                };
                let target = input
                    .target_canonical_id
                    .and_then(|id| replica_proto(&self.identity, id));
                if input.target_canonical_id.is_some() && target.is_none() {
                    return None;
                }
                let projected = crate::game_proto::TeamAcceptedInput {
                    player_id: input.player_id,
                    input_id: input.input_id,
                    action_kind: input.action_kind,
                    actor,
                    target,
                    sanitized_payload: input.sanitized_payload,
                    stable_sub_index: 0,
                };
                if super::input_scheme_transport::is_owner_setting_action(projected.action_kind)
                    && super::input_scheme_transport::validate_owner_setting_ack(&projected).is_err() { return None; }
                if projected.action_kind==super::direction_transport::ACTION_KIND
                    && super::direction_transport::validate_projected_ack(&projected).is_err() {return None;}
                if super::direction_transport::reject_kind_alias(&projected).is_err() {return None;}
                if super::summon_transport::reject_kind_alias(&projected).is_err() {return None;}
                if projected.action_kind==super::summon_transport::ACTION_KIND
                    && super::summon_transport::validate_projected_ack(&projected).is_err() {return None;}
                Some(projected)
            })
            .collect::<Vec<_>>();
        accepted_inputs.sort_by_key(|input| (input.player_id, input.input_id));
        for (index, input) in accepted_inputs.iter_mut().enumerate() {
            input.stable_sub_index = index as u32;
        }
        Ok(Step {
            accepted_inputs,
            public_events,
            random_tapes: Vec::new(),
            external_effects,
        })
    }
}

fn audience_allows(audience: &FactAudience, team: u32) -> bool {
    matches!(audience, FactAudience::AllPlayers)
        || matches!(audience, FactAudience::Team(value) if *value == team)
        || matches!(audience, FactAudience::VisibilityPolicy(_))
}

fn assign_presentation_ordinals(public: &mut [TeamPublicEvent], external: &mut [SanitizedExternalEffect]) {
    for (index, effect) in external.iter_mut().enumerate() {
        effect.stable_sub_index = u32::try_from(index).expect("bounded team frame effects");
    }
    // Both lists can become renderer one-shots. Producer-local ordinals are
    // neither globally unique nor safe event identities across these lists.
    for (index, event) in public.iter_mut().enumerate() {
        event.stable_sub_index = u32::try_from(external.len().checked_add(index)
            .expect("bounded team frame events")).expect("bounded team frame events");
    }
}

fn project_fact(
    ordered: &OrderedFact,
    visible: &BTreeSet<u64>,
    identity: &TeamIdentityState,
    public: &mut Vec<TeamPublicEvent>,
    external: &mut Vec<SanitizedExternalEffect>,
) {
    let (source, target, payload) = fact_entities_and_payload(&ordered.fact, identity.team_id());
    let source_visible = source.is_some_and(|id| visible.contains(&id));
    let target_visible = target.is_some_and(|id| visible.contains(&id));
    // Source-less impact is not a global event. Remembered/forgotten targets
    // cannot anchor it, even if an inconsistent visibility set contains them.
    if matches!(ordered.fact, ObservableFact::ProjectileImpact { .. } | ObservableFact::AttackAvoided { .. } | ObservableFact::CriticalHit { .. })
        && (!target_visible || target.and_then(|id| replica_proto(identity, id)).is_none()) {
        return;
    }
    // Ability payload identifies the caster's skill, not a target-only impact.
    // A visible victim must never disclose a hidden/remembered caster's cast.
    if matches!(ordered.fact, ObservableFact::Ability { .. } | ObservableFact::AbilityArea { .. })
        && (!source_visible || source.and_then(|id| replica_proto(identity, id)).is_none()) {
        return;
    }
    let target_replica = target.and_then(|id| replica_proto(identity, id));
    let authority_owned_combat = matches!(&ordered.audience,
        FactAudience::VisibilityPolicy(id)
            if id == omb_script_abi::types::projection_policy_ids::EXTERNAL_DIRECT_COMBAT);
    let hidden_source_requires_sanitizing = (!source_visible || authority_owned_combat)
        && target_visible
        && matches!(
            ordered.fact,
            ObservableFact::DirectCombat { .. }
                | ObservableFact::Buff { .. }
                | ObservableFact::Projectile { .. }
                | ObservableFact::AreaEffect { .. }
                | ObservableFact::Collision { .. }
                | ObservableFact::RandomOutcome { .. }
        );
    if hidden_source_requires_sanitizing {
        external.push(SanitizedExternalEffect {
            effect_kind: ordered.key.fact_kind as u32,
            visible_target: target_replica,
            sanitized_payload: payload,
            stable_sub_index: ordered.key.local_ordinal,
        });
    } else if !authority_owned_combat && (source_visible || target_visible || source.is_none()) {
        public.push(TeamPublicEvent {
            event_kind: ordered.key.fact_kind as u32,
            subject: source
                .and_then(|id| replica_proto(identity, id))
                .or(target_replica),
            sanitized_payload: payload,
            stable_sub_index: ordered.key.local_ordinal,
        });
    }
}

fn fact_entities_and_payload(fact: &ObservableFact, viewer_team: u32) -> (Option<u64>, Option<u64>, Vec<u8>) {
    let mut payload = Vec::new();
    match fact {
        ObservableFact::ProjectileImpact { target } => (None, Some(*target), b"HIT1".to_vec()),
        ObservableFact::AttackAvoided { target } => (None, Some(*target), b"AVD1".to_vec()),
        ObservableFact::CriticalHit {target}=>(None,Some(*target),b"CRT1".to_vec()),
        ObservableFact::CommittedMana { source, state } => (Some(*source), None, state.encode()),
        ObservableFact::OwnerControl { source, batch, row } => (Some(*source), None, row.encode(*batch)),
        ObservableFact::CommittedAbilityRanks { source, ranks } => {
            for rank in ranks { payload.extend(rank.to_le_bytes()); }
            (Some(*source), None, payload)
        }
        ObservableFact::OwnerEconomy { state, .. } => (None, None, state.encode()),
        ObservableFact::OwnerShopUndo { state, .. } => (None, None, state.encode()),
        ObservableFact::CommittedEconomy { source, state } => (Some(*source), None, state.encode()),
        ObservableFact::CommittedEquipmentStats { source, hp_raw, max_hp_raw, speed_raw, armor_raw, attack_raw } => {
            for value in [hp_raw, max_hp_raw, speed_raw, armor_raw, attack_raw] { payload.extend(value.to_le_bytes()); }
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedProgression { source, level, experience, experience_to_next, skill_points } => {
            for value in [level, experience, experience_to_next, skill_points] { payload.extend(value.to_le_bytes()); }
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedAttack { source, elapsed_raw, sequence, phase } => {
            payload.extend(elapsed_raw.to_le_bytes());
            payload.extend(sequence.to_le_bytes());
            payload.push(*phase);
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedVitals { source, hp_raw, max_hp_raw } => {
            payload.extend(hp_raw.to_le_bytes());
            payload.extend(max_hp_raw.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedIncomingDamage { source, bonus_raw } => {
            payload.extend(bonus_raw.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedStructure { source, state } => {
            payload.extend(state);
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedCooldown { source, slot, remaining_raw } => {
            payload.extend(slot.to_le_bytes());
            payload.extend(remaining_raw.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::CommittedCharge { source, slot, charges, max, recharge_raw, remaining_raw } => {
            // 24 bytes LE: slot u32, charges u16, max u16, recharge i64, remaining i64.
            payload.extend(slot.to_le_bytes());
            payload.extend(charges.to_le_bytes());
            payload.extend(max.to_le_bytes());
            payload.extend(recharge_raw.to_le_bytes());
            payload.extend(remaining_raw.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::MovementPriority { source, active } => {
            payload.push(u8::from(*active));
            (Some(*source), None, payload)
        }
        ObservableFact::Movement {
            source,
            x_mm,
            y_mm,
            facing_ticks,
        } => {
            payload.extend(x_mm.to_le_bytes());
            payload.extend(y_mm.to_le_bytes());
            if let Some(ticks) = facing_ticks {
                payload.extend(ticks.to_le_bytes());
            }
            (Some(*source), None, payload)
        }
        ObservableFact::Spawn {
            source,
            template_id,
            team,
        } => {
            payload.extend(template_id.to_le_bytes());
            payload.extend(team.to_le_bytes());
            (*source, None, payload)
        }
        ObservableFact::Death { source, killer } => (Some(*source), *killer, payload),
        ObservableFact::Ownership { source, team } => {
            payload.extend(team.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::DirectCombat {
            source,
            target,
            amount_milli,
        } => {
            payload.extend(amount_milli.to_le_bytes());
            (Some(*source), Some(*target), payload)
        }
        ObservableFact::Projectile {
            source,
            target,
            effect_id,
            active,
        } => {
            payload.extend(effect_id.to_le_bytes());
            payload.push(u8::from(*active));
            (Some(*source), *target, payload)
        }
        ObservableFact::AreaEffect {
            source,
            x_mm,
            y_mm,
            radius_mm,
        } => {
            payload.extend(x_mm.to_le_bytes());
            payload.extend(y_mm.to_le_bytes());
            payload.extend(radius_mm.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::Buff {
            source,
            target,
            effect_id,
            active,
        } => {
            payload.extend(effect_id.to_le_bytes());
            payload.push(u8::from(*active));
            (Some(*source), Some(*target), payload)
        }
        ObservableFact::AbilityArea {source, team, ability_id,rank,x_raw,y_raw,radius_raw,duration_raw} => {
            if *team == 0 || *team != viewer_team {return (None,None,Vec::new());}
            payload = crate::runtime::presentation_cue::AbilityAreaPresentationCue::public_payload(
                *ability_id,*rank,*x_raw,*y_raw,*radius_raw,*duration_raw);
            (Some(*source),None,payload)
        },
        ObservableFact::Ability {
            source,
            ability_id,
            rank,
            caster_relocation,
            target,
        } => {
            payload = crate::runtime::presentation_cue::AbilityPresentationCue::public_payload(*ability_id, *rank);
            if let Some((_, x, y)) = caster_relocation.filter(|(team, _, _)| *team != 0 && *team == viewer_team) {
                payload = crate::runtime::presentation_cue::AbilityPresentationCue::public_relocation_payload(*ability_id, *rank, x, y);
            }
            (Some(*source), *target, payload)
        }
        ObservableFact::Tower { source, action_id } => {
            payload.extend(action_id.to_le_bytes());
            (Some(*source), None, payload)
        }
        ObservableFact::Item {
            source,
            item_id,
            target,
        } => {
            payload.extend(item_id.to_le_bytes());
            (Some(*source), *target, payload)
        }
        ObservableFact::Hud {
            team,
            metric_id,
            value,
        } => {
            payload.extend(team.to_le_bytes());
            payload.extend(metric_id.to_le_bytes());
            payload.extend(value.to_le_bytes());
            (None, None, payload)
        }
        ObservableFact::Terminal {
            result_code,
            winning_team,
        } => {
            payload.extend(result_code.to_le_bytes());
            if let Some(team) = winning_team {
                payload.extend(team.to_le_bytes());
            }
            (None, None, payload)
        }
        ObservableFact::Collision {
            source,
            target,
            x_raw,
            y_raw,
        } => {
            payload.extend(x_raw.to_le_bytes());
            payload.extend(y_raw.to_le_bytes());
            (Some(*source), Some(*target), payload)
        }
        ObservableFact::RandomOutcome {
            source,
            target,
            value,
        } => {
            payload.extend(value.to_le_bytes());
            (Some(*source), Some(*target), payload)
        }
    }
}

fn fact_subject_replica(fact: &OrderedFact, identity: &TeamIdentityState) -> u64 {
    fact_entities_and_payload(&fact.fact, identity.team_id())
        .0
        .and_then(|id| replica_proto(identity, id))
        .map_or(0, |id| id.value)
}

fn replica_proto(identity: &TeamIdentityState, canonical: u64) -> Option<ProtoReplicaEntityId> {
    identity
        .replica_for(unpack_canonical(canonical))
        .filter(|mapping| mapping.visibility == MappingVisibility::Disclosed)
        .map(|mapping| ProtoReplicaEntityId {
            value: mapping.replica_id.get(),
        })
}

fn unpack_canonical(value: u64) -> CanonicalEntityKey {
    CanonicalEntityKey {
        id: value as u32,
        generation: (value >> 32) as u32,
    }
}

fn transition_sort_key(value: &Transition) -> (u8, u64, u32) {
    match value.transition.as_ref() {
        Some(transition::Transition::Reveal(item)) => (
            0,
            item.replica_entity_id.as_ref().map_or(0, |id| id.value),
            item.stable_sub_index,
        ),
        Some(transition::Transition::Replace(item)) => (
            1,
            item.replica_entity_id.as_ref().map_or(0, |id| id.value),
            item.stable_sub_index,
        ),
        Some(transition::Transition::Hide(item)) => (
            2,
            item.replica_entity_id.as_ref().map_or(0, |id| id.value),
            item.stable_sub_index,
        ),
        Some(transition::Transition::Forget(item)) => (
            3,
            item.replica_entity_id.as_ref().map_or(0, |id| id.value),
            item.stable_sub_index,
        ),
        None => (u8::MAX, 0, 0),
    }
}

fn expected_team_hash(
    team: u32,
    tick: u64,
    entities: &BTreeMap<u64, (u64, u32, BTreeMap<u32, Vec<u8>>)>,
    undo_material: &[u8],
    combat_preferences: Option<&crate::game_proto::CombatPreferencesProjection>,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(crate::runtime::shop_undo_wire::CANONICAL_TEAM_VIEW_DOMAIN);
    hasher.update(team.to_be_bytes());
    hasher.update(tick.to_be_bytes());
    for (replica_id, (epoch, entity_kind, components)) in entities {
        hasher.update(replica_id.to_be_bytes());
        hasher.update(epoch.to_be_bytes());
        hasher.update(entity_kind.to_be_bytes());
        for (schema_id, bytes) in components {
            hasher.update(schema_id.to_be_bytes());
            hasher.update((bytes.len() as u64).to_be_bytes());
            hasher.update(bytes);
        }
    }
    crate::runtime::shop_undo_wire::mix_team_undo_material(&mut hasher, undo_material);
    crate::runtime::combat_transport::mix_projection_hash(&mut hasher, combat_preferences);
    hasher.finalize().into()
}

fn team_safe_components(bytes: &[u8], team: u32) -> BTreeMap<u32, Vec<u8>> {
    let mut components = decode_safe_components_for_hash(bytes);
    let owns=components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
        .and_then(|bytes|crate::runtime::decode_demo_render_state(bytes))
        .is_some_and(|state|state.team_id==team && state.owner_player_id!=0);
    // Another team sees only the public statuses' icons (v3 `reveal = "public"`, E216); the owner team sees all.
    if !owns {
        let schema = crate::runtime::buff_visual_state::SCHEMA_ID;
        match components.get(&schema).map(|bytes| crate::runtime::buff_visual_state::public_snapshot(bytes)) {
            Some(Some(public)) => { components.insert(schema, public); }
            Some(None) => { components.remove(&schema); }
            None => {}
        }
    }
    if components.contains_key(&crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID) {
        let owned = components.get(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
            .and_then(|bytes| crate::runtime::decode_demo_render_state(bytes))
            .is_some_and(|state| state.team_id == team && state.owner_player_id != 0);
        if !owned {
            for schema in [crate::runtime::DISCLOSED_GOLD_COMPONENT_SCHEMA_ID,
                crate::runtime::DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID,
                crate::runtime::DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID] {
                components.remove(&schema);
            }
        }
    }
    components
}

fn decode_safe_components_for_hash(bytes: &[u8]) -> BTreeMap<u32, Vec<u8>> {
    let mut result = BTreeMap::new();
    let Some(count) = bytes
        .get(..4)
        .map(|value| u32::from_be_bytes(value.try_into().unwrap()))
    else {
        return result;
    };
    let mut cursor = 4usize;
    for _ in 0..count {
        let Some(schema) = bytes.get(cursor..cursor + 4) else {
            return BTreeMap::new();
        };
        cursor += 4;
        let Some(len) = bytes.get(cursor..cursor + 4) else {
            return BTreeMap::new();
        };
        cursor += 4;
        let schema = u32::from_be_bytes(schema.try_into().unwrap());
        let len = u32::from_be_bytes(len.try_into().unwrap()) as usize;
        let Some(value) = bytes.get(cursor..cursor + len) else {
            return BTreeMap::new();
        };
        cursor += len;
        result.insert(schema, value.to_vec());
    }
    if cursor == bytes.len() {
        result
    } else {
        BTreeMap::new()
    }
}

fn pad_frame(
    mut frame: TeamTickFrame,
    buckets: &[usize],
) -> Result<PaddedTeamFrame, ProjectionError> {
    if buckets.is_empty() {
        return Err(ProjectionError::EmptyPaddingBuckets);
    }
    let canonical_bytes = frame.encode_to_vec();
    let steady_empty = frame
        .pre_step
        .as_ref()
        .is_none_or(|pre| pre.transitions.is_empty())
        && frame.step.as_ref().is_none_or(|step| {
            step.accepted_inputs.is_empty()
                && step.public_events.is_empty()
                && step.random_tapes.is_empty()
                && step.external_effects.is_empty()
        })
        && frame.post_step.as_ref().is_none_or(|post| {
            post.component_repairs.is_empty()
                && post.entity_replaces.is_empty()
                && post.hash_checkpoint.is_none()
                && post.rebase_notice.is_none()
        });
    // Hidden-only activity has already been filtered before this point, so an
    // empty heartbeat has one canonical shape and carries no activity oracle.
    // Avoiding padding here keeps 120 Hz steady-state traffic within budget.
    if steady_empty {
        return Ok(PaddedTeamFrame {
            frame,
            canonical_bytes: canonical_bytes.clone(),
            wire_bytes: canonical_bytes,
            padding_len: 0,
        });
    }
    for bucket in buckets
        .iter()
        .copied()
        .filter(|size| *size >= canonical_bytes.len() && *size <= MAX_PADDED_TEAM_FRAME_BYTES)
    {
        let mut low = 0usize;
        let mut high = bucket - canonical_bytes.len();
        while low <= high {
            let padding_len = low + (high - low) / 2;
            frame.padding.resize(padding_len, 0);
            let wire_bytes = frame.encode_to_vec();
            if wire_bytes.len() == bucket {
                return Ok(PaddedTeamFrame {
                    frame,
                    canonical_bytes,
                    wire_bytes,
                    padding_len,
                });
            }
            if wire_bytes.len() < bucket {
                low = padding_len.saturating_add(1);
            } else if padding_len == 0 {
                break;
            } else {
                high = padding_len - 1;
            }
        }
    }
    Err(ProjectionError::FrameTooLarge)
}

pub fn safe_mismatch_metadata(
    team_id: u32,
    replica_tick: u64,
    expected: &[u8; 32],
    received: &[u8; 32],
    disclosed_entity_count: usize,
) -> SafeMismatchMetadata {
    let mut expected_hash_prefix = [0; 8];
    let mut received_hash_prefix = [0; 8];
    expected_hash_prefix.copy_from_slice(&expected[..8]);
    received_hash_prefix.copy_from_slice(&received[..8]);
    SafeMismatchMetadata {
        team_id,
        replica_tick,
        expected_hash_prefix,
        received_hash_prefix,
        disclosed_entity_count: disclosed_entity_count.min(u32::MAX as usize) as u32,
    }
}

/// Explicit sanitizer used when an authoritative hidden AOE changes a visible
/// target. Only the effect class, visible replica target and bounded numeric
/// result cross the boundary; source, origin, radius and affected count do not.
pub fn sanitize_hidden_aoe_effect(
    visible_target: u64,
    amount_milli: i64,
    stable_sub_index: u32,
) -> SanitizedExternalEffect {
    SanitizedExternalEffect {
        effect_kind: crate::runtime::FactKind::AreaEffect as u32,
        visible_target: Some(ProtoReplicaEntityId {
            value: visible_target,
        }),
        sanitized_payload: amount_milli.to_le_bytes().to_vec(),
        stable_sub_index,
    }
}

#[cfg(test)]
mod ability_cue_projection_tests {
    use super::*;

    #[test]
    fn buff_visual_state_shows_every_icon_to_the_owner_team_and_public_ones_to_others() {
        use crate::runtime::buff_visual_state::{BuffVisualState,SCHEMA_ID};
        // E216: stun is public (every v3 status held as `stun`); a hand-written icon no v3 status declares is not.
        let stun=omoba_template_ids::buff_by_name("stun").unwrap();
        assert!(omoba_template_ids::buff_is_public(stun));
        let private=(1..u16::MAX).map(omoba_template_ids::BuffId).find(|id|omoba_template_ids::buff_id_str(*id)!=""
            && !omoba_template_ids::buff_is_public(*id)).expect("a non-public buff");
        let state=BuffVisualState(vec![(stun.raw(),1024),(private.raw(),2048)]);
        let public=BuffVisualState(vec![(stun.raw(),1024)]).encode().unwrap();
        for owner in [0,7] {
            let bytes=crate::runtime::encode_disclosed_baseline(&[
                (crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,crate::runtime::encode_demo_render_state(
                    crate::runtime::DemoRenderState {x_raw:0,y_raw:0,team_id:1,kind:1,owner_player_id:owner})),
                (SCHEMA_ID,state.encode().unwrap())]);
            let own=team_safe_components(&bytes,1);
            assert_eq!(own.get(&SCHEMA_ID),Some(&if owner!=0 {state.encode().unwrap()} else {public.clone()}),"owner {owner}");
            assert_eq!(team_safe_components(&bytes,2).get(&SCHEMA_ID),Some(&public),"the other team sees the public icon only");
        }
    }

    #[cfg(feature="kcp")]
    #[tokio::test]
    async fn authored_map_bursts_use_bounded_padding_and_round_trip_real_kcp_framing() {
        let config=TeamProjectorConfig::default();
        assert_eq!(config.size_buckets.last(),Some(&MAX_PADDED_TEAM_FRAME_BYTES));
        for size in [17*1024,50*1024,MAX_PADDED_TEAM_FRAME_BYTES+1] {
            let frame=TeamTickFrame {step:Some(Step {public_events:vec![TeamPublicEvent {
                event_kind:1,sanitized_payload:vec![7;size],..Default::default()}],..Default::default()}),..Default::default()};
            let result=pad_frame(frame,&config.size_buckets);
            if size>MAX_PADDED_TEAM_FRAME_BYTES {assert!(matches!(result,Err(ProjectionError::FrameTooLarge)));continue;}
            let padded=result.unwrap();
            assert!(config.size_buckets.contains(&padded.wire_bytes.len()));
            assert!(padded.wire_bytes.len()<=MAX_PADDED_TEAM_FRAME_BYTES);
            let (mut writer,mut reader)=tokio::io::duplex(2*MAX_PADDED_TEAM_FRAME_BYTES);
            crate::kcp::framing::write_framed(&mut writer,crate::wire_tags::TAG_TEAM_TICK_FRAME_V2,&padded.wire_bytes).await.unwrap();
            let (tag,bytes,wire_len)=crate::kcp::framing::read_framed(&mut reader).await.unwrap().unwrap();
            assert_eq!(tag,crate::wire_tags::TAG_TEAM_TICK_FRAME_V2);
            assert_eq!(bytes,padded.wire_bytes);
            assert!(wire_len<=MAX_PADDED_TEAM_FRAME_BYTES+5);
            assert_eq!(TeamTickFrame::decode(bytes.as_slice()).unwrap(),padded.frame);
        }
    }

    #[test]
    fn buff_visual_state_projector_publishes_current_countdown_and_empty_state() {
        use crate::runtime::buff_visual_state::{BuffVisualState,SCHEMA_ID};
        assert!(crate::runtime::secure_replica_component_allowlist().contains(&SCHEMA_ID));
        let canonical=CanonicalEntityKey {id:7,generation:1};let source=(1u64<<32)|7;
        let id=omoba_template_ids::buff_by_name("slow").unwrap().raw();
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
        let mapping=projector.identity.disclose(canonical).unwrap();
        projector.hash_entities.insert(mapping.replica_id.get(),(mapping.disclosure_epoch,0,BTreeMap::new()));
        for (tick,state) in [(1,BuffVisualState(vec![(id,2048)])),(2,BuffVisualState(vec![(id,1024)])),(3,BuffVisualState(vec![]))] {
            let baseline=crate::runtime::encode_disclosed_baseline(&[
                (crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,crate::runtime::encode_demo_render_state(
                    crate::runtime::DemoRenderState {x_raw:0,y_raw:0,team_id:1,kind:1,owner_player_id:7})),
                (SCHEMA_ID,state.encode().unwrap())]);
            let visible=BTreeSet::from([source]);
            projector.refresh_expected_visible_components(tick,&visible,&BTreeMap::from([(source,baseline)]));
            let step=projector.build_step(tick,&visible,&[],vec![],&[],&[]).unwrap();
            let events:Vec<_>=step.public_events.iter().filter(|event|event.event_kind==crate::runtime::FactKind::OwnerBuffVisual as u32).collect();
            assert_eq!(events.len(),1);assert_eq!(events[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
            assert_eq!(events[0].sanitized_payload,state.encode().unwrap());
        }
    }

    #[test]
    fn public_channel_duration_remains_owner_team_only_and_clears() {
        use crate::runtime::buff_visual_state::{BuffVisualSnapshot,BuffVisualState,SCHEMA_ID};
        let id=omoba_template_ids::buff_by_name("recovery_channel").unwrap().raw();
        let canonical=CanonicalEntityKey {id:7,generation:1};let source=(1u64<<32)|7;
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());let mapping=projector.identity.disclose(canonical).unwrap();
        projector.hash_entities.insert(mapping.replica_id.get(),(mapping.disclosure_epoch,0,BTreeMap::new()));
        for (tick,remaining) in [(1,1536),(2,1024),(3,0)] {
            let state=if remaining==0 {BuffVisualSnapshot {buffs:BuffVisualState(vec![]),channel_totals:vec![]}}
                else {BuffVisualSnapshot {buffs:BuffVisualState(vec![(id,remaining)]),channel_totals:vec![(id,3072)]}};
            let bytes=state.encode().unwrap();let baseline=crate::runtime::encode_disclosed_baseline(&[
                (crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,crate::runtime::encode_demo_render_state(crate::runtime::DemoRenderState {x_raw:0,y_raw:0,team_id:1,kind:1,owner_player_id:7})),(SCHEMA_ID,bytes.clone())]);
            // recovery_channel is no v3 status: the other team gets the empty public snapshot.
            assert!(team_safe_components(&baseline,1).contains_key(&SCHEMA_ID));
            assert_eq!(team_safe_components(&baseline,2).get(&SCHEMA_ID),Some(&BuffVisualState(vec![]).encode().unwrap()));
            let visible=BTreeSet::from([source]);projector.refresh_expected_visible_components(tick,&visible,&BTreeMap::from([(source,baseline)]));
            let step=projector.build_step(tick,&visible,&[],vec![],&[],&[]).unwrap();
            let events:Vec<_>=step.public_events.iter().filter(|event|event.event_kind==crate::runtime::FactKind::OwnerBuffVisual as u32).collect();assert_eq!(events.len(),1);assert_eq!(events[0].sanitized_payload,bytes);
        }
        projector.hash_entities.clear();let hidden=projector.build_step(4,&BTreeSet::new(),&[],vec![],&[],&[]).unwrap();
        assert!(!hidden.public_events.iter().any(|event|event.event_kind==crate::runtime::FactKind::OwnerBuffVisual as u32));
    }

    #[test]
    fn public_range_projector_only_updates_current_disclosures() {
        use crate::runtime::attack_visual_state::{RANGE_SCHEMA_ID as SCHEMA,encode_range};
        let canonical=CanonicalEntityKey{id:7,generation:1};let source=(1u64<<32)|7;let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());let mapping=projector.identity.disclose(canonical).unwrap();projector.hash_entities.insert(mapping.replica_id.get(),(mapping.disclosure_epoch,0,BTreeMap::new()));
        for (tick,raw) in [(1,256001),(2,102400),(3,0)]{let bytes=encode_range(omoba_sim::Fixed64::from_raw(raw)).unwrap();let baseline=crate::runtime::encode_disclosed_baseline(&[(crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,crate::runtime::encode_demo_render_state(crate::runtime::DemoRenderState{x_raw:0,y_raw:0,team_id:2,kind:3,owner_player_id:0})),(SCHEMA,bytes.clone())]);let visible=BTreeSet::from([source]);projector.refresh_expected_visible_components(tick,&visible,&BTreeMap::from([(source,baseline)]));let step=projector.build_step(tick,&visible,&[],vec![],&[],&[]).unwrap();let events:Vec<_>=step.public_events.iter().filter(|e|e.event_kind==crate::runtime::FactKind::DisclosedAttackRange as u32).collect();assert_eq!(events.len(),1);assert_eq!(events[0].subject.as_ref().unwrap().value,mapping.replica_id.get());assert_eq!(events[0].sanitized_payload,bytes);}
        projector.hash_entities.clear();assert!(!projector.build_step(4,&BTreeSet::new(),&[],vec![],&[],&[]).unwrap().public_events.iter().any(|e|e.event_kind==crate::runtime::FactKind::DisclosedAttackRange as u32));
    }
    #[test]
    fn attack_visual_state_projector_updates_visible_enemy_and_idle() {
        use crate::runtime::attack_visual_state::{AttackVisualState, SCHEMA_ID};
        assert!(crate::runtime::secure_replica_component_allowlist().contains(&SCHEMA_ID));
        let canonical = CanonicalEntityKey { id: 7, generation: 1 };
        let source = (1u64 << 32) | 7;
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let mapping = projector.identity.disclose(canonical).unwrap();
        projector.hash_entities.insert(mapping.replica_id.get(), (mapping.disclosure_epoch, 0, BTreeMap::new()));
        for (tick, state) in [(1, AttackVisualState { sequence: 9, phase: 1, paused: true, elapsed_raw: 40, duration_raw: 100 }),
            (2, AttackVisualState { sequence: 9, phase: 2, paused: false, elapsed_raw: 60, duration_raw: 200 }), (3, Default::default())] {
            let baseline = crate::runtime::encode_disclosed_baseline(&[
                (crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID, crate::runtime::encode_demo_render_state(
                    crate::runtime::DemoRenderState { x_raw: 0, y_raw: 0, team_id: 2, kind: 1, owner_player_id: 9 })),
                (SCHEMA_ID, state.encode().unwrap())]);
            let visible = BTreeSet::from([source]);
            projector.refresh_expected_visible_components(tick, &visible, &BTreeMap::from([(source, baseline)]));
            let step = projector.build_step(tick, &visible, &[], vec![], &[], &[]).unwrap();
            let events: Vec<_> = step.public_events.iter().filter(|event| event.event_kind == crate::runtime::FactKind::AttackVisual as u32).collect();
            assert_eq!(events.len(), 1);
            assert_eq!(events[0].subject.as_ref().unwrap().value, mapping.replica_id.get());
            assert_eq!(events[0].sanitized_payload, state.encode().unwrap());
        }
        projector.hash_entities.clear();
        let hidden = projector.build_step(4, &BTreeSet::new(), &[], vec![], &[], &[]).unwrap();
        assert!(!hidden.public_events.iter().any(|event| event.event_kind == crate::runtime::FactKind::AttackVisual as u32));
    }

    #[test]
    fn combat_relation_projector_updates_current_disclosure_only() {
        use crate::runtime::{DisclosedCombatRelation,encode_disclosed_combat_relation,decode_disclosed_combat_relation,DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID as SCHEMA};
        let canonical=CanonicalEntityKey{id:7,generation:1};let source=(1u64<<32)|7;
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
        let mapping=projector.identity.disclose(canonical).unwrap();
        projector.hash_entities.insert(mapping.replica_id.get(),(mapping.disclosure_epoch,0,BTreeMap::new()));
        for (tick,alive,attackable) in [(1,true,false),(2,true,true),(3,false,false),(4,true,true)] {
            let relation=DisclosedCombatRelation{faction_kind:2,team_id:2,alive,attackable};
            let baseline=crate::runtime::encode_disclosed_baseline(&[(SCHEMA,encode_disclosed_combat_relation(relation).unwrap())]);
            let visible=BTreeSet::from([source]);projector.refresh_expected_visible_components(tick,&visible,&BTreeMap::from([(source,baseline)]));
            let step=projector.build_step(tick,&visible,&[],vec![],&[],&[]).unwrap();
            let events:Vec<_>=step.public_events.iter().filter(|event|event.event_kind==crate::runtime::FactKind::DisclosedCombatRelation as u32).collect();
            assert_eq!(events.len(),1);assert_eq!(events[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
            assert_eq!(decode_disclosed_combat_relation(&events[0].sanitized_payload),Some(relation));assert_eq!(events[0].sanitized_payload.len(),8);
        }
        projector.hash_entities.clear();let hidden=projector.build_step(5,&BTreeSet::new(),&[],vec![],&[],&[]).unwrap();
        assert!(!hidden.public_events.iter().any(|event|event.event_kind==crate::runtime::FactKind::DisclosedCombatRelation as u32));
    }

    #[test]
    fn channel_visual_state_projector_discloses_visible_enemy_and_clears_channel() {
        use crate::runtime::channel_visual_state::{ChannelVisualState,SCHEMA_ID};
        assert!(crate::runtime::secure_replica_component_allowlist().contains(&SCHEMA_ID));
        let canonical=CanonicalEntityKey{id:7,generation:1};let source=(1u64<<32)|7;
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
        let mapping=projector.identity.disclose(canonical).unwrap();
        projector.hash_entities.insert(mapping.replica_id.get(),(mapping.disclosure_epoch,0,BTreeMap::new()));
        for (tick,state) in [(1,ChannelVisualState::None),(2,ChannelVisualState::Recall),(3,ChannelVisualState::None)] {
            let baseline=crate::runtime::encode_disclosed_baseline(&[
                (crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID,crate::runtime::encode_demo_render_state(
                    crate::runtime::DemoRenderState{x_raw:0,y_raw:0,team_id:2,kind:1,owner_player_id:9})),
                (SCHEMA_ID,state.encode())]);
            let visible=BTreeSet::from([source]);
            projector.refresh_expected_visible_components(tick,&visible,&BTreeMap::from([(source,baseline)]));
            let step=projector.build_step(tick,&visible,&[],vec![],&[],&[]).unwrap();
            let events:Vec<_>=step.public_events.iter().filter(|event|event.event_kind==crate::runtime::FactKind::ChannelVisual as u32).collect();
            assert_eq!(events.len(),1);assert_eq!(events[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
            assert_eq!(ChannelVisualState::decode(&events[0].sanitized_payload),Some(state));
            assert_eq!(events[0].sanitized_payload.len(),5,"channel projection carries no private countdown or destination");
        }
        projector.hash_entities.clear();
        let hidden=projector.build_step(4,&BTreeSet::new(),&[],vec![],&[],&[]).unwrap();
        assert!(!hidden.public_events.iter().any(|event|event.event_kind==crate::runtime::FactKind::ChannelVisual as u32));
    }

    #[test]
    fn critical_hit_cue_requires_visible_disclosed_target() {
        use crate::runtime::{FactKind, FactOrderingKey, FactPhase};
        let target = CanonicalEntityKey { id: 42, generation: 1 };
        let canonical = (u64::from(target.generation) << 32) | u64::from(target.id);
        let mut identity = TeamIdentityState::new(1);
        let mapping = identity.disclose(target).unwrap();
        let fact = OrderedFact {
            key: FactOrderingKey {tick:17, phase:FactPhase::PostStep,
                canonical_source_order:canonical, local_ordinal:2, fact_kind:FactKind::CriticalHit},
            audience:FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
            fact:ObservableFact::CriticalHit {target:canonical},
        };
        let mut public=Vec::new(); let mut external=Vec::new();
        let visible=BTreeSet::from([canonical]);
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert_eq!(public.len(),1);
        assert_eq!(public[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
        assert_eq!(public[0].sanitized_payload,b"CRT1");
        assert!(external.is_empty());
        public.clear();
        project_fact(&fact,&BTreeSet::new(),&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.remember(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.forget(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty() && external.is_empty());
    }

    #[test]
    fn attack_avoidance_cue_requires_visible_disclosed_target() {
        use crate::runtime::{FactKind, FactOrderingKey, FactPhase};
        let target = CanonicalEntityKey { id: 42, generation: 1 };
        let canonical = (u64::from(target.generation) << 32) | u64::from(target.id);
        let mut identity = TeamIdentityState::new(1);
        let mapping = identity.disclose(target).unwrap();
        let fact = OrderedFact {
            key: FactOrderingKey {tick:17, phase:FactPhase::PostStep,
                canonical_source_order:canonical, local_ordinal:2, fact_kind:FactKind::AttackAvoided},
            audience:FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
            fact:ObservableFact::AttackAvoided {target:canonical},
        };
        let mut public=Vec::new(); let mut external=Vec::new();
        let visible=BTreeSet::from([canonical]);
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert_eq!(public.len(),1);
        assert_eq!(public[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
        assert_eq!(public[0].sanitized_payload,b"AVD1");
        assert!(external.is_empty());
        public.clear();
        project_fact(&fact,&BTreeSet::new(),&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.remember(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.forget(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty() && external.is_empty());
    }

    #[test]
    fn projectile_impact_cue_requires_visible_disclosed_target() {
        use crate::runtime::{FactKind, FactOrderingKey, FactPhase};
        let target = CanonicalEntityKey { id: 42, generation: 1 };
        let canonical = (u64::from(target.generation) << 32) | u64::from(target.id);
        let mut identity = TeamIdentityState::new(1);
        let mapping = identity.disclose(target).unwrap();
        let fact = OrderedFact {
            key: FactOrderingKey {tick:17, phase:FactPhase::PostStep,
                canonical_source_order:canonical, local_ordinal:2, fact_kind:FactKind::ProjectileImpact},
            audience:FactAudience::VisibilityPolicy(omb_script_abi::types::projection_policy_ids::PROJECTILE.to_owned()),
            fact:ObservableFact::ProjectileImpact {target:canonical},
        };
        let mut public=Vec::new(); let mut external=Vec::new();
        let visible=BTreeSet::from([canonical]);
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert_eq!(public.len(),1);
        assert_eq!(public[0].subject.as_ref().unwrap().value,mapping.replica_id.get());
        assert_eq!(public[0].sanitized_payload,b"HIT1");
        assert!(external.is_empty());
        public.clear();
        project_fact(&fact,&BTreeSet::new(),&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.remember(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty());
        identity.forget(target).unwrap();
        project_fact(&fact,&visible,&identity,&mut public,&mut external);
        assert!(public.is_empty() && external.is_empty());
    }

    #[test]
    fn ability_cue_ordinals_do_not_collide_with_damage_or_other_producers() {
        let mut external = vec![SanitizedExternalEffect {stable_sub_index: 0, ..Default::default()}; 2];
        let mut public = vec![TeamPublicEvent {stable_sub_index: 0, ..Default::default()}; 3];
        assign_presentation_ordinals(&mut public, &mut external);
        let ids: Vec<_> = external.iter().map(|event| event.stable_sub_index)
            .chain(public.iter().map(|event| event.stable_sub_index)).collect();
        assert_eq!(ids, vec![0, 1, 2, 3, 4]);
        assign_presentation_ordinals(&mut public, &mut external);
        assert_eq!(public.iter().map(|event| event.stable_sub_index).collect::<Vec<_>>(), vec![2, 3, 4]);
    }

    #[test]
    fn ability_cue_projection_never_uses_visible_target_for_hidden_caster() {
        use crate::runtime::{FactKind, FactOrderingKey, FactPhase};
        let caster = CanonicalEntityKey { id: 41, generation: 1 };
        let target = CanonicalEntityKey { id: 42, generation: 1 };
        let canonical = |key: CanonicalEntityKey| (u64::from(key.generation) << 32) | u64::from(key.id);
        let mut identity = TeamIdentityState::new(1);
        let caster_mapping = identity.disclose(caster).unwrap();
        identity.disclose(target).unwrap();
        identity.remember(caster).unwrap();
        let mut fact = OrderedFact {
            key: FactOrderingKey { tick: 17, phase: FactPhase::PostStep,
                canonical_source_order: canonical(caster), local_ordinal: 2, fact_kind: FactKind::Ability },
            audience: FactAudience::AllPlayers,
            fact: ObservableFact::Ability {source: canonical(caster), ability_id: 123, rank: 0, caster_relocation: None, target: Some(canonical(target))},
        };
        let mut public = Vec::new();
        let mut external = Vec::new();
        let target_only = BTreeSet::from([canonical(target)]);
        project_fact(&fact, &target_only, &identity, &mut public, &mut external);
        assert!(public.is_empty() && external.is_empty());
        project_fact(&fact, &BTreeSet::from([canonical(caster), canonical(target)]),
            &identity, &mut public, &mut external);
        assert!(public.is_empty() && external.is_empty());
        identity.forget(caster).unwrap();
        project_fact(&fact, &target_only, &identity, &mut public, &mut external);
        assert!(public.is_empty() && external.is_empty());
        let current = identity.disclose(caster).unwrap();
        assert_ne!(current.replica_id, caster_mapping.replica_id);
        // Disclosed caster may cast at a hidden target, but the public payload
        // must remain exactly the skill ID, with no target or coordinates.
        project_fact(&fact, &BTreeSet::from([canonical(caster)]), &identity, &mut public, &mut external);
        assert_eq!(public.len(), 1);
        assert_eq!(public[0].subject.as_ref().unwrap().value, current.replica_id.get());
        assert_eq!(public[0].sanitized_payload, 123_u64.to_le_bytes());
        assert!(external.is_empty());
        public.clear();
        fact.fact = ObservableFact::Ability {source: canonical(caster), ability_id: 123, rank: 0, caster_relocation: None, target: None};
        project_fact(&fact, &BTreeSet::new(), &identity, &mut public, &mut external);
        assert!(public.is_empty() && external.is_empty());
    }
}

/// Ticks of expected checkpoint digests retained per team for diagnostics.
pub const EXPECTED_DIGEST_TICKS: usize = 128;
/// Full expected snapshots kept for the exact bytes of a recent mismatch.
pub const EXPECTED_SNAPSHOT_TICKS: usize = 4;

/// Every disclosed component's bytes keyed by (replica id, schema id).
pub type ComponentBytes = BTreeMap<(u64, u32), Vec<u8>>;

pub fn component_bytes<E: ComponentSource>(entities: &BTreeMap<u64, E>) -> ComponentBytes {
    let mut out = BTreeMap::new();
    for (replica_id, entity) in entities {
        for (schema_id, bytes) in entity.components() { out.insert((*replica_id, *schema_id), bytes.clone()); }
    }
    out
}

/// One disclosed component of a team view: (replica id, schema, FNV-1a 64 of
/// the bytes). Carries no component content, only identity and a digest.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ComponentDigestRow { pub replica_id: u64, pub schema_id: u32, pub digest: u64 }

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3))
}

pub fn component_digest_rows<E>(entities: &BTreeMap<u64, E>) -> Vec<ComponentDigestRow>
where E: ComponentSource {
    let mut rows = Vec::new();
    for (replica_id, entity) in entities {
        for (schema_id, bytes) in entity.components() {
            rows.push(ComponentDigestRow { replica_id: *replica_id, schema_id: *schema_id, digest: fnv64(bytes) });
        }
    }
    rows
}

/// Both the projector's expected rows and a replica's disclosed world expose components.
pub trait ComponentSource { fn components(&self) -> &BTreeMap<u32, Vec<u8>>; }
impl ComponentSource for (u64, u32, BTreeMap<u32, Vec<u8>>) { fn components(&self) -> &BTreeMap<u32, Vec<u8>> { &self.2 } }
impl ComponentSource for crate::runtime::ReplicaEntityState { fn components(&self) -> &BTreeMap<u32, Vec<u8>> { &self.components } }

impl TeamProjectionRuntime {
    /// Canonical id behind a team's disclosed replica id (diagnostics only).
    pub fn canonical_for_replica(&self, team: u32, replica_id: u64) -> Option<u64> {
        self.projectors.get(&team)?.identity.disclosed_mappings().into_iter()
            .find(|(_, mapping)| mapping.replica_id.get() == replica_id)
            .map(|(key, _)| (u64::from(key.generation) << 32) | u64::from(key.id))
    }

    /// The team view's current expected bytes for one component (diagnostics only).
    pub fn expected_component_bytes(&self, team: u32, replica_id: u64, schema_id: u32) -> Option<&[u8]> {
        self.projectors.get(&team)?.hash_entities.get(&replica_id)?.2.get(&schema_id).map(Vec::as_slice)
    }

    /// Exact expected bytes of one component at a retained checkpoint (diagnostics only).
    pub fn expected_checkpoint_bytes(&self, team: u32, replica_tick: u64, replica_id: u64, schema_id: u32) -> Option<&[u8]> {
        self.projectors.get(&team)?.expected_snapshots.iter()
            .find(|(tick, _)| *tick == replica_tick)?.1.get(&(replica_id, schema_id)).map(Vec::as_slice)
    }

    /// Retained expected digests for `team` at checkpoint `replica_tick`.
    pub fn expected_component_digests(&self, team: u32, replica_tick: u64) -> Option<&[ComponentDigestRow]> {
        self.projectors.get(&team)?.expected_digests.iter()
            .find(|(tick, _)| *tick == replica_tick).map(|(_, rows)| rows.as_slice())
    }
}

/// Human-readable, bounded diff of expected vs observed component digests.
/// Names only replica ids, schema ids and digests of the team's own disclosed
/// view, never undisclosed state.
pub fn describe_component_divergence(expected: Option<&[ComponentDigestRow]>, observed: &[ComponentDigestRow], limit: usize) -> String {
    describe_component_divergence_with(expected, observed, limit, &|_, _| String::new())
}

/// As [`describe_component_divergence`]; `label(replica_id, schema_id)` appends
/// host-side context (canonical id, kind, current team-view bytes) per row.
pub fn describe_component_divergence_with(expected: Option<&[ComponentDigestRow]>, observed: &[ComponentDigestRow], limit: usize,
    label: &dyn Fn(u64, u32) -> String) -> String {
    let Some(expected) = expected else {
        return format!("expected digests unavailable (older than {EXPECTED_DIGEST_TICKS} checkpoints); observed {} rows", observed.len());
    };
    let index = |rows: &[ComponentDigestRow]| rows.iter().map(|row| ((row.replica_id, row.schema_id), row.digest)).collect::<BTreeMap<_, _>>();
    let (expected, observed) = (index(expected), index(observed));
    let mut lines = Vec::new();
    let mut total = 0usize;
    for key in expected.keys().chain(observed.keys()).collect::<BTreeSet<_>>() {
        let (left, right) = (expected.get(key), observed.get(key));
        if left == right { continue; }
        total += 1;
        if lines.len() < limit {
            lines.push(format!("entity={} schema=0x{:08x} expected={} observed={}{}", key.0, key.1,
                left.map_or("absent".into(), |v| format!("{v:016x}")), right.map_or("absent".into(), |v| format!("{v:016x}")), label(key.0, key.1)));
        }
    }
    format!("{total} divergent component(s); first {}: [{}]", lines.len(), lines.join("; "))
}

#[cfg(test)]
mod component_divergence_tests {
    use super::*;

    fn row(replica_id: u64, schema_id: u32, digest: u64) -> ComponentDigestRow { ComponentDigestRow { replica_id, schema_id, digest } }

    #[test]
    fn divergence_report_lists_changed_missing_and_extra_components_up_to_limit() {
        let expected = [row(1, 0x10, 1), row(1, 0x11, 2), row(2, 0x10, 3), row(3, 0x10, 4)];
        let observed = [row(1, 0x10, 1), row(1, 0x11, 9), row(3, 0x10, 4), row(4, 0x10, 5)];
        let report = describe_component_divergence(Some(&expected), &observed, 2);
        assert!(report.starts_with("3 divergent component(s); first 2:"), "{report}");
        assert!(report.contains("entity=1 schema=0x00000011 expected=0000000000000002 observed=0000000000000009"), "{report}");
        assert!(report.contains("entity=2 schema=0x00000010 expected=0000000000000003 observed=absent"), "{report}");
        assert!(!report.contains("entity=4"), "limit keeps the log bounded: {report}");
        assert!(describe_component_divergence(Some(&expected), &expected, 16).starts_with("0 divergent"));
        assert!(describe_component_divergence(None, &observed, 16).contains("unavailable"));
    }

    #[test]
    fn digest_rows_cover_every_component_and_change_with_bytes() {
        let mut entities = BTreeMap::new();
        entities.insert(7u64, (1u64, 0u32, BTreeMap::from([(0x20u32, vec![1u8, 2]), (0x21, vec![3])])));
        let first = component_digest_rows(&entities);
        assert_eq!(first.len(), 2);
        entities.get_mut(&7).unwrap().2.insert(0x21, vec![4]);
        let second = component_digest_rows(&entities);
        assert_eq!(first[0], second[0]);
        assert_ne!(first[1].digest, second[1].digest);
    }

    #[test]
    fn target_disclosure_rule_requires_a_disclosed_mapping_for_selective_teams() {
        let mut runtime = TeamProjectionRuntime::default();
        assert!(runtime.target_disclosed(1, 42), "non-selective teams keep targets");
        let canonical = CanonicalEntityKey { id: 7, generation: 1 };
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        projector.identity.disclose(canonical).unwrap();
        runtime.projectors.insert(1, projector);
        assert!(runtime.target_disclosed(1, (1u64 << 32) | 7));
        assert!(!runtime.target_disclosed(1, (1u64 << 32) | 8));
        runtime.projectors.get_mut(&1).unwrap().identity.remember(canonical).unwrap();
        assert!(!runtime.target_disclosed(1, (1u64 << 32) | 7), "a remembered silhouette is not targetable");
    }
}
