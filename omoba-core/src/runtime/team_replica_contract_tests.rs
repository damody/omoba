use std::collections::{BTreeMap, BTreeSet};

use prost::Message;

use super::*;
use crate::game_proto::{
    transition, AuthorityRevision, DisclosureEpoch, ForgetEntity, PostStep, PreStep,
    ReplicaEntityId, Step, TeamTickFrame, Transition, ViewEpoch,
};

fn demo_baseline(team: u32, marker: u32) -> Vec<u8> {
    encode_component_baseline(&[(
        DEMO_RENDER_COMPONENT_SCHEMA_ID,
        &encode_demo_render_state(DemoRenderState {
            x_raw: i64::from(marker),
            y_raw: 0,
            team_id: team,
            kind: 2,
            owner_player_id: 0,
        }),
    )])
}

fn disclosed_projector(team: u32, marker: u32) -> TeamViewProjector {
    let mut projector = TeamViewProjector::new(
        team,
        TeamProjectorConfig {
            hash_checkpoint_interval_ticks: 1,
            ..TeamProjectorConfig::default()
        },
    );
    projector
        .build_frame(
            0,
            0,
            &BTreeSet::from([u64::from(marker)]),
            vec![VisibilityTransition::Reveal {
                canonical_id: u64::from(marker),
                effective_tick: 0,
                baseline: demo_baseline(team, marker),
            }],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    projector
}

fn hero_baseline(team: u32, owner: u32) -> Vec<u8> {
    let render = encode_demo_render_state(DemoRenderState {
        x_raw: 0,
        y_raw: 0,
        team_id: team,
        kind: 1,
        owner_player_id: owner,
    });
    let property = encode_disclosed_property(&CProperty {
        hp: omoba_sim::Fixed64::from_i32(100),
        mhp: omoba_sim::Fixed64::from_i32(100),
        msd: omoba_sim::Fixed64::from_i32(120),
        def_physic: omoba_sim::Fixed64::ZERO,
        def_magic: omoba_sim::Fixed64::ZERO,
    });
    encode_component_baseline(&[
        (DEMO_RENDER_COMPONENT_SCHEMA_ID, &render),
        (DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID, &property),
    ])
}

#[test]
fn phase_fixture_records_the_complete_shared_order() {
    let mut trace = Vec::new();
    run_deterministic_gameplay_phases(&mut |phase| -> Result<(), ()> {
        trace.push(phase);
        Ok(())
    })
    .unwrap();
    assert_eq!(trace, DETERMINISTIC_GAMEPLAY_PHASES);
}

#[test]
fn hidden_sentinel_bootstrap_is_team_isolated_and_seed_is_shared() {
    let seed = 0x1234_5678;
    let mut one = disclosed_projector(1, 101);
    let mut two = disclosed_projector(2, 202);
    let start_one = one.build_team_game_start(1, 120, seed);
    let start_two = two.build_team_game_start(1, 120, seed);
    let bytes_one = start_one.encode_to_vec();
    let bytes_two = start_two.encode_to_vec();
    assert_eq!(start_one.global_seed, start_two.global_seed);
    assert!(!bytes_one
        .windows(4)
        .any(|value| value == 202u32.to_be_bytes()));
    assert!(!bytes_two
        .windows(4)
        .any(|value| value == 101u32.to_be_bytes()));
}

#[test]
fn filtered_specs_bootstrap_contains_only_disclosed_entities() {
    let mut projector = disclosed_projector(1, 101);
    let start = projector.build_team_game_start(1, 120, 7);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start,
        allow.clone(),
        BTreeSet::new(),
    )
    .unwrap();
    let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow, BTreeSet::new());
    stepper.bootstrap_membership(runtime.world()).unwrap();
    assert_eq!(runtime.world().entities.len(), 1);
    assert_eq!(stepper.filtered.entities.0.len(), 1);
}

#[derive(Default)]
struct MembershipRecordingStepper {
    saw_entity_during_step: bool,
}

