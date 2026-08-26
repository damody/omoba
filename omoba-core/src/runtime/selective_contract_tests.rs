use std::collections::{BTreeSet, HashSet};
use std::sync::Arc;

use omoba_sim::{Fixed64, Vec2};
use prost::Message;

use super::*;
use crate::game_proto::{
    AuthorityRevision, BoundedRandomTape, DisclosureEpoch, PostStep, PreStep,
    ReplicaEntityId as WireReplicaEntityId, Step, TeamTickFrame, ViewEpoch,
};
use crate::runtime::native::comp::{RememberDisposition, ReplicationScopeKind};

const TEAM: u32 = 7;
const COMPONENT: u32 = 11;

fn allowlist() -> BTreeSet<u32> { BTreeSet::from([COMPONENT]) }

fn empty_frame(tick: u64, sequence: u64, revision: u64) -> TeamTickFrame {
    TeamTickFrame {
        protocol_version: 2,
        frame_schema_version: 1,
        content_schema_version: 1,
        team_id: TEAM,
        server_tick: tick,
        replica_tick: tick,
        team_sequence: sequence,
        view_epoch: Some(ViewEpoch { value: 1 }),
        authority_revision: Some(AuthorityRevision { value: revision }),
        pre_step: Some(PreStep::default()),
        step: Some(Step::default()),
        post_step: Some(PostStep::default()),
        padding: Vec::new(),
    }
}

fn runtime(tick: u64, sequence: u64) -> SelectiveReplicaRuntime {
    SelectiveReplicaRuntime::new(TEAM, tick, sequence, 1, allowlist(), BTreeSet::new())
}

#[test]
fn protocol_encode_decode_round_trip_is_schema_stable() {
    let frame = empty_frame(42, 9, 3);
    let bytes = encode_v2_player_payload(&frame);
    let decoded = TeamTickFrame::decode(bytes.as_slice()).unwrap();
    assert_eq!(decoded, frame);
    assert_eq!(decoded.protocol_version, 2);
    assert_eq!(decoded.frame_schema_version, 1);
    assert!(ensure_filtered_snapshot_schema(FILTERED_SNAPSHOT_SCHEMA_VERSION).is_ok());
    assert!(ensure_filtered_snapshot_schema(FILTERED_SNAPSHOT_SCHEMA_VERSION + 1).is_err());
}

#[test]
fn projection_policy_catalogue_is_complete_and_unknown_ids_fail_closed() {
    let defaults = ProjectionPolicyRegistry::secure_defaults();
    assert!(validate_secure_match_startup(&defaults).is_ok());
    let empty = ProjectionPolicyRegistry::default();
    assert_eq!(empty.validate_complete().unwrap_err().len(), REQUIRED_PROJECTION_POLICIES.len());
}

#[test]
fn team_identity_is_scoped_monotonic_and_never_reuses_retired_ids() {
    let key = CanonicalEntityKey { id: 4, generation: 2 };
    let mut a = TeamIdentityState::new(TEAM);
    let mut b = TeamIdentityState::new(TEAM + 1);
    let first = a.disclose(key).unwrap();
    assert_eq!(b.disclose(key).unwrap().replica_id.get(), 1);
    a.remember(key).unwrap();
    let revealed = a.disclose(key).unwrap();
    assert_eq!(revealed.replica_id, first.replica_id);
    assert_eq!(revealed.disclosure_epoch, first.disclosure_epoch + 1);
    assert!(matches!(a.canonical_for(first.replica_id, first.disclosure_epoch), Err(TeamIdentityError::StaleDisclosureEpoch)));
    let retired = a.forget(key).unwrap();
    let next = a.disclose(CanonicalEntityKey { id: 5, generation: 2 }).unwrap();
    assert_ne!(retired, next.replica_id);
}

#[test]
fn visibility_resolution_schedules_then_commits_a_fresh_reveal() {
    let entity = CommittedEntityView {
        canonical_id: 99,
        team: 2,
        position: Vec2::new(Fixed64::from_i32(0), Fixed64::from_i32(0)),
        scope: ReplicationScopeKind::Vision,
        owner_team: None,
        stealth_level: 0,
        overrides: Vec::new(),
        remember: RememberDisposition::LastKnown,
        disclosed_baseline: b"fresh".to_vec(),
    };
    let source = CommittedVisionSource {
        canonical_id: 1,
        team: TEAM,
        position: entity.position,
        radius: Fixed64::from_i32(10),
        detection_level: 0,
    };
    let mut state = TeamVisibilityState::new(TEAM, 8);
    let at_10 = WaveBReadView { tick: 10, entities: Arc::from([entity.clone()]), vision_sources: Arc::from([source]) };
    assert!(state.resolve(&at_10, 1).is_empty());
    let at_11 = WaveBReadView { tick: 11, entities: Arc::from([entity]), vision_sources: at_10.vision_sources.clone() };
    assert_eq!(state.resolve(&at_11, 1), vec![VisibilityTransition::Reveal { canonical_id: 99, effective_tick: 11, baseline: b"fresh".to_vec() }]);
    assert!(state.history.was_visible(11, 99));
}

