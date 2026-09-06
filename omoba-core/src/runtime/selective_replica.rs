use std::collections::{BTreeMap, BTreeSet};

use prost::Message;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::game_proto::{
    self, transition, BoundedRandomTape, ComponentRepair, EntityReplace, FilteredTeamSnapshot,
    PostStep, PreStep, SanitizedExternalEffect, Step, TeamAcceptedInput, TeamPublicEvent,
    TeamTickFrame, TeamViewRebase,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplicaEntityState {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub entity_kind: u32,
    pub authority_revision: u64,
    pub components: BTreeMap<u32, Vec<u8>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisclosedReplicaWorld {
    pub tick: u64,
    pub entities: BTreeMap<u64, ReplicaEntityState>,
    pub resources: BTreeMap<u32, Vec<u8>>,
}

/// Test-evidence description of one disclosed component. It contains only
/// per-team disclosed state; hidden canonical ids and component bytes are not
/// exposed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DisclosedComponentDigest {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub entity_kind: u32,
    pub component_schema_id: u32,
    pub byte_len: usize,
    pub sha256: String,
    pub disclosed_value_hex: String,
}

fn evidence_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

pub fn disclosed_component_digests(
    entities: &BTreeMap<u64, ReplicaEntityState>,
) -> Vec<DisclosedComponentDigest> {
    let mut rows = Vec::new();
    for state in entities.values() {
        for (schema_id, bytes) in &state.components {
            rows.push(DisclosedComponentDigest {
                replica_id: state.replica_id,
                disclosure_epoch: state.disclosure_epoch,
                entity_kind: state.entity_kind,
                component_schema_id: *schema_id,
                byte_len: bytes.len(),
                sha256: evidence_hex(&Sha256::digest(bytes)),
                disclosed_value_hex: evidence_hex(bytes),
            });
        }
    }
    rows
}

#[derive(Clone, Debug, Default)]
pub struct StepInjections {
    pub accepted_inputs: Vec<TeamAcceptedInput>,
    pub public_events: Vec<TeamPublicEvent>,
    pub random_tapes: Vec<BoundedRandomTape>,
    pub external_effects: Vec<SanitizedExternalEffect>,
}

pub trait DisclosedWorldStepper {
    fn fixed_step(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        injections: &StepInjections,
        component_allowlist: &BTreeSet<u32>,
        resource_allowlist: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError>;
}

#[derive(Default)]
pub struct NoopDisclosedWorldStepper;

impl DisclosedWorldStepper for NoopDisclosedWorldStepper {
    fn fixed_step(
        &mut self,
        _world: &mut DisclosedReplicaWorld,
        _injections: &StepInjections,
        _component_allowlist: &BTreeSet<u32>,
        _resource_allowlist: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError> {
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplicaStallState {
    Running,
    MissingSequence { expected: u64, received: u64 },
    MissingReplicaTick { expected: u64, received: u64 },
    AwaitingRebase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplicaRuntimeError {
    Decode,
    WrongProtocol,
    WrongTeam,
    SequenceGap,
    ReplicaTickGap,
    StaleViewEpoch,
    FutureViewEpoch,
    StaleDisclosureEpoch,
    StaleAuthorityRevision,
    ConflictingEqualRevision,
    UnknownEntity,
    DuplicateEntity,
    ComponentNotAllowlisted,
    ResourceNotAllowlisted,
    MalformedBaseline,
    MalformedTransition,
    MalformedRandomTape,
    UnverifiedRebase,
    GameplayStep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaApplyPhase {
    PreStep,
    Step,
    PostStep,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaApplyOperation {
    RevealDependency,
    Hide,
    Forget,
    AcceptedInputActor,
    AcceptedInputTarget,
    ExternalEffectTarget,
    RandomTapeEntity,
    ComponentRepair,
    EntityReplace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplicaApplyFault {
    pub phase: ReplicaApplyPhase,
    pub operation: ReplicaApplyOperation,
    pub replica_id: u64,
    pub disclosure_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameApplyResult {
    Applied {
        replica_tick: u64,
        team_sequence: u64,
        pre_repair_observed_hash: [u8; 32],
        post_repair_hash: [u8; 32],
        /// Compatibility alias. Always equals `pre_repair_observed_hash`.
        team_hash: [u8; 32],
    },
    Duplicate,
    Stalled(ReplicaStallState),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderMemoryDirective {
    Hide {
        replica_id: u64,
        disclosure_epoch: u64,
        remember_policy: u32,
        sanitized_presentation: Vec<u8>,
    },
    Forget {
        replica_id: u64,
        disclosure_epoch: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilteredRenderEntity {
    pub replica_id: u64,
    pub disclosure_epoch: u64,
    pub entity_kind: u32,
    pub components: BTreeMap<u32, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FilteredRenderSnapshot {
    pub team_id: u32,
    pub replica_tick: u64,
    pub entities: Vec<FilteredRenderEntity>,
    pub public_events: Vec<TeamPublicEvent>,
    pub external_effects: Vec<SanitizedExternalEffect>,
    pub memory_directives: Vec<RenderMemoryDirective>,
}

pub struct SelectiveReplicaRuntime {
    team_id: u32,
    expected_team_sequence: u64,
    expected_replica_tick: u64,
    view_epoch: u64,
    authority_revision: u64,
    global_seed: u64,
    stall: ReplicaStallState,
    world: DisclosedReplicaWorld,
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
    last_injections: StepInjections,
    memory_directives: Vec<RenderMemoryDirective>,
    remembered_presentations: BTreeMap<(u64, u64), Vec<u8>>,
    applied_transition_revisions: BTreeMap<(u8, u64, u64), u64>,
    retired_replica_filter: Box<[u64]>,
    last_apply_fault: Option<ReplicaApplyFault>,
}

impl SelectiveReplicaRuntime {
    pub fn new(
        team_id: u32,
        replica_start_tick: u64,
        next_team_sequence: u64,
        view_epoch: u64,
        component_allowlist: BTreeSet<u32>,
        resource_allowlist: BTreeSet<u32>,
    ) -> Self {
        Self {
            team_id,
            expected_team_sequence: next_team_sequence,
            expected_replica_tick: replica_start_tick,
            view_epoch,
            authority_revision: 0,
            global_seed: 0,
            stall: ReplicaStallState::Running,
            world: DisclosedReplicaWorld {
                tick: replica_start_tick,
                ..DisclosedReplicaWorld::default()
            },
            component_allowlist,
            resource_allowlist,
            last_injections: StepInjections::default(),
            memory_directives: Vec::new(),
            remembered_presentations: BTreeMap::new(),
            applied_transition_revisions: BTreeMap::new(),
            retired_replica_filter: vec![0; 131_072].into_boxed_slice(),
            last_apply_fault: None,
        }
    }

    pub fn stall_state(&self) -> &ReplicaStallState {
        &self.stall
    }

    pub fn bootstrap_from_team_game_start(
        start: &game_proto::TeamGameStart,
        component_allowlist: BTreeSet<u32>,
        resource_allowlist: BTreeSet<u32>,
    ) -> Result<Self, ReplicaRuntimeError> {
        if start.protocol_version != 2 {
            return Err(ReplicaRuntimeError::WrongProtocol);
        }
        let snapshot = start
            .filtered_snapshot
            .as_ref()
            .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
        if snapshot.team_id != start.team_id
            || snapshot.filtered_snapshot_hash
                != Sha256::digest(&snapshot.disclosed_world).as_slice()
        {
            return Err(ReplicaRuntimeError::UnverifiedRebase);
        }
        let entities = decode_disclosed_world(&snapshot.disclosed_world, &component_allowlist)?;
        let mut runtime = Self::new(
            start.team_id,
            start.replica_start_tick,
            start.next_team_sequence,
            start.view_epoch.as_ref().map_or(0, |epoch| epoch.value),
            component_allowlist,
            resource_allowlist,
        );
        runtime.world = DisclosedReplicaWorld {
            tick: snapshot.authoritative_tick,
            entities,
            resources: BTreeMap::new(),
        };
        runtime.expected_replica_tick = start.replica_start_tick;
        runtime.global_seed = start.global_seed;
        Ok(runtime)
    }

    pub fn global_seed(&self) -> u64 {
        self.global_seed
    }

    pub fn view_epoch(&self) -> u64 {
        self.view_epoch
    }

    pub fn next_replica_tick(&self) -> u64 {
        self.expected_replica_tick
    }

    pub fn world(&self) -> &DisclosedReplicaWorld {
        &self.world
    }

    pub fn last_apply_fault(&self) -> Option<&ReplicaApplyFault> {
        self.last_apply_fault.as_ref()
    }

    pub fn disclosed_component_digests(&self) -> Vec<DisclosedComponentDigest> {
        disclosed_component_digests(&self.world.entities)
    }

    /// Test-mode corruption hook used by the three-process recovery gate.
    /// Mutating the disclosed source of truth (rather than only the Specs
    /// mirror) guarantees the fault survives the next membership sync and is
    /// observable in the pre-repair hash.
    pub fn inject_test_only_disclosed_position_fault(&mut self) -> bool {
        for entity in self.world.entities.values_mut() {
            let Some(bytes) = entity
                .components
                .get_mut(&crate::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
            else {
                continue;
            };
            let Some(mut render) = crate::runtime::decode_demo_render_state(bytes) else {
                continue;
            };
            render.x_raw = render.x_raw.saturating_add(1 << 10);
            *bytes = crate::runtime::encode_demo_render_state(render);
            return true;
        }
        false
    }

    pub fn apply_encoded_frame(
        &mut self,
        bytes: &[u8],
        stepper: &mut impl DisclosedWorldStepper,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        let frame = TeamTickFrame::decode(bytes).map_err(|_| ReplicaRuntimeError::Decode)?;
        self.apply_frame(frame, stepper)
    }

    pub fn apply_frame(
        &mut self,
        frame: TeamTickFrame,
        stepper: &mut impl DisclosedWorldStepper,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        self.last_apply_fault = None;
        self.memory_directives.clear();
        self.applied_transition_revisions.clear();
        if frame.protocol_version != 2 {
            return Err(ReplicaRuntimeError::WrongProtocol);
        }
        if frame.team_id != self.team_id {
            return Err(ReplicaRuntimeError::WrongTeam);
        }
        if frame.team_sequence < self.expected_team_sequence {
            return Ok(FrameApplyResult::Duplicate);
        }
        if frame.team_sequence > self.expected_team_sequence {
            self.stall = ReplicaStallState::MissingSequence {
                expected: self.expected_team_sequence,
                received: frame.team_sequence,
            };
            return Ok(FrameApplyResult::Stalled(self.stall.clone()));
        }
        if frame.replica_tick != self.expected_replica_tick {
            self.stall = ReplicaStallState::MissingReplicaTick {
                expected: self.expected_replica_tick,
                received: frame.replica_tick,
            };
            return Ok(FrameApplyResult::Stalled(self.stall.clone()));
        }
        let frame_view_epoch = frame.view_epoch.as_ref().map_or(0, |epoch| epoch.value);
        if frame_view_epoch < self.view_epoch {
            return Err(ReplicaRuntimeError::StaleViewEpoch);
        }
        if frame_view_epoch > self.view_epoch {
            self.stall = ReplicaStallState::AwaitingRebase;
            return Err(ReplicaRuntimeError::FutureViewEpoch);
        }

        let frame_revision = frame
            .authority_revision
            .as_ref()
            .map_or(0, |revision| revision.value);
        self.preflight_entity_references(&frame)?;
        self.apply_pre_step(frame.pre_step.as_ref(), frame_revision)?;
        self.last_injections = self.inject_step(frame.step.as_ref())?;
        // Reveal/Replace baselines are captured from the authoritative world
        // after this replica tick has already executed. Keep them out of the
        // local step for this frame, then merge them back as the tick result.
        // Otherwise a newly visible moving entity advances twice on its reveal
        // tick and immediately breaks player-view lockstep.
        let baseline_ids = post_tick_baseline_ids(frame.pre_step.as_ref());
        let staged_baselines: Vec<_> = baseline_ids
            .iter()
            .filter_map(|id| self.world.entities.remove_entry(id))
            .collect();
        self.last_injections.accepted_inputs.retain(|input| {
            let actor = input.actor.as_ref().map_or(0, |id| id.value);
            let target = input.target.as_ref().map_or(0, |id| id.value);
            !baseline_ids.contains(&actor) && !baseline_ids.contains(&target)
        });
        self.last_injections.external_effects.retain(|effect| {
            let target = effect.visible_target.as_ref().map_or(0, |id| id.value);
            !baseline_ids.contains(&target)
        });
        self.last_injections.public_events.retain(|event| {
            let subject = event.subject.as_ref().map_or(0, |id| id.value);
            !baseline_ids.contains(&subject)
        });
        self.last_injections.random_tapes.retain(|tape| {
            let entity = tape.replica_entity_id.as_ref().map_or(0, |id| id.value);
            !baseline_ids.contains(&entity)
        });
        stepper.fixed_step(
            &mut self.world,
            &self.last_injections,
            &self.component_allowlist,
            &self.resource_allowlist,
        )?;
        self.world.entities.extend(staged_baselines);
        self.validate_world_allowlist()?;
        self.world.tick = self.expected_replica_tick + 1;
        let pre_repair_observed_hash = self.canonical_team_hash();
        self.apply_post_step(frame.post_step.as_ref(), frame_revision)?;
        let post_repair_hash = self.canonical_team_hash();
        self.expected_replica_tick += 1;
        self.expected_team_sequence += 1;
        self.stall = ReplicaStallState::Running;
        Ok(FrameApplyResult::Applied {
            replica_tick: frame.replica_tick,
            team_sequence: frame.team_sequence,
            pre_repair_observed_hash,
            post_repair_hash,
            team_hash: pre_repair_observed_hash,
        })
    }

    /// Validate entity lifecycle references before mutating either the
    /// disclosed world or the Specs mirror.  This keeps an invalid server frame
    /// from becoming a half-applied client state while authoritative recovery
    /// is requested.
    fn preflight_entity_references(
        &mut self,
        frame: &TeamTickFrame,
    ) -> Result<(), ReplicaRuntimeError> {
        let mut entities: BTreeSet<u64> = self.world.entities.keys().copied().collect();
        if let Some(pre_step) = &frame.pre_step {
            for transition in &pre_step.transitions {
                match transition.transition.as_ref() {
                    Some(transition::Transition::Reveal(reveal)) => {
                        if let Some(dependency) = reveal
                            .disclosed_dependencies
                            .iter()
                            .find(|dependency| !entities.contains(&dependency.value))
                        {
                            self.last_apply_fault = Some(ReplicaApplyFault {
                                phase: ReplicaApplyPhase::PreStep,
                                operation: ReplicaApplyOperation::RevealDependency,
                                replica_id: dependency.value,
                                disclosure_epoch: 0,
                            });
                            return Err(ReplicaRuntimeError::UnknownEntity);
                        }
                        entities.insert(reveal.replica_entity_id.as_ref().map_or(0, |id| id.value));
                    }
                    Some(transition::Transition::Replace(replace)) => {
                        entities
                            .insert(replace.replica_entity_id.as_ref().map_or(0, |id| id.value));
                    }
                    Some(transition::Transition::Hide(hide)) => {
                        let replica_id = hide.replica_entity_id.as_ref().map_or(0, |id| id.value);
                        if !entities.remove(&replica_id) {
                            self.last_apply_fault = Some(ReplicaApplyFault {
                                phase: ReplicaApplyPhase::PreStep,
                                operation: ReplicaApplyOperation::Hide,
                                replica_id,
                                disclosure_epoch: epoch_value(&hide.disclosure_epoch),
                            });
                            return Err(ReplicaRuntimeError::UnknownEntity);
                        }
                    }
                    Some(transition::Transition::Forget(forget)) => {
                        let replica_id = forget.replica_entity_id.as_ref().map_or(0, |id| id.value);
                        if !entities.remove(&replica_id) {
                            self.last_apply_fault = Some(ReplicaApplyFault {
                                phase: ReplicaApplyPhase::PreStep,
                                operation: ReplicaApplyOperation::Forget,
                                replica_id,
                                disclosure_epoch: epoch_value(&forget.disclosure_epoch),
                            });
                            return Err(ReplicaRuntimeError::UnknownEntity);
                        }
                    }
                    None => {}
                }
            }
        }
        if let Some(step) = &frame.step {
            for input in &step.accepted_inputs {
                let actor = input.actor.as_ref().map_or(0, |id| id.value);
                if actor == 0 || !entities.contains(&actor) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::Step,
                        operation: ReplicaApplyOperation::AcceptedInputActor,
                        replica_id: actor,
                        disclosure_epoch: 0,
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
                if let Some(target) = input.target.as_ref() {
                    if target.value != 0 && !entities.contains(&target.value) {
                        self.last_apply_fault = Some(ReplicaApplyFault {
                            phase: ReplicaApplyPhase::Step,
                            operation: ReplicaApplyOperation::AcceptedInputTarget,
                            replica_id: target.value,
                            disclosure_epoch: 0,
                        });
                        return Err(ReplicaRuntimeError::UnknownEntity);
                    }
                }
            }
            for effect in &step.external_effects {
                let target = effect.visible_target.as_ref().map_or(0, |id| id.value);
                if target == 0 || !entities.contains(&target) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::Step,
                        operation: ReplicaApplyOperation::ExternalEffectTarget,
                        replica_id: target,
                        disclosure_epoch: 0,
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
            }
            for tape in &step.random_tapes {
                let replica_id = tape.replica_entity_id.as_ref().map_or(0, |id| id.value);
                if !entities.contains(&replica_id) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::Step,
                        operation: ReplicaApplyOperation::RandomTapeEntity,
                        replica_id,
                        disclosure_epoch: epoch_value(&tape.disclosure_epoch),
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
            }
        }
        if let Some(post_step) = &frame.post_step {
            for repair in &post_step.component_repairs {
                let replica_id = repair.replica_entity_id.as_ref().map_or(0, |id| id.value);
                if !entities.contains(&replica_id) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::PostStep,
                        operation: ReplicaApplyOperation::ComponentRepair,
                        replica_id,
                        disclosure_epoch: epoch_value(&repair.disclosure_epoch),
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
            }
            for replace in &post_step.entity_replaces {
                let replica_id = replace.replica_entity_id.as_ref().map_or(0, |id| id.value);
                if !entities.contains(&replica_id) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::PostStep,
                        operation: ReplicaApplyOperation::EntityReplace,
                        replica_id,
                        disclosure_epoch: epoch_value(&replace.disclosure_epoch),
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
            }
        }
        Ok(())
    }

    fn apply_pre_step(
        &mut self,
        pre_step: Option<&PreStep>,
        frame_revision: u64,
    ) -> Result<(), ReplicaRuntimeError> {
        let Some(pre_step) = pre_step else {
            return Ok(());
        };
        for transition in &pre_step.transitions {
            match transition.transition.as_ref() {
                Some(transition::Transition::Reveal(reveal)) => {
                    if reveal.effective_tick != self.expected_replica_tick {
                        return Err(ReplicaRuntimeError::MalformedTransition);
                    }
                    let replica_id = reveal.replica_entity_id.as_ref().map_or(0, |id| id.value);
                    let disclosure_epoch = epoch_value(&reveal.disclosure_epoch);
                    if self.retired_filter_contains(replica_id) {
                        return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
                    }
                    if reveal
                        .disclosed_dependencies
                        .iter()
                        .any(|dependency| !self.world.entities.contains_key(&dependency.value))
                    {
                        if let Some(dependency) = reveal
                            .disclosed_dependencies
                            .iter()
                            .find(|dependency| !self.world.entities.contains_key(&dependency.value))
                        {
                            self.last_apply_fault = Some(ReplicaApplyFault {
                                phase: ReplicaApplyPhase::PreStep,
                                operation: ReplicaApplyOperation::RevealDependency,
                                replica_id: dependency.value,
                                disclosure_epoch: 0,
                            });
                        }
                        return Err(ReplicaRuntimeError::UnknownEntity);
                    }
                    if self.transition_already_applied(
                        0,
                        replica_id,
                        disclosure_epoch,
                        frame_revision,
                    ) {
                        continue;
                    }
                    if let Some(existing) = self.world.entities.get(&replica_id) {
                        if existing.disclosure_epoch == epoch_value(&reveal.disclosure_epoch)
                            && existing.authority_revision >= frame_revision
                        {
                            continue;
                        }
                        return Err(ReplicaRuntimeError::DuplicateEntity);
                    }
                    let components =
                        decode_components(&reveal.safe_baseline, &self.component_allowlist)?;
                    self.world.entities.insert(
                        replica_id,
                        ReplicaEntityState {
                            replica_id,
                            disclosure_epoch,
                            entity_kind: reveal.entity_kind,
                            authority_revision: frame_revision,
                            components,
                        },
                    );
                    self.remembered_presentations
                        .retain(|(remembered_id, _), _| *remembered_id != replica_id);
                    self.record_transition(0, replica_id, disclosure_epoch, frame_revision);
                }
                Some(transition::Transition::Replace(replace)) => {
                    if replace.effective_tick != self.expected_replica_tick {
                        return Err(ReplicaRuntimeError::MalformedTransition);
                    }
                    let replica_id = replace.replica_entity_id.as_ref().map_or(0, |id| id.value);
                    let revision = revision_value(&replace.authority_revision);
                    let disclosure_epoch = epoch_value(&replace.disclosure_epoch);
                    if self.transition_already_applied(1, replica_id, disclosure_epoch, revision) {
                        continue;
                    }
                    let entity_kind = self
                        .world
                        .entities
                        .get(&replica_id)
                        .map_or(0, |entity| entity.entity_kind);
                    let components =
                        decode_components(&replace.safe_baseline, &self.component_allowlist)?;
                    self.server_wins_replace(
                        replica_id,
                        disclosure_epoch,
                        entity_kind,
                        revision,
                        components,
                    )?;
                    self.record_transition(1, replica_id, disclosure_epoch, revision);
                }
                Some(transition::Transition::Hide(hide)) => {
                    if hide.effective_tick != self.expected_replica_tick {
                        return Err(ReplicaRuntimeError::MalformedTransition);
                    }
                    let replica_id = hide.replica_entity_id.as_ref().map_or(0, |id| id.value);
                    let disclosure_epoch = epoch_value(&hide.disclosure_epoch);
                    if self.transition_already_applied(
                        2,
                        replica_id,
                        disclosure_epoch,
                        frame_revision,
                    ) {
                        continue;
                    }
                    if let Err(error) = self.remove_with_epoch(replica_id, disclosure_epoch) {
                        self.last_apply_fault = Some(ReplicaApplyFault {
                            phase: ReplicaApplyPhase::PreStep,
                            operation: ReplicaApplyOperation::Hide,
                            replica_id,
                            disclosure_epoch,
                        });
                        return Err(error);
                    }
                    self.memory_directives.push(RenderMemoryDirective::Hide {
                        replica_id,
                        disclosure_epoch,
                        remember_policy: hide.remember_policy,
                        sanitized_presentation: hide.sanitized_remembered_presentation.clone(),
                    });
                    self.remembered_presentations.insert(
                        (replica_id, disclosure_epoch),
                        hide.sanitized_remembered_presentation.clone(),
                    );
                    self.record_transition(2, replica_id, disclosure_epoch, frame_revision);
                }
                Some(transition::Transition::Forget(forget)) => {
                    if forget.effective_tick != self.expected_replica_tick {
                        return Err(ReplicaRuntimeError::MalformedTransition);
                    }
                    let replica_id = forget.replica_entity_id.as_ref().map_or(0, |id| id.value);
                    let disclosure_epoch = epoch_value(&forget.disclosure_epoch);
                    if self.transition_already_applied(
                        3,
                        replica_id,
                        disclosure_epoch,
                        frame_revision,
                    ) {
                        continue;
                    }
                    if let Err(error) = self.remove_with_epoch(replica_id, disclosure_epoch) {
                        self.last_apply_fault = Some(ReplicaApplyFault {
                            phase: ReplicaApplyPhase::PreStep,
                            operation: ReplicaApplyOperation::Forget,
                            replica_id,
                            disclosure_epoch,
                        });
                        return Err(error);
                    }
                    self.retired_filter_insert(replica_id);
                    self.remembered_presentations
                        .remove(&(replica_id, disclosure_epoch));
                    self.memory_directives.push(RenderMemoryDirective::Forget {
                        replica_id,
                        disclosure_epoch,
                    });
                    self.record_transition(3, replica_id, disclosure_epoch, frame_revision);
                }
                None => {}
            }
        }
        Ok(())
    }

    fn remove_with_epoch(
        &mut self,
        replica_id: u64,
        disclosure_epoch: u64,
    ) -> Result<(), ReplicaRuntimeError> {
        let entity = self
            .world
            .entities
            .get(&replica_id)
            .ok_or(ReplicaRuntimeError::UnknownEntity)?;
        if entity.disclosure_epoch != disclosure_epoch {
            return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
        }
        self.world.entities.remove(&replica_id);
        Ok(())
    }

    fn retired_filter_indices(&self, replica_id: u64) -> [usize; 4] {
        let bits = self.retired_replica_filter.len() * 64;
        let mix = |mut value: u64| {
            value ^= value >> 30;
            value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
            value ^= value >> 27;
            value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
            value ^ (value >> 31)
        };
        [
            mix(replica_id) as usize % bits,
            mix(replica_id ^ 0x9e37_79b9_7f4a_7c15) as usize % bits,
            mix(replica_id ^ 0xd1b5_4a32_d192_ed03) as usize % bits,
            mix(replica_id ^ 0x94d0_49bb_1331_11eb) as usize % bits,
        ]
    }

    fn retired_filter_contains(&self, replica_id: u64) -> bool {
        self.retired_filter_indices(replica_id)
            .into_iter()
            .all(|bit| self.retired_replica_filter[bit / 64] & (1u64 << (bit % 64)) != 0)
    }

    fn retired_filter_insert(&mut self, replica_id: u64) {
        for bit in self.retired_filter_indices(replica_id) {
            self.retired_replica_filter[bit / 64] |= 1u64 << (bit % 64);
        }
    }

    fn inject_step(&mut self, step: Option<&Step>) -> Result<StepInjections, ReplicaRuntimeError> {
        let Some(step) = step else {
            return Ok(StepInjections::default());
        };
        for input in &step.accepted_inputs {
            let actor = input.actor.as_ref().map_or(0, |id| id.value);
            if actor == 0 || !self.world.entities.contains_key(&actor) {
                self.last_apply_fault = Some(ReplicaApplyFault {
                    phase: ReplicaApplyPhase::Step,
                    operation: ReplicaApplyOperation::AcceptedInputActor,
                    replica_id: actor,
                    disclosure_epoch: 0,
                });
                return Err(ReplicaRuntimeError::UnknownEntity);
            }
            if let Some(target) = input.target.as_ref() {
                if target.value != 0 && !self.world.entities.contains_key(&target.value) {
                    self.last_apply_fault = Some(ReplicaApplyFault {
                        phase: ReplicaApplyPhase::Step,
                        operation: ReplicaApplyOperation::AcceptedInputTarget,
                        replica_id: target.value,
                        disclosure_epoch: 0,
                    });
                    return Err(ReplicaRuntimeError::UnknownEntity);
                }
            }
        }
        for effect in &step.external_effects {
            let target = effect.visible_target.as_ref().map_or(0, |id| id.value);
            if target == 0 || !self.world.entities.contains_key(&target) {
                self.last_apply_fault = Some(ReplicaApplyFault {
                    phase: ReplicaApplyPhase::Step,
                    operation: ReplicaApplyOperation::ExternalEffectTarget,
                    replica_id: target,
                    disclosure_epoch: 0,
                });
                return Err(ReplicaRuntimeError::UnknownEntity);
            }
        }
        for tape in &step.random_tapes {
            let tape_end = tape
                .first_tick
                .checked_add(u64::from(tape.tick_count))
                .ok_or(ReplicaRuntimeError::StaleDisclosureEpoch)?;
            let replica_id = tape.replica_entity_id.as_ref().map_or(0, |id| id.value);
            let Some(entity) = self.world.entities.get(&replica_id) else {
                self.last_apply_fault = Some(ReplicaApplyFault {
                    phase: ReplicaApplyPhase::Step,
                    operation: ReplicaApplyOperation::RandomTapeEntity,
                    replica_id,
                    disclosure_epoch: epoch_value(&tape.disclosure_epoch),
                });
                return Err(ReplicaRuntimeError::UnknownEntity);
            };
            if tape.tick_count == 0
                || tape.values.len() < tape.tick_count as usize
                || self.expected_replica_tick < tape.first_tick
                || self.expected_replica_tick >= tape_end
                || entity.disclosure_epoch != epoch_value(&tape.disclosure_epoch)
            {
                return Err(ReplicaRuntimeError::MalformedRandomTape);
            }
        }
        Ok(StepInjections {
            accepted_inputs: step.accepted_inputs.clone(),
            public_events: step.public_events.clone(),
            random_tapes: step.random_tapes.clone(),
            external_effects: step.external_effects.clone(),
        })
    }

    fn validate_world_allowlist(&self) -> Result<(), ReplicaRuntimeError> {
        if self.world.entities.values().any(|entity| {
            entity
                .components
                .keys()
                .any(|schema_id| !self.component_allowlist.contains(schema_id))
        }) {
            return Err(ReplicaRuntimeError::ComponentNotAllowlisted);
        }
        if self
            .world
            .resources
            .keys()
            .any(|schema_id| !self.resource_allowlist.contains(schema_id))
        {
            return Err(ReplicaRuntimeError::ResourceNotAllowlisted);
        }
        Ok(())
    }

    fn transition_already_applied(
        &self,
        kind: u8,
        replica_id: u64,
        disclosure_epoch: u64,
        revision: u64,
    ) -> bool {
        self.applied_transition_revisions
            .get(&(kind, replica_id, disclosure_epoch))
            .is_some_and(|applied| *applied >= revision)
    }

    fn record_transition(
        &mut self,
        kind: u8,
        replica_id: u64,
        disclosure_epoch: u64,
        revision: u64,
    ) {
        self.applied_transition_revisions
            .insert((kind, replica_id, disclosure_epoch), revision);
    }

    fn apply_post_step(
        &mut self,
        post_step: Option<&PostStep>,
        frame_revision: u64,
    ) -> Result<(), ReplicaRuntimeError> {
        if frame_revision < self.authority_revision {
            return Err(ReplicaRuntimeError::StaleAuthorityRevision);
        }
        let Some(post_step) = post_step else {
            self.authority_revision = self.authority_revision.max(frame_revision);
            return Ok(());
        };
        for repair in &post_step.component_repairs {
            if repair.effective_tick != self.expected_replica_tick {
                return Err(ReplicaRuntimeError::MalformedTransition);
            }
            self.apply_component_repair(repair)?;
        }
        for replace in &post_step.entity_replaces {
            if replace.effective_tick != self.expected_replica_tick {
                return Err(ReplicaRuntimeError::MalformedTransition);
            }
            self.apply_entity_replace(replace)?;
        }
        self.authority_revision = self.authority_revision.max(frame_revision);
        Ok(())
    }

    fn apply_component_repair(
        &mut self,
        repair: &ComponentRepair,
    ) -> Result<(), ReplicaRuntimeError> {
        if !self
            .component_allowlist
            .contains(&repair.component_schema_id)
        {
            return Err(ReplicaRuntimeError::ComponentNotAllowlisted);
        }
        let replica_id = repair.replica_entity_id.as_ref().map_or(0, |id| id.value);
        let revision = revision_value(&repair.authority_revision);
        let Some(entity) = self.world.entities.get_mut(&replica_id) else {
            self.last_apply_fault = Some(ReplicaApplyFault {
                phase: ReplicaApplyPhase::PostStep,
                operation: ReplicaApplyOperation::ComponentRepair,
                replica_id,
                disclosure_epoch: epoch_value(&repair.disclosure_epoch),
            });
            return Err(ReplicaRuntimeError::UnknownEntity);
        };
        if entity.disclosure_epoch != epoch_value(&repair.disclosure_epoch) {
            return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
        }
        if revision < entity.authority_revision {
            return Err(ReplicaRuntimeError::StaleAuthorityRevision);
        }
        if revision == entity.authority_revision {
            if entity.components.get(&repair.component_schema_id)
                == Some(&repair.replacement_fields)
            {
                return Ok(());
            }
            // One authoritative post-step may repair several schemas on the
            // same entity under one revision. Frame ordering and disclosure
            // epoch validation already prevent stale cross-frame rewrites.
            entity.components.insert(
                repair.component_schema_id,
                repair.replacement_fields.clone(),
            );
            return Ok(());
        }
        entity.components.insert(
            repair.component_schema_id,
            repair.replacement_fields.clone(),
        );
        entity.authority_revision = revision;
        Ok(())
    }

    fn apply_entity_replace(&mut self, replace: &EntityReplace) -> Result<(), ReplicaRuntimeError> {
        let replica_id = replace.replica_entity_id.as_ref().map_or(0, |id| id.value);
        let Some(current) = self.world.entities.get(&replica_id) else {
            self.last_apply_fault = Some(ReplicaApplyFault {
                phase: ReplicaApplyPhase::PostStep,
                operation: ReplicaApplyOperation::EntityReplace,
                replica_id,
                disclosure_epoch: epoch_value(&replace.disclosure_epoch),
            });
            return Err(ReplicaRuntimeError::UnknownEntity);
        };
        let current_kind = current.entity_kind;
        let components = decode_components(&replace.safe_baseline, &self.component_allowlist)?;
        self.server_wins_replace(
            replica_id,
            epoch_value(&replace.disclosure_epoch),
            current_kind,
            revision_value(&replace.authority_revision),
            components,
        )
    }

    fn server_wins_replace(
        &mut self,
        replica_id: u64,
        disclosure_epoch: u64,
        entity_kind: u32,
        revision: u64,
        components: BTreeMap<u32, Vec<u8>>,
    ) -> Result<(), ReplicaRuntimeError> {
        if let Some(current) = self.world.entities.get(&replica_id) {
            if disclosure_epoch < current.disclosure_epoch {
                return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
            }
            if revision < current.authority_revision {
                return Err(ReplicaRuntimeError::StaleAuthorityRevision);
            }
            if revision == current.authority_revision && current.components != components {
                return Err(ReplicaRuntimeError::ConflictingEqualRevision);
            }
        }
        self.world.entities.insert(
            replica_id,
            ReplicaEntityState {
                replica_id,
                disclosure_epoch,
                entity_kind,
                authority_revision: revision,
                components,
            },
        );
        self.authority_revision = self.authority_revision.max(revision);
        Ok(())
    }

    pub fn canonical_team_hash(&self) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"omoba-canonical-team-view-v1\0");
        digest.update(self.team_id.to_be_bytes());
        digest.update(self.world.tick.to_be_bytes());
        for (replica_id, entity) in &self.world.entities {
            digest.update(replica_id.to_be_bytes());
            digest.update(entity.disclosure_epoch.to_be_bytes());
            digest.update(entity.entity_kind.to_be_bytes());
            for (schema_id, bytes) in &entity.components {
                if self.component_allowlist.contains(schema_id) {
                    digest.update(schema_id.to_be_bytes());
                    digest.update((bytes.len() as u64).to_be_bytes());
                    digest.update(bytes);
                }
            }
        }
        for (schema_id, bytes) in &self.world.resources {
            if self.resource_allowlist.contains(schema_id) {
                digest.update(schema_id.to_be_bytes());
                digest.update((bytes.len() as u64).to_be_bytes());
                digest.update(bytes);
            }
        }
        digest.finalize().into()
    }

    pub fn extract_filtered_render_snapshot(&mut self) -> FilteredRenderSnapshot {
        FilteredRenderSnapshot {
            team_id: self.team_id,
            replica_tick: self.world.tick,
            entities: self
                .world
                .entities
                .values()
                .map(|entity| FilteredRenderEntity {
                    replica_id: entity.replica_id,
                    disclosure_epoch: entity.disclosure_epoch,
                    entity_kind: entity.entity_kind,
                    components: entity.components.clone(),
                })
                .collect(),
            public_events: std::mem::take(&mut self.last_injections.public_events),
            external_effects: std::mem::take(&mut self.last_injections.external_effects),
            memory_directives: std::mem::take(&mut self.memory_directives),
        }
    }

    pub fn remembered_presentations(&self) -> &BTreeMap<(u64, u64), Vec<u8>> {
        &self.remembered_presentations
    }

    pub fn take_accepted_inputs(&mut self) -> Vec<TeamAcceptedInput> {
        std::mem::take(&mut self.last_injections.accepted_inputs)
    }

    pub fn apply_verified_rebase(
        &mut self,
        snapshot: &FilteredTeamSnapshot,
        manifest: &TeamViewRebase,
        verified_snapshot_bytes: &[u8],
    ) -> Result<(), ReplicaRuntimeError> {
        if !super::selective::verify_snapshot_manifest(manifest)
            || manifest.team_id != self.team_id
            || manifest.filtered_snapshot_hash != Sha256::digest(verified_snapshot_bytes).as_slice()
            || snapshot.team_id != self.team_id
            || snapshot.disclosed_world != verified_snapshot_bytes
        {
            return Err(ReplicaRuntimeError::UnverifiedRebase);
        }
        let entities = decode_disclosed_world(verified_snapshot_bytes, &self.component_allowlist)?;
        self.world = DisclosedReplicaWorld {
            tick: snapshot.authoritative_tick,
            entities,
            resources: BTreeMap::new(),
        };
        self.view_epoch = snapshot.view_epoch.as_ref().map_or(0, |epoch| epoch.value);
        self.authority_revision = revision_value(&manifest.authority_revision);
        self.expected_replica_tick = snapshot.authoritative_tick;
        self.expected_team_sequence = manifest.resume_team_sequence;
        self.last_injections = StepInjections::default();
        self.memory_directives.clear();
        self.remembered_presentations.clear();
        self.applied_transition_revisions.clear();
        self.retired_replica_filter.fill(0);
        self.stall = ReplicaStallState::Running;
        Ok(())
    }
}

fn post_tick_baseline_ids(pre_step: Option<&PreStep>) -> BTreeSet<u64> {
    pre_step
        .into_iter()
        .flat_map(|pre| &pre.transitions)
        .filter_map(|transition| match transition.transition.as_ref() {
            Some(transition::Transition::Reveal(reveal)) => {
                reveal.replica_entity_id.as_ref().map(|id| id.value)
            }
            Some(transition::Transition::Replace(replace)) => {
                replace.replica_entity_id.as_ref().map(|id| id.value)
            }
            _ => None,
        })
        .collect()
}

fn epoch_value(epoch: &Option<game_proto::DisclosureEpoch>) -> u64 {
    epoch.as_ref().map_or(0, |epoch| epoch.value)
}

fn revision_value(revision: &Option<game_proto::AuthorityRevision>) -> u64 {
    revision.as_ref().map_or(0, |revision| revision.value)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, ReplicaRuntimeError> {
    let value = bytes
        .get(*cursor..*cursor + 4)
        .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
    *cursor += 4;
    Ok(u32::from_be_bytes(value.try_into().unwrap()))
}

fn read_u64(bytes: &[u8], cursor: &mut usize) -> Result<u64, ReplicaRuntimeError> {
    let value = bytes
        .get(*cursor..*cursor + 8)
        .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
    *cursor += 8;
    Ok(u64::from_be_bytes(value.try_into().unwrap()))
}

fn decode_components(
    bytes: &[u8],
    allowlist: &BTreeSet<u32>,
) -> Result<BTreeMap<u32, Vec<u8>>, ReplicaRuntimeError> {
    let mut cursor = 0;
    let count = read_u32(bytes, &mut cursor)?;
    let mut components = BTreeMap::new();
    for _ in 0..count {
        let schema_id = read_u32(bytes, &mut cursor)?;
        if !allowlist.contains(&schema_id) {
            return Err(ReplicaRuntimeError::ComponentNotAllowlisted);
        }
        let len = read_u32(bytes, &mut cursor)? as usize;
        let value = bytes
            .get(cursor..cursor + len)
            .ok_or(ReplicaRuntimeError::MalformedBaseline)?
            .to_vec();
        cursor += len;
        components.insert(schema_id, value);
    }
    if cursor != bytes.len() {
        return Err(ReplicaRuntimeError::MalformedBaseline);
    }
    Ok(components)
}

fn decode_disclosed_world(
    bytes: &[u8],
    allowlist: &BTreeSet<u32>,
) -> Result<BTreeMap<u64, ReplicaEntityState>, ReplicaRuntimeError> {
    let mut cursor = 0;
    let mut entities = BTreeMap::new();
    while cursor < bytes.len() {
        let replica_id = read_u64(bytes, &mut cursor)?;
        let disclosure_epoch = read_u64(bytes, &mut cursor)?;
        let entity_kind = read_u32(bytes, &mut cursor)?;
        let component_count = read_u32(bytes, &mut cursor)?;
        let component_start = cursor - 4;
        for _ in 0..component_count {
            let _schema = read_u32(bytes, &mut cursor)?;
            let len = read_u32(bytes, &mut cursor)? as usize;
            cursor = cursor
                .checked_add(len)
                .filter(|end| *end <= bytes.len())
                .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
        }
        let components = decode_components(&bytes[component_start..cursor], allowlist)?;
        entities.insert(
            replica_id,
            ReplicaEntityState {
                replica_id,
                disclosure_epoch,
                entity_kind,
                authority_revision: 0,
                components,
            },
        );
    }
    Ok(entities)
}
