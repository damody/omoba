//! Two-wave selective visibility pipeline.
//!
//! Wave A commits ordered gameplay outputs. Wave B receives an immutable view
//! of State[T+1] and resolves each team independently (and in parallel).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use omoba_sim::{Fixed64, Vec2};
use rayon::prelude::*;

use crate::runtime::native::comp::{
    line_of_sight, LosResult, RememberDisposition, ReplicationScopeKind, VisibilityOverrideKind,
    VisionOccluder,
};
use crate::runtime::{OrderedFact, OrderedOutput, StableOutputError};
use specs::{Join, World, WorldExt};

pub const DEMO_RENDER_COMPONENT_SCHEMA_ID: u32 = 0x464f4701;
pub const DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID: u32 = 0x464f4702;
pub const DISCLOSED_DEMO_PATROL_COMPONENT_SCHEMA_ID: u32 = 0x464f4704;
pub const DISCLOSED_HERO_COMPONENT_SCHEMA_ID: u32 = 0x464f4705;
pub const DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID: u32 = 0x464f4706;
pub const DISCLOSED_FACING_COMPONENT_SCHEMA_ID: u32 = 0x464f4707;
pub const DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID: u32 = 0x464f4708;
pub const DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID: u32 = 0x464f4709;
pub const DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID: u32 = 0x464f470a;
pub const DISCLOSED_TOWER_COMPONENT_SCHEMA_ID: u32 = 0x464f470b;
pub const DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID: u32 = 0x464f470c;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DemoRenderState {
    pub x_raw: i64,
    pub y_raw: i64,
    pub team_id: u32,
    pub kind: u8,
    pub owner_player_id: u32,
}

pub fn encode_demo_render_state(state: DemoRenderState) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(25);
    bytes.extend_from_slice(&state.x_raw.to_be_bytes());
    bytes.extend_from_slice(&state.y_raw.to_be_bytes());
    bytes.extend_from_slice(&state.team_id.to_be_bytes());
    bytes.push(state.kind);
    bytes.extend_from_slice(&state.owner_player_id.to_be_bytes());
    bytes
}

pub fn decode_demo_render_state(bytes: &[u8]) -> Option<DemoRenderState> {
    if bytes.len() != 25 {
        return None;
    }
    Some(DemoRenderState {
        x_raw: i64::from_be_bytes(bytes[0..8].try_into().ok()?),
        y_raw: i64::from_be_bytes(bytes[8..16].try_into().ok()?),
        team_id: u32::from_be_bytes(bytes[16..20].try_into().ok()?),
        kind: bytes[20],
        owner_player_id: u32::from_be_bytes(bytes[21..25].try_into().ok()?),
    })
}

fn encode_disclosed_baseline(components: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(components.len() as u32).to_be_bytes());
    for (schema_id, value) in components {
        bytes.extend_from_slice(&schema_id.to_be_bytes());
        bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    bytes
}

pub fn encode_disclosed_property(property: &crate::runtime::CProperty) -> Vec<u8> {
    [
        property.hp.raw(),
        property.mhp.raw(),
        property.msd.raw(),
        property.def_physic.raw(),
        property.def_magic.raw(),
    ]
    .into_iter()
    .flat_map(i64::to_be_bytes)
    .collect()
}

pub fn encode_disclosed_demo_patrol(patrol: &crate::runtime::DemoPatrol) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(45);
    bytes.extend_from_slice(&patrol.stable_index.to_be_bytes());
    bytes.extend_from_slice(&patrol.endpoint_a.x.raw().to_be_bytes());
    bytes.extend_from_slice(&patrol.endpoint_a.y.raw().to_be_bytes());
    bytes.extend_from_slice(&patrol.endpoint_b.x.raw().to_be_bytes());
    bytes.extend_from_slice(&patrol.endpoint_b.y.raw().to_be_bytes());
    bytes.push(u8::from(patrol.target_b));
    bytes.extend_from_slice(&patrol.speed_per_tick.raw().to_be_bytes());
    bytes
}

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

/// Immutable, stable-sorted Wave B copy of the validated map occluder.
pub type CommittedVisionOccluder = VisionOccluder;

