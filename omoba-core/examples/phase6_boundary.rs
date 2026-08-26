use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use omoba_core::runtime::*;
use omoba_sim::{Fixed64, Vec2};

const TEAM: u32 = 1;
const OTHER: u32 = 2;
const ENTITY: u64 = (1u64 << 32) | 10;
const TARGET: u64 = (1u64 << 32) | 11;
const COMPONENT: u32 = 7;

fn entity(scope: ReplicationScopeKind, owner: Option<u32>, overrides: Vec<CommittedVisibilityOverride>, remember: RememberDisposition) -> CommittedEntityView {
    CommittedEntityView { canonical_id: ENTITY, team: OTHER, position: Vec2::new(Fixed64::ZERO, Fixed64::ZERO), scope, owner_team: owner, stealth_level: 0, overrides, remember, disclosed_baseline: encode_component_baseline(&[(COMPONENT, b"fresh")]) }
}

fn view(tick: u64, value: CommittedEntityView, source: bool) -> WaveBReadView {
    WaveBReadView { tick, entities: Arc::from([value]), vision_sources: if source { Arc::from([CommittedVisionSource { canonical_id: 1, team: TEAM, position: Vec2::new(Fixed64::ZERO, Fixed64::ZERO), radius: Fixed64::from_i32(5), detection_level: 0 }]) } else { Arc::from([]) } }
}

fn visible(value: CommittedEntityView, source: bool) -> bool {
    let mut state = TeamVisibilityState::new(TEAM, 8);
    state.resolve(&view(1, value, source), 0);
    state.index.current.contains(&ENTITY)
}

fn projector_with_target() -> (TeamViewProjector, BTreeSet<u64>) {
    let mut projector = TeamViewProjector::new(TEAM, TeamProjectorConfig { component_allowlist: BTreeSet::from([COMPONENT]), ..Default::default() });
    let set = BTreeSet::from([TARGET]);
    projector.build_frame(0, 0, &set, vec![VisibilityTransition::Reveal { canonical_id: TARGET, effective_tick: 0, baseline: encode_component_baseline(&[(COMPONENT, b"target")]) }], &[], &ProjectionDependencyGraph::default()).unwrap();
    (projector, set)
}

fn boundary_fact(kind: FactKind, active: bool) -> OrderedFact {
    OrderedFact { key: FactOrderingKey { tick: 1, phase: FactPhase::Step, canonical_source_order: ENTITY, local_ordinal: 0, fact_kind: kind }, audience: FactAudience::AllPlayers, fact: match kind {
        FactKind::DirectCombat => ObservableFact::DirectCombat { source: ENTITY, target: TARGET, amount_milli: 5 },
        FactKind::Buff => ObservableFact::Buff { source: ENTITY, target: TARGET, effect_id: 9, active },
        FactKind::Projectile => ObservableFact::Projectile { source: ENTITY, target: Some(TARGET), effect_id: 4, active },
        FactKind::AreaEffect => ObservableFact::DirectCombat { source: ENTITY, target: TARGET, amount_milli: 3 },
        _ => unreachable!(),
    }}
}

