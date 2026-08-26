//! Safe per-team projection from committed facts/state into protocol-v2 frames.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use prost::Message;
use sha2::{Digest, Sha256};

use crate::game_proto::{
    transition, AuthorityRevision, DisclosureEpoch, ForgetEntity, HideEntity, PostStep, PreStep,
    ReplicaEntityId as ProtoReplicaEntityId, RevealEntity, SanitizedExternalEffect, Step,
    TeamHashCheckpoint, TeamPublicEvent, TeamTickFrame, Transition, ViewEpoch,
};
use crate::runtime::{
    CanonicalEntityKey, ClassifiedComponentRecord, DisclosureClass, FactAudience,
    MappingVisibility, ObservableFact, OrderedFact, RememberDisposition, TeamIdentityError,
    TeamIdentityState, VisibilityTransition,
};
use specs::{World, WorldExt};

pub const TEAM_FRAME_SCHEMA_VERSION: u32 = 1;
pub const CONTENT_SCHEMA_VERSION: u32 = 1;

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
}

impl From<TeamIdentityError> for ProjectionError {
    fn from(value: TeamIdentityError) -> Self { Self::Identity(value) }
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
        if !visited.insert(entity) { continue; }
        if graph.server_only.contains(&entity) { return Err(ProjectionError::ServerOnlyDependency(entity)); }
        if entity != root && !visible.contains(&entity) { return Err(ProjectionError::HiddenDependency(entity)); }
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
}

impl Default for TeamProjectorConfig {
    fn default() -> Self {
        Self {
            component_allowlist: BTreeSet::new(),
            size_buckets: vec![256, 512, 1024, 2048, 4096, 8192, 16384],
            mass_reveal_chunk_entities: 64,
            rebase_chunks_per_tick: 2,
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
}

#[derive(Default)]
pub struct TeamProjectionRuntime {
    projectors: BTreeMap<u32, TeamViewProjector>,
    pub latest_frames: BTreeMap<u32, PaddedTeamFrame>,
    pub dependency_graph: ProjectionDependencyGraph,
}

pub fn run_team_projection_after_wave_b(world: &mut World, server_tick: u64) -> Result<(), ProjectionError> {
    let visibility = world.read_resource::<crate::runtime::TeamVisibilityRuntime>().clone();
    let committed = world.read_resource::<crate::runtime::CommittedProjectionBatch>().clone();
    if !committed.barrier_reached { return Ok(()); }
    let mut runtime = world.write_resource::<TeamProjectionRuntime>();
    let graph = runtime.dependency_graph.clone();
    let mut frames = BTreeMap::new();
    for (team, state) in &visibility.teams {
        let transitions = visibility.last_transitions.get(team).cloned().unwrap_or_default();
        let projector = runtime.projectors.entry(*team)
            .or_insert_with(|| TeamViewProjector::new(*team, TeamProjectorConfig::default()));
        let frame = projector.build_frame(
            server_tick,
            committed.tick,
            &state.index.current,
            transitions,
            &committed.facts,
            &graph,
        )?;
        frames.insert(*team, frame);
    }
    runtime.latest_frames = frames;
    Ok(())
}

impl TeamViewProjector {
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
        }
    }

    pub fn enqueue_rebase_chunks(&mut self, chunks: impl IntoIterator<Item = Vec<u8>>) {
        self.pending_rebase_chunks.extend(chunks);
    }

