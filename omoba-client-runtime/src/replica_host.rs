use omoba_core::{
    comp::replica_stage::{ReplicaStageDurations, ReplicaStagePhase},
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
    pub encoded_frame_hash: [u8; 32],
}

#[cfg(test)]
mod map_contract_tests {
    use omoba_core::comp::replica_stage::{ReplicaStageDurations, ReplicaStagePhase};

    use super::*;

    #[test]
    fn bootstrap_rejects_geometry_mismatch_before_loading_scripts() {
        let map = omoba_template_ids::moba_map_by_name("three_lane_training").unwrap();
        let mut metadata = omoba_core::runtime::moba_map_layout::bootstrap_metadata(map);
        metadata[1].value.clear();
        let result = ReplicaHost::bootstrap(&TeamGameStart {
            public_metadata: metadata,
            ..Default::default()
        });
        assert!(matches!(result, Err(ClientRuntimeError::Replica(error))
            if error == "compiled MOBA terrain mismatch"));
    }

    #[test]
    fn replica_stage_host_finalize_keeps_only_an_applied_profile() {
        let mut stale = ReplicaStageDurations::default();
        stale.set_phase(ReplicaStagePhase::Decode, 12);
        stale.set_phase(ReplicaStagePhase::FixedStep, 40);
        assert!(super::commit_stage_profile(true, false, Some(stale), 9).is_none());
        assert!(super::commit_stage_profile(false, true, Some(stale), 9).is_none());
        let kept = super::commit_stage_profile(true, true, Some(stale), 7).unwrap();
        assert_eq!(kept.phase_ns(ReplicaStagePhase::Decode), 12);
        assert_eq!(kept.phase_ns(ReplicaStagePhase::FixedStep), 40);
        assert_eq!(kept.phase_ns(ReplicaStagePhase::HostFinalize), 7);
        assert_eq!(kept.phase_ns(ReplicaStagePhase::Staging), 0);
    }

    #[test]
    fn replica_host_coalesces_owner_ward_recovery_after_a_rejected_envelope() {
        use omoba_core::game_proto::{FilteredTeamSnapshot, TeamGameStart, ViewEpoch};
        use omoba_core::runtime::ward_transport::{
            encode_owner_ward_projection, hash_owner_ward_projection, owner_ward_session_binding,
            seal_owner_ward_envelope, stamp_owner_ward_admission, OwnerWardProjection,
        };
        use sha2::{Digest, Sha256};
        let row = omoba_template_ids::TRINKET_CATALOG.iter().find(|row| row.catalog_id == 1).unwrap();
        let projection = OwnerWardProjection {
            player_id: 7,
            view_epoch: 1,
            catalog_id: row.catalog_id,
            charges: 1,
            max_charges: row.max_charges,
            recharge_remaining_raw: 0,
            revision: 2,
            placement_capable: true,
            elapsed_raw: 8,
        };
        let bytes = encode_owner_ward_projection(&projection).unwrap();
        let disclosed_world = Vec::new();
        let mut start = TeamGameStart {
            protocol_version: omoba_core::transport::SELECTIVE_LOCKSTEP_PROTOCOL_VERSION,
            player_id: 7,
            team_id: 1,
            replica_start_tick: 1,
            next_team_sequence: 1,
            tick_rate_hz: 20,
            view_epoch: Some(ViewEpoch { value: 1 }),
            filtered_snapshot: Some(FilteredTeamSnapshot {
                team_id: 1,
                authoritative_tick: 1,
                view_epoch: Some(ViewEpoch { value: 1 }),
                disclosed_world: disclosed_world.clone(),
                filtered_snapshot_hash: Sha256::digest(&disclosed_world).to_vec(),
                ..Default::default()
            }),
            ..Default::default()
        };
        let session = owner_ward_session_binding("host-recovery", 7);
        stamp_owner_ward_admission(&mut start, true, session, &bytes);
        assert_eq!(
            start.owner_ward_envelope.as_ref().unwrap().projection_hash.as_slice(),
            hash_owner_ward_projection(&bytes),
        );
        let mut host = ReplicaHost::bootstrap_disclosed(&start).unwrap();
        assert!(host.owner_ward_recovery_enabled());
        assert_eq!(host.runtime.owner_ward_projection(), bytes.as_slice());
        let mut poisoned = start.owner_ward_envelope.clone().unwrap();
        poisoned.authoritative_tick = u64::MAX;
        poisoned.projection_hash[0] ^= 0xff;
        assert!(host.apply_owner_ward_envelope(&poisoned).is_err());
        assert!(host.runtime.owner_ward_projection().is_empty());
        assert_eq!(host.runtime.owner_ward_tick(), 1);
        assert_eq!(host.poll_owner_ward_recovery(), Some((1, 1)));
        assert_eq!(host.poll_owner_ward_recovery(), Some((2, 1)));
        assert_eq!(host.poll_owner_ward_recovery(), None);
        let recovered = seal_owner_ward_envelope(7, session, 1, 1, &bytes);
        host.apply_owner_ward_envelope(&recovered).unwrap();
        assert_eq!(host.runtime.owner_ward_projection(), bytes.as_slice());
        assert_eq!(host.poll_owner_ward_recovery(), None);
    }
}