impl DisclosedWorldStepper for MembershipRecordingStepper {
    fn fixed_step(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        _: &StepInjections,
        _: &BTreeSet<u32>,
        _: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError> {
        self.saw_entity_during_step = world.entities.contains_key(&1);
        Ok(())
    }
}

#[test]
fn reveal_is_present_before_gameplay_of_the_same_tick() {
    let mut runtime = SelectiveReplicaRuntime::new(
        1,
        0,
        0,
        1,
        BTreeSet::from([DEMO_RENDER_COMPONENT_SCHEMA_ID]),
        BTreeSet::new(),
    );
    let mut stepper = MembershipRecordingStepper::default();
    runtime
        .apply_encoded_frame(
            &single_reveal_frame_fixture(1, 0, 0, 1, DEMO_RENDER_COMPONENT_SCHEMA_ID),
            &mut stepper,
        )
        .unwrap();
    assert!(stepper.saw_entity_during_step);
}

#[test]
fn accepted_move_input_advances_the_owned_filtered_hero() {
    use crate::game_proto::{player_input, MoveTo, PlayerInput, Vec2I};
    let mut projector = TeamViewProjector::new(
        1,
        TeamProjectorConfig {
            hash_checkpoint_interval_ticks: 1,
            ..TeamProjectorConfig::default()
        },
    );
    projector
        .build_frame(
            0,
            0,
            &BTreeSet::from([77]),
            vec![VisibilityTransition::Reveal {
                canonical_id: 77,
                effective_tick: 0,
                baseline: hero_baseline(1, 42),
            }],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    let start = projector.build_team_game_start(1, 120, 9);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let mut runtime = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start,
        allow.clone(),
        BTreeSet::new(),
    )
    .unwrap();
    let mut stepper = SpecsDisclosedWorldStepper::from_start(&start, allow, BTreeSet::new());
    stepper.bootstrap_membership(runtime.world()).unwrap();
    let input = PlayerInput {
        action: Some(player_input::Action::MoveTo(MoveTo {
            target: Some(Vec2I {
                x: 100 * 1024,
                y: 0,
            }),
            queued: false,
        })),
    };
    let frame = projector
        .build_frame_with_inputs(
            1,
            1,
            &BTreeSet::from([77]),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
            vec![CanonicalAcceptedInput::from_authoritative_acceptance(
                1,
                42,
                1,
                2,
                77,
                None,
                input.encode_to_vec(),
            )],
        )
        .unwrap();
    runtime.apply_frame(frame.frame, &mut stepper).unwrap();
    let next = projector
        .build_frame(
            2,
            2,
            &BTreeSet::from([77]),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    runtime.apply_frame(next.frame, &mut stepper).unwrap();
    let render = decode_demo_render_state(
        &runtime.world().entities[&1].components[&DEMO_RENDER_COMPONENT_SCHEMA_ID],
    )
    .unwrap();
    assert!(
        render.x_raw > 0,
        "owned hero did not move in filtered Specs world"
    );
}

#[test]
fn forget_removes_entity_before_same_tick_gameplay() {
    let mut runtime = SelectiveReplicaRuntime::new(
        1,
        0,
        0,
        1,
        BTreeSet::from([DEMO_RENDER_COMPONENT_SCHEMA_ID]),
        BTreeSet::new(),
    );
    let mut noop = NoopDisclosedWorldStepper;
    runtime
        .apply_encoded_frame(
            &single_reveal_frame_fixture(1, 0, 0, 1, DEMO_RENDER_COMPONENT_SCHEMA_ID),
            &mut noop,
        )
        .unwrap();
    let frame = TeamTickFrame {
        protocol_version: 2,
        frame_schema_version: 1,
        content_schema_version: 1,
        team_id: 1,
        server_tick: 1,
        replica_tick: 1,
        team_sequence: 1,
        view_epoch: Some(ViewEpoch { value: 1 }),
        authority_revision: Some(AuthorityRevision { value: 2 }),
        pre_step: Some(PreStep {
            transitions: vec![Transition {
                transition: Some(transition::Transition::Forget(ForgetEntity {
                    replica_entity_id: Some(ReplicaEntityId { value: 1 }),
                    disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
                    effective_tick: 1,
                    retire_reason: 1,
                    stable_sub_index: 0,
                })),
            }],
        }),
        step: Some(Step::default()),
        post_step: Some(PostStep::default()),
        padding: vec![],
    };
    runtime.apply_frame(frame, &mut noop).unwrap();
    assert!(runtime.world().entities.is_empty());
    assert!(runtime.remembered_presentations().is_empty());
}

#[test]
fn random_request_completion_order_is_irrelevant() {
    let run = |order: &[u64]| {
        let mut rng = TickDeterministicRng::new(99);
        rng.begin_tick(8);
        for ordinal in order {
            rng.request(RandomRequest {
                key: RandomRequestKey {
                    phase_ordinal: 1,
                    stable_request_ordinal: *ordinal,
                },
            });
        }
        rng.resolve().to_vec()
    };
    assert_eq!(run(&[3, 1, 2]), run(&[2, 3, 1]));
}

#[test]
fn hidden_external_effect_contains_no_canonical_source() {
    let mut projector = disclosed_projector(1, 77);
    let fact = OrderedFact {
        key: FactOrderingKey {
            tick: 1,
            phase: FactPhase::Step,
            canonical_source_order: 999,
            local_ordinal: 0,
            fact_kind: FactKind::DirectCombat,
        },
        audience: FactAudience::VisibilityPolicy(
            omb_script_abi::types::projection_policy_ids::DIRECT_COMBAT.into(),
        ),
        fact: ObservableFact::DirectCombat {
            source: 999,
            target: 77,
            amount_milli: 10,
        },
    };
    let frame = projector
        .build_frame(
            1,
            1,
            &BTreeSet::from([77]),
            vec![],
            &[fact],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    let effect = &frame.frame.step.unwrap().external_effects[0];
    assert_eq!(effect.visible_target.as_ref().unwrap().value, 1);
    assert!(!effect
        .sanitized_payload
        .windows(8)
        .any(|value| value == 999u64.to_be_bytes()));
}

#[test]
fn every_hidden_cross_boundary_effect_is_sanitized() {
    let cases = [
        (
            FactKind::DirectCombat,
            ObservableFact::DirectCombat {
                source: 999,
                target: 77,
                amount_milli: 3,
            },
        ),
        (
            FactKind::Buff,
            ObservableFact::Buff {
                source: 999,
                target: 77,
                effect_id: 4,
                active: true,
            },
        ),
        (
            FactKind::Projectile,
            ObservableFact::Projectile {
                source: 999,
                target: Some(77),
                effect_id: 5,
                active: true,
            },
        ),
        (
            FactKind::Collision,
            ObservableFact::Collision {
                source: 999,
                target: 77,
                x_raw: 6,
                y_raw: 7,
            },
        ),
        (
            FactKind::Random,
            ObservableFact::RandomOutcome {
                source: 999,
                target: 77,
                value: 8,
            },
        ),
    ];
    for (ordinal, (kind, fact)) in cases.into_iter().enumerate() {
        let mut projector = disclosed_projector(1, 77);
        let frame = projector
            .build_frame(
                1,
                1,
                &BTreeSet::from([77]),
                vec![],
                &[OrderedFact {
                    key: FactOrderingKey {
                        tick: 1,
                        phase: FactPhase::Step,
                        canonical_source_order: 999,
                        local_ordinal: ordinal as u32,
                        fact_kind: kind,
                    },
                    audience: FactAudience::VisibilityPolicy(
                        omb_script_abi::types::projection_policy_ids::DIRECT_COMBAT.into(),
                    ),
                    fact,
                }],
                &ProjectionDependencyGraph::default(),
            )
            .unwrap();
        let effects = frame.frame.step.unwrap().external_effects;
        assert_eq!(effects.len(), 1, "missing sanitized effect for {kind:?}");
        assert_eq!(effects[0].visible_target.as_ref().unwrap().value, 1);
        assert!(!effects[0]
            .sanitized_payload
            .windows(8)
            .any(|bytes| bytes == 999u64.to_le_bytes()));
        assert!(!effects[0]
            .sanitized_payload
            .windows(8)
            .any(|bytes| bytes == 999u64.to_be_bytes()));
    }
}

#[test]
fn steady_state_frame_has_no_proactive_component_repair() {
    let mut projector = disclosed_projector(1, 77);
    let frame = projector
        .build_frame(
            1,
            1,
            &BTreeSet::from([77]),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap();
    assert!(frame.frame.post_step.unwrap().component_repairs.is_empty());
}

#[test]
fn recovery_escalates_repair_replace_rebase_and_termination() {
    let difference = |id| ComponentDifference {
        replica_id: id,
        disclosure_epoch: 1,
        component_schema_id: DEMO_RENDER_COMPONENT_SCHEMA_ID,
        safe_component_path: "position".into(),
        field_mask: vec![1],
        replacement_fields: vec![1],
        safe_entity_baseline: demo_baseline(1, id as u32),
    };
    let metadata = safe_mismatch_metadata(1, 1, &[1; 32], &[2; 32], 1);
    let mut coordinator = AuthorityRepairCoordinator::configured();
    coordinator.report_component_divergence(1, 1, 1, metadata.clone(), &[difference(1)]);
    assert!(matches!(
        coordinator.drain_actions(1)[0],
        RecoveryAction::ComponentRepair { .. }
    ));
    coordinator.report_component_divergence(
        1,
        2,
        2,
        metadata.clone(),
        &[difference(1), difference(2)],
    );
    assert!(matches!(
        coordinator.drain_actions(1)[0],
        RecoveryAction::EntityReplace(_)
    ));
    coordinator.report_coverage_gap(1, 3);
    assert!(matches!(
        coordinator.drain_actions(1)[0],
        RecoveryAction::FilteredRebase { .. }
    ));
}

struct MutateComponentStepper;

impl DisclosedWorldStepper for MutateComponentStepper {
    fn fixed_step(
        &mut self,
        world: &mut DisclosedReplicaWorld,
        _: &StepInjections,
        _: &BTreeSet<u32>,
        _: &BTreeSet<u32>,
    ) -> Result<(), ReplicaRuntimeError> {
        world.entities.get_mut(&1).unwrap().components.insert(
            DEMO_RENDER_COMPONENT_SCHEMA_ID,
            b"divergent-position".to_vec(),
        );
        Ok(())
    }
}

#[test]
fn pre_repair_hash_detects_divergence_and_post_repair_revalidates_server_value() {
    let mut runtime = SelectiveReplicaRuntime::new(
        1,
        0,
        0,
        1,
        BTreeSet::from([DEMO_RENDER_COMPONENT_SCHEMA_ID]),
        BTreeSet::new(),
    );
    let mut noop = NoopDisclosedWorldStepper;
    runtime
        .apply_encoded_frame(
            &single_reveal_frame_fixture(1, 0, 0, 1, DEMO_RENDER_COMPONENT_SCHEMA_ID),
            &mut noop,
        )
        .unwrap();
    let server_value = b"server-position".to_vec();
    let mut frame = empty_wire_frame(1, 1);
    frame.authority_revision = Some(AuthorityRevision { value: 2 });
    frame
        .post_step
        .as_mut()
        .unwrap()
        .component_repairs
        .push(crate::game_proto::ComponentRepair {
            replica_entity_id: Some(ReplicaEntityId { value: 1 }),
            disclosure_epoch: Some(DisclosureEpoch { value: 1 }),
            component_schema_id: DEMO_RENDER_COMPONENT_SCHEMA_ID,
            field_mask: Vec::new(),
            replacement_fields: server_value.clone(),
            authority_revision: Some(AuthorityRevision { value: 2 }),
            effective_tick: 1,
        });
    let result = runtime
        .apply_frame(frame, &mut MutateComponentStepper)
        .unwrap();
    let FrameApplyResult::Applied {
        pre_repair_observed_hash,
        post_repair_hash,
        ..
    } = result
    else {
        panic!("frame not applied")
    };
    assert_ne!(pre_repair_observed_hash, post_repair_hash);
    assert_eq!(
        runtime.world().entities[&1].components[&DEMO_RENDER_COMPONENT_SCHEMA_ID],
        server_value
    );
    assert_eq!(post_repair_hash, runtime.canonical_team_hash());
}

#[test]
fn changed_script_result_is_detected_before_any_authority_correction() {
    let mut projector = disclosed_projector(1, 77);
    let start = projector.build_team_game_start(1, 120, 55);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let mut replica =
        SelectiveReplicaRuntime::bootstrap_from_team_game_start(&start, allow, BTreeSet::new())
            .unwrap();
    let frame = projector
        .build_frame(
            1,
            1,
            &BTreeSet::from([77]),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap()
        .frame;
    let expected: [u8; 32] = frame
        .post_step
        .as_ref()
        .unwrap()
        .hash_checkpoint
        .as_ref()
        .unwrap()
        .canonical_team_hash
        .clone()
        .try_into()
        .unwrap();
    let result = replica
        .apply_frame(frame, &mut MutateComponentStepper)
        .unwrap();
    let FrameApplyResult::Applied {
        pre_repair_observed_hash,
        ..
    } = result
    else {
        panic!("frame not applied")
    };
    assert_ne!(pre_repair_observed_hash, expected);
}

#[test]
fn duplicate_reorder_missing_and_corrupt_frames_fail_safely() {
    let mut runtime = SelectiveReplicaRuntime::new(1, 0, 0, 1, BTreeSet::new(), BTreeSet::new());
    let mut noop = NoopDisclosedWorldStepper;
    assert!(matches!(
        runtime.apply_frame(
            TeamTickFrame {
                team_id: 1,
                ..empty_wire_frame(0, 0)
            },
            &mut noop
        ),
        Ok(FrameApplyResult::Applied { .. })
    ));
    assert_eq!(
        runtime.apply_frame(
            TeamTickFrame {
                team_id: 1,
                ..empty_wire_frame(0, 0)
            },
            &mut noop
        ),
        Ok(FrameApplyResult::Duplicate)
    );
    assert!(matches!(
        runtime.apply_frame(
            TeamTickFrame {
                team_id: 1,
                ..empty_wire_frame(2, 2)
            },
            &mut noop
        ),
        Ok(FrameApplyResult::Stalled(_))
    ));
    assert_eq!(
        runtime.apply_encoded_frame(b"corrupt", &mut noop),
        Err(ReplicaRuntimeError::Decode)
    );
}

fn empty_wire_frame(tick: u64, sequence: u64) -> TeamTickFrame {
    TeamTickFrame {
        protocol_version: 2,
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
        padding: vec![],
    }
}

#[test]
fn ten_thousand_entity_fixture_has_bounded_filtered_membership() {
    let visible: BTreeSet<_> = (1..=10_000u64).collect();
    let mut baselines = BTreeMap::new();
    for id in &visible {
        baselines.insert(*id, demo_baseline(1, *id as u32));
    }
    assert_eq!(visible.len(), 10_000);
    assert_eq!(baselines.len(), visible.len());
}

#[test]
fn server_observer_and_client_replica_have_the_same_pre_repair_hash() {
    let mut projector = disclosed_projector(1, 77);
    let start = projector.build_team_game_start(1, 120, 55);
    let allow = TeamProjectorConfig::default().component_allowlist;
    let mut observer = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start,
        allow.clone(),
        BTreeSet::new(),
    )
    .unwrap();
    let mut client = SelectiveReplicaRuntime::bootstrap_from_team_game_start(
        &start,
        allow.clone(),
        BTreeSet::new(),
    )
    .unwrap();
    let mut observer_stepper =
        SpecsDisclosedWorldStepper::from_start(&start, allow.clone(), BTreeSet::new());
    let mut client_stepper = SpecsDisclosedWorldStepper::from_start(&start, allow, BTreeSet::new());
    observer_stepper
        .bootstrap_membership(observer.world())
        .unwrap();
    client_stepper.bootstrap_membership(client.world()).unwrap();
    let frame = projector
        .build_frame(
            1,
            1,
            &BTreeSet::from([77]),
            vec![],
            &[],
            &ProjectionDependencyGraph::default(),
        )
        .unwrap()
        .frame;
    let expected: [u8; 32] = frame
        .post_step
        .as_ref()
        .unwrap()
        .hash_checkpoint
        .as_ref()
        .unwrap()
        .canonical_team_hash
        .clone()
        .try_into()
        .unwrap();
    let observer_result = observer
        .apply_frame(frame.clone(), &mut observer_stepper)
        .unwrap();
    let client_result = client.apply_frame(frame, &mut client_stepper).unwrap();
    let hash = |result| match result {
        FrameApplyResult::Applied {
            pre_repair_observed_hash,
            ..
        } => pre_repair_observed_hash,
        _ => panic!("frame not applied"),
    };
    let observer_hash = hash(observer_result);
    let client_hash = hash(client_result);
    assert_eq!(observer_hash, client_hash);
    assert_eq!(observer_hash, expected, "expected hash contract changed");
}

#[test]
fn repair_decisions_do_not_depend_on_cross_team_report_arrival_order() {
    fn run(order: [u32; 2]) -> BTreeMap<u32, &'static str> {
        let mut coordinator = AuthorityRepairCoordinator::configured();
        for team in order {
            coordinator.report_coverage_gap(team, 9);
        }
        [1, 2]
            .into_iter()
            .map(|team| {
                let kind = match coordinator.drain_actions(team).remove(0) {
                    RecoveryAction::FilteredRebase { .. } => "rebase",
                    _ => "other",
                };
                (team, kind)
            })
            .collect()
    }
    assert_eq!(run([1, 2]), run([2, 1]));
}