#[derive(Clone, Debug)]
pub struct WaveBReadView {
    pub tick: u64,
    pub entities: Arc<[CommittedEntityView]>,
    pub vision_sources: Arc<[CommittedVisionSource]>,
    pub vision_occluders: Arc<[CommittedVisionOccluder]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VisibilityTransition {
    Reveal {
        canonical_id: u64,
        effective_tick: u64,
        baseline: Vec<u8>,
    },
    Hide {
        canonical_id: u64,
        effective_tick: u64,
        disposition: RememberDisposition,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct VisibilityCandidate {
    desired_visible: bool,
    effective_tick: u64,
}

#[derive(Clone, Debug, Default)]
pub struct TeamVisibilityIndex {
    pub current: BTreeSet<u64>,
    candidates: BTreeMap<u64, VisibilityCandidate>,
}

#[derive(Clone, Debug)]
pub struct VisibilityHistoryEntry {
    pub tick: u64,
    pub visible: BTreeSet<u64>,
}

#[derive(Clone, Debug)]
pub struct TeamVisibilityHistory {
    capacity: usize,
    entries: VecDeque<VisibilityHistoryEntry>,
}

impl TeamVisibilityHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            entries: VecDeque::new(),
        }
    }
    pub fn push(&mut self, tick: u64, visible: &BTreeSet<u64>) {
        self.entries.push_back(VisibilityHistoryEntry {
            tick,
            visible: visible.clone(),
        });
        while self.entries.len() > self.capacity {
            self.entries.pop_front();
        }
    }
    pub fn was_visible(&self, tick: u64, entity: u64) -> bool {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.tick <= tick)
            .is_some_and(|entry| entry.visible.contains(&entity))
    }
    pub fn snapshot(&self) -> BTreeMap<u64, BTreeSet<u64>> {
        self.entries
            .iter()
            .map(|entry| (entry.tick, entry.visible.clone()))
            .collect()
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
        Self {
            team,
            index: TeamVisibilityIndex::default(),
            history: TeamVisibilityHistory::new(history_capacity),
        }
    }

    pub fn resolve(
        &mut self,
        view: &WaveBReadView,
        transition_delay_ticks: u64,
    ) -> Vec<VisibilityTransition> {
        let sources: Vec<_> = view
            .vision_sources
            .iter()
            .filter(|source| source.team == self.team)
            .copied()
            .collect();
        let mut transitions = Vec::new();
        for entity in view.entities.iter() {
            let desired =
                entity_visible_to_team(entity, self.team, &sources, &view.vision_occluders);
            let current = self.index.current.contains(&entity.canonical_id);
            if desired == current {
                self.index.candidates.remove(&entity.canonical_id);
                continue;
            }
            let effective_tick = view.tick.saturating_add(transition_delay_ticks);
            let candidate =
                self.index
                    .candidates
                    .entry(entity.canonical_id)
                    .or_insert(VisibilityCandidate {
                        desired_visible: desired,
                        effective_tick,
                    });
            if candidate.desired_visible != desired {
                *candidate = VisibilityCandidate {
                    desired_visible: desired,
                    effective_tick,
                };
            }
            if view.tick < candidate.effective_tick {
                continue;
            }
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
            VisibilityTransition::Reveal { canonical_id, .. }
            | VisibilityTransition::Hide { canonical_id, .. } => *canonical_id,
        });
        self.history.push(view.tick, &self.index.current);
        transitions
    }
}