pub struct ReplicaHost {
    team_id: u32,
    tick_rate_hz: u32,
    public_metadata: Vec<omoba_core::game_proto::DeterministicMetadata>,
    runtime: SelectiveReplicaRuntime,
    stepper: SpecsDisclosedWorldStepper,
    staging: IncompleteSnapshotStaging,
    expected_team_sequence: u64,
    fog: omoba_core::runtime::fog_grid::FogGridRetention,
    stage_profile: Option<ReplicaStageDurations>,
    owner_ward_recovery: bool,
    owner_ward_recovery_budget: omoba_core::runtime::ward_transport::OwnerWardRecoveryBudget,
}

pub(crate) fn commit_stage_profile(
    measured: bool,
    applied: bool,
    mut core: Option<ReplicaStageDurations>,
    host_finalize_ns: u128,
) -> Option<ReplicaStageDurations> {
    if !measured || !applied {
        return None;
    }
    if let Some(durations) = core.as_mut() {
        durations.set_phase(ReplicaStagePhase::HostFinalize, host_finalize_ns);
    }
    core
}

impl ReplicaHost {
    pub fn bootstrap(start: &TeamGameStart) -> Result<Self, ClientRuntimeError> {
        Self::open(start, true)
    }

    /// Same production open as [`Self::bootstrap`]. Tests skip only the script
    /// registry requirement so a disclosed roster can be read without
    /// `base_content.dll`.
    #[cfg(test)]
    pub(crate) fn bootstrap_disclosed(start: &TeamGameStart) -> Result<Self, ClientRuntimeError> {
        Self::open(start, false)
    }

