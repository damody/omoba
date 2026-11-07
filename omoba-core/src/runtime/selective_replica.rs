use std::collections::{BTreeMap, BTreeSet};

use prost::Message;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::comp::replica_stage::{
    MeasuringReplicaStageClock, NoopReplicaStageClock, ReplicaPhaseGuard, ReplicaStageClock,
    ReplicaStageDurations, ReplicaStagePhase,
};
use crate::game_proto::{
    self, transition, BoundedRandomTape, ComponentRepair, EntityReplace, FilteredTeamSnapshot,
    PostStep, PreStep, SanitizedExternalEffect, Step, TeamAcceptedInput, TeamPublicEvent,
    TeamTickFrame, TeamViewRebase,
};

#[cfg(test)]
mod damage_drain_tests {
    use super::*;

    #[test]
    fn damage_drain_preserves_other_effects_and_is_one_shot() {
        let mut runtime = SelectiveReplicaRuntime::new(1, 0, 1, 7, BTreeSet::new(), BTreeSet::new());
        let damage = SanitizedExternalEffect {effect_kind: super::super::FactKind::DirectCombat as u32, ..Default::default()};
        let state = SanitizedExternalEffect {effect_kind: u32::MAX, ..Default::default()};
        runtime.last_injections.external_effects = vec![damage.clone(), state.clone()];
        assert_eq!(runtime.take_damage_presentation_effects(), vec![damage]);
        assert!(runtime.take_damage_presentation_effects().is_empty());
        assert_eq!(runtime.last_injections.external_effects, vec![state]);
    }
}

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
    pub combat_preferences: Option<game_proto::CombatPreferencesProjection>,
    /// M3' ② dev hot reload: script DLLs to swap before this step (frame field 16).
    pub script_module_swaps: Vec<game_proto::ScriptModuleSwap>,
}

pub trait DisclosedWorldStepper {
    fn fixed_step(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        injections: &StepInjections,
        component_allowlist: &BTreeSet<u32>,
        resource_allowlist: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError>;

    fn fixed_step_profiled(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        injections: &StepInjections,
        component_allowlist: &BTreeSet<u32>,
        resource_allowlist: &BTreeSet<u32>,
    ) -> Result<Option<crate::comp::fixed_step_detail::FixedStepDetail>, ReplicaRuntimeError> {
        self.fixed_step(world, injections, component_allowlist, resource_allowlist)?;
        Ok(None)
    }
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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FilteredRenderSnapshot {
    pub team_id: u32,
    pub replica_tick: u64,
    pub entities: Vec<FilteredRenderEntity>,
    pub public_events: Vec<TeamPublicEvent>,
    pub external_effects: Vec<SanitizedExternalEffect>,
    pub memory_directives: Vec<RenderMemoryDirective>,
    /// Frozen, server-sanitized presentation; never part of the gameplay world.
    pub remembered_presentations: BTreeMap<(u64, u64), Vec<u8>>,
    /// Owner-team summoner roster. Standalone extracts stay empty. The replica
    /// host copies the stepper resource after this extract.
    pub disclosed_summoners: super::summoner_transport::DisclosedSummonerProjection,
    /// Last mode installed by an applied frame. Rebase and a successful frame
    /// with an absent or rejected mode clear it. Rejected frames do not.
    pub authoritative_mode: Option<game_proto::AuthoritativeModePresentation>,
    pub combat_preferences: Option<game_proto::CombatPreferencesProjection>,
    /// Computed by the current filtered Specs world, scoped by body identity and tick.
    pub hero_attributes: BTreeMap<(u64, u64), super::hero_attributes::DisclosedHeroAttributes>,
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
    /// Last measured apply. Cleared on every entry and only set for `Applied`.
    /// Not part of `world` and not hashed.
    stage_profile: Option<ReplicaStageDurations>,
    presented_mode: Option<game_proto::AuthoritativeModePresentation>,
    combat_preferences: Option<game_proto::CombatPreferencesProjection>,
    /// Owner-undo fact bytes from the last successfully injected step, or from
    /// the verified bootstrap/rebase that restored that step. Empty leaves the
    /// v1 team digest unchanged and is the no-fact, unavailable-without-a-fact,
    /// and retirement-cleared state.
    owner_undo_hash_material: Vec<u8>,
    /// Private owner trinket bytes. Empty stays outside the team digest.
    /// A rebase does not carry this fact. A verified owner envelope restores it.
    owner_ward_projection: Vec<u8>,
    owner_ward_verified: Vec<u8>,
    owner_ward_player: u32,
    owner_ward_epoch: u64,
    owner_ward_session: u64,
    owner_ward_tick: u64,
    owner_ward_revision: u64,
    owner_ward_needs_recovery: bool,
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
            stage_profile: None,
            presented_mode: None,
            combat_preferences: None,
            owner_undo_hash_material: Vec::new(),
            owner_ward_projection: Vec::new(),
            owner_ward_verified: Vec::new(),
            owner_ward_player: 0,
            owner_ward_epoch: 0,
            owner_ward_session: 0,
            owner_ward_tick: 0,
            owner_ward_revision: 0,
            owner_ward_needs_recovery: false,
        }
    }

    pub fn owner_ward_projection(&self) -> &[u8] {
        &self.owner_ward_projection
    }

    pub fn owner_ward_needs_recovery(&self) -> bool {
        self.owner_ward_needs_recovery
    }

    pub fn owner_ward_tick(&self) -> u64 {
        self.owner_ward_tick
    }

    pub fn owner_ward_revision(&self) -> u64 {
        self.owner_ward_revision
    }

    pub fn owner_ward_session(&self) -> u64 {
        self.owner_ward_session
    }

    pub fn owner_ward_player(&self) -> u32 {
        self.owner_ward_player
    }