fn main() -> Result<(), String> {
    let mut passed = Vec::new();
    macro_rules! check { ($name:expr, $condition:expr) => {{ if !$condition { return Err(format!("scenario failed: {}", $name)); } passed.push($name); }} }

    check!("team-shared-vision", visible(entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::Forget), true));

    let (mut projector, target_set) = projector_with_target();
    let damage = projector.build_frame(1, 1, &target_set, vec![], &[boundary_fact(FactKind::DirectCombat, true)], &ProjectionDependencyGraph::default()).unwrap();
    check!("hidden-attacker-damage", damage.frame.step.as_ref().unwrap().external_effects.len() == 1);

    let mut replica = SelectiveReplicaRuntime::new(TEAM, 0, 0, 1, BTreeSet::from([COMPONENT]), BTreeSet::new());
    let mut stepper = NoopDisclosedWorldStepper;
    replica.apply_encoded_frame(&single_reveal_frame_fixture(TEAM, 0, 0, 1, COMPONENT), &mut stepper).unwrap();
    replica.apply_encoded_frame(&single_hide_frame_fixture(TEAM, 1, 1, 1), &mut stepper).unwrap();
    let remembered = replica.extract_filtered_render_snapshot();
    check!("remembered-ghost", replica.world().entities.is_empty() && matches!(remembered.memory_directives.first(), Some(RenderMemoryDirective::Hide { .. })));

    check!("owner-only-resource", visible(entity(ReplicationScopeKind::OwnerTeam, Some(TEAM), vec![], RememberDisposition::Forget), false));
    let mut history = TeamVisibilityHistory::new(3); history.push(4, &BTreeSet::from([ENTITY])); history.push(5, &BTreeSet::new());
    check!("input-tick-visibility-history", history.was_visible(4, ENTITY) && !history.was_visible(5, ENTITY));

    let rules = vec![CommittedVisibilityOverride { team: Some(TEAM), kind: VisibilityOverrideKind::ForceShow, priority: 100, stable_rule_id: 1 }, CommittedVisibilityOverride { team: Some(TEAM), kind: VisibilityOverrideKind::ForceHide, priority: -100, stable_rule_id: 2 }];
    check!("force-hide-precedence", !visible(entity(ReplicationScopeKind::Public, None, rules, RememberDisposition::Forget), false));
    let tie_a = vec![CommittedVisibilityOverride { team: Some(TEAM), kind: VisibilityOverrideKind::ForceShow, priority: 4, stable_rule_id: 20 }, CommittedVisibilityOverride { team: Some(TEAM), kind: VisibilityOverrideKind::ForceShow, priority: 4, stable_rule_id: 10 }];
    let mut tie_b = tie_a.clone(); tie_b.reverse();
    check!("stable-rule-tiebreak", visible(entity(ReplicationScopeKind::Vision, None, tie_a, RememberDisposition::Forget), false) == visible(entity(ReplicationScopeKind::Vision, None, tie_b, RememberDisposition::Forget), false));

    let mut state = TeamVisibilityState::new(TEAM, 8);
    state.resolve(&view(1, entity(ReplicationScopeKind::Public, None, vec![], RememberDisposition::Forget), false), 0);
    check!("override-expiry-boundary", matches!(state.resolve(&view(2, entity(ReplicationScopeKind::Public, None, vec![CommittedVisibilityOverride { team: Some(TEAM), kind: VisibilityOverrideKind::ForceHide, priority: 1, stable_rule_id: 1 }], RememberDisposition::Forget), false), 0).first(), Some(VisibilityTransition::Hide { .. })));

    let mut cancel = TeamVisibilityState::new(TEAM, 8);
    cancel.resolve(&view(1, entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::Forget), true), 2);
    check!("reveal-candidate-cancellation", cancel.resolve(&view(2, entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::Forget), false), 2).is_empty() && cancel.index.current.is_empty());
    cancel.resolve(&view(3, entity(ReplicationScopeKind::Public, None, vec![], RememberDisposition::Forget), false), 0);
    cancel.resolve(&view(4, entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::Forget), false), 2);
    check!("hide-candidate-cancellation", cancel.resolve(&view(5, entity(ReplicationScopeKind::Public, None, vec![], RememberDisposition::Forget), false), 2).is_empty() && cancel.index.current.contains(&ENTITY));

    let mut scheduled = TeamVisibilityState::new(TEAM, 8);
    scheduled.resolve(&view(7, entity(ReplicationScopeKind::Public, None, vec![], RememberDisposition::LastKnown), false), 1);
    let reveal = scheduled.resolve(&view(8, entity(ReplicationScopeKind::Public, None, vec![], RememberDisposition::LastKnown), false), 1);
    check!("scheduled-reveal-fresh-baseline", matches!(reveal.first(), Some(VisibilityTransition::Reveal { baseline, .. }) if baseline.ends_with(b"fresh")));
    scheduled.resolve(&view(9, entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::LastKnown), false), 1);
    check!("scheduled-hide", matches!(scheduled.resolve(&view(10, entity(ReplicationScopeKind::Vision, None, vec![], RememberDisposition::LastKnown), false), 1).first(), Some(VisibilityTransition::Hide { .. })));

    let key = CanonicalEntityKey { id: 10, generation: 1 }; let mut identity = TeamIdentityState::new(TEAM);
    let first = identity.disclose(key).unwrap(); identity.remember(key).unwrap(); let again = identity.disclose(key).unwrap();
    check!("rereveal-identity-association", first.replica_id == again.replica_id && again.disclosure_epoch > first.disclosure_epoch);
    let retired = identity.forget(key).unwrap();
    check!("authoritative-forget-retirement", matches!(identity.canonical_for(retired, again.disclosure_epoch), Err(TeamIdentityError::RetiredReplica)));

    for (name, kind, active) in [("hidden-source-buff", FactKind::Buff, true), ("hidden-source-debuff", FactKind::Buff, false), ("projectile-enter-visibility", FactKind::Projectile, true), ("projectile-leave-visibility", FactKind::Projectile, false), ("aoe-cross-boundary", FactKind::AreaEffect, true)] {
        let (mut p, set) = projector_with_target(); let frame = p.build_frame(1, 1, &set, vec![], &[boundary_fact(kind, active)], &ProjectionDependencyGraph::default()).unwrap();
        check!(name, frame.frame.step.unwrap().external_effects.len() == 1);
    }
    check!("remembered-fog-death-nondisclosure", remembered.public_events.is_empty());
    let silhouette = RenderMemoryDirective::Hide { replica_id: 2, disclosure_epoch: 1, remember_policy: 2, sanitized_presentation: vec![1] };
    check!("custom-remember-policy", matches!(silhouette, RenderMemoryDirective::Hide { remember_policy: 2, .. }));

    let mut validation = SecureInputValidationSnapshot::default();
    validation.teams.insert(TEAM, TeamInputValidationView { view_epoch: 3, replicas: BTreeMap::from([(1, ReplicaValidationRecord { canonical_id: ENTITY, disclosure_epoch: 2, owner_team: Some(TEAM) }), (2, ReplicaValidationRecord { canonical_id: TARGET, disclosure_epoch: 4, owner_team: Some(OTHER) })]), visible_by_tick: BTreeMap::from([(9, BTreeSet::from([ENTITY, TARGET])), (10, BTreeSet::from([ENTITY]))]) });
    let actor = SecureTargetReference { replica_id: 1, view_epoch: 3, disclosure_epoch: 2 };
    let target = SecureTargetReference { replica_id: 2, view_epoch: 3, disclosure_epoch: 4 };
    check!("remembered-target-rejection", validation.validate(TEAM, 10, actor, target).is_err());

    let safe = vec![ClassifiedComponentRecord { schema_id: COMPONENT, class: DisclosureClass::TeamPrivate, bytes: vec![1] }];
    check!("team-private-resource", redact_component_fields(&safe, &BTreeSet::from([COMPONENT])).unwrap().len() == 1);
    let public = vec![ClassifiedComponentRecord { schema_id: COMPONENT, class: DisclosureClass::Public, bytes: vec![2] }];
    check!("public-resource", redact_component_fields(&public, &BTreeSet::from([COMPONENT])).unwrap().len() == 1);
    let server = vec![ClassifiedComponentRecord { schema_id: COMPONENT, class: DisclosureClass::ServerOnly, bytes: vec![9] }];
    check!("server-only-resource-nondisclosure", matches!(redact_component_fields(&server, &BTreeSet::from([COMPONENT])), Err(ProjectionError::ServerOnlyComponent(_))));

    let (mut audience_projector, set) = projector_with_target();
    let retained = OrderedFact { key: FactOrderingKey { tick: 1, phase: FactPhase::Step, canonical_source_order: 0, local_ordinal: 0, fact_kind: FactKind::Hud }, audience: FactAudience::Team(TEAM), fact: ObservableFact::Hud { team: TEAM, metric_id: 1, value: 2 } };
    let audience_frame = audience_projector.build_frame(1, 1, &set, vec![], &[retained], &ProjectionDependencyGraph::default()).unwrap();
    check!("retained-event-audience", audience_frame.frame.step.unwrap().public_events.len() == 1);
    check!("stale-disclosure-epoch-input", validation.validate(TEAM, 9, actor, SecureTargetReference { disclosure_epoch: 3, ..target }).is_err());
    check!("ownership-invalid-input", validation.validate(TEAM, 9, SecureTargetReference { replica_id: 2, disclosure_epoch: 4, ..actor }, target).is_err());
    check!("accepted-disclosed-target", validation.validate(TEAM, 9, actor, target).is_ok());
    check!("rejected-hidden-target", validation.validate(TEAM, 10, actor, target).is_err());

    if passed.len() != 30 { return Err(format!("expected 30 scenarios, got {}", passed.len())); }
    println!("phase6-boundary ok count={} scenarios={}", passed.len(), passed.join(","));
    Ok(())
}