    fn open(start: &TeamGameStart, require_scripts: bool) -> Result<Self, ClientRuntimeError> {
        omoba_core::runtime::moba_map_layout::validate_metadata(&start.public_metadata)
            .map_err(|error| ClientRuntimeError::Replica(error.into()))?;
        let fog = omoba_core::runtime::fog_grid::FogGridRetention::bootstrap(start)
            .map_err(|error| ClientRuntimeError::Replica(error.into()))?;
        let components = secure_replica_component_allowlist();
        let resources = secure_replica_resource_allowlist();
        let runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            start,
            components.clone(),
            resources.clone(),
        )
        .map_err(|error| ClientRuntimeError::Replica(format!("bootstrap: {error:?}")))?;
        let mut stepper = SpecsDisclosedWorldStepper::from_start(start, components, resources);
        if require_scripts && stepper.script_registry.is_empty() {
            return Err(ClientRuntimeError::Replica(
                "no gameplay scripts loaded; set OMB_SCRIPTS_DIR to the directory containing base_content".into(),
            ));
        }
        stepper
            .bootstrap_membership(runtime.world())
            .map_err(|error| ClientRuntimeError::Replica(format!("Specs bootstrap: {error:?}")))?;
        Ok(Self {
            team_id: start.team_id,
            tick_rate_hz: start.tick_rate_hz.max(1),
            public_metadata: start.public_metadata.clone(),
            runtime,
            stepper,
            staging: IncompleteSnapshotStaging::default(),
            expected_team_sequence: start.next_team_sequence,
            fog,
            stage_profile: None,
            owner_ward_recovery: omoba_core::runtime::ward_transport::negotiate_ward_protocol(
                start.ward_protocol_version,
                &start.ward_rules_hash,
            )
            .unwrap_or(false),
            owner_ward_recovery_budget: omoba_core::runtime::ward_transport::OwnerWardRecoveryBudget::default(),
        })
    }

    pub fn owner_ward_recovery_enabled(&self) -> bool {
        self.owner_ward_recovery
    }

    pub fn owner_ward_protocol(&self) -> u32 {
        if self.owner_ward_recovery {
            omoba_core::runtime::ward_transport::WARD_PROTOCOL_VERSION
        } else {
            0
        }
    }

    /// Verified private bytes currently held by the replica. Empty means unavailable.
    pub fn verified_owner_ward_bytes(&self) -> &[u8] {
        self.runtime.owner_ward_projection()
    }

    pub fn apply_owner_ward_envelope(
        &mut self,
        envelope: &omoba_core::game_proto::OwnerWardEnvelope,
    ) -> Result<(), &'static str> {
        let result = self.runtime.apply_verified_owner_ward_envelope(envelope);
        if result.is_ok() {
            self.owner_ward_recovery_budget.reset();
        }
        result
    }

    /// View epoch for one coalesced read-only recovery query, when the replica
    /// still needs one and this epoch has attempts left.
    pub fn poll_owner_ward_recovery(&mut self) -> Option<(u64, u64)> {
        let epoch = self.owner_ward_recovery_budget.poll(
            self.owner_ward_recovery,
            self.runtime.owner_ward_needs_recovery(),
            self.runtime.view_epoch(),
        )?;
        Some((self.owner_ward_recovery_budget.request_id(), epoch))
    }

    pub fn apply_encoded_frame(
        &mut self,
        encoded: &[u8],
        authority_revision: u64,
    ) -> Result<Option<ReplicaApplyReport>, ClientRuntimeError> {
        self.apply_encoded_frame_inner(encoded, authority_revision, false)
    }

    pub fn apply_encoded_frame_profiled(
        &mut self,
        encoded: &[u8],
        authority_revision: u64,
    ) -> Result<Option<ReplicaApplyReport>, ClientRuntimeError> {
        self.apply_encoded_frame_inner(encoded, authority_revision, true)
    }

    pub fn take_stage_profile(&mut self) -> Option<ReplicaStageDurations> {
        self.stage_profile.take()
    }

    fn apply_encoded_frame_inner(
        &mut self,
        encoded: &[u8],
        authority_revision: u64,
        measure: bool,
    ) -> Result<Option<ReplicaApplyReport>, ClientRuntimeError> {
        self.stage_profile = None;
        let outcome = if measure {
            self.runtime
                .apply_encoded_frame_profiled(encoded, &mut self.stepper)
        } else {
            self.runtime.apply_encoded_frame(encoded, &mut self.stepper)
        };
        match outcome {
            Ok(FrameApplyResult::Applied {
                replica_tick,
                team_sequence,
                pre_repair_observed_hash,
                post_repair_hash,
                ..
            }) => {
                let host_started = measure.then(std::time::Instant::now);
                let epoch = self.runtime.view_epoch();
                if self.fog.latest().is_some_and(|grid| grid.view_epoch != epoch) { self.fog.clear(); }
                if let Err(error) = self.fog.ingest_events(self.runtime.applied_public_events(), self.team_id, epoch, replica_tick) {
                    self.fog.clear();
                    log::warn!("fog presentation rejected: {error}");
                }
                let report = ReplicaApplyReport {
                    replica_tick,
                    team_sequence,
                    authority_revision,
                    pre_repair_hash: pre_repair_observed_hash,
                    post_repair_hash,
                    encoded_frame_hash: <sha2::Sha256 as sha2::Digest>::digest(encoded).into(),
                };
                self.expected_team_sequence = team_sequence.saturating_add(1);
                let host_finalize_ns = host_started.map(|started| started.elapsed().as_nanos()).unwrap_or(0);
                self.stage_profile = commit_stage_profile(
                    measure,
                    true,
                    self.runtime.take_stage_profile(),
                    host_finalize_ns,
                );
                Ok(Some(report))
            }
            Ok(FrameApplyResult::Duplicate) => {
                self.stage_profile = commit_stage_profile(
                    measure,
                    false,
                    self.runtime.take_stage_profile(),
                    0,
                );
                Ok(None)
            }
            Ok(FrameApplyResult::Stalled(state)) => {
                self.stage_profile = commit_stage_profile(
                    measure,
                    false,
                    self.runtime.take_stage_profile(),
                    0,
                );
                Err(ClientRuntimeError::Replica(format!(
                    "frame barrier stalled: {state:?}"
                )))
            }
            Err(error) => {
                self.stage_profile = commit_stage_profile(
                    measure,
                    false,
                    self.runtime.take_stage_profile(),
                    0,
                );
                let fault = self.runtime.last_apply_fault().cloned();
                Err(ClientRuntimeError::Replica(format!(
                    "frame rejected: {error:?} fault={fault:?}"
                )))
            }
        }
    }

    pub fn extract_presentation_source(&mut self) -> FilteredRenderSnapshot {
        let mut source = self.runtime.extract_filtered_render_snapshot();
        let epoch = self.runtime.view_epoch();
        if self.fog.latest().is_some_and(|grid| grid.view_epoch != epoch) { self.fog.clear(); }
        source.public_events.retain(|event| event.event_kind != omoba_core::runtime::fog_grid::FOG_GRID_EVENT_KIND);
        if let Some(grid) = self.fog.latest() {
            source.public_events.push(grid.presentation_event().expect("validated retained fog"));
        }
        source.disclosed_summoners = self.stepper.disclosed_summoners();
        source.hero_attributes = self.stepper.disclosed_hero_attributes();
        source
    }

    pub fn take_damage_presentation(&mut self, tick: u64) -> Vec<omoba_core::game_proto::PresentationEffect> {
        use omoba_core::runtime::presentation_cue::{DamagePresentationCue, presentation_effect_id};
        self.runtime.take_damage_presentation_effects().into_iter().filter_map(|effect| {
            let target_id = effect.visible_target?.value;
            let target = self.runtime.world().entities.get(&target_id)?;
            let amount_milli = i64::from_le_bytes(effect.sanitized_payload.get(..8)?.try_into().ok()?);
            if amount_milli <= 0 || target.disclosure_epoch == 0 { return None; }
            Some(omoba_core::game_proto::PresentationEffect {
                effect_id: presentation_effect_id(tick, effect.stable_sub_index)?,
                safe_payload: DamagePresentationCue {tick, target_id, disclosure_epoch: target.disclosure_epoch, amount_milli}.encode(),
            })
        }).collect()
    }

    pub fn team_id(&self) -> u32 {
        self.team_id
    }

    pub fn ability_presentation(&self, tick: u64) -> Vec<omoba_core::game_proto::PresentationEffect> {
        use omoba_core::runtime::presentation_cue::PresentationCue;
        self.runtime.applied_public_events().iter().filter_map(|event| {
            let (effect_id, cue) = PresentationCue::from_public_event(tick, event, |id| {
                self.runtime.world().entities.get(&id).map(|entity| entity.disclosure_epoch)
            })?;
            Some(omoba_core::game_proto::PresentationEffect {effect_id, safe_payload: cue.encode()})
        }).collect()
    }

    pub fn global_seed(&self) -> u64 {
        self.runtime.global_seed()
    }

    pub fn current_team_hash(&self) -> [u8; 32] {
        self.runtime.canonical_team_hash()
    }

    pub fn disclosed_component_digests(
        &self,
    ) -> Vec<omoba_core::runtime::DisclosedComponentDigest> {
        self.runtime.disclosed_component_digests()
    }

    pub fn view_epoch(&self) -> u64 {
        self.runtime.view_epoch()
    }

    pub fn combat_preferences_for_owner(&self, owner: u32) -> Option<omoba_core::runtime::combat_controls::OwnerCombatPreferences> {
        self.runtime.combat_preferences_for_owner(owner)
    }

    pub fn input_scheme_for_owner(&self, owner: u32) -> Option<omoba_core::runtime::input_scheme_transport::AdmittedOwnerInputScheme> {
        self.runtime.input_scheme_for_owner(owner)
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
        let recovered_fog = omoba_core::runtime::fog_grid::FogGridRetention::from_rebase(manifest)
            .map_err(|error| ClientRuntimeError::Replica(error.into()))?;
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
        let team_private_metadata = if manifest.admitted_summoner_projection.is_empty() {
            Vec::new()
        } else {
            vec![omoba_core::game_proto::DeterministicMetadata {
                namespace: omoba_core::runtime::summoner_transport::SUMMONER_PROJECTION_NAMESPACE.into(),
                key: omoba_core::runtime::summoner_transport::SUMMONER_PROJECTION_KEY.into(),
                schema_version: omoba_core::runtime::summoner_transport::SUMMONER_PROJECTION_SCHEMA,
                value: manifest.admitted_summoner_projection.clone(),
            }]
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
                    tick_rate_hz: self.tick_rate_hz,
                    public_metadata: self.public_metadata.clone(),
                    team_private_metadata,
                    ..TeamGameStart::default()
                },
                secure_replica_component_allowlist(),
                secure_replica_resource_allowlist(),
                self.runtime.world(),
            )
            .map_err(|error| ClientRuntimeError::Replica(format!("Specs rebase: {error:?}")))?;
        self.expected_team_sequence = manifest.resume_team_sequence;
        self.fog = recovered_fog;
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
        let mut matches = self
            .runtime
            .world()
            .entities
            .iter()
            .filter_map(|(id, entity)| {
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
            });
        let render_id=matches.next()?;
        if matches.next().is_some() {return None;}
        self.secure_reference(render_id)
    }

    pub fn attack_relation_legal(&self, source_player_id: u32, render_id: u64) -> bool {
        let rows: Vec<_> = self.runtime.world().entities.iter()
            .map(|(id, entity)| (*id, &entity.components))
            .collect();
        omoba_core::runtime::disclosed_attack_target_admitted(&rows, source_player_id, render_id)
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
        self.runtime.inject_test_only_disclosed_position_fault()
    }
}
