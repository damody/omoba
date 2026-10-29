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
    fn summon_wire_client_constructor_uses_only_current_owned_kind4_disclosure() {
        use omoba_core::runtime::*;use std::collections::BTreeSet;
        let mut projector=TeamViewProjector::new(1,TeamProjectorConfig::default());
        let ids=[(1_u64<<32)|1,(1_u64<<32)|2,(1_u64<<32)|3,(1_u64<<32)|4];
        let visible:BTreeSet<_>=ids.into_iter().collect();
        let transitions=ids.into_iter().zip([(1,7),(4,7),(4,8),(2,7)]).map(|(canonical_id,(kind,owner_player_id))| {
            let bytes=encode_demo_render_state(DemoRenderState{x_raw:0,y_raw:0,team_id:1,kind,owner_player_id});
            VisibilityTransition::Reveal{canonical_id,effective_tick:1,baseline:encode_component_baseline(&[
                (DEMO_RENDER_COMPONENT_SCHEMA_ID,bytes.as_slice())])}
        }).collect();
        projector.build_frame_with_inputs(1,1,&visible,transitions,&[],&ProjectionDependencyGraph::default(),vec![]).unwrap();
        let mut start=projector.build_team_game_start(1,60,0);start.player_id=7;
        let host=ReplicaHost::bootstrap_disclosed(&start).unwrap();
        let rows:Vec<_>=host.runtime.world().entities.iter().map(|(id,e)|(*id,decode_demo_render_state(&e.components[&DEMO_RENDER_COMPONENT_SCHEMA_ID]).unwrap())).collect();
        let summon=rows.iter().find(|(_,r)|r.kind==4&&r.owner_player_id==7).unwrap().0;
        for operation in [1,2] {let input=host.owned_summon_command(7,summon,operation,(operation==1).then_some(omoba_core::game_proto::Vec2I{x:100,y:0})).unwrap();
            let intent=omoba_core::renderer_protocol::player_input_to_renderer_intent(&input).unwrap();
            let renderer=omoba_core::game_proto::RendererInput{request_id:91,player_id:7,disclosure_epoch:host.view_epoch(),intent:Some(intent)};
            let mut gateway=crate::input_bridge::InputBridge::resume_after(0);
            assert!(matches!(gateway.validate(renderer.clone(),7,&host),crate::input_bridge::InputDecision::Rejected{..}));
            assert_eq!(gateway.reserved_input_id(),0);gateway.set_commands_enabled(true);
            let crate::input_bridge::InputDecision::Accepted{input:admitted,..}=gateway.validate(renderer.clone(),7,&host) else {panic!("current summon renderer command refused");};
            assert_eq!(admitted,input);
            let mut stale=renderer;
            if let Some(omoba_core::game_proto::renderer_input::Intent::SummonCommand(value))=stale.intent.as_mut(){value.summon.as_mut().unwrap().disclosure_epoch.as_mut().unwrap().value+=1;}
            assert!(matches!(gateway.validate(stale,7,&host),crate::input_bridge::InputDecision::Rejected{..}));assert_eq!(gateway.reserved_input_id(),1);
            let Some(omoba_core::game_proto::player_input::Action::SummonCommand(value))=input.action else {panic!("wrong client action");};
            summon_transport::validate_client(&value).unwrap();assert_eq!(value.owner.unwrap(),host.owned_hero_reference(7).unwrap());}
        for (id,row) in rows {if id!=summon {assert!(host.owned_summon_command(7,id,2,None).is_none(),"kind{} owner{}",row.kind,row.owner_player_id);}}
        assert!(host.owned_summon_command(0,summon,2,None).is_none());
        assert!(host.owned_summon_command(8,summon,2,None).is_none());
        assert!(host.owned_summon_command(7,u64::MAX,2,None).is_none());
        assert!(host.owned_summon_command(7,summon,2,Some(Default::default())).is_none());
    }

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
    /// Runtime facts of the latest applied frame (runtime_fact_state), replayed until the next.
    runtime_facts: Vec<omoba_core::game_proto::TeamPublicEvent>,
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
            runtime_facts: Vec::new(),
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
        source.public_events.retain(|event| event.event_kind != crate::presentation_bridge::FALLEN_TREES_EVENT_KIND);
        if let Some(fallen) = self.stepper.fallen_trees() {
            source.public_events.push(crate::presentation_bridge::fallen_trees_event(&fallen));
        }
        // Runtime facts and the night metric come with every applied MOBA frame; between frames
        // the latest ones stay (runtime_fact_state).
        if source.public_events.iter().any(crate::presentation_bridge::is_night_metric) {
            let facts: Vec<_> = source.public_events.iter()
                .filter(|event| crate::presentation_bridge::is_runtime_fact(event)).cloned().collect();
            if facts.len() != self.runtime_facts.len() {
                log::info!("runtime facts: {} event(s) (night metric included)", facts.len());
            }
            self.runtime_facts = facts;
        } else {
            source.public_events.retain(|event| !crate::presentation_bridge::is_runtime_fact(event));
            source.public_events.extend(self.runtime_facts.iter().cloned());
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

    /// Own inputs the replica rejected under the authority's rules since the
    /// last call, as (input id, code). Drained on read.
    pub fn take_owner_command_rejections(&mut self, player_id: u32) -> Vec<(u32, &'static str)> {
        self.stepper.take_owner_command_rejections(player_id)
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
        // Replica ids of the old view mean nothing after a rebase.
        self.runtime_facts.clear();
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

    /// Construct only from this committed disclosure. Never accept canonical
    /// entity IDs, another player's summon or a same-team creep as a source.
    pub fn owned_summon_command(&self,player_id:u32,render_id:u64,operation:u32,point:Option<omoba_core::game_proto::Vec2I>)
        ->Option<omoba_core::game_proto::PlayerInput> {
        if player_id==0 {return None;}
        let entity=self.runtime.world().entities.get(&render_id)?;
        let render=entity.components.get(&omoba_core::runtime::DEMO_RENDER_COMPONENT_SCHEMA_ID)
            .and_then(|bytes|omoba_core::runtime::decode_demo_render_state(bytes))?;
        if render.kind!=4 || render.team_id!=self.team_id || render.owner_player_id!=player_id {return None;}
        let value=omoba_core::runtime::summon_transport::client_from_references(
            self.owned_hero_reference(player_id)?,self.secure_reference(render_id)?,operation,point).ok()?;
        Some(omoba_core::game_proto::PlayerInput {action:Some(omoba_core::game_proto::player_input::Action::SummonCommand(value))})
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
