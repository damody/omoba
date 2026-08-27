use std::collections::BTreeSet;
use std::sync::Arc;

use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};

fn main() -> Result<(), String> {
    let team_id = 7;
    let tick = 41;
    let canonical_id = (3u64 << 32) | 19;
    let mut projector = TeamViewProjector::new(team_id, TeamProjectorConfig::default());
    let start = projector.build_team_game_start(tick, 120);
    let mut client = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start,
        BTreeSet::new(),
        BTreeSet::new(),
    )
    .map_err(|error| format!("filtered join failed: {error:?}"))?;

    let view = WaveBReadView {
        tick,
        entities: Arc::from([CommittedEntityView {
            canonical_id,
            team: team_id,
            position: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            scope: ReplicationScopeKind::Public,
            owner_team: Some(team_id),
            stealth_level: 0,
            overrides: Vec::new(),
            remember: RememberDisposition::Forget,
            disclosed_baseline: 0u32.to_be_bytes().to_vec(),
        }]),
        vision_occluders: Arc::from([]),
        vision_sources: Arc::from([]),
    };
    let mut visibility = TeamVisibilityState::new(team_id, 16);
    let transitions = visibility.resolve(&view, 0);
    let frame = projector
        .build_frame(
            tick,
            tick,
            &visibility.index.current,
            transitions,
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("team frame projection failed: {error:?}"))?;
    let result = client
        .apply_encoded_frame(&frame.wire_bytes, &mut NoopDisclosedWorldStepper)
        .map_err(|error| format!("team frame receive failed: {error:?}"))?;
    if !matches!(result, FrameApplyResult::Applied { .. }) || client.world().entities.len() != 1 {
        return Err("synthetic client did not apply the first filtered team frame".into());
    }
    println!(
        "phase4-filtered-join-smoke ok team={} join_tick={} frame_sequence={} disclosed_entities={} acceptance=false",
        team_id,
        start.replica_start_tick,
        frame.frame.team_sequence,
        client.world().entities.len(),
    );
    Ok(())
}