    /// Verified owner envelope. Rejected input clears placement capability and
    /// requests recovery, but it cannot move the last verified tick or revision.
    pub fn apply_verified_owner_ward_envelope(
        &mut self,
        envelope: &game_proto::OwnerWardEnvelope,
    ) -> Result<(), &'static str> {
        let opened = match crate::runtime::ward_transport::open_owner_ward_envelope(envelope) {
            Ok(opened) => opened,
            Err(error) => {
                self.reject_unverified_owner_ward();
                return Err(error);
            }
        };
        if envelope.view_epoch != self.view_epoch
            || self.owner_ward_player == 0
            || envelope.player_id != self.owner_ward_player
        {
            self.reject_unverified_owner_ward();
            return Err("OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
        }
        if self.owner_ward_session != 0 && envelope.session_id != self.owner_ward_session {
            self.reject_unverified_owner_ward();
            return Err("OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
        }
        let (incoming, revision) = match &opened {
            crate::runtime::ward_transport::OwnerWardEnvelopeBody::Clear => (&[][..], 0),
            crate::runtime::ward_transport::OwnerWardEnvelopeBody::Apply { projection, .. } => {
                (envelope.projection.as_slice(), projection.revision)
            }
        };
        let newer = envelope.authoritative_tick > self.owner_ward_tick
            || (envelope.authoritative_tick == self.owner_ward_tick && revision > self.owner_ward_revision);
        let same = envelope.authoritative_tick == self.owner_ward_tick && revision == self.owner_ward_revision;
        if !newer && !same {
            return Err("OWNER_WARD_STALE");
        }
        if same {
            if incoming != self.owner_ward_verified.as_slice() {
                self.reject_unverified_owner_ward();
                return Err("NONCANONICAL_OWNER_WARD_SAME_TICK_BYTES");
            }
            self.owner_ward_projection = incoming.to_vec();
            if self.owner_ward_session == 0 {
                self.owner_ward_session = envelope.session_id;
            }
            self.owner_ward_needs_recovery = false;
            return Ok(());
        }
        self.owner_ward_projection = incoming.to_vec();
        self.owner_ward_verified = incoming.to_vec();
        self.owner_ward_player = envelope.player_id;
        self.owner_ward_epoch = envelope.view_epoch;
        self.owner_ward_session = envelope.session_id;
        self.owner_ward_tick = envelope.authoritative_tick;
        self.owner_ward_revision = revision;
        self.owner_ward_needs_recovery = false;
        Ok(())
    }

    /// Byte-level continuing apply. Prefer [`Self::apply_verified_owner_ward_envelope`].
    /// Decode failure clears. A mismatched owner or epoch clears.
    pub fn apply_continuing_owner_ward_projection(
        &mut self,
        player_id: u32,
        bytes: &[u8],
    ) -> Result<(), &'static str> {
        if bytes.is_empty() || player_id == 0 {
            self.clear_owner_ward_bytes();
            return Ok(());
        }
        let decoded = match crate::runtime::ward_transport::decode_owner_ward_projection(bytes) {
            Ok(decoded) => decoded,
            Err(error) => {
                self.clear_owner_ward_bytes();
                return Err(error);
            }
        };
        if decoded.player_id != player_id || decoded.view_epoch != self.view_epoch {
            self.clear_owner_ward_bytes();
            return Err("OWNER_WARD_OWNER_OR_EPOCH_MISMATCH");
        }
        self.owner_ward_projection = bytes.to_vec();
        self.owner_ward_verified = bytes.to_vec();
        self.owner_ward_player = decoded.player_id;
        self.owner_ward_epoch = decoded.view_epoch;
        self.owner_ward_revision = decoded.revision;
        Ok(())
    }

    /// Drops usable capability. The last verified owner, session, epoch, tick,
    /// and revision stay where the last accepted envelope left them.
    fn reject_unverified_owner_ward(&mut self) {
        self.owner_ward_projection.clear();
        if self.owner_ward_player != 0 || self.owner_ward_session != 0 {
            self.owner_ward_needs_recovery = true;
        }
    }

    fn clear_owner_ward_bytes(&mut self) {
        self.owner_ward_projection.clear();
    }

    fn clear_owner_ward(&mut self) {
        self.owner_ward_projection.clear();
        self.owner_ward_verified.clear();
        self.owner_ward_player = 0;
        self.owner_ward_epoch = 0;
        self.owner_ward_session = 0;
        self.owner_ward_tick = 0;
        self.owner_ward_revision = 0;
        self.owner_ward_needs_recovery = false;
    }

    fn restore_owner_ward_bootstrap(
        &mut self,
        start: &game_proto::TeamGameStart,
    ) -> Result<(), ReplicaRuntimeError> {
        let negotiated = crate::runtime::ward_transport::negotiate_ward_protocol(
            start.ward_protocol_version,
            &start.ward_rules_hash,
        )
        .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        if !negotiated {
            // Version 0 is an explicit disable and must not smuggle a projection.
            // Any other unsupported version, including previously accepted protocol 2,
            // disables only ward. Its payload is not decoded and does not fail bootstrap.
            let carries = start.owner_ward_envelope.is_some() || !start.owner_ward_projection.is_empty();
            if start.ward_protocol_version == 0 && carries {
                return Err(ReplicaRuntimeError::MalformedBaseline);
            }
            self.clear_owner_ward();
            return Ok(());
        }
        let Some(envelope) = start.owner_ward_envelope.as_ref() else {
            return Err(ReplicaRuntimeError::MalformedBaseline);
        };
        if start.player_id == 0 || envelope.projection.as_slice() != start.owner_ward_projection.as_slice() {
            return Err(ReplicaRuntimeError::MalformedBaseline);
        }
        let opened = crate::runtime::ward_transport::open_owner_ward_envelope(envelope)
            .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        let epoch = start.view_epoch.as_ref().map_or(0, |epoch| epoch.value);
        let tick = crate::runtime::ward_transport::owner_ward_bootstrap_tick(start);
        if envelope.player_id != start.player_id
            || envelope.view_epoch != epoch
            || envelope.authoritative_tick != tick
        {
            return Err(ReplicaRuntimeError::MalformedBaseline);
        }
        let (bytes, revision) = match opened {
            crate::runtime::ward_transport::OwnerWardEnvelopeBody::Clear => (Vec::new(), 0),
            crate::runtime::ward_transport::OwnerWardEnvelopeBody::Apply { projection, .. } => {
                (envelope.projection.clone(), projection.revision)
            }
        };
        self.owner_ward_projection = bytes.clone();
        self.owner_ward_verified = bytes;
        self.owner_ward_player = envelope.player_id;
        self.owner_ward_epoch = epoch;
        self.owner_ward_session = envelope.session_id;
        self.owner_ward_tick = tick;
        self.owner_ward_revision = revision;
        self.owner_ward_needs_recovery = false;
        Ok(())
    }

