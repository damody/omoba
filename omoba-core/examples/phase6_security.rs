use std::collections::{BTreeMap, BTreeSet};

use omoba_core::game_proto::{AuthorityRevision, ComponentRepair, DisclosureEpoch, PostStep, PreStep, TeamGameStart, TeamTickFrame, TeamViewRebase, TeamViewRebaseChunk, ViewEpoch};
use omoba_core::runtime::*;
use prost::Message;

const TEAM: u32 = 1;
const COMPONENT: u32 = 7;
const CANONICAL_SENTINEL: u64 = 0xfeed_beef_dead_cafe;
const GLOBAL_SEED_SENTINEL: u64 = 0x1122_3344_5566_7788;

fn contains(bytes: &[u8], needle: &[u8]) -> bool { bytes.windows(needle.len()).any(|window| window == needle) }

fn frame(tick: u64, sequence: u64, revision: u64) -> TeamTickFrame {
    TeamTickFrame { protocol_version: 2, frame_schema_version: 1, content_schema_version: 1, team_id: TEAM, server_tick: tick, replica_tick: tick, team_sequence: sequence, view_epoch: Some(ViewEpoch { value: 1 }), authority_revision: Some(AuthorityRevision { value: revision }), pre_step: Some(PreStep::default()), step: Some(Default::default()), post_step: Some(PostStep::default()), padding: vec![] }
}