fn entity_visible_to_team(
    entity: &CommittedEntityView,
    team: u32,
    sources: &[CommittedVisionSource],
    occluders: &[CommittedVisionOccluder],
) -> bool {
    // Deny rules have absolute precedence.
    if entity.scope == ReplicationScopeKind::ServerOnly {
        return false;
    }
    if resolve_override(&entity.overrides, team) == Some(VisibilityOverrideKind::ForceHide) {
        return false;
    }
    // Explicit/public grants precede owner and geometry checks.
    if entity.scope == ReplicationScopeKind::Public
        || resolve_override(&entity.overrides, team) == Some(VisibilityOverrideKind::ForceShow)
    {
        return true;
    }
    // 只有 OwnerTeam scope 才能讓擁有者無條件看見。Vision scope 即使是
    // 同隊單位也必須通過幾何視野，否則兩隊會各自直接取得整張地圖的同隊單位。
    if entity.scope == ReplicationScopeKind::OwnerTeam && entity.owner_team == Some(team) {
        return true;
    }
    sources.iter().any(|source| {
        if source.team != team {
            return false;
        }
        if source.detection_level < entity.stealth_level {
            return false;
        }
        let delta = entity.position - source.position;
        delta.length_squared() <= source.radius * source.radius
            && line_of_sight(occluders, source.position, entity.position) == LosResult::Clear
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(scope: ReplicationScopeKind, owner_team: Option<u32>, x: i32) -> CommittedEntityView {
        CommittedEntityView {
            canonical_id: 7,
            team: owner_team.unwrap_or_default(),
            position: Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO),
            scope,
            owner_team,
            stealth_level: 0,
            overrides: Vec::new(),
            remember: RememberDisposition::Forget,
            disclosed_baseline: Vec::new(),
        }
    }

    fn source(team: u32, radius: i32) -> CommittedVisionSource {
        CommittedVisionSource {
            canonical_id: 1,
            team,
            position: Vec2::new(Fixed64::ZERO, Fixed64::ZERO),
            radius: Fixed64::from_i32(radius),
            detection_level: 0,
        }
    }

    #[test]
    fn same_team_vision_entity_outside_radius_is_hidden() {
        let target = entity(ReplicationScopeKind::Vision, Some(1), 100);
        assert!(!entity_visible_to_team(&target, 1, &[source(1, 20)], &[]));
    }

    #[test]
    fn owner_team_hero_is_visible_to_owner_outside_radius() {
        let target = entity(ReplicationScopeKind::OwnerTeam, Some(1), 100);
        assert!(entity_visible_to_team(&target, 1, &[source(1, 20)], &[]));
    }

    #[test]
    fn owner_team_entity_can_be_revealed_to_enemy_by_geometry() {
        let target = entity(ReplicationScopeKind::OwnerTeam, Some(1), 10);
        assert!(entity_visible_to_team(&target, 2, &[source(2, 20)], &[]));
        assert!(!entity_visible_to_team(&target, 2, &[source(2, 5)], &[]));
    }

    #[test]
    fn one_clear_source_reveals_target_while_all_blocked_sources_hide_it() {
        use crate::runtime::native::comp::{VisionAabb, VisionTreeCircle};
        let target = entity(ReplicationScopeKind::Vision, Some(2), 10);
        let radius = Fixed64::from_i32(2);
        let center = Vec2::new(Fixed64::from_i32(5), Fixed64::ZERO);
        let extent = Vec2::new(radius, radius);
        let tree = VisionOccluder::Tree(VisionTreeCircle {
            stable_id: 1,
            center,
            radius,
            aabb: VisionAabb {
                min: center - extent,
                max: center + extent,
            },
        });
        assert!(!entity_visible_to_team(
            &target,
            1,
            &[source(1, 20)],
            &[tree.clone()]
        ));
        let mut second = source(1, 20);
        second.canonical_id = 2;
        second.position = Vec2::new(Fixed64::ZERO, Fixed64::from_i32(10));
        assert!(entity_visible_to_team(
            &target,
            1,
            &[source(1, 20), second],
            &[tree]
        ));
    }

    #[test]
    fn occlusion_changes_emit_canonical_forget_then_fresh_reveal() {
        use crate::runtime::native::comp::{VisionAabb, VisionTreeCircle};
        let target = entity(ReplicationScopeKind::Vision, Some(2), 10);
        let source = source(1, 20);
        let mut state = TeamVisibilityState::new(1, 8);
        let clear = WaveBReadView {
            tick: 1,
            entities: vec![target.clone()].into(),
            vision_sources: vec![source].into(),
            vision_occluders: Vec::new().into(),
        };
        assert!(state.resolve(&clear, 0).iter().any(|value| matches!(
            value,
            VisibilityTransition::Reveal {
                canonical_id: 7,
                ..
            }
        )));

        let radius = Fixed64::from_i32(2);
        let center = Vec2::new(Fixed64::from_i32(5), Fixed64::ZERO);
        let extent = Vec2::new(radius, radius);
        let blocked = WaveBReadView {
            tick: 2,
            entities: vec![target.clone()].into(),
            vision_sources: vec![source].into(),
            vision_occluders: vec![VisionOccluder::Tree(VisionTreeCircle {
                stable_id: 1,
                center,
                radius,
                aabb: VisionAabb {
                    min: center - extent,
                    max: center + extent,
                },
            })]
            .into(),
        };
        assert_eq!(
            state.resolve(&blocked, 0),
            vec![VisibilityTransition::Hide {
                canonical_id: 7,
                effective_tick: 2,
                disposition: RememberDisposition::Forget,
            }]
        );

        let clear_again = WaveBReadView { tick: 3, ..clear };
        assert!(state.resolve(&clear_again, 0).iter().any(|value| matches!(
            value,
            VisibilityTransition::Reveal {
                canonical_id: 7,
                effective_tick: 3,
                ..
            }
        )));
    }
}

fn resolve_override(
    overrides: &[CommittedVisibilityOverride],
    team: u32,
) -> Option<VisibilityOverrideKind> {
    let matching: Vec<_> = overrides
        .iter()
        .filter(|rule| rule.team.is_none() || rule.team == Some(team))
        .copied()
        .collect();
    let kind = if matching
        .iter()
        .any(|rule| rule.kind == VisibilityOverrideKind::ForceHide)
    {
        VisibilityOverrideKind::ForceHide
    } else if matching
        .iter()
        .any(|rule| rule.kind == VisibilityOverrideKind::ForceShow)
    {
        VisibilityOverrideKind::ForceShow
    } else {
        return None;
    };
    matching
        .into_iter()
        .filter(|rule| rule.kind == kind)
        .min_by_key(|rule| (-i32::from(rule.priority), rule.stable_rule_id))
        .map(|rule| rule.kind)
}

pub fn run_team_wave_b_parallel(
    view: &WaveBReadView,
    teams: &mut [TeamVisibilityState],
    transition_delay_ticks: u64,
) -> Vec<(u32, Vec<VisibilityTransition>)> {
    let mut results: Vec<_> = teams
        .par_iter_mut()
        .map(|state| (state.team, state.resolve(view, transition_delay_ticks)))
        .collect();
    results.sort_by_key(|(team, _)| *team);
    results
}

#[derive(Clone, Debug, Default)]
pub struct TeamVisibilityRuntime {
    pub teams: BTreeMap<u32, TeamVisibilityState>,
    pub last_transitions: BTreeMap<u32, Vec<VisibilityTransition>>,
    pub latest_owner_by_canonical: BTreeMap<u64, Option<u32>>,
    pub latest_demo_render_by_canonical: BTreeMap<u64, Vec<u8>>,
    pub latest_disclosed_baseline_by_canonical: BTreeMap<u64, Vec<u8>>,
}

impl TeamVisibilityRuntime {
    pub fn ensure_team(&mut self, team: u32) {
        self.teams
            .entry(team)
            .or_insert_with(|| TeamVisibilityState::new(team, 512));
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
    let units = world.read_storage::<Unit>();
    let heroes = world.read_storage::<Hero>();
    let owners = world.read_storage::<PlayerOwner>();
    let properties = world.read_storage::<CProperty>();
    let patrols = world.read_storage::<DemoPatrol>();
    let attacks = world.read_storage::<crate::runtime::TAttack>();
    let facings = world.read_storage::<crate::runtime::Facing>();
    let turn_speeds = world.read_storage::<crate::runtime::TurnSpeed>();
    let collision_radii = world.read_storage::<crate::runtime::CollisionRadius>();
    let inventories = world.read_storage::<crate::runtime::Inventory>();
    let towers = world.read_storage::<crate::runtime::Tower>();
    let script_tags = world.read_storage::<crate::runtime::ScriptUnitTag>();

    let mut committed_entities: Vec<_> = (&entities, &positions)
        .join()
        .map(|(entity, position)| {
            let canonical_id = ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id());
            let team = factions
                .get(entity)
                .map(|faction| faction.team_id.max(0) as u32)
                .unwrap_or(0);
            let scope = scopes.get(entity).copied().unwrap_or(ReplicationScope {
                kind: ReplicationScopeKind::Vision,
                owner_team: Some(team),
            });
            let render_state = DemoRenderState {
                x_raw: position.0.x.raw(),
                y_raw: position.0.y.raw(),
                team_id: team,
                kind: if heroes.get(entity).is_some() {
                    1
                } else if units.get(entity).is_some() {
                    2
                } else {
                    0
                },
                owner_player_id: owners.get(entity).map_or(0, |owner| owner.player_id),
            };
            let mut disclosed = vec![(
                DEMO_RENDER_COMPONENT_SCHEMA_ID,
                encode_demo_render_state(render_state),
            )];
            if let Some(property) = properties.get(entity) {
                disclosed.push((
                    DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
                    encode_disclosed_property(property),
                ));
            }
            if let Some(patrol) = patrols.get(entity) {
                disclosed.push((
                    DISCLOSED_DEMO_PATROL_COMPONENT_SCHEMA_ID,
                    encode_disclosed_demo_patrol(patrol),
                ));
            }
            macro_rules! disclose_json {
                ($storage:expr, $schema:expr) => {
                    if let Some(value) = $storage.get(entity) {
                        disclosed.push((
                            $schema,
                            serde_json::to_vec(value).expect("safe disclosed component"),
                        ));
                    }
                };
            }
            disclose_json!(heroes, DISCLOSED_HERO_COMPONENT_SCHEMA_ID);
            disclose_json!(attacks, DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID);
            disclose_json!(facings, DISCLOSED_FACING_COMPONENT_SCHEMA_ID);
            disclose_json!(turn_speeds, DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID);
            disclose_json!(
                collision_radii,
                DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID
            );
            disclose_json!(inventories, DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID);
            disclose_json!(script_tags, DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID);
            if let Some(tower) = towers.get(entity) {
                let mut safe = tower.clone();
                safe.nearby_creeps.clear();
                safe.block_creeps.clear();
                disclosed.push((
                    DISCLOSED_TOWER_COMPONENT_SCHEMA_ID,
                    serde_json::to_vec(&safe).expect("safe tower component"),
                ));
            }
            let baseline = encode_disclosed_baseline(&disclosed);
            CommittedEntityView {
                canonical_id,
                team,
                position: position.0,
                scope: scope.kind,
                owner_team: scope.owner_team,
                stealth_level: stealth
                    .get(entity)
                    .map(|value| value.stealth_level)
                    .unwrap_or(0),
                overrides: overrides
                    .get(entity)
                    .map(|rule| {
                        vec![CommittedVisibilityOverride {
                            team: rule.team,
                            kind: rule.kind,
                            priority: rule.priority,
                            stable_rule_id: rule.stable_rule_id,
                        }]
                    })
                    .unwrap_or_default(),
                remember: remembers
                    .get(entity)
                    .map(|value| value.disposition)
                    .unwrap_or(RememberDisposition::Forget),
                disclosed_baseline: baseline,
            }
        })
        .collect();
    committed_entities.sort_by_key(|entity| entity.canonical_id);

    let mut vision_sources: Vec<_> = (&entities, &positions, &vision)
        .join()
        .map(|(entity, position, source)| CommittedVisionSource {
            canonical_id: ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id()),
            team: source.team,
            position: position.0,
            radius: source.radius,
            detection_level: source.detection_level,
        })
        .collect();
    vision_sources.sort_by_key(|source| (source.team, source.canonical_id));
    let vision_occluders: Arc<[CommittedVisionOccluder]> =
        world.read_resource::<VisionOccluderSet>().0.clone().into();
    WaveBReadView {
        tick,
        entities: committed_entities.into(),
        vision_sources: vision_sources.into(),
        vision_occluders,
    }
}