    pub fn take_rate_limited_rebase_chunks(&mut self) -> Vec<Vec<u8>> {
        (0..self.config.rebase_chunks_per_tick.max(1))
            .filter_map(|_| self.pending_rebase_chunks.pop_front()).collect()
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
        self.pending_reveals.extend(transitions);
        let pre_step = self.build_pre_step(visible, graph)?;
        let step = self.build_step(visible, facts);
        let hash = expected_team_hash(self.team_id, replica_tick, visible, &pre_step, &step);
        let post_step = PostStep {
            component_repairs: Vec::new(),
            entity_replaces: Vec::new(),
            hash_checkpoint: Some(TeamHashCheckpoint {
                replica_tick,
                canonical_team_hash: hash.to_vec(),
                authority_revision: Some(AuthorityRevision { value: self.authority_revision }),
            }),
            rebase_notice: None,
        };
        let frame = TeamTickFrame {
            protocol_version: 2,
            frame_schema_version: TEAM_FRAME_SCHEMA_VERSION,
            content_schema_version: CONTENT_SCHEMA_VERSION,
            team_id: self.team_id,
            server_tick,
            replica_tick,
            team_sequence: self.sequence,
            view_epoch: Some(ViewEpoch { value: self.view_epoch }),
            authority_revision: Some(AuthorityRevision { value: self.authority_revision }),
            pre_step: Some(pre_step),
            step: Some(step),
            post_step: Some(post_step),
            padding: Vec::new(),
        };
        self.sequence = self.sequence.saturating_add(1);
        self.authority_revision = self.authority_revision.saturating_add(1);
        pad_frame(frame, &self.config.size_buckets)
    }

    fn build_pre_step(
        &mut self,
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
                VisibilityTransition::Reveal { canonical_id, effective_tick, baseline } => {
                    let key = unpack_canonical(canonical_id);
                    let mapping = self.identity.disclose(key)?;
                    let dependencies = disclosed_dependency_closure(canonical_id, visible, graph)?
                        .into_iter().filter_map(|dependency| {
                            self.identity.replica_for(unpack_canonical(dependency))
                                .map(|mapping| ProtoReplicaEntityId { value: mapping.replica_id.get() })
                        }).collect();
                    transitions.push(Transition { transition: Some(transition::Transition::Reveal(RevealEntity {
                        replica_entity_id: Some(ProtoReplicaEntityId { value: mapping.replica_id.get() }),
                        disclosure_epoch: Some(DisclosureEpoch { value: mapping.disclosure_epoch }),
                        effective_tick,
                        entity_kind: 0,
                        safe_baseline: baseline,
                        disclosed_dependencies: dependencies,
                        stable_sub_index: reveal_count as u32,
                    })) });
                    reveal_count += 1;
                }
                VisibilityTransition::Hide { canonical_id, effective_tick, disposition } => {
                    let key = unpack_canonical(canonical_id);
                    let mapping = self.identity.replica_for(key).ok_or(TeamIdentityError::UnknownCanonical)?;
                    let transition = match disposition {
                        RememberDisposition::Forget => {
                            self.identity.forget(key)?;
                            transition::Transition::Forget(ForgetEntity {
                                replica_entity_id: Some(ProtoReplicaEntityId { value: mapping.replica_id.get() }),
                                disclosure_epoch: Some(DisclosureEpoch { value: mapping.disclosure_epoch }),
                                effective_tick,
                                retire_reason: 1,
                                stable_sub_index: transitions.len() as u32,
                            })
                        }
                        RememberDisposition::LastKnown | RememberDisposition::Silhouette => {
                            self.identity.remember(key)?;
                            transition::Transition::Hide(HideEntity {
                                replica_entity_id: Some(ProtoReplicaEntityId { value: mapping.replica_id.get() }),
                                disclosure_epoch: Some(DisclosureEpoch { value: mapping.disclosure_epoch }),
                                effective_tick,
                                remember_policy: if disposition == RememberDisposition::LastKnown { 1 } else { 2 },
                                sanitized_remembered_presentation: Vec::new(),
                                stable_sub_index: transitions.len() as u32,
                            })
                        }
                    };
                    transitions.push(Transition { transition: Some(transition) });
                }
            }
        }
        self.pending_reveals = deferred;
        transitions.sort_by_key(transition_sort_key);
        Ok(PreStep { transitions })
    }

    fn build_step(&self, visible: &BTreeSet<u64>, facts: &[OrderedFact]) -> Step {
        let mut public_events = Vec::new();
        let mut external_effects = Vec::new();
        let mut ordered: Vec<_> = facts.iter().filter(|fact| audience_allows(&fact.audience, self.team_id)).collect();
        ordered.sort_by_key(|fact| (fact.key.fact_kind, fact_subject_replica(fact, &self.identity), fact.key.local_ordinal));
        for fact in ordered {
            project_fact(fact, visible, &self.identity, &mut public_events, &mut external_effects);
        }
        public_events.sort_by_key(|event| (event.event_kind, event.subject.as_ref().map_or(0, |id| id.value), event.stable_sub_index));
        external_effects.sort_by_key(|effect| (effect.effect_kind, effect.visible_target.as_ref().map_or(0, |id| id.value), effect.stable_sub_index));
        Step { accepted_inputs: Vec::new(), public_events, random_tapes: Vec::new(), external_effects }
    }
}

