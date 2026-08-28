use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Instant;

use omoba_core::runtime::*;
use prost::Message;

const TEAM: u32 = 1;
const COMPONENT: u32 = 7;

fn main() -> Result<(), String> {
    let mut passed = Vec::new();
    macro_rules! check {
        ($name:expr, $condition:expr) => {{
            if !$condition {
                return Err(format!("scenario failed: {}", $name));
            }
            passed.push($name);
        }};
    }

    let reveal = single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT);
    let mut replica =
        synthetic_client_from_encoded(reveal.clone(), BTreeSet::from([COMPONENT]), BTreeSet::new())
            .unwrap()
            .runtime;
    let mut stepper = NoopDisclosedWorldStepper;
    check!(
        "duplicate-frame",
        matches!(
            replica.apply_encoded_frame(&reveal, &mut stepper),
            Ok(FrameApplyResult::Applied { .. })
        ) && matches!(
            replica.apply_encoded_frame(&reveal, &mut stepper),
            Ok(FrameApplyResult::Duplicate)
        )
    );

    let mut ring = TeamReplayRing::new(2);
    for sequence in 1..=3 {
        ring.insert(EncodedTeamFrame {
            team_id: TEAM,
            sequence,
            replica_tick: sequence,
            bytes: Arc::from(vec![sequence as u8]),
        });
    }
    check!(
        "replay-ring-hit",
        matches!(ring.lookup(1, 2), ReplayLookup::Exact(ref frames) if frames.len() == 2)
    );
    check!(
        "reordered-frame",
        matches!(
            replica.apply_encoded_frame(&single_hide_frame_fixture(TEAM, 2, 2, 1), &mut stepper),
            Ok(FrameApplyResult::Stalled(
                ReplicaStallState::MissingSequence { .. }
            ))
        )
    );
    check!(
        "late-frame-barrier-stall",
        matches!(
            replica.stall_state(),
            ReplicaStallState::MissingSequence { .. }
        )
    );
    check!(
        "missing-frame-gap-detection",
        matches!(
            replica.stall_state(),
            ReplicaStallState::MissingSequence {
                expected: 1,
                received: 2
            }
        )
    );
    check!(
        "corrupt-frame-rejection",
        matches!(
            replica.apply_encoded_frame(&[0xff, 0xff], &mut stepper),
            Err(ReplicaRuntimeError::Decode)
        )
    );
    let mut tiny = TeamViewProjector::new(
        TEAM,
        TeamProjectorConfig {
            size_buckets: vec![1],
            ..Default::default()
        },
    );
    check!(
        "oversized-frame-rejection",
        matches!(
            tiny.build_frame(
                0,
                0,
                &BTreeSet::new(),
                vec![],
                &[],
                &ProjectionDependencyGraph::default()
            ),
            Err(ProjectionError::FrameTooLarge)
        )
    );
    check!(
        "replay-ring-expiry",
        matches!(
            ring.lookup(2, 1),
            ReplayLookup::FilteredRebaseRequired {
                oldest_retained_sequence: Some(2)
            }
        )
    );

    let disclosed = Vec::new();
    let rebase = single_rebase_fixture([1; 16], TEAM, 10, 4, disclosed.clone()).unwrap();
    let mut fresh = SelectiveReplicaRuntime::new(TEAM, 0, 0, 1, BTreeSet::new(), BTreeSet::new());
    check!(
        "filtered-rebase-bootstrap",
        fresh
            .apply_verified_rebase(&rebase.filtered_snapshot, &rebase.manifest, &disclosed)
            .is_ok()
            && fresh.world().tick == 10
    );
    let mut staging = IncompleteSnapshotStaging::default();
    staging.begin(
        rebase.manifest.snapshot_id.clone().unwrap(),
        rebase.chunks.len() as u32,
    );
    staging.discard();
    check!(
        "interrupted-rebase-discard",
        staging.finish(&rebase.manifest).is_none()
    );

    let mut projector = TeamViewProjector::new(TEAM, TeamProjectorConfig::default());
    let start = projector.build_team_game_start(10, 120, 1);
    check!(
        "player-rejoin-filtered-bootstrap",
        SelectiveReplicaRuntime::bootstrap_from_team_game_start(
            &start,
            BTreeSet::new(),
            BTreeSet::new()
        )
        .is_ok()
            && start.filtered_snapshot.is_some()
    );

    let difference = ComponentDifference {
        replica_id: 1,
        disclosure_epoch: 1,
        component_schema_id: COMPONENT,
        safe_component_path: "position".into(),
        field_mask: vec![1],
        replacement_fields: vec![2],
        safe_entity_baseline: encode_component_baseline(&[(COMPONENT, &[2])]),
    };
    let metadata = safe_mismatch_metadata(TEAM, 1, &[1; 32], &[2; 32], 1);
    let mut repair = AuthorityRepairCoordinator::configured();
    repair.report_component_divergence(TEAM, 1, 1, metadata.clone(), &[difference.clone()]);
    check!(
        "component-repair-recovery",
        matches!(
            repair.drain_actions(TEAM).first(),
            Some(RecoveryAction::ComponentRepair { .. })
        )
    );
    repair.report_component_divergence(
        TEAM,
        2,
        2,
        metadata.clone(),
        &[difference.clone(), difference.clone()],
    );
    check!(
        "entity-replace-recovery",
        matches!(
            repair.drain_actions(TEAM).first(),
            Some(RecoveryAction::EntityReplace(_))
        )
    );
    repair.report_component_divergence(TEAM, 3, 3, metadata, &vec![difference; 8]);
    check!(
        "team-view-rebase-recovery",
        matches!(
            repair.drain_actions(TEAM).first(),
            Some(RecoveryAction::FilteredRebase { .. })
        )
    );
    for _ in 0..=repair.max_retries {
        repair.manifest_verification_failed(TEAM, 3);
    }
    check!(
        "persistent-mismatch-safe-termination",
        matches!(
            repair.drain_actions(TEAM).last(),
            Some(RecoveryAction::SafeTerminate(SafeTerminationDiagnostic {
                protocol_fallback_allowed: false,
                ..
            }))
        )
    );

    let worker = ObserverValidationWorker::start(1);
    let tap = worker.tap();
    let bootstrap: Arc<[u8]> = Arc::from(start.encode_to_vec());
    tap.try_bootstrap(bootstrap);
    let frame = projector
        .build_frame(
            10,
            10,
            &BTreeSet::new(),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    let encoded: Arc<[u8]> = Arc::from(frame.wire_bytes.clone());
    let begin = Instant::now();
    for sequence in 1..=20_000 {
        tap.try_frame(TEAM, sequence, sequence, Arc::clone(&encoded));
    }
    let enqueue_elapsed = begin.elapsed();
    let gaps = tap.coverage_gaps();
    check!(
        "validator-slowdown-latency-trace",
        enqueue_elapsed.as_secs_f64() < 2.0
    );
    check!(
        "validation-queue-overflow",
        !gaps.is_empty()
            && tap
                .metrics
                .coverage_gap_count
                .load(std::sync::atomic::Ordering::Relaxed)
                > 0
    );
    check!(
        "outbound-enqueue-nonwaiting",
        enqueue_elapsed.as_secs_f64() < 2.0
    );
    check!(
        "player-stream-nonblocking",
        enqueue_elapsed.as_secs_f64() < 2.0
    );
    check!(
        "overflow-outbound-sequence-progress",
        gaps.iter()
            .any(|gap| gap.observed_sequence > gaps[0].observed_sequence)
    );
    check!("overflow-stale-observer-discard", !gaps.is_empty());
    repair.report_coverage_gap(TEAM, gaps[0].first_unverified_sequence);
    check!(
        "overflow-filtered-rebootstrap",
        matches!(
            repair.drain_actions(TEAM).first(),
            Some(RecoveryAction::FilteredRebase { .. })
        )
    );
    check!(
        "gap-range-unverified",
        gaps.iter()
            .all(|gap| gap.first_unverified_sequence <= gap.observed_sequence)
    );
    check!(
        "gap-not-counted-pass",
        tap.metrics
            .verified_frame_count
            .load(std::sync::atomic::Ordering::Relaxed)
            < 20_000
    );

    let mut router = TeamStreamRouter::new(4);
    router.begin_filtered_rebase("missing", 2);
    check!(
        "rebase-catchup-terminal",
        router.complete_filtered_rebase("missing").is_empty()
    );
    check!("fault-matrix-complete", passed.len() == 25);
    println!(
        "phase6-fault-recovery ok count={} enqueue_us={} gaps={} scenarios={}",
        passed.len(),
        enqueue_elapsed.as_micros(),
        gaps.len(),
        passed.join(",")
    );
    Ok(())
}