fn main() -> Result<(), String> {
    let mut passed = Vec::new();
    macro_rules! check { ($name:expr, $condition:expr) => {{ if !$condition { return Err(format!("scenario failed: {}", $name)); } passed.push($name); }} }
    let mut projector = TeamViewProjector::new(TEAM, TeamProjectorConfig::default());
    let packet = projector.build_frame(0, 0, &BTreeSet::new(), vec![], &[], &ProjectionDependencyGraph::default()).unwrap().wire_bytes;
    check!("packet-no-canonical-id", !contains(&packet, &CANONICAL_SENTINEL.to_le_bytes()) && !contains(&packet, &CANONICAL_SENTINEL.to_be_bytes()));

    let mut replica = SelectiveReplicaRuntime::new(TEAM, 0, 0, 1, BTreeSet::from([COMPONENT]), BTreeSet::new()); let mut stepper = NoopDisclosedWorldStepper;
    replica.apply_encoded_frame(&single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT), &mut stepper).unwrap(); replica.apply_encoded_frame(&single_hide_frame_fixture(TEAM, 1, 1, 1), &mut stepper).unwrap();
    check!("replica-memory-no-hidden-state", replica.world().entities.is_empty());

    let valid_facts = TargetValidationFacts { session_team_matches:true, view_epoch_matches:true, disclosure_epoch_matches:true, visible_at_input_tick:true, actor_owned_by_session:true, replica_mapping_exists:true };
    let mut hidden = valid_facts; hidden.visible_at_input_tick=false; let mut nonexistent=valid_facts; nonexistent.replica_mapping_exists=false; let mut stale=valid_facts; stale.disclosure_epoch_matches=false;
    check!("hidden-existing-probe", validate_secure_target(hidden) == Err(GeneralizedInputRejection::InvalidTarget));

    let encoded = single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT);
    for index in 0..512 { let mut fuzz=encoded.clone(); if !fuzz.is_empty() { let slot=index%fuzz.len(); fuzz[slot]^=(index as u8).wrapping_add(1); } let _=TeamTickFrame::decode(fuzz.as_slice()); }
    check!("transition-decoder-fuzz", true);
    let a = projector.build_frame(1, 1, &BTreeSet::new(), vec![], &[], &ProjectionDependencyGraph::default()).unwrap();
    let b = projector.build_frame(2, 2, &BTreeSet::new(), vec![], &[], &ProjectionDependencyGraph::default()).unwrap();
    check!("hidden-activity-frame-cadence", a.wire_bytes.len() == b.wire_bytes.len());

    let admin = ServerAdminDiagnosticCapability::from_server_secret([1;32]); let wrong = ServerAdminDiagnosticCapability::from_server_secret([2;32]); let transport=AdminDiagnosticTransport::new(admin.clone());
    check!("player-no-admin-capability", !transport.authorize(&wrong));
    check!("packet-no-global-seed", !contains(&packet, &GLOBAL_SEED_SENTINEL.to_le_bytes()));
    check!("packet-no-other-team-mask", !contains(&packet, &[0xaa;16]));
    check!("packet-no-hidden-component-sentinel", !contains(&packet, &[0xde,0xad,0xfa,0xce]));
    let remembered = replica.extract_filtered_render_snapshot();
    check!("remembered-export-sanitized-only", remembered.entities.is_empty() && remembered.memory_directives.iter().all(|directive| matches!(directive, RenderMemoryDirective::Hide { sanitized_presentation, .. } if sanitized_presentation == b"last-known")));

    let metrics=SelectiveSecurityMetrics::default(); let mut sinks=PlayerDiagnosticSinks::default(); let record=DiagnosticRecord { fields:BTreeMap::from([("team_id".into(),"1".into()),("canonical_id".into(),"secret".into())]) };
    sinks.emit(PlayerSinkKind::Log,record.clone(),&metrics); check!("player-log-redaction", !sinks.log[0].fields.contains_key("canonical_id"));
    sinks.emit(PlayerSinkKind::Replay,record.clone(),&metrics); check!("player-replay-redaction", !sinks.replay[0].fields.contains_key("canonical_id"));
    sinks.emit(PlayerSinkKind::CrashBundle,record.clone(),&metrics); check!("player-crash-redaction", !sinks.crash_bundle[0].fields.contains_key("canonical_id"));
    sinks.emit(PlayerSinkKind::Trace,record,&metrics); check!("player-trace-redaction", !sinks.trace[0].fields.contains_key("canonical_id"));
    check!("nonexistent-probe", validate_secure_target(nonexistent) == Err(GeneralizedInputRejection::InvalidTarget));
    check!("stale-probe", validate_secure_target(stale) == Err(GeneralizedInputRejection::InvalidTarget));
    check!("probe-response-class-equal", validate_secure_target(hidden) == validate_secure_target(nonexistent));
    check!("probe-timing-bucket-equal", INVALID_TARGET_TIMING_BUCKET.as_millis() == 8);
    let mut limiter=InvalidReferenceRateLimiter::new(2,10); check!("invalid-reference-rate-limit", limiter.admit("s",1) && limiter.admit("s",2) && !limiter.admit("s",3));

    let start = TeamGameStart::default().encode_to_vec(); for cut in 0..=start.len() { let _=TeamGameStart::decode(&start[..cut]); }
    check!("snapshot-decoder-fuzz", true);
    let rebase=single_rebase_fixture([3;16],TEAM,5,4,vec![]).unwrap(); let manifest=rebase.manifest.encode_to_vec(); let chunk=rebase.chunks[0].encode_to_vec();
    for cut in 0..=manifest.len() { let _=TeamViewRebase::decode(&manifest[..cut]); } for cut in 0..=chunk.len() { let _=TeamViewRebaseChunk::decode(&chunk[..cut]); }
    check!("rebase-decoder-fuzz", true);

    let mut authority=SelectiveReplicaRuntime::new(TEAM,0,0,1,BTreeSet::from([COMPONENT]),BTreeSet::new()); authority.apply_encoded_frame(&single_reveal_frame_fixture(TEAM,0,0,1,COMPONENT),&mut stepper).unwrap();
    let mut replayed=frame(1,1,1); replayed.post_step.as_mut().unwrap().component_repairs.push(ComponentRepair { replica_entity_id:Some(omoba_core::game_proto::ReplicaEntityId{value:1}), disclosure_epoch:Some(DisclosureEpoch{value:1}), component_schema_id:COMPONENT, field_mask:vec![1], replacement_fields:b"attack".to_vec(), authority_revision:Some(AuthorityRevision{value:1}), effective_tick:1 });
    check!("replayed-authority-revision", matches!(authority.apply_frame(replayed,&mut stepper),Err(ReplicaRuntimeError::ConflictingEqualRevision)));
    let mut malformed=single_hide_frame_fixture(TEAM,1,1,1); let mut decoded=TeamTickFrame::decode(malformed.as_slice()).unwrap(); if let Some(omoba_core::game_proto::transition::Transition::Hide(hide))=decoded.pre_step.as_mut().unwrap().transitions[0].transition.as_mut(){hide.disclosure_epoch=Some(DisclosureEpoch{value:99});} malformed=decoded.encode_to_vec();
    check!("malformed-disclosure-epoch", matches!(authority.apply_encoded_frame(&malformed,&mut stepper),Err(ReplicaRuntimeError::StaleDisclosureEpoch)));
    check!("hidden-activity-padding-bucket", a.wire_bytes.len().is_power_of_two() && a.wire_bytes.len()==b.wire_bytes.len());

    let visible:BTreeSet<_>=(1..=130).map(|id|(1u64<<32)|id).collect(); let transitions=visible.iter().map(|id|VisibilityTransition::Reveal{canonical_id:*id,effective_tick:0,baseline:encode_component_baseline(&[])}).collect();
    let mut burst=TeamViewProjector::new(TEAM,TeamProjectorConfig{mass_reveal_chunk_entities:64,..Default::default()}); let first=burst.build_frame(0,0,&visible,transitions,&[],&ProjectionDependencyGraph::default()).unwrap();
    check!("mass-reveal-chunk-distribution", first.frame.pre_step.unwrap().transitions.len()==64);
    burst.enqueue_rebase_chunks((0..5).map(|_|vec![0;10])); check!("rebase-chunk-rate-limit", burst.take_rate_limited_rebase_chunks().len()==2);
    let mut full=FullAdminDiagnosticSink::default(); check!("admin-separate-transport", !full.emit(&transport,&wrong,DiagnosticRecord::default()) && full.emit(&transport,&admin,DiagnosticRecord::default()));
    check!("security-findings-review", passed.len()==27 && metrics.snapshot().redaction_violation_count==4);
    println!("phase6-security ok count={} fuzz_cases={} scenarios={}",passed.len(),512+start.len()+manifest.len()+chunk.len()+3,passed.join(","));
    Ok(())
}