fn audience_allows(audience: &FactAudience, team: u32) -> bool {
    matches!(audience, FactAudience::AllPlayers)
        || matches!(audience, FactAudience::Team(value) if *value == team)
        || matches!(audience, FactAudience::VisibilityPolicy(_))
}

fn project_fact(
    ordered: &OrderedFact,
    visible: &BTreeSet<u64>,
    identity: &TeamIdentityState,
    public: &mut Vec<TeamPublicEvent>,
    external: &mut Vec<SanitizedExternalEffect>,
) {
    let (source, target, payload) = fact_entities_and_payload(&ordered.fact);
    let source_visible = source.is_some_and(|id| visible.contains(&id));
    let target_visible = target.is_some_and(|id| visible.contains(&id));
    let target_replica = target.and_then(|id| replica_proto(identity, id));
    let hidden_source_requires_sanitizing = !source_visible && target_visible && matches!(
        ordered.fact,
        ObservableFact::DirectCombat { .. } | ObservableFact::Buff { .. }
            | ObservableFact::Projectile { .. } | ObservableFact::AreaEffect { .. }
    );
    if hidden_source_requires_sanitizing {
        external.push(SanitizedExternalEffect {
            effect_kind: ordered.key.fact_kind as u32,
            visible_target: target_replica,
            sanitized_payload: payload,
            stable_sub_index: ordered.key.local_ordinal,
        });
    } else if source_visible || target_visible || source.is_none() {
        public.push(TeamPublicEvent {
            event_kind: ordered.key.fact_kind as u32,
            subject: source.and_then(|id| replica_proto(identity, id)).or(target_replica),
            sanitized_payload: payload,
            stable_sub_index: ordered.key.local_ordinal,
        });
    }
}

fn fact_entities_and_payload(fact: &ObservableFact) -> (Option<u64>, Option<u64>, Vec<u8>) {
    let mut payload = Vec::new();
    match fact {
        ObservableFact::Movement { source, x_mm, y_mm } => { payload.extend(x_mm.to_le_bytes()); payload.extend(y_mm.to_le_bytes()); (Some(*source), None, payload) }
        ObservableFact::Spawn { source, template_id, team } => { payload.extend(template_id.to_le_bytes()); payload.extend(team.to_le_bytes()); (*source, None, payload) }
        ObservableFact::Death { source, killer } => (Some(*source), *killer, payload),
        ObservableFact::Ownership { source, team } => { payload.extend(team.to_le_bytes()); (Some(*source), None, payload) }
        ObservableFact::DirectCombat { source, target, amount_milli } => { payload.extend(amount_milli.to_le_bytes()); (Some(*source), Some(*target), payload) }
        ObservableFact::Projectile { source, target, effect_id, active } => { payload.extend(effect_id.to_le_bytes()); payload.push(u8::from(*active)); (Some(*source), *target, payload) }
        ObservableFact::AreaEffect { source, x_mm, y_mm, radius_mm } => { payload.extend(x_mm.to_le_bytes()); payload.extend(y_mm.to_le_bytes()); payload.extend(radius_mm.to_le_bytes()); (Some(*source), None, payload) }
        ObservableFact::Buff { source, target, effect_id, active } => { payload.extend(effect_id.to_le_bytes()); payload.push(u8::from(*active)); (Some(*source), Some(*target), payload) }
        ObservableFact::Ability { source, ability_id, target } => { payload.extend(ability_id.to_le_bytes()); (Some(*source), *target, payload) }
        ObservableFact::Tower { source, action_id } => { payload.extend(action_id.to_le_bytes()); (Some(*source), None, payload) }
        ObservableFact::Item { source, item_id, target } => { payload.extend(item_id.to_le_bytes()); (Some(*source), *target, payload) }
        ObservableFact::Hud { team, metric_id, value } => { payload.extend(team.to_le_bytes()); payload.extend(metric_id.to_le_bytes()); payload.extend(value.to_le_bytes()); (None, None, payload) }
        ObservableFact::Terminal { result_code, winning_team } => { payload.extend(result_code.to_le_bytes()); if let Some(team) = winning_team { payload.extend(team.to_le_bytes()); } (None, None, payload) }
    }
}