/// Called only after Wave A outcome/fact reduction and `World::maintain`.
pub fn run_committed_visibility_wave_b(world: &mut World, tick: u64, delay: u64) {
    let view = build_wave_b_read_view(world, tick);
    let mut runtime = world.write_resource::<TeamVisibilityRuntime>();
    runtime.latest_owner_by_canonical = view
        .entities
        .iter()
        .map(|entity| (entity.canonical_id, entity.owner_team))
        .collect();
    runtime.latest_demo_render_by_canonical = view
        .entities
        .iter()
        .filter_map(|entity| {
            (entity.disclosed_baseline.len() >= 12).then(|| {
                let len = u32::from_be_bytes(entity.disclosed_baseline[8..12].try_into().unwrap())
                    as usize;
                (
                    entity.canonical_id,
                    entity.disclosed_baseline[12..12 + len].to_vec(),
                )
            })
        })
        .collect();
    runtime.latest_disclosed_baseline_by_canonical = view
        .entities
        .iter()
        .map(|entity| (entity.canonical_id, entity.disclosed_baseline.clone()))
        .collect();
    // This game mode is intentionally fixed to the two opposing LoL-style
    // teams. Neutral faction 0 never receives a disclosure stream.
    for team in crate::runtime::SUPPORTED_REPLICA_TEAMS {
        runtime.ensure_team(team);
    }
    let mut states: Vec<_> = std::mem::take(&mut runtime.teams).into_values().collect();
    let results = run_team_wave_b_parallel(&view, &mut states, delay);
    runtime.teams = states
        .into_iter()
        .map(|state| (state.team, state))
        .collect();
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
    for outcome in &outcomes {
        outcome.key.validate()?;
    }
    for fact in &facts {
        fact.key.validate()?;
    }
    outcomes.sort_by_key(|outcome| outcome.key);
    facts.sort();
    facts.dedup();
    Ok(WaveACommit {
        tick,
        ordered_outcomes: outcomes,
        ordered_facts: facts,
        barrier_reached: true,
    })
}