#[test]
fn authority_repair_selection_is_monotonic_and_server_authoritative() {
    let difference = ComponentDifference {
        replica_id: 5,
        disclosure_epoch: 2,
        component_schema_id: COMPONENT,
        safe_component_path: "movement.position".into(),
        field_mask: vec![1],
        replacement_fields: vec![8],
        safe_entity_baseline: encode_component_baseline(&[(COMPONENT, &[8])]),
    };
    let metadata = safe_mismatch_metadata(TEAM, 4, &[1; 32], &[2; 32], 1);
    let mut coordinator = AuthorityRepairCoordinator::configured();
    coordinator.report_component_divergence(TEAM, 4, 4, metadata.clone(), &[difference.clone()]);
    coordinator.report_component_divergence(TEAM, 5, 5, metadata, &[difference]);
    let actions = coordinator.drain_actions(TEAM);
    let revisions: Vec<_> = actions.into_iter().filter_map(|action| match action {
        RecoveryAction::ComponentRepair(value) => value.authority_revision.map(|r| r.value),
        _ => None,
    }).collect();
    assert_eq!(revisions, vec![1, 2]);
}

#[test]
fn bounded_random_tape_accepts_only_current_disclosed_epoch_and_sufficient_values() {
    let mut replica = runtime(0, 0);
    let mut stepper = NoopDisclosedWorldStepper;
    replica.apply_encoded_frame(&single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT), &mut stepper).unwrap();
    let mut valid = empty_frame(1, 1, 2);
    valid.step.as_mut().unwrap().random_tapes.push(BoundedRandomTape {
        tape_id: 1,
        disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
        first_tick: 1,
        tick_count: 1,
        algorithm_id: 1,
        values: vec![123],
        consumer_kind: 1,
        replica_entity_id: Some(WireReplicaEntityId { value: 1 }),
        stable_sub_index: 0,
    });
    replica.apply_frame(valid, &mut stepper).unwrap();

    let mut malformed = empty_frame(2, 2, 3);
    malformed.step.as_mut().unwrap().random_tapes.push(BoundedRandomTape {
        tape_id: 2,
        disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
        first_tick: 2,
        tick_count: 2,
        algorithm_id: 1,
        values: vec![123],
        consumer_kind: 1,
        replica_entity_id: Some(WireReplicaEntityId { value: 1 }),
        stable_sub_index: 0,
    });
    assert_eq!(replica.apply_frame(malformed, &mut stepper), Err(ReplicaRuntimeError::MalformedRandomTape));
}

#[test]
fn canonical_merge_is_independent_of_shard_completion_order() {
    let make = |ordinal| OrderedFact {
        key: FactOrderingKey { tick: 3, phase: FactPhase::Step, canonical_source_order: ordinal, local_ordinal: 0, fact_kind: FactKind::Movement },
        audience: FactAudience::AllPlayers,
        fact: ObservableFact::Movement { source: ordinal, x_mm: ordinal as i64, y_mm: 0 },
    };
    let buffer = ShardedStableBuffer::new(4).unwrap();
    buffer.push(3, make(4)).unwrap();
    buffer.push(0, make(1)).unwrap();
    buffer.push(2, make(3)).unwrap();
    buffer.push(1, make(2)).unwrap();
    let order: Vec<_> = buffer.drain_sorted().unwrap().into_iter().map(|f| f.key.canonical_source_order).collect();
    assert_eq!(order, vec![1, 2, 3, 4]);
}

#[test]
fn malformed_transition_and_rebase_are_rejected() {
    let mut replica = runtime(0, 0);
    let mut stepper = NoopDisclosedWorldStepper;
    let mut reveal = TeamTickFrame::decode(single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT).as_slice()).unwrap();
    if let Some(crate::game_proto::transition::Transition::Reveal(value)) = reveal.pre_step.as_mut().unwrap().transitions[0].transition.as_mut() {
        value.effective_tick = 9;
    }
    assert_eq!(replica.apply_frame(reveal, &mut stepper), Err(ReplicaRuntimeError::MalformedTransition));

    let disclosed = Vec::new();
    let mut fixture = single_rebase_fixture([9; 16], TEAM, 10, 5, disclosed.clone()).unwrap();
    fixture.manifest.manifest_hash[0] ^= 0xff;
    assert_eq!(replica.apply_verified_rebase(&fixture.filtered_snapshot, &fixture.manifest, &disclosed), Err(ReplicaRuntimeError::UnverifiedRebase));
}

#[test]
fn remembered_presentation_is_excluded_from_simulation_and_hash() {
    let mut with_memory = runtime(0, 0);
    let mut stepper = NoopDisclosedWorldStepper;
    with_memory.apply_encoded_frame(&single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT), &mut stepper).unwrap();
    with_memory.apply_encoded_frame(&single_hide_frame_fixture(TEAM, 1, 1, 1), &mut stepper).unwrap();
    let remembered = with_memory.extract_filtered_render_snapshot();
    assert!(with_memory.world().entities.is_empty());
    assert_eq!(remembered.memory_directives.len(), 1);

    let mut empty = runtime(0, 0);
    empty.apply_frame(empty_frame(0, 0, 1), &mut stepper).unwrap();
    empty.apply_frame(empty_frame(1, 1, 2), &mut stepper).unwrap();
    assert_eq!(with_memory.canonical_team_hash(), empty.canonical_team_hash());
    let ids: HashSet<_> = with_memory.world().entities.keys().copied().collect();
    assert!(!ids.contains(&1));
}
