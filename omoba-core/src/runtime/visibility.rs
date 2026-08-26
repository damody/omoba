//! Two-wave selective visibility pipeline.
//!
//! Wave A commits ordered gameplay outputs. Wave B receives an immutable view
//! of State[T+1] and resolves each team independently (and in parallel).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use omoba_sim::{Fixed64, Vec2};
use rayon::prelude::*;

use crate::runtime::native::comp::{
    RememberDisposition, ReplicationScopeKind, VisibilityOverrideKind,
};
use crate::runtime::{OrderedFact, OrderedOutput, StableOutputError};
use specs::{Join, World, WorldExt};

#[derive(Clone, Debug)]
pub struct CommittedEntityView {
    pub canonical_id: u64,
    pub team: u32,
    pub position: Vec2,
    pub scope: ReplicationScopeKind,
    pub owner_team: Option<u32>,
    pub stealth_level: u16,
    pub overrides: Vec<CommittedVisibilityOverride>,
    pub remember: RememberDisposition,
    /// Fresh authoritative, already allowlisted component baseline.
    pub disclosed_baseline: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommittedVisibilityOverride {
    pub team: Option<u32>,
    pub kind: VisibilityOverrideKind,
    pub priority: i16,
    pub stable_rule_id: u64,
}

#[derive(Clone, Copy, Debug)]
pub struct CommittedVisionSource {
    pub canonical_id: u64,
    pub team: u32,
    pub position: Vec2,
    pub radius: Fixed64,
    pub detection_level: u16,
}

#[derive(Clone, Debug)]
pub struct WaveBReadView {
    pub tick: u64,
    pub entities: Arc<[CommittedEntityView]>,
    pub vision_sources: Arc<[CommittedVisionSource]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VisibilityTransition {
    Reveal { canonical_id: u64, effective_tick: u64, baseline: Vec<u8> },
    Hide { canonical_id: u64, effective_tick: u64, disposition: RememberDisposition },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VisibilityCandidate { desired_visible: bool, effective_tick: u64 }

#[derive(Clone, Debug, Default)]
pub struct TeamVisibilityIndex {
    pub current: BTreeSet<u64>,
    candidates: BTreeMap<u64, VisibilityCandidate>,
}

#[derive(Clone, Debug)]
pub struct VisibilityHistoryEntry { pub tick: u64, pub visible: BTreeSet<u64> }

#[derive(Clone, Debug)]
pub struct TeamVisibilityHistory {
    capacity: usize,
    entries: VecDeque<VisibilityHistoryEntry>,
}

impl TeamVisibilityHistory {
    pub fn new(capacity: usize) -> Self { Self { capacity: capacity.max(1), entries: VecDeque::new() } }
    pub fn push(&mut self, tick: u64, visible: &BTreeSet<u64>) {
        self.entries.push_back(VisibilityHistoryEntry { tick, visible: visible.clone() });
        while self.entries.len() > self.capacity { self.entries.pop_front(); }
    }
    pub fn was_visible(&self, tick: u64, entity: u64) -> bool {
        self.entries.iter().rev().find(|entry| entry.tick <= tick)
            .is_some_and(|entry| entry.visible.contains(&entity))
    }
}

#[derive(Clone, Debug)]
pub struct TeamVisibilityState {
    pub team: u32,
    pub index: TeamVisibilityIndex,
    pub history: TeamVisibilityHistory,
}

impl TeamVisibilityState {
    pub fn new(team: u32, history_capacity: usize) -> Self {
        Self { team, index: TeamVisibilityIndex::default(), history: TeamVisibilityHistory::new(history_capacity) }
    }

    pub fn resolve(&mut self, view: &WaveBReadView, transition_delay_ticks: u64) -> Vec<VisibilityTransition> {
        let sources: Vec<_> = view.vision_sources.iter().filter(|source| source.team == self.team).copied().collect();
        let mut transitions = Vec::new();
        for entity in view.entities.iter() {
            let desired = entity_visible_to_team(entity, self.team, &sources);
            let current = self.index.current.contains(&entity.canonical_id);
            if desired == current {
                self.index.candidates.remove(&entity.canonical_id);
                continue;
            }
            let effective_tick = view.tick.saturating_add(transition_delay_ticks);
            let candidate = self.index.candidates.entry(entity.canonical_id).or_insert(VisibilityCandidate {
                desired_visible: desired,
                effective_tick,
            });
            if candidate.desired_visible != desired {
                *candidate = VisibilityCandidate { desired_visible: desired, effective_tick };
            }
            if view.tick < candidate.effective_tick { continue; }
            if desired {
                self.index.current.insert(entity.canonical_id);
                transitions.push(VisibilityTransition::Reveal {
                    canonical_id: entity.canonical_id,
                    effective_tick: view.tick,
                    baseline: entity.disclosed_baseline.clone(),
                });
            } else {
                self.index.current.remove(&entity.canonical_id);
                transitions.push(VisibilityTransition::Hide {
                    canonical_id: entity.canonical_id,
                    effective_tick: view.tick,
                    disposition: entity.remember,
                });
            }
            self.index.candidates.remove(&entity.canonical_id);
        }
        transitions.sort_by_key(|transition| match transition {
            VisibilityTransition::Reveal { canonical_id, .. } | VisibilityTransition::Hide { canonical_id, .. } => *canonical_id,
        });
        self.history.push(view.tick, &self.index.current);
        transitions
    }
}

fn entity_visible_to_team(entity: &CommittedEntityView, team: u32, sources: &[CommittedVisionSource]) -> bool {
    // Deny rules have absolute precedence.
    if entity.scope == ReplicationScopeKind::ServerOnly { return false; }
    if resolve_override(&entity.overrides, team) == Some(VisibilityOverrideKind::ForceHide) { return false; }
    // Explicit/public grants precede owner and geometry checks.
    if entity.scope == ReplicationScopeKind::Public
        || resolve_override(&entity.overrides, team) == Some(VisibilityOverrideKind::ForceShow) { return true; }
    if entity.scope == ReplicationScopeKind::OwnerTeam && entity.owner_team == Some(team) { return true; }
    sources.iter().any(|source| {
        if source.detection_level < entity.stealth_level { return false; }
        let delta = entity.position - source.position;
        delta.length_squared() <= source.radius * source.radius
    })
}

fn resolve_override(overrides: &[CommittedVisibilityOverride], team: u32) -> Option<VisibilityOverrideKind> {
    let matching: Vec<_> = overrides.iter()
        .filter(|rule| rule.team.is_none() || rule.team == Some(team)).copied().collect();
    let kind = if matching.iter().any(|rule| rule.kind == VisibilityOverrideKind::ForceHide) {
        VisibilityOverrideKind::ForceHide
    } else if matching.iter().any(|rule| rule.kind == VisibilityOverrideKind::ForceShow) {
        VisibilityOverrideKind::ForceShow
    } else {
        return None;
    };
    matching.into_iter().filter(|rule| rule.kind == kind)
        .min_by_key(|rule| (-i32::from(rule.priority), rule.stable_rule_id))
        .map(|rule| rule.kind)
}

pub fn run_team_wave_b_parallel(
    view: &WaveBReadView,
    teams: &mut [TeamVisibilityState],
    transition_delay_ticks: u64,
) -> Vec<(u32, Vec<VisibilityTransition>)> {
    let mut results: Vec<_> = teams.par_iter_mut().map(|state| {
        (state.team, state.resolve(view, transition_delay_ticks))
    }).collect();
    results.sort_by_key(|(team, _)| *team);
    results
}

#[derive(Clone, Debug, Default)]
pub struct TeamVisibilityRuntime {
    pub teams: BTreeMap<u32, TeamVisibilityState>,
    pub last_transitions: BTreeMap<u32, Vec<VisibilityTransition>>,
}

impl TeamVisibilityRuntime {
    pub fn ensure_team(&mut self, team: u32) {
        self.teams.entry(team).or_insert_with(|| TeamVisibilityState::new(team, 512));
    }
}

pub fn build_wave_b_read_view(world: &World, tick: u64) -> WaveBReadView {
    use crate::runtime::native::comp::*;
    let entities = world.entities();
    let positions = world.read_storage::<Pos>();
    let factions = world.read_storage::<Faction>();
    let scopes = world.read_storage::<ReplicationScope>();
    let stealth = world.read_storage::<StealthProfile>();
    let overrides = world.read_storage::<VisibilityOverride>();
    let remembers = world.read_storage::<RememberPolicy>();
    let vision = world.read_storage::<VisionSource>();

    let mut committed_entities: Vec<_> = (&entities, &positions).join().map(|(entity, position)| {
        let canonical_id = ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id());
        let team = factions.get(entity).map(|faction| faction.team_id.max(0) as u32).unwrap_or(0);
        let scope = scopes.get(entity).copied().unwrap_or(ReplicationScope {
            kind: ReplicationScopeKind::Vision,
            owner_team: Some(team),
        });
        let mut baseline = Vec::with_capacity(28);
        baseline.extend_from_slice(&canonical_id.to_le_bytes());
        baseline.extend_from_slice(&position.0.x.raw().to_le_bytes());
        baseline.extend_from_slice(&position.0.y.raw().to_le_bytes());
        baseline.extend_from_slice(&team.to_le_bytes());
        CommittedEntityView {
            canonical_id,
            team,
            position: position.0,
            scope: scope.kind,
            owner_team: scope.owner_team,
            stealth_level: stealth.get(entity).map(|value| value.stealth_level).unwrap_or(0),
            overrides: overrides.get(entity).map(|rule| vec![CommittedVisibilityOverride {
                team: rule.team,
                kind: rule.kind,
                priority: rule.priority,
                stable_rule_id: rule.stable_rule_id,
            }]).unwrap_or_default(),
            remember: remembers.get(entity).map(|value| value.disposition).unwrap_or(RememberDisposition::Forget),
            disclosed_baseline: baseline,
        }
    }).collect();
    committed_entities.sort_by_key(|entity| entity.canonical_id);

    let mut vision_sources: Vec<_> = (&entities, &positions, &vision).join().map(|(entity, position, source)| {
        CommittedVisionSource {
            canonical_id: ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id()),
            team: source.team,
            position: position.0,
            radius: source.radius,
            detection_level: source.detection_level,
        }
    }).collect();
    vision_sources.sort_by_key(|source| (source.team, source.canonical_id));
    WaveBReadView { tick, entities: committed_entities.into(), vision_sources: vision_sources.into() }
}

/// Called only after Wave A outcome/fact reduction and `World::maintain`.
pub fn run_committed_visibility_wave_b(world: &mut World, tick: u64, delay: u64) {
    let view = build_wave_b_read_view(world, tick);
    let discovered_teams: BTreeSet<_> = view.entities.iter().map(|entity| entity.team)
        .chain(view.vision_sources.iter().map(|source| source.team)).collect();
    let mut runtime = world.write_resource::<TeamVisibilityRuntime>();
    for team in discovered_teams { runtime.ensure_team(team); }
    let mut states: Vec<_> = std::mem::take(&mut runtime.teams).into_values().collect();
    let results = run_team_wave_b_parallel(&view, &mut states, delay);
    runtime.teams = states.into_iter().map(|state| (state.team, state)).collect();
    runtime.last_transitions = results.into_iter().collect();
}

#[derive(Clone, Debug)]
pub struct WaveACommit<T> {
    pub tick: u64,
    pub ordered_outcomes: Vec<OrderedOutput<T>>,
    pub ordered_facts: Vec<OrderedFact>,
    pub barrier_reached: bool,
}

#[derive(Clone, Debug, Default)]
pub struct CommittedProjectionBatch {
    pub tick: u64,
    pub ordered_outcome_count: usize,
    pub facts: Vec<OrderedFact>,
    pub barrier_reached: bool,
}

pub fn commit_wave_a<T>(
    tick: u64,
    mut outcomes: Vec<OrderedOutput<T>>,
    mut facts: Vec<OrderedFact>,
) -> Result<WaveACommit<T>, StableOutputError> {
    for outcome in &outcomes { outcome.key.validate()?; }
    for fact in &facts { fact.key.validate()?; }
    outcomes.sort_by_key(|outcome| outcome.key);
    facts.sort();
    facts.dedup();
    Ok(WaveACommit { tick, ordered_outcomes: outcomes, ordered_facts: facts, barrier_reached: true })
}
