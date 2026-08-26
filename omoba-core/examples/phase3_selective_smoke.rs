use std::sync::Arc;

use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};
use prost::Message;

fn main() -> Result<(), String> {
    let fact = OrderedFact {
        key: FactOrderingKey {
            tick: 7,
            phase: FactPhase::PostStep,
            canonical_source_order: 1,
            local_ordinal: 0,
            fact_kind: FactKind::Hud,
        },
        audience: FactAudience::AllPlayers,
        fact: ObservableFact::Hud { team: 1, metric_id: 9, value: 42 },
    };
    let committed = commit_wave_a::<()>(7, Vec::new(), vec![fact])
        .map_err(|error| format!("Wave A failed: {error:?}"))?;
    if !committed.barrier_reached { return Err("commit barrier missing".into()); }

    let canonical_id = (1u64 << 32) | 17;
    let view = WaveBReadView {
        tick: 7,
        entities: Arc::from([CommittedEntityView {
            canonical_id,
            team: 1,
            position: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            scope: ReplicationScopeKind::Public,
            owner_team: Some(1),
            stealth_level: 0,
            overrides: Vec::new(),
            remember: RememberDisposition::Forget,
            disclosed_baseline: vec![1, 2, 3],
        }]),
        vision_sources: Arc::from([]),
    };
    let mut visibility = TeamVisibilityState::new(1, 16);
    let transitions = visibility.resolve(&view, 0);
    if !visibility.index.current.contains(&canonical_id) || transitions.is_empty() {
        return Err("Wave B did not observe committed public entity".into());
    }

    let mut projector = TeamViewProjector::new(1, TeamProjectorConfig::default());
    let padded = projector.build_frame(
        7,
        7,
        &visibility.index.current,
        transitions,
        &committed.ordered_facts,
        &ProjectionDependencyGraph::default(),
    ).map_err(|error| format!("projection failed: {error:?}"))?;
    let decoded = omoba_core::game_proto::TeamTickFrame::decode(padded.wire_bytes.as_slice())
        .map_err(|error| format!("encoded frame invalid: {error}"))?;
    if decoded.team_id != 1 || decoded.replica_tick != 7 || decoded.pre_step.is_none()
        || decoded.step.is_none() || decoded.post_step.is_none() {
        return Err("encoded frame fields incomplete".into());
    }
    println!(
        "phase3-smoke ok barrier={} visible={} frame_bytes={} padding={}",
        committed.barrier_reached,
        visibility.index.current.len(),
        padded.wire_bytes.len(),
        padded.padding_len,
    );
    Ok(())
}
