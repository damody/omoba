use std::collections::{BTreeMap, BTreeSet};

use prost::Message;
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
    UnverifiedRebase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FrameApplyResult {
    Applied {
        replica_tick: u64,
        team_sequence: u64,
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
    stall: ReplicaStallState,
    world: DisclosedReplicaWorld,
    component_allowlist: BTreeSet<u32>,
    resource_allowlist: BTreeSet<u32>,
    last_injections: StepInjections,
    memory_directives: Vec<RenderMemoryDirective>,
    applied_transition_revisions: BTreeMap<(u8, u64, u64), u64>,
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
            stall: ReplicaStallState::Running,
            world: DisclosedReplicaWorld {
                tick: replica_start_tick,
                ..DisclosedReplicaWorld::default()
            },
            component_allowlist,
            resource_allowlist,
            last_injections: StepInjections::default(),
            memory_directives: Vec::new(),
            applied_transition_revisions: BTreeMap::new(),
        }
    }

    pub fn stall_state(&self) -> &ReplicaStallState {
        &self.stall
    }

    pub fn world(&self) -> &DisclosedReplicaWorld {
        &self.world
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
        self.apply_pre_step(frame.pre_step.as_ref(), frame_revision)?;
        self.last_injections = self.inject_step(frame.step.as_ref())?;
        stepper.fixed_step(
            &mut self.world,
            &self.last_injections,
            &self.component_allowlist,
            &self.resource_allowlist,
        )?;
        self.validate_world_allowlist()?;
        self.apply_post_step(frame.post_step.as_ref(), frame_revision)?;

        self.world.tick = self.expected_replica_tick + 1;
        self.expected_replica_tick += 1;
        self.expected_team_sequence += 1;
        self.stall = ReplicaStallState::Running;
        let team_hash = self.canonical_team_hash();
        Ok(FrameApplyResult::Applied {
            replica_tick: frame.replica_tick,
            team_sequence: frame.team_sequence,
            team_hash,
        })
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
                    let replica_id = reveal.replica_entity_id.as_ref().map_or(0, |id| id.value);
                    let disclosure_epoch = epoch_value(&reveal.disclosure_epoch);
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
                    self.record_transition(0, replica_id, disclosure_epoch, frame_revision);
                }
                Some(transition::Transition::Replace(replace)) => {
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
                    self.remove_with_epoch(replica_id, disclosure_epoch)?;
                    self.memory_directives.push(RenderMemoryDirective::Hide {
                        replica_id,
                        disclosure_epoch,
                        remember_policy: hide.remember_policy,
                        sanitized_presentation: hide.sanitized_remembered_presentation.clone(),
                    });
                    self.record_transition(2, replica_id, disclosure_epoch, frame_revision);
                }
                Some(transition::Transition::Forget(forget)) => {
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
                    self.remove_with_epoch(replica_id, disclosure_epoch)?;
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

    fn inject_step(&self, step: Option<&Step>) -> Result<StepInjections, ReplicaRuntimeError> {
        let Some(step) = step else {
            return Ok(StepInjections::default());
        };
        for tape in &step.random_tapes {
            let tape_end = tape
                .first_tick
                .checked_add(u64::from(tape.tick_count))
                .ok_or(ReplicaRuntimeError::StaleDisclosureEpoch)?;
            let replica_id = tape.replica_entity_id.as_ref().map_or(0, |id| id.value);
            let entity = self
                .world
                .entities
                .get(&replica_id)
                .ok_or(ReplicaRuntimeError::UnknownEntity)?;
            if tape.tick_count == 0
                || self.expected_replica_tick < tape.first_tick
                || self.expected_replica_tick >= tape_end
                || entity.disclosure_epoch != epoch_value(&tape.disclosure_epoch)
            {
                return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
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
            self.apply_component_repair(repair)?;
        }
        for replace in &post_step.entity_replaces {
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
        let entity = self
            .world
            .entities
            .get_mut(&replica_id)
            .ok_or(ReplicaRuntimeError::UnknownEntity)?;
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
            return Err(ReplicaRuntimeError::ConflictingEqualRevision);
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
        let current_kind = self
            .world
            .entities
            .get(&replica_id)
            .ok_or(ReplicaRuntimeError::UnknownEntity)?
            .entity_kind;
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
                    entity_kind: entity.entity_kind,
                    components: entity.components.clone(),
                })
                .collect(),
            public_events: std::mem::take(&mut self.last_injections.public_events),
            external_effects: std::mem::take(&mut self.last_injections.external_effects),
            memory_directives: std::mem::take(&mut self.memory_directives),
        }
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
        self.applied_transition_revisions.clear();
        self.stall = ReplicaStallState::Running;
        Ok(())
    }
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
