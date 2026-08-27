use std::collections::{BTreeMap, BTreeSet};

use omoba_core::runtime::*;
use sha2::{Digest, Sha256};

const TEAM: u32 = 1;
const COMPONENT: u32 = 11;
const VISIBLE: u64 = (1u64 << 32) | 17;
const HIDDEN: u64 = (1u64 << 32) | 18;

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn config() -> TeamProjectorConfig {
    TeamProjectorConfig {
        component_allowlist: BTreeSet::from([COMPONENT]),
        size_buckets: vec![512, 1024, 2048],
        mass_reveal_chunk_entities: 64,
        rebase_chunks_per_tick: 2,
        hash_checkpoint_interval_ticks: 1,
    }
}

fn fact(kind: FactKind, source: u64, value: i64, ordinal: u32) -> OrderedFact {
    OrderedFact {
        key: FactOrderingKey {
            tick: 8,
            phase: FactPhase::Step,
            canonical_source_order: source,
            local_ordinal: ordinal,
            fact_kind: kind,
        },
        audience: FactAudience::AllPlayers,
        fact: match kind {
            FactKind::Movement => ObservableFact::Movement {
                source,
                x_mm: value,
                y_mm: value,
            },
            FactKind::Death => ObservableFact::Death {
                source,
                killer: None,
            },
            _ => ObservableFact::Hud {
                team: TEAM,
                metric_id: source,
                value,
            },
        },
    }
}

fn bootstrapped_projector() -> Result<
    (
        TeamViewProjector,
        SelectiveReplicaRuntime,
        SelectiveReplicaRuntime,
    ),
    String,
> {
    let mut projector = TeamViewProjector::new(TEAM, config());
    let visible = BTreeSet::from([VISIBLE]);
    let baseline = encode_component_baseline(&[(COMPONENT, b"visible")]);
    let frame = projector
        .build_frame(
            7,
            7,
            &visible,
            vec![VisibilityTransition::Reveal {
                canonical_id: VISIBLE,
                effective_tick: 7,
                baseline,
            }],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("initial projection: {error:?}"))?;
    let mut client = synthetic_client_from_encoded(
        frame.wire_bytes.clone(),
        BTreeSet::from([COMPONENT]),
        BTreeSet::new(),
    )
    .map_err(|error| error.to_string())?
    .runtime;
    let mut observer = synthetic_observer_from_encoded(
        frame.wire_bytes.clone(),
        BTreeSet::from([COMPONENT]),
        BTreeSet::new(),
    )
    .map_err(|error| error.to_string())?
    .runtime;
    let mut stepper = NoopDisclosedWorldStepper;
    let client_result = client
        .apply_encoded_frame(&frame.wire_bytes, &mut stepper)
        .map_err(|error| format!("client: {error:?}"))?;
    let observer_result = observer
        .apply_encoded_frame(&frame.wire_bytes, &mut stepper)
        .map_err(|error| format!("observer: {error:?}"))?;
    let expected = frame
        .frame
        .post_step
        .as_ref()
        .and_then(|post| post.hash_checkpoint.as_ref())
        .unwrap()
        .canonical_team_hash
        .clone();
    for result in [client_result, observer_result] {
        let FrameApplyResult::Applied { team_hash, .. } = result else {
            return Err("replica did not step".into());
        };
        if team_hash.as_slice() != expected {
            return Err("three-way checkpoint hash mismatch".into());
        }
    }
    println!(
        "parity checkpoint={} frame={}",
        hash(&expected),
        hash(&frame.wire_bytes)
    );
    Ok((projector, client, observer))
}

fn paired_hidden_case(
    label: &str,
    left: OrderedFact,
    right: OrderedFact,
) -> Result<String, String> {
    let visible = BTreeSet::from([VISIBLE]);
    let mut a = TeamViewProjector::new(TEAM, config());
    let mut b = TeamViewProjector::new(TEAM, config());
    let baseline = encode_component_baseline(&[(COMPONENT, b"visible")]);
    for projector in [&mut a, &mut b] {
        projector
            .build_frame(
                7,
                7,
                &visible,
                vec![VisibilityTransition::Reveal {
                    canonical_id: VISIBLE,
                    effective_tick: 7,
                    baseline: baseline.clone(),
                }],
                &[],
                &ProjectionDependencyGraph::default(),
            )
            .map_err(|error| format!("{error:?}"))?;
    }
    let fa = a
        .build_frame(
            8,
            8,
            &visible,
            Vec::new(),
            &[left],
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let fb = b
        .build_frame(
            8,
            8,
            &visible,
            Vec::new(),
            &[right],
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("{error:?}"))?;
    if fa.wire_bytes != fb.wire_bytes {
        return Err(format!("{label} leaked into team frame"));
    }
    let digest = hash(&fa.wire_bytes);
    println!("noninterference {label}={digest}");
    Ok(digest)
}

fn main() -> Result<(), String> {
    let (_projector, client, observer) = bootstrapped_projector()?;
    if client.canonical_team_hash() != observer.canonical_team_hash() {
        return Err("client/observer parity mismatch".into());
    }

    let movement = paired_hidden_case(
        "movement",
        fact(FactKind::Movement, HIDDEN, 10, 0),
        fact(FactKind::Movement, HIDDEN, 9999, 0),
    )?;
    let component = paired_hidden_case(
        "component",
        fact(FactKind::Movement, HIDDEN, -7, 0),
        fact(FactKind::Movement, HIDDEN, 77, 0),
    )?;
    let rng = paired_hidden_case(
        "rng",
        fact(FactKind::Movement, HIDDEN, 12345, 0),
        fact(FactKind::Movement, HIDDEN, 98765, 0),
    )?;
    let death = paired_hidden_case(
        "death",
        fact(FactKind::Death, HIDDEN, 0, 0),
        fact(FactKind::Movement, HIDDEN, 1, 0),
    )?;
    if BTreeSet::from([movement, component, rng, death]).len() != 1 {
        return Err("paired cases did not converge to one public frame".into());
    }

    let visible = BTreeSet::new();
    let facts = vec![
        fact(FactKind::Hud, 3, 30, 2),
        fact(FactKind::Hud, 1, 10, 0),
        fact(FactKind::Hud, 2, 20, 1),
    ];
    let mut reversed = facts.clone();
    reversed.reverse();
    let mut a = TeamViewProjector::new(TEAM, config());
    let mut b = TeamViewProjector::new(TEAM, config());
    let fa = a
        .build_frame(
            7,
            7,
            &visible,
            Vec::new(),
            &facts,
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("{error:?}"))?;
    let fb = b
        .build_frame(
            7,
            7,
            &visible,
            Vec::new(),
            &reversed,
            &ProjectionDependencyGraph::default(),
        )
        .map_err(|error| format!("{error:?}"))?;
    if fa.wire_bytes != fb.wire_bytes {
        return Err("completion-order permutation changed bytes".into());
    }
    println!("permutation frame={}", hash(&fa.wire_bytes));

    let report = BTreeMap::from([
        ("checkpoint", hash(&client.canonical_team_hash())),
        ("noninterference", hash(&fa.wire_bytes)),
    ]);
    println!("phase6-differential ok {report:?}");
    Ok(())
}