fn fact_subject_replica(fact: &OrderedFact, identity: &TeamIdentityState) -> u64 {
    fact_entities_and_payload(&fact.fact).0.and_then(|id| replica_proto(identity, id)).map_or(0, |id| id.value)
}

fn replica_proto(identity: &TeamIdentityState, canonical: u64) -> Option<ProtoReplicaEntityId> {
    identity.replica_for(unpack_canonical(canonical)).filter(|mapping| mapping.visibility == MappingVisibility::Disclosed)
        .map(|mapping| ProtoReplicaEntityId { value: mapping.replica_id.get() })
}

fn unpack_canonical(value: u64) -> CanonicalEntityKey {
    CanonicalEntityKey { id: value as u32, generation: (value >> 32) as u32 }
}

fn transition_sort_key(value: &Transition) -> (u8, u64, u32) {
    match value.transition.as_ref() {
        Some(transition::Transition::Reveal(item)) => (0, item.replica_entity_id.as_ref().map_or(0, |id| id.value), item.stable_sub_index),
        Some(transition::Transition::Replace(item)) => (1, item.replica_entity_id.as_ref().map_or(0, |id| id.value), item.stable_sub_index),
        Some(transition::Transition::Hide(item)) => (2, item.replica_entity_id.as_ref().map_or(0, |id| id.value), item.stable_sub_index),
        Some(transition::Transition::Forget(item)) => (3, item.replica_entity_id.as_ref().map_or(0, |id| id.value), item.stable_sub_index),
        None => (u8::MAX, 0, 0),
    }
}

fn expected_team_hash(team: u32, tick: u64, visible: &BTreeSet<u64>, pre: &PreStep, step: &Step) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(team.to_le_bytes());
    hasher.update(tick.to_le_bytes());
    for id in visible { hasher.update(id.to_le_bytes()); }
    hasher.update(pre.encode_to_vec());
    hasher.update(step.encode_to_vec());
    hasher.finalize().into()
}

fn pad_frame(mut frame: TeamTickFrame, buckets: &[usize]) -> Result<PaddedTeamFrame, ProjectionError> {
    if buckets.is_empty() { return Err(ProjectionError::EmptyPaddingBuckets); }
    let canonical_bytes = frame.encode_to_vec();
    for bucket in buckets.iter().copied().filter(|size| *size >= canonical_bytes.len()) {
        let mut low = 0usize;
        let mut high = bucket - canonical_bytes.len();
        while low <= high {
            let padding_len = low + (high - low) / 2;
            frame.padding.resize(padding_len, 0);
            let wire_bytes = frame.encode_to_vec();
            if wire_bytes.len() == bucket {
                return Ok(PaddedTeamFrame { frame, canonical_bytes, wire_bytes, padding_len });
            }
            if wire_bytes.len() < bucket { low = padding_len.saturating_add(1); }
            else if padding_len == 0 { break; }
            else { high = padding_len - 1; }
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
        visible_target: Some(ProtoReplicaEntityId { value: visible_target }),
        sanitized_payload: amount_milli.to_le_bytes().to_vec(),
        stable_sub_index,
    }
}