    pub fn stall_state(&self) -> &ReplicaStallState {
        &self.stall
    }

    pub fn bootstrap_from_team_game_start(
        start: &game_proto::TeamGameStart,
        component_allowlist: BTreeSet<u32>,
        resource_allowlist: BTreeSet<u32>,
    ) -> Result<Self, ReplicaRuntimeError> {
        if start.protocol_version != crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION {
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
        runtime.owner_undo_hash_material = retained_owner_undo_for_bootstrap(start, snapshot)?;
        runtime.restore_owner_ward_bootstrap(start)?;
        super::combat_transport::decode_projection(start.combat_preferences.as_ref(), runtime.view_epoch)
            .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
        runtime.combat_preferences = start.combat_preferences.clone();
        Ok(runtime)
    }

    pub fn global_seed(&self) -> u64 {
        self.global_seed
    }

    pub fn view_epoch(&self) -> u64 {
        self.view_epoch
    }

    pub fn combat_preferences_for_owner(&self, owner: u32) -> Option<super::combat_controls::OwnerCombatPreferences> {
        super::combat_transport::decode_projection(self.combat_preferences.as_ref(), self.view_epoch)
            .ok()?.get(&owner).copied()
    }

    pub fn input_scheme_for_owner(&self, owner: u32) -> Option<super::input_scheme_transport::AdmittedOwnerInputScheme> {
        super::input_scheme_transport::for_owner(self.combat_preferences.as_ref(),self.view_epoch,owner)
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
        self.apply_encoded_frame_with(bytes, stepper, &mut NoopReplicaStageClock)
    }

    /// Same apply as [`Self::apply_encoded_frame`]. Phase durations are wall-clock
    /// only and are stored for [`Self::take_stage_profile`] when the result is `Applied`.
    pub fn apply_encoded_frame_profiled(
        &mut self,
        bytes: &[u8],
        stepper: &mut impl DisclosedWorldStepper,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        self.apply_encoded_frame_with(bytes, stepper, &mut MeasuringReplicaStageClock::new())
    }

    pub fn take_stage_profile(&mut self) -> Option<ReplicaStageDurations> {
        self.stage_profile.take()
    }

    fn apply_encoded_frame_with<C: ReplicaStageClock>(
        &mut self,
        bytes: &[u8],
        stepper: &mut impl DisclosedWorldStepper,
        clock: &mut C,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        self.stage_profile = None;
        let frame = {
            let decode = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::Decode);
            let frame = TeamTickFrame::decode(bytes).map_err(|_| ReplicaRuntimeError::Decode)?;
            decode.close();
            frame
        };
        self.apply_frame_with(frame, stepper, clock)
    }

    pub fn apply_frame(
        &mut self,
        frame: TeamTickFrame,
        stepper: &mut impl DisclosedWorldStepper,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        self.apply_frame_with(frame, stepper, &mut NoopReplicaStageClock)
    }

    fn apply_frame_with<C: ReplicaStageClock>(
        &mut self,
        frame: TeamTickFrame,
        stepper: &mut impl DisclosedWorldStepper,
        clock: &mut C,
    ) -> Result<FrameApplyResult, ReplicaRuntimeError> {
        self.stage_profile = None;
        self.last_apply_fault = None;
        self.memory_directives.clear();
        self.applied_transition_revisions.clear();
        if frame.protocol_version != crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION {
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
        let staged_baselines = {
            super::combat_transport::decode_projection(frame.combat_preferences.as_ref(), frame_view_epoch)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            super::input_scheme_transport::decode_direction_steps(frame.combat_preferences.as_ref(), frame_view_epoch, self.expected_replica_tick)
                .map_err(|_| ReplicaRuntimeError::MalformedBaseline)?;
            let staging = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::Staging);
            self.preflight_entity_references(&frame)?;
            self.apply_pre_step(frame.pre_step.as_ref(), frame_revision)?;
            self.last_injections = self.inject_step(frame.step.as_ref())?;
            self.last_injections.combat_preferences = frame.combat_preferences.clone();
            self.last_injections.script_module_swaps = frame.script_module_swaps.clone();
            self.combat_preferences = frame.combat_preferences.clone();
            self.owner_undo_hash_material = crate::runtime::shop_undo_wire::owner_undo_hash_material(
                frame.step.as_ref().map(|step| step.public_events.as_slice()).unwrap_or(&[]),
            );
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
            staging.close();
            staged_baselines
        };
        let measuring = clock.measuring();
        let stepped = {
            let fixed = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::FixedStep);
            let stepped = if measuring { stepper.fixed_step_profiled(
                &mut self.world,
                &self.last_injections,
                &self.component_allowlist,
                &self.resource_allowlist,
            ) } else { stepper.fixed_step(
                &mut self.world, &self.last_injections,
                &self.component_allowlist, &self.resource_allowlist,
            ).map(|()| None) };
            fixed.close();
            stepped
        };
        let fixed_step_detail = stepped?;
        let pre_repair_observed_hash = {
            let pre_repair = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::PreRepair);
            self.world.entities.extend(staged_baselines);
            self.validate_world_allowlist()?;
            self.world.tick = self.expected_replica_tick + 1;
            let hash = self.canonical_team_hash();
            pre_repair.close();
            hash
        };
        {
            let post_step = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::PostStepRepair);
            self.apply_post_step(frame.post_step.as_ref(), frame_revision)?;
            post_step.close();
        }
        let post_repair_hash = {
            let post_hash = ReplicaPhaseGuard::enter(clock, ReplicaStagePhase::PostRepairHash);
            let hash = self.canonical_team_hash();
            post_hash.close();
            hash
        };
        self.expected_replica_tick += 1;
        self.expected_team_sequence += 1;
        self.stall = ReplicaStallState::Running;
        if clock.measuring() {
            let mut durations = clock.durations();
            durations.fixed_step_detail = fixed_step_detail;
            self.stage_profile = Some(durations);
        }
        self.presented_mode = frame.authoritative_mode.filter(|mode| {
            super::authoritative_mode::accepted(Some(mode), frame_view_epoch)
                != super::authoritative_mode::UNAVAILABLE
        });
        self.combat_preferences = frame.combat_preferences;
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
        let mut remembered: BTreeSet<(u64, u64)> = self.remembered_presentations.keys().copied().collect();
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
                        let replica_id = reveal.replica_entity_id.as_ref().map_or(0, |id| id.value);
                        entities.insert(replica_id);
                        remembered.retain(|(id, _)| *id != replica_id);
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
                        remembered.insert((replica_id, epoch_value(&hide.disclosure_epoch)));
                    }
                    Some(transition::Transition::Forget(forget)) => {
                        let replica_id = forget.replica_entity_id.as_ref().map_or(0, |id| id.value);
                        let epoch = epoch_value(&forget.disclosure_epoch);
                        if !entities.remove(&replica_id) && !remembered.remove(&(replica_id, epoch)) {
                            self.last_apply_fault = Some(ReplicaApplyFault {
                                phase: ReplicaApplyPhase::PreStep,
                                operation: ReplicaApplyOperation::Forget,
                                replica_id,
                                disclosure_epoch: epoch_value(&forget.disclosure_epoch),
                            });
                            return Err(if remembered.iter().any(|(id, _)| *id == replica_id) {
                                ReplicaRuntimeError::StaleDisclosureEpoch
                            } else {
                                ReplicaRuntimeError::UnknownEntity
                            });
                        }
                    }
                    None => {}
                }
            }
        }
        if let Some(step) = &frame.step {
            for input in &step.accepted_inputs {
                super::summon_transport::reject_kind_alias(input).map_err(|_|ReplicaRuntimeError::Decode)?;
                if input.action_kind==super::summon_transport::ACTION_KIND {
                    super::summon_transport::validate_projected_ack(input).map_err(|_|ReplicaRuntimeError::Decode)?;
                }
                super::direction_transport::reject_kind_alias(input).map_err(|_|ReplicaRuntimeError::Decode)?;
                if input.action_kind==super::direction_transport::ACTION_KIND {
                    super::direction_transport::validate_projected_ack(input).map_err(|_|ReplicaRuntimeError::Decode)?;
                    if !super::direction_transport::available(frame.combat_preferences.as_ref(),frame.view_epoch.as_ref().map_or(0,|epoch|epoch.value),input.player_id) {
                        return Err(ReplicaRuntimeError::WrongTeam);
                    }
                }
                if super::input_scheme_transport::is_owner_setting_action(input.action_kind) {
                    super::input_scheme_transport::validate_owner_setting_ack(input).map_err(|_| ReplicaRuntimeError::Decode)?;
                    if !super::input_scheme_transport::owner_setting_available(frame.combat_preferences.as_ref(),frame.view_epoch.as_ref().map_or(0,|epoch|epoch.value),input.action_kind,input.player_id) {
                        return Err(ReplicaRuntimeError::WrongTeam);
                    }
                    continue;
                }
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
                    if let Err(error) = self.forget_with_epoch(replica_id, disclosure_epoch) {
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

    fn forget_with_epoch(&mut self, replica_id: u64, disclosure_epoch: u64) -> Result<(), ReplicaRuntimeError> {
        if self.world.entities.contains_key(&replica_id) {
            return self.remove_with_epoch(replica_id, disclosure_epoch);
        }
        if self.remembered_presentations.contains_key(&(replica_id, disclosure_epoch)) {
            return Ok(());
        }
        if self.remembered_presentations.keys().any(|(id, _)| *id == replica_id) {
            return Err(ReplicaRuntimeError::StaleDisclosureEpoch);
        }
        Err(ReplicaRuntimeError::UnknownEntity)
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
            super::summon_transport::reject_kind_alias(input).map_err(|_|ReplicaRuntimeError::Decode)?;
            if input.action_kind==super::summon_transport::ACTION_KIND {
                super::summon_transport::validate_projected_ack(input).map_err(|_|ReplicaRuntimeError::Decode)?;
            }
            super::direction_transport::reject_kind_alias(input).map_err(|_|ReplicaRuntimeError::Decode)?;
            if input.action_kind==super::direction_transport::ACTION_KIND {
                super::direction_transport::validate_projected_ack(input).map_err(|_|ReplicaRuntimeError::Decode)?;
            }
            if super::input_scheme_transport::is_owner_setting_action(input.action_kind) {
                super::input_scheme_transport::validate_owner_setting_ack(input).map_err(|_| ReplicaRuntimeError::Decode)?;
                continue;
            }
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
            combat_preferences: None,
            script_module_swaps: Vec::new(),
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
        digest.update(crate::runtime::shop_undo_wire::CANONICAL_TEAM_VIEW_DOMAIN);
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
        crate::runtime::shop_undo_wire::mix_team_undo_material(&mut digest, &self.owner_undo_hash_material);
        super::combat_transport::mix_projection_hash(&mut digest, self.combat_preferences.as_ref());
        digest.finalize().into()
    }

    /// Borrow presentation events immediately after a successful apply, before
    /// catch-up can replace the injection batch or rendering can drain it.
    pub fn applied_public_events(&self) -> &[TeamPublicEvent] {
        &self.last_injections.public_events
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
            remembered_presentations: self.remembered_presentations.iter()
                .filter(|(_, presentation)| !presentation.is_empty())
                .map(|(key, presentation)| (*key, presentation.clone()))
                .collect(),
            disclosed_summoners: super::summoner_transport::DisclosedSummonerProjection::default(),
            authoritative_mode: self.presented_mode.clone(),
            combat_preferences: self.combat_preferences.clone(),
            hero_attributes: BTreeMap::new(),
        }
    }

    pub fn remembered_presentations(&self) -> &BTreeMap<(u64, u64), Vec<u8>> {
        &self.remembered_presentations
    }

    pub fn take_accepted_inputs(&mut self) -> Vec<TeamAcceptedInput> {
        std::mem::take(&mut self.last_injections.accepted_inputs)
    }

    /// Drain only one-shot damage after stepping, without cloning the disclosed world.
    pub fn take_damage_presentation_effects(&mut self) -> Vec<SanitizedExternalEffect> {
        let (damage, other) = std::mem::take(&mut self.last_injections.external_effects)
            .into_iter().partition(|effect| effect.effect_kind == super::FactKind::DirectCombat as u32);
        self.last_injections.external_effects = other;
        damage
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
            || snapshot.view_epoch != manifest.view_epoch
            || snapshot.authoritative_tick != manifest.authoritative_tick
            || snapshot.snapshot_id != manifest.snapshot_id
            || manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value) == 0
            || manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value) < self.view_epoch
        {
            return Err(ReplicaRuntimeError::UnverifiedRebase);
        }
        let retained_owner_undo = retained_owner_undo_for_rebase(snapshot, manifest)?;
        super::combat_transport::decode_projection(manifest.combat_preferences.as_ref(),
            manifest.view_epoch.as_ref().map_or(0, |epoch| epoch.value))
            .map_err(|_| ReplicaRuntimeError::UnverifiedRebase)?;
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
        self.owner_undo_hash_material = retained_owner_undo;
        // The rebase manifest stays free of private charges. Keep the bound
        // session and require a newer verified owner envelope to restore.
        let session = self.owner_ward_session;
        let player = self.owner_ward_player;
        self.owner_ward_projection.clear();
        self.owner_ward_verified.clear();
        self.owner_ward_session = session;
        self.owner_ward_epoch = self.view_epoch;
        self.owner_ward_tick = snapshot.authoritative_tick;
        self.owner_ward_revision = 0;
        self.owner_ward_needs_recovery = session != 0 || player != 0;
        self.memory_directives.clear();
        self.remembered_presentations.clear();
        self.applied_transition_revisions.clear();
        self.retired_replica_filter.fill(0);
        self.stall = ReplicaStallState::Running;
        self.stage_profile = None;
        self.presented_mode = None;
        self.combat_preferences = manifest.combat_preferences.clone();
        Ok(())
    }
}

fn retained_owner_undo_for_bootstrap(
    start: &game_proto::TeamGameStart,
    snapshot: &game_proto::FilteredTeamSnapshot,
) -> Result<Vec<u8>, ReplicaRuntimeError> {
    if crate::runtime::shop_undo_wire::metadata_declares_owner_undo(&start.public_metadata)
        || crate::runtime::shop_undo_wire::metadata_declares_owner_undo(&snapshot.public_metadata)
    {
        return Err(ReplicaRuntimeError::MalformedBaseline);
    }
    let from_start = crate::runtime::shop_undo_wire::retained_owner_undo_from_metadata(
        &start.team_private_metadata,
    )
    .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
    let from_snapshot = crate::runtime::shop_undo_wire::retained_owner_undo_from_metadata(
        &snapshot.team_private_metadata,
    )
    .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
    if from_start != from_snapshot {
        return Err(ReplicaRuntimeError::MalformedBaseline);
    }
    Ok(from_start)
}

/// The verified manifest is authoritative. Snapshot metadata may echo the same
/// bytes and must not contradict them. Omitting the echo is the chunk-reassembly
/// path, which has no private metadata of its own. A public-channel copy fails.
fn retained_owner_undo_for_rebase(
    snapshot: &game_proto::FilteredTeamSnapshot,
    manifest: &game_proto::TeamViewRebase,
) -> Result<Vec<u8>, ReplicaRuntimeError> {
    if crate::runtime::shop_undo_wire::metadata_declares_owner_undo(&snapshot.public_metadata) {
        return Err(ReplicaRuntimeError::MalformedBaseline);
    }
    let echoed = crate::runtime::shop_undo_wire::retained_owner_undo_from_metadata(
        &snapshot.team_private_metadata,
    )
    .ok_or(ReplicaRuntimeError::MalformedBaseline)?;
    let retained = if manifest.retained_owner_undo_material.is_empty() {
        Vec::new()
    } else {
        crate::runtime::shop_undo_wire::canonical_retained_owner_undo_material(
            &manifest.retained_owner_undo_material,
        )
        .ok_or(ReplicaRuntimeError::UnverifiedRebase)?
    };
    if !echoed.is_empty() && echoed != retained {
        return Err(ReplicaRuntimeError::UnverifiedRebase);
    }
    Ok(retained)
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

#[cfg(test)]
mod replica_stage_tests {
    use super::{DisclosedWorldStepper, DisclosedReplicaWorld, StepInjections};
    use std::collections::BTreeSet;

    use prost::Message;

    use super::{
        FrameApplyResult, NoopDisclosedWorldStepper, ReplicaRuntimeError, SelectiveReplicaRuntime,
    };
    use crate::comp::replica_stage::{ReplicaStagePhase, REPLICA_STAGE_PHASE_COUNT};
    use crate::game_proto::{AuthorityRevision, PostStep, PreStep, Step, TeamTickFrame, ViewEpoch};

    fn encoded_frame(tick: u64, sequence: u64) -> Vec<u8> {
        TeamTickFrame {
            protocol_version: crate::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            frame_schema_version: 1,
            content_schema_version: 1,
            team_id: 1,
            server_tick: tick,
            replica_tick: tick,
            team_sequence: sequence,
            view_epoch: Some(ViewEpoch { value: 1 }),
            authority_revision: Some(AuthorityRevision { value: 1 }),
            pre_step: Some(PreStep::default()),
            step: Some(Step::default()),
            post_step: Some(PostStep::default()),
            padding: Vec::new(),
            authoritative_mode: None, combat_preferences: None, script_module_swaps: Vec::new(),
        }
        .encode_to_vec()
    }

    fn runtime() -> SelectiveReplicaRuntime {
        SelectiveReplicaRuntime::new(1, 0, 0, 1, BTreeSet::new(), BTreeSet::new())
    }

    struct DetailStepper;
    impl DisclosedWorldStepper for DetailStepper {
        fn fixed_step(&mut self, _: &mut DisclosedReplicaWorld, _: &StepInjections,
            _: &BTreeSet<u32>, _: &BTreeSet<u32>) -> Result<(), ReplicaRuntimeError> {
            Ok(())
        }
        fn fixed_step_profiled(&mut self, world: &mut DisclosedReplicaWorld, injections: &StepInjections,
            components: &BTreeSet<u32>, resources: &BTreeSet<u32>)
            -> Result<Option<crate::comp::fixed_step_detail::FixedStepDetail>, ReplicaRuntimeError> {
            self.fixed_step(world, injections, components, resources)?;
            Ok(Some(crate::comp::fixed_step_detail::FixedStepDetail {
                process_cpu_ns: Some(999), ..Default::default()
            }))
        }
    }

    #[test]
    fn replica_stage_detail_is_opt_in_and_does_not_change_hash() {
        let mut plain = runtime();
        let mut measured = runtime();
        let frame = encoded_frame(0, 0);
        assert_eq!(plain.apply_encoded_frame(&frame, &mut DetailStepper).unwrap(),
            measured.apply_encoded_frame_profiled(&frame, &mut DetailStepper).unwrap());
        assert!(plain.take_stage_profile().is_none());
        assert_eq!(measured.take_stage_profile().unwrap().fixed_step_detail.unwrap().process_cpu_ns,
            Some(999));
        assert_eq!(plain.canonical_team_hash(), measured.canonical_team_hash());
    }

    #[test]
    fn replica_stage_measured_and_unmeasured_apply_share_result_and_hash() {
        let first = encoded_frame(0, 0);
        let second = encoded_frame(1, 1);
        let mut plain = runtime();
        let mut measured = runtime();
        let mut plain_step = NoopDisclosedWorldStepper;
        let mut measured_step = NoopDisclosedWorldStepper;
        let plain_first = plain
            .apply_encoded_frame(&first, &mut plain_step)
            .expect("unmeasured apply");
        let measured_first = measured
            .apply_encoded_frame_profiled(&first, &mut measured_step)
            .expect("measured apply");
        assert_eq!(plain_first, measured_first);
        assert!(plain.take_stage_profile().is_none());
        let profile = measured
            .take_stage_profile()
            .expect("applied profile");
        assert_eq!(profile.phases_ns.len(), REPLICA_STAGE_PHASE_COUNT);
        assert_eq!(profile.phase_ns(ReplicaStagePhase::HostFinalize), 0);
        let plain_second = plain
            .apply_encoded_frame(&second, &mut plain_step)
            .expect("unmeasured second");
        let measured_second = measured
            .apply_encoded_frame_profiled(&second, &mut measured_step)
            .expect("measured second");
        assert_eq!(plain_second, measured_second);
        assert_eq!(plain.canonical_team_hash(), measured.canonical_team_hash());
        assert_eq!(plain.world(), measured.world());
        assert!(plain.world().resources.is_empty());
        assert!(measured.take_stage_profile().is_some());
        assert!(plain.take_stage_profile().is_none());
    }

    #[test]
    fn replica_stage_duplicate_stall_and_error_drop_stale_profile() {
        let mut runtime = runtime();
        let mut stepper = DetailStepper;
        runtime
            .apply_encoded_frame_profiled(&encoded_frame(0, 0), &mut stepper)
            .unwrap();
        assert!(runtime.take_stage_profile().is_some());
        runtime
            .apply_encoded_frame_profiled(&encoded_frame(1, 1), &mut stepper)
            .unwrap();
        let after_apply = runtime.canonical_team_hash();
        assert!(matches!(
            runtime
                .apply_encoded_frame_profiled(&encoded_frame(0, 0), &mut stepper)
                .unwrap(),
            FrameApplyResult::Duplicate
        ));
        assert!(runtime.take_stage_profile().is_none());
        assert_eq!(runtime.canonical_team_hash(), after_apply);

        runtime
            .apply_encoded_frame_profiled(&encoded_frame(2, 2), &mut stepper)
            .unwrap();
        let after_next = runtime.canonical_team_hash();
        assert!(matches!(
            runtime
                .apply_encoded_frame_profiled(&encoded_frame(9, 9), &mut stepper)
                .unwrap(),
            FrameApplyResult::Stalled(_)
        ));
        assert!(runtime.take_stage_profile().is_none());
        assert_eq!(runtime.canonical_team_hash(), after_next);

        runtime
            .apply_encoded_frame_profiled(&encoded_frame(3, 3), &mut stepper)
            .unwrap();
        let after_recovered = runtime.canonical_team_hash();
        let err = runtime.apply_encoded_frame_profiled(b"\xff\xff", &mut stepper);
        assert_eq!(err, Err(ReplicaRuntimeError::Decode));
        assert!(runtime.take_stage_profile().is_none());
        assert_eq!(runtime.canonical_team_hash(), after_recovered);
        assert!(runtime.world().resources.is_empty());
    }
}

#[cfg(test)]
mod owner_undo_baseline_tests {
    use super::{NoopDisclosedWorldStepper, SelectiveReplicaRuntime};
    use crate::game_proto::{FilteredTeamSnapshot, TeamViewRebase, TeamViewRebaseChunk};
    use crate::runtime::shop::{UNDO_REASON_CAN_UNDO, UNDO_REASON_RETIRED};
    use crate::runtime::shop_undo_wire::{
        metadata_declares_owner_undo, owner_undo_hash_material, retained_owner_undo_from_metadata,
        OwnerShopUndoProjection, OWNER_UNDO_RETAINED_NAMESPACE,
    };
    use crate::runtime::{
        verify_snapshot_manifest, FactAudience, FactKind, FactOrderingKey, FactPhase, FrameApplyResult,
        IncompleteSnapshotStaging, ObservableFact, OrderedFact, ProjectionDependencyGraph,
        TeamProjectorConfig, TeamViewProjector,
    };
    use std::collections::BTreeSet;

    fn allow() -> BTreeSet<u32> {
        BTreeSet::new()
    }

    fn undo_fact(phase: u8, can_undo: bool, reason: u8, sequence: u64) -> OrderedFact {
        OrderedFact {
            key: FactOrderingKey {
                tick: 1,
                phase: if phase == 0 { FactPhase::PreStep } else { FactPhase::PostStep },
                canonical_source_order: 0,
                local_ordinal: u32::from(phase),
                fact_kind: FactKind::OwnerShopUndo,
            },
            audience: FactAudience::Team(1),
            fact: ObservableFact::OwnerShopUndo {
                team: 1,
                state: OwnerShopUndoProjection {
                    phase,
                    player_id: 7,
                    can_undo,
                    reason,
                    newest_sequence: sequence,
                    commitment: [9; 32],
                },
            },
        }
    }

    fn bootstrap(start: &crate::game_proto::TeamGameStart) -> Result<SelectiveReplicaRuntime, super::ReplicaRuntimeError> {
        SelectiveReplicaRuntime::bootstrap_from_team_game_start(start, allow(), BTreeSet::new())
    }

    fn undo_slot(entries: &mut [crate::game_proto::DeterministicMetadata]) -> &mut crate::game_proto::DeterministicMetadata {
        entries.iter_mut().find(|entry| entry.namespace == OWNER_UNDO_RETAINED_NAMESPACE).expect("retained owner undo")
    }

    fn assemble(manifest: &TeamViewRebase, chunks: &[TeamViewRebaseChunk]) -> (FilteredTeamSnapshot, Vec<u8>) {
        let mut staging = IncompleteSnapshotStaging::default();
        staging.begin(manifest.snapshot_id.clone().expect("snapshot id"), chunks.len() as u32);
        for chunk in chunks {
            assert!(staging.insert(chunk), "rebase chunk");
        }
        let bytes = staging.finish(manifest).expect("verified rebase bytes");
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
        (snapshot, bytes)
    }

    #[test]
    fn continuing_purchase_bootstrap_rebase_and_retirement_share_one_hash() {
        let graph = ProjectionDependencyGraph::default();
        let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
        let baseline = projector.build_team_game_start(1, 120, 9);
        let mut continued = bootstrap(&baseline).expect("empty baseline");
        assert!(retained_owner_undo_from_metadata(&baseline.team_private_metadata).unwrap().is_empty());

        let frame = projector
            .build_frame(1, 1, &BTreeSet::new(), vec![], &[undo_fact(1, true, UNDO_REASON_CAN_UNDO, 4)], &graph)
            .expect("purchase frame");
        let events = frame.frame.step.as_ref().unwrap().public_events.clone();
        let purchase = events.iter().find(|event| event.event_kind == FactKind::OwnerShopUndo as u32).expect("owner undo fact");
        let decoded = OwnerShopUndoProjection::decode(&purchase.sanitized_payload).expect("purchase projection");
        assert!(decoded.can_undo && decoded.player_id == 7 && decoded.newest_sequence == 4);
        assert!(matches!(
            continued.apply_frame(frame.frame, &mut NoopDisclosedWorldStepper).unwrap(),
            FrameApplyResult::Applied { .. }
        ));
        let purchased = projector.build_team_game_start(2, 120, 9);
        let fresh = bootstrap(&purchased).expect("purchase baseline");
        assert_eq!(continued.canonical_team_hash(), fresh.canonical_team_hash());
        assert_eq!(
            retained_owner_undo_from_metadata(&purchased.team_private_metadata).unwrap(),
            owner_undo_hash_material(&events)
        );
        let mut stripped = purchased.clone();
        stripped.team_private_metadata.retain(|entry| entry.namespace != OWNER_UNDO_RETAINED_NAMESPACE);
        stripped.filtered_snapshot.as_mut().unwrap().team_private_metadata.retain(|entry| entry.namespace != OWNER_UNDO_RETAINED_NAMESPACE);
        assert_ne!(bootstrap(&stripped).unwrap().canonical_team_hash(), continued.canonical_team_hash());

        let mut enemy = TeamViewProjector::new(2, TeamProjectorConfig::default());
        let enemy_frame = enemy
            .build_frame(1, 1, &BTreeSet::new(), vec![], &[undo_fact(1, true, UNDO_REASON_CAN_UNDO, 4)], &graph)
            .unwrap();
        assert!(enemy_frame.frame.step.unwrap().public_events.iter().all(|event| event.event_kind != FactKind::OwnerShopUndo as u32));
        let enemy_start = enemy.build_team_game_start(2, 120, 9);
        let quiet = TeamViewProjector::new(2, TeamProjectorConfig::default()).build_team_game_start(2, 120, 9);
        assert!(retained_owner_undo_from_metadata(&enemy_start.team_private_metadata).unwrap().is_empty());
        assert_eq!(bootstrap(&enemy_start).unwrap().canonical_team_hash(), bootstrap(&quiet).unwrap().canonical_team_hash());

        let bundle = projector.build_filtered_rebase(2, 2, 1).expect("purchase rebase");
        assert_eq!(bundle.manifest.retained_owner_undo_material, owner_undo_hash_material(&events));
        assert!(verify_snapshot_manifest(&bundle.manifest));
        let (snapshot, bytes) = assemble(&bundle.manifest, &bundle.chunks);
        let mut rejoined = SelectiveReplicaRuntime::new(1, 0, 0, 1, allow(), BTreeSet::new());
        rejoined.apply_verified_rebase(&snapshot, &bundle.manifest, &bytes).expect("rejoin");
        assert_eq!(rejoined.canonical_team_hash(), fresh.canonical_team_hash());

        let mut contradicted = snapshot.clone();
        contradicted.team_private_metadata = purchased.team_private_metadata.clone();
        let commitment_at = 4 + 8 + 16;
        undo_slot(&mut contradicted.team_private_metadata).value[commitment_at] ^= 0xff;
        let mut untouched = SelectiveReplicaRuntime::new(1, 0, 0, 1, allow(), BTreeSet::new());
        assert!(untouched.apply_verified_rebase(&contradicted, &bundle.manifest, &bytes).is_err());
        assert_eq!(untouched.world().tick, 0);
        assert_ne!(untouched.canonical_team_hash(), fresh.canonical_team_hash());
        let mut truncated_echo = snapshot.clone();
        truncated_echo.team_private_metadata = purchased.team_private_metadata.clone();
        undo_slot(&mut truncated_echo.team_private_metadata).value.pop();
        assert!(untouched.apply_verified_rebase(&truncated_echo, &bundle.manifest, &bytes).is_err());
        assert_eq!(untouched.world().tick, 0);

        let mut bad_manifest = bundle.manifest.clone();
        bad_manifest.retained_owner_undo_material.push(0);
        bad_manifest.manifest_hash = crate::runtime::manifest_hash(&bad_manifest).to_vec();
        assert!(!verify_snapshot_manifest(&bad_manifest));
        assert!(untouched.apply_verified_rebase(&snapshot, &bad_manifest, &bytes).is_err());

        let mut truncated = purchased.clone();
        undo_slot(&mut truncated.team_private_metadata).value.pop();
        undo_slot(&mut truncated.filtered_snapshot.as_mut().unwrap().team_private_metadata).value.pop();
        assert!(bootstrap(&truncated).is_err());
        let mut one_sided = purchased.clone();
        undo_slot(&mut one_sided.team_private_metadata).value.pop();
        assert!(bootstrap(&one_sided).is_err());
        let mut duplicate = purchased.clone();
        let extra = undo_slot(&mut duplicate.team_private_metadata).clone();
        duplicate.team_private_metadata.push(extra.clone());
        duplicate.filtered_snapshot.as_mut().unwrap().team_private_metadata.push(extra);
        assert!(bootstrap(&duplicate).is_err());
        let mut public = purchased.clone();
        let leaked = undo_slot(&mut public.team_private_metadata).clone();
        public.public_metadata.push(leaked);
        assert!(metadata_declares_owner_undo(&public.public_metadata));
        assert!(bootstrap(&public).is_err());

        let retired = projector
            .build_frame(2, 2, &BTreeSet::new(), vec![], &[undo_fact(1, false, UNDO_REASON_RETIRED, 0)], &graph)
            .expect("retired frame");
        assert!(matches!(
            continued.apply_frame(retired.frame, &mut NoopDisclosedWorldStepper).unwrap(),
            FrameApplyResult::Applied { .. }
        ));
        let retired_start = projector.build_team_game_start(3, 120, 9);
        assert!(!retained_owner_undo_from_metadata(&retired_start.team_private_metadata).unwrap().is_empty());
        assert_eq!(continued.canonical_team_hash(), bootstrap(&retired_start).unwrap().canonical_team_hash());

        let cleared_frame = projector
            .build_frame(3, 3, &BTreeSet::new(), vec![], &[], &graph)
            .expect("cleared frame");
        assert!(matches!(
            continued.apply_frame(cleared_frame.frame, &mut NoopDisclosedWorldStepper).unwrap(),
            FrameApplyResult::Applied { .. }
        ));
        let cleared = projector.build_team_game_start(4, 120, 9);
        assert!(retained_owner_undo_from_metadata(&cleared.team_private_metadata).unwrap().is_empty());
        assert!(cleared.filtered_snapshot.as_ref().unwrap().team_private_metadata.iter().all(|entry| entry.namespace != OWNER_UNDO_RETAINED_NAMESPACE));
        let cleared_fresh = bootstrap(&cleared).unwrap();
        assert_eq!(continued.canonical_team_hash(), cleared_fresh.canonical_team_hash());
        let clear_bundle = projector.build_filtered_rebase(4, 4, 1).expect("cleared rebase");
        assert!(clear_bundle.manifest.retained_owner_undo_material.is_empty());
        assert_eq!(crate::runtime::manifest_hash(&clear_bundle.manifest).as_slice(), clear_bundle.manifest.manifest_hash.as_slice());
        let (clear_snapshot, clear_bytes) = assemble(&clear_bundle.manifest, &clear_bundle.chunks);
        let mut rejoined_clear = SelectiveReplicaRuntime::new(1, 0, 0, 1, allow(), BTreeSet::new());
        rejoined_clear.apply_verified_rebase(&clear_snapshot, &clear_bundle.manifest, &clear_bytes).unwrap();
        assert_eq!(rejoined_clear.canonical_team_hash(), cleared_fresh.canonical_team_hash());
    }
}
