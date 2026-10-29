//! Two-wave selective visibility pipeline.
//!
//! Wave A commits ordered gameplay outputs. Wave B receives an immutable view
//! of State[T+1] and resolves each team independently (and in parallel).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;

use omoba_sim::{Fixed64, Vec2};
use rayon::prelude::*;
use serde::Serialize;

use crate::runtime::native::comp::{
    IndexedVisionOccluders, LineOfSight, LosResult, RememberDisposition, ReplicationScopeKind,
    VisibilityOverrideKind, VisionOccluder,
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
pub const DISCLOSED_GOLD_COMPONENT_SCHEMA_ID: u32 = 0x464f470d;
pub const DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID: u32 = 0x464f470e;
/// Version 1: one signed Q10 incoming damage bonus, big endian.
pub const DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID: u32 = 0x464f470f;
/// Version 1: role (1 tower, 2 base), current damage admission (0 or 1).
pub const DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID: u32 = 0x464f4710;
/// Version 1 combat relation. 0x464f4703 stays unallocated. 0x464f4711 and
/// 0x464f4712 are visual schemas. Eight bytes: version, faction discriminant,
/// big-endian team id, attackable, alive. Absence is unknown, not team 0.
pub const DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID: u32 = 0x464f4713;
pub const DISCLOSED_COMBAT_RELATION_VERSION: u8 = 1;
/// Version 1 disclosed ward body: catalog, generation, direct policy, automatic
/// policy. Not a unit, and not private owner charge state.
pub const DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID: u32 = 0x464f4714;

/// Presentation template identity only. Does not bind or execute a script.
/// Version, ASCII byte length, then a bounded author-defined template name.
pub const DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID: u32 = 0x464f4715;

pub fn encode_disclosed_presentation_identity(id: &str) -> Option<Vec<u8>> {
    if id.is_empty() || id.len() > 96 || !id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
        return None;
    }
    let mut bytes = vec![1, id.len() as u8];
    bytes.extend_from_slice(id.as_bytes());
    Some(bytes)
}

pub fn decode_disclosed_presentation_identity(bytes: &[u8]) -> Option<&str> {
    if bytes.len() < 3 || bytes[0] != 1 || usize::from(bytes[1]) + 2 != bytes.len() {
        return None;
    }
    let id = std::str::from_utf8(&bytes[2..]).ok()?;
    encode_disclosed_presentation_identity(id)?;
    Some(id)
}

/// Disclosed faction/team/attackability. This is not owner identity and not a
/// minimap team. `faction_kind` is 1 Player, 2 Enemy, 3 Neutral, 4 Ally,
/// 5 HostileNeutral.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DisclosedCombatRelation {
    pub faction_kind: u8,
    pub team_id: i32,
    pub attackable: bool,
    pub alive: bool,
}

pub fn faction_kind_discriminant(kind: &crate::runtime::FactionType) -> Option<u8> {
    Some(match kind {
        crate::runtime::FactionType::Player => 1,
        crate::runtime::FactionType::Enemy => 2,
        crate::runtime::FactionType::Neutral => 3,
        crate::runtime::FactionType::Ally => 4,
        crate::runtime::FactionType::HostileNeutral => 5,
    })
}

pub fn faction_from_disclosed_kind(kind: u8, team_id: i32) -> Option<crate::runtime::Faction> {
    let faction_id = match kind {
        1 => crate::runtime::FactionType::Player,
        2 => crate::runtime::FactionType::Enemy,
        3 => crate::runtime::FactionType::Neutral,
        4 => crate::runtime::FactionType::Ally,
        5 => crate::runtime::FactionType::HostileNeutral,
        _ => return None,
    };
    Some(crate::runtime::Faction::new(faction_id, team_id))
}

/// Attackable only while the body is alive, a published structure admits
/// damage when that byte exists, and a jungle camp is not returning.
pub fn disclosed_attackable(alive: bool, structure_admission: Option<bool>, jungle_returning: bool) -> bool {
    alive && structure_admission.unwrap_or(true) && !jungle_returning
}

pub fn encode_disclosed_combat_relation(relation: DisclosedCombatRelation) -> Option<Vec<u8>> {
    if !matches!(relation.faction_kind, 1..=5) || (relation.attackable && !relation.alive) {
        return None;
    }
    let mut bytes = Vec::with_capacity(8);
    bytes.push(DISCLOSED_COMBAT_RELATION_VERSION);
    bytes.push(relation.faction_kind);
    bytes.extend_from_slice(&relation.team_id.to_be_bytes());
    bytes.push(u8::from(relation.attackable));
    bytes.push(u8::from(relation.alive));
    Some(bytes)
}

pub fn decode_disclosed_combat_relation(bytes: &[u8]) -> Option<DisclosedCombatRelation> {
    if bytes.len() != 8 || bytes[0] != DISCLOSED_COMBAT_RELATION_VERSION || !matches!(bytes[1], 1..=5) {
        return None;
    }
    if bytes[6] > 1 || bytes[7] > 1 { return None; }
    let attackable = bytes[6] == 1;
    let alive = bytes[7] == 1;
    if attackable && !alive { return None; }
    Some(DisclosedCombatRelation {
        faction_kind: bytes[1],
        team_id: i32::from_be_bytes(bytes[2..6].try_into().ok()?),
        attackable,
        alive,
    })
}

/// Presentation admission. Native entity id 0 is a different namespace and is
/// not a render id. A missing, malformed, dead, or non-hostile row refuses.
/// The lowest owned living hero with a valid combat component is the source.
pub fn disclosed_attack_target_admitted(
    entities: &[(u64, &BTreeMap<u32, Vec<u8>>)],
    source_player_id: u32,
    target_render_id: u64,
) -> bool {
    if source_player_id == 0 || target_render_id == 0 { return false; }
    let mut source: Option<(u64, DisclosedCombatRelation, bool)> = None;
    for (render_id, components) in entities {
        let Some((render, combat, hp_positive)) = disclosed_relation_row(components) else { continue };
        if render.kind != 1 || render.owner_player_id != source_player_id { continue; }
        let replace = source.as_ref().map(|(current, _, _)| render_id < current).unwrap_or(true);
        if replace { source = Some((*render_id, combat, hp_positive)); }
    }
    let Some((_, source_combat, source_hp)) = source else { return false; };
    if !source_combat.alive || !source_hp { return false; }
    let Some(source_faction) = faction_from_disclosed_kind(source_combat.faction_kind, source_combat.team_id) else {
        return false;
    };
    let Some((_, target_components)) = entities.iter().find(|(render_id, _)| *render_id == target_render_id) else {
        return false;
    };
    let Some((_, target_combat, target_hp)) = disclosed_relation_row(target_components) else { return false; };
    if !target_combat.alive || !target_combat.attackable || !target_hp { return false; }
    if let Some(bytes) = target_components.get(&DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID) {
        let Some(body) = crate::runtime::ward_transport::decode_disclosed_ward_body(bytes) else {
            return false;
        };
        if !crate::runtime::ward_transport::disclosed_direct_attack_allowed(body.direct_attack) {
            return false;
        }
    }
    faction_from_disclosed_kind(target_combat.faction_kind, target_combat.team_id)
        .is_some_and(|target_faction| source_faction.is_hostile_to(&target_faction))
}

fn disclosed_relation_row(components: &BTreeMap<u32, Vec<u8>>) -> Option<(DemoRenderState, DisclosedCombatRelation, bool)> {
    let render = decode_demo_render_state(components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID)?)?;
    let combat = decode_disclosed_combat_relation(components.get(&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID)?)?;
    let hp_positive = components.get(&DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID)
        .and_then(|bytes| decode_disclosed_health(bytes))
        .is_some_and(|(hp, _)| hp > 0);
    Some((render, combat, hp_positive))
}

pub fn decode_disclosed_structure(bytes: &[u8]) -> Option<(u8,bool)> {
    if bytes.len()!=2 || !matches!(bytes[0],1|2) || bytes[1]>1 {return None;}
    Some((bytes[0],bytes[1]==1))
}

pub(crate) fn decode_disclosed_incoming_damage(bytes: &[u8]) -> Option<Fixed64> {
    let raw = i64::from_be_bytes(bytes.try_into().ok()?);
    raw.checked_add(Fixed64::ONE.raw())?;
    Some(Fixed64::from_raw(raw))
}

pub(crate) fn canonical_disclosed_json<T: Serialize>(value: &T) -> Vec<u8> {
    fn sort_objects(value: serde_json::Value) -> serde_json::Value {
        match value {
            serde_json::Value::Array(values) => {
                serde_json::Value::Array(values.into_iter().map(sort_objects).collect())
            }
            serde_json::Value::Object(values) => {
                let mut entries: Vec<_> = values.into_iter().collect();
                entries.sort_by(|left, right| left.0.cmp(&right.0));
                let mut sorted = serde_json::Map::new();
                for (key, value) in entries {
                    sorted.insert(key, sort_objects(value));
                }
                serde_json::Value::Object(sorted)
            }
            scalar => scalar,
        }
    }

    let value = serde_json::to_value(value).expect("safe disclosed component");
    serde_json::to_vec(&sort_objects(value)).expect("safe disclosed component")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DemoRenderState {
    pub x_raw: i64,
    pub y_raw: i64,
    pub team_id: u32,
    /// Presentation category: 1 hero, 2 unit/creep, 3 neutral unit, 4 summon. Unknown values stay generic.
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

pub(crate) fn encode_disclosed_baseline(components: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(components.len() as u32).to_be_bytes());
    for (schema_id, value) in components {
        bytes.extend_from_slice(&schema_id.to_be_bytes());
        bytes.extend_from_slice(&(value.len() as u32).to_be_bytes());
        bytes.extend_from_slice(value);
    }
    bytes
}

/// Borrowed, bounded decoder for the committed baseline component framing.
/// Validate the entire frame before any caller uses a partial component.
pub fn disclosed_component<'a>(baseline: &'a [u8], schema: u32) -> Option<&'a [u8]> {
    decode_disclosed_components(baseline)?.remove(&schema)
}

pub(crate) fn decode_disclosed_components(bytes: &[u8]) -> Option<BTreeMap<u32, &[u8]>> {
    let count = u32::from_be_bytes(bytes.get(..4)?.try_into().ok()?) as usize;
    let mut rest = bytes.get(4..)?;
    if count > rest.len() / 8 { return None; }
    let mut result = BTreeMap::new();
    for _ in 0..count {
        let schema = u32::from_be_bytes(rest.get(..4)?.try_into().ok()?);
        let len = u32::from_be_bytes(rest.get(4..8)?.try_into().ok()?) as usize;
        rest = rest.get(8..)?;
        let value = rest.get(..len)?;
        if result.insert(schema,value).is_some() { return None; }
        rest = rest.get(len..)?;
    }
    rest.is_empty().then_some(result)
}

pub fn decode_disclosed_health(bytes: &[u8]) -> Option<(i64,i64)> {
    if bytes.len() != 40 { return None; }
    let hp = i64::from_be_bytes(bytes[..8].try_into().ok()?);
    let max_hp = i64::from_be_bytes(bytes[8..16].try_into().ok()?);
    (hp >= 0 && max_hp > 0 && hp <= max_hp).then_some((hp,max_hp))
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
/// The committed occluder list with its broad phase (E187 §5.5: the nobu map's trees).
pub type CommittedVisionOccluders = IndexedVisionOccluders;

#[derive(Clone, Debug)]
pub struct WaveBReadView {
    pub tick: u64,
    pub entities: Arc<[CommittedEntityView]>,
    pub vision_sources: Arc<[CommittedVisionSource]>,
    pub vision_occluders: CommittedVisionOccluders,
    pub brush_regions: crate::runtime::brush_vision::BrushVisionRegions,
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

/// One tick's committed visible set. Entries never change after `push`, so
/// snapshots and runtime clones share them instead of deep-copying 512 sets.
pub type SharedVisibleSet = Arc<BTreeSet<u64>>;

#[derive(Clone, Debug)]
pub struct VisibilityHistoryEntry {
    pub tick: u64,
    pub visible: SharedVisibleSet,
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
            visible: Arc::new(visible.clone()),
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
    pub fn snapshot(&self) -> BTreeMap<u64, SharedVisibleSet> {
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
    last_remember: BTreeMap<u64, RememberDisposition>,
    /// Entities this team's vision actually covers at the latest wave B,
    /// before input grace. Gameplay input admission may target only these.
    pub sighted: BTreeSet<u64>,
    /// Targets of this tick's admitted inputs. A currently disclosed target
    /// stays disclosed for this frame so the team's replica can resolve the
    /// same input the authority executed; it never chains (admission needs
    /// `sighted`, which grace does not extend).
    pub input_grace: BTreeSet<u64>,
    #[cfg(test)]
    pub(crate) witness_worker_count: usize,
    #[cfg(test)]
    pub(crate) witness_thread: Option<std::thread::ThreadId>,
}

impl TeamVisibilityState {
    pub fn new(team: u32, history_capacity: usize) -> Self {
        Self {
            team,
            index: TeamVisibilityIndex::default(),
            history: TeamVisibilityHistory::new(history_capacity),
            last_remember: BTreeMap::new(),
            sighted: BTreeSet::new(),
            input_grace: BTreeSet::new(),
            #[cfg(test)]
            witness_worker_count: 0,
            #[cfg(test)]
            witness_thread: None,
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
        self.sighted.clear();
        let grace = std::mem::take(&mut self.input_grace);
        let existing: BTreeSet<_> = view
            .entities
            .iter()
            .map(|entity| entity.canonical_id)
            .collect();
        // Deleted/generation-replaced entities no longer occur in the view loop.
        // Retire only a previously disclosed identity, using its existing memory
        // policy; absence must not disclose a hidden death or hidden coordinates.
        // A target of this tick's admitted input that was deleted during the
        // step (it died) stays disclosed for this one frame, like a target that
        // left sight: the team's replica still holds it when it executes the
        // same input, and it retires on the next resolve (E175).
        let absent: Vec<_> = self.index.current.difference(&existing).copied()
            .filter(|canonical_id| !grace.contains(canonical_id)).collect();
        for canonical_id in absent {
            self.index.current.remove(&canonical_id);
            self.index.candidates.remove(&canonical_id);
            transitions.push(VisibilityTransition::Hide {
                canonical_id,
                effective_tick: view.tick,
                disposition: self
                    .last_remember
                    .remove(&canonical_id)
                    .unwrap_or(RememberDisposition::Forget),
            });
        }
        for entity in view.entities.iter() {
            let sighted =
                entity_visible_with_brush(entity, self.team, &sources, &view.vision_occluders, &view.brush_regions);
            if sighted { self.sighted.insert(entity.canonical_id); }
            let current = self.index.current.contains(&entity.canonical_id);
            let desired = sighted || (current && grace.contains(&entity.canonical_id));
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
                self.last_remember
                    .insert(entity.canonical_id, entity.remember);
                transitions.push(VisibilityTransition::Reveal {
                    canonical_id: entity.canonical_id,
                    effective_tick: view.tick,
                    baseline: entity.disclosed_baseline.clone(),
                });
            } else {
                self.index.current.remove(&entity.canonical_id);
                self.last_remember.remove(&entity.canonical_id);
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

#[cfg(test)]
fn entity_visible_to_team(
    entity: &CommittedEntityView, team: u32, sources: &[CommittedVisionSource], occluders: &[CommittedVisionOccluder],
) -> bool { entity_visible_with_brush(entity, team, sources, occluders, &Default::default()) }

pub(crate) fn has_brush_reveal(entity:&CommittedEntityView,team:u32)->bool {
    entity.overrides.iter().any(|rule|rule.kind==VisibilityOverrideKind::RevealBrush
        && (rule.team.is_none() || rule.team==Some(team)))
}

pub(crate) fn entity_visible_with_brush(
    entity: &CommittedEntityView,
    team: u32,
    sources: &[CommittedVisionSource],
    occluders: &(impl LineOfSight + ?Sized),
    brushes: &crate::runtime::brush_vision::BrushVisionRegions,
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
            && (has_brush_reveal(entity,team)
                || crate::runtime::brush_vision::brush_allows_vision(brushes, source.position, entity.position))
            && occluders.line_of_sight(source.position, entity.position) == LosResult::Clear
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn presentation_identity_is_bounded_and_canonical() {
        use super::*;
        for id in ["single_lane_tower", "single_lane_base", "single_lane_creep", "custom_42"] {
            let bytes = encode_disclosed_presentation_identity(id).unwrap();
            assert_eq!(decode_disclosed_presentation_identity(&bytes), Some(id));
            for end in 0..bytes.len() { assert!(decode_disclosed_presentation_identity(&bytes[..end]).is_none()); }
            let mut extra = bytes.clone(); extra.push(0);
            assert!(decode_disclosed_presentation_identity(&extra).is_none());
        }
        for id in ["", "../escape", "not a template", "單位", "unit.path"] {
            assert!(encode_disclosed_presentation_identity(id).is_none());
        }
        assert!(encode_disclosed_presentation_identity(&"a".repeat(96)).is_some());
        assert!(encode_disclosed_presentation_identity(&"a".repeat(97)).is_none());
        assert!(decode_disclosed_presentation_identity(&[2, 1, b'a']).is_none());
    }
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
    fn brush_disclosure_respects_sources_detection_and_explicit_scope_grants() {
        use crate::runtime::brush_vision::BrushVisionRegion;
        let mut target=entity(ReplicationScopeKind::Vision,Some(2),15);
        let brushes=vec![BrushVisionRegion {min:Vec2::new(Fixed64::from_i32(10),Fixed64::from_i32(-5)),
            max:Vec2::new(Fixed64::from_i32(20),Fixed64::from_i32(5))}].into();
        let outside=source(1,100);
        assert!(!entity_visible_with_brush(&target,1,&[outside],&[],&brushes));
        let mut inside=outside;inside.position=target.position;
        assert!(entity_visible_with_brush(&target,1,&[inside],&[],&brushes));
        inside.team=2;
        assert!(!entity_visible_with_brush(&target,1,&[inside],&[],&brushes));
        inside.team=1;target.stealth_level=1;
        assert!(!entity_visible_with_brush(&target,1,&[inside],&[],&brushes));
        target.scope=ReplicationScopeKind::Public;
        assert!(entity_visible_with_brush(&target,1,&[outside],&[],&brushes));
        target.scope=ReplicationScopeKind::OwnerTeam;target.owner_team=Some(1);
        assert!(entity_visible_with_brush(&target,1,&[],&[],&brushes));
        target.scope=ReplicationScopeKind::ServerOnly;
        assert!(!entity_visible_with_brush(&target,1,&[inside],&[],&brushes));
    }

    #[test]
    fn same_team_vision_entity_outside_radius_is_hidden() {
        let target = entity(ReplicationScopeKind::Vision, Some(1), 100);
        assert!(!entity_visible_to_team(&target, 1, &[source(1, 20)], &[]));
    }

    #[test]
    fn compiled_wall_disclosure_blocks_then_reveals_from_another_friendly_source() {
        use omoba_template_ids::{MobaMapConst,MobaTerrainConst};
        const WALLS:[MobaTerrainConst;1]=[MobaTerrainConst {id:"custom_wall",min:(4,-3),max:(6,3)}];
        let map=MobaMapConst {terrain:&WALLS,..omoba_template_ids::MOBA_MAPS[0]};
        let occluders=crate::runtime::moba_map_layout::compiled_vision_occluders(&map);
        let target=entity(ReplicationScopeKind::Vision,Some(2),10);
        assert!(!entity_visible_to_team(&target,1,&[source(1,20)],&occluders.0));
        let mut friendly=source(1,20);friendly.position=target.position;
        assert!(entity_visible_to_team(&target,1,&[source(1,20),friendly],&occluders.0));
        friendly.team=2;
        assert!(!entity_visible_to_team(&target,1,&[source(1,20),friendly],&occluders.0));
        let mut public=target;public.scope=ReplicationScopeKind::Public;
        assert!(entity_visible_to_team(&public,1,&[],&occluders.0));
    }

    #[test]
    fn owner_team_hero_is_visible_to_owner_outside_radius() {
        let target = entity(ReplicationScopeKind::OwnerTeam, Some(1), 100);
        assert!(entity_visible_to_team(&target, 1, &[source(1, 20)], &[]));
    }

    #[test]
    fn input_grace_keeps_a_targeted_entity_one_frame_without_extending_sight() {
        // split18: an admitted input's target left vision that same tick and
        // its Forget reached the replica before the input; grace holds it once.
        let enemy = entity(ReplicationScopeKind::Vision, Some(2), 10);
        let mut state = TeamVisibilityState::new(1, 4);
        let view = |tick, sources: Vec<CommittedVisionSource>| WaveBReadView { tick, entities: vec![enemy.clone()].into(),
            vision_sources: sources.into(), brush_regions: Vec::new().into(), vision_occluders: vec![].into() };
        assert_eq!(state.resolve(&view(1, vec![source(1, 20)]), 0).len(), 1);
        assert!(state.sighted.contains(&7));
        state.input_grace.insert(7);
        assert!(state.resolve(&view(2, vec![]), 0).is_empty(), "graced target is not hidden this frame");
        assert!(state.index.current.contains(&7) && !state.sighted.contains(&7), "grace is not sight");
        assert!(state.input_grace.is_empty(), "grace lasts exactly one resolve");
        assert!(matches!(state.resolve(&view(3, vec![]), 0).as_slice(), [VisibilityTransition::Hide { canonical_id: 7, .. }]));
        state.input_grace.insert(7);
        assert!(state.resolve(&view(4, vec![]), 0).is_empty() && !state.index.current.contains(&7), "grace never reveals");
    }

    #[test]
    fn input_grace_keeps_a_target_deleted_during_the_step_one_frame() {
        // E175: an admitted AttackTarget/CastAbility named a creep that died in the
        // same step. Its immediate Hide reached the team's replica before the
        // input, the projector dropped the input, and only the authority cleared
        // the queue and cancelled the windup. Grace holds a deleted target once.
        let enemy = entity(ReplicationScopeKind::Vision, Some(2), 10);
        let mut state = TeamVisibilityState::new(1, 4);
        let view = |tick, entities: Vec<CommittedEntityView>| WaveBReadView { tick, entities: entities.into(),
            vision_sources: vec![source(1, 20)].into(), brush_regions: Vec::new().into(), vision_occluders: vec![].into() };
        assert_eq!(state.resolve(&view(1, vec![enemy.clone()]), 0).len(), 1);
        state.input_grace.insert(7);
        assert!(state.resolve(&view(2, vec![]), 0).is_empty(), "a graced deleted target is not hidden this frame");
        assert!(state.index.current.contains(&7));
        assert!(matches!(state.resolve(&view(3, vec![]), 0).as_slice(), [VisibilityTransition::Hide { canonical_id: 7, .. }]),
            "it retires on the next resolve");
        assert!(!state.index.current.contains(&7));
        // Without grace a deletion still hides at once.
        let mut state = TeamVisibilityState::new(1, 4);
        state.resolve(&view(1, vec![enemy]), 0);
        assert!(matches!(state.resolve(&view(2, vec![]), 0).as_slice(), [VisibilityTransition::Hide { canonical_id: 7, .. }]));
    }

    #[test]
    fn missing_disclosed_entity_keeps_its_existing_last_known_policy() {
        let mut target = entity(ReplicationScopeKind::OwnerTeam, Some(1), 100);
        target.remember = RememberDisposition::LastKnown;
        let mut state = TeamVisibilityState::new(1, 4);
        let mut view = WaveBReadView { tick: 1, entities: vec![target].into(),
            vision_sources: vec![].into(), brush_regions: Vec::new().into(), vision_occluders: vec![].into() };
        assert_eq!(state.resolve(&view, 0).len(), 1);
        view.tick = 2; view.entities = vec![].into();
        assert!(matches!(state.resolve(&view, 0).as_slice(),
            [VisibilityTransition::Hide { canonical_id: 7, disposition: RememberDisposition::LastKnown, .. }]));
        assert!(state.index.current.is_empty());
        assert!(state.resolve(&view, 0).is_empty());
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
            brush_regions: Vec::new().into(),
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
            brush_regions: Vec::new().into(),
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

    #[test]
    fn disclosed_combat_relation_codec_rejects_malformed_bytes_and_emits_only_real_factions() {
        use crate::runtime::*;
        use omoba_sim::{Fixed64, Vec2};
        use specs::{Builder, WorldExt};
        assert!(decode_disclosed_combat_relation(&[]).is_none());
        assert!(decode_disclosed_combat_relation(&[1, 1, 0, 0, 0, 1]).is_none());
        assert!(decode_disclosed_combat_relation(&[2, 1, 0, 0, 0, 1, 1, 1]).is_none());
        assert!(decode_disclosed_combat_relation(&[1, 0, 0, 0, 0, 1, 1, 1]).is_none());
        assert!(decode_disclosed_combat_relation(&[1, 6, 0, 0, 0, 1, 1, 1]).is_none());
        assert!(decode_disclosed_combat_relation(&[1, 2, 0, 0, 0, 2, 1, 0]).is_none());
        assert!(encode_disclosed_combat_relation(DisclosedCombatRelation {
            faction_kind: 2, team_id: 2, attackable: true, alive: false,
        }).is_none());
        assert!(!disclosed_attackable(false, Some(true), false));
        assert!(!disclosed_attackable(true, Some(false), false));
        assert!(!disclosed_attackable(true, Some(true), true));
        assert!(disclosed_attackable(true, None, false));
        let alive = encode_disclosed_combat_relation(DisclosedCombatRelation {
            faction_kind: 2, team_id: -3, attackable: true, alive: true,
        }).unwrap();
        assert_eq!(decode_disclosed_combat_relation(&alive), Some(DisclosedCombatRelation {
            faction_kind: 2, team_id: -3, attackable: true, alive: true,
        }));
        assert_eq!(
            faction_from_disclosed_kind(5, 0).unwrap().faction_id,
            FactionType::HostileNeutral
        );

        let mut world = specs::World::new();
        world.register::<Pos>();
        world.register::<Faction>();
        world.register::<ReplicationScope>();
        world.register::<StealthProfile>();
        world.register::<VisibilityOverride>();
        world.register::<RememberPolicy>();
        world.register::<VisionSource>();
        world.register::<Unit>();
        world.register::<Hero>();
        world.register::<PlayerOwner>();
        world.register::<CProperty>();
        world.register::<DemoPatrol>();
        world.register::<TAttack>();
        world.register::<Facing>();
        world.register::<TurnSpeed>();
        world.register::<CollisionRadius>();
        world.register::<Inventory>();
        world.register::<Gold>();
        world.register::<ItemEffects>();
        world.register::<Tower>();
        world.register::<ScriptUnitTag>();
        world.register::<PlacedWard>();
        world.insert(VisionOccluderSet::default());
        let property = |hp: i32| CProperty {
            hp: Fixed64::from_i32(hp),
            mhp: Fixed64::from_i32(10),
            msd: Fixed64::ZERO,
            def_physic: Fixed64::ZERO,
            def_magic: Fixed64::ZERO,
        };
        let place = |x: i32| Pos(Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO));
        let presentation_only = world.create_entity().with(place(1))
            .with(Unit::new("custom_public_unit".into(), "Public".into(), UnitType::Creep)).build();
        world.create_entity().with(place(2)).with(Faction::new(FactionType::Enemy, 2))
            .with(property(10)).build();
        world.create_entity().with(place(3)).with(Faction::new(FactionType::Ally, 1))
            .with(property(0)).build();
        let view = build_wave_b_read_view(&world, 4);
        assert_eq!(view.entities.len(), 3);
        let row = view.entities.iter().find(|row| row.canonical_id == crate::runtime::canonical_entity_id(presentation_only)).unwrap();
        let components = decode_disclosed_components(&row.disclosed_baseline).unwrap();
        assert_eq!(decode_disclosed_presentation_identity(&components[&DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID]), Some("custom_public_unit"));
        assert!(!components.contains_key(&DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID));
        assert!(world.read_storage::<ScriptUnitTag>().get(presentation_only).is_none(), "read view cannot enable scripting");
        let mut saw_unfactioned = false;
        let mut saw_enemy = false;
        let mut saw_dead_ally = false;
        for entity in view.entities.iter() {
            let components = decode_disclosed_components(&entity.disclosed_baseline).unwrap();
            let render = decode_demo_render_state(
                components.get(&DEMO_RENDER_COMPONENT_SCHEMA_ID).copied().unwrap(),
            ).unwrap();
            let combat = components.get(&DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID).copied()
                .map(|bytes| decode_disclosed_combat_relation(bytes).unwrap());
            match render.x_raw {
                raw if raw == Fixed64::from_i32(1).raw() => {
                    assert!(combat.is_none(), "no faction stays unknown");
                    saw_unfactioned = true;
                }
                raw if raw == Fixed64::from_i32(2).raw() => {
                    assert_eq!(combat, Some(DisclosedCombatRelation {
                        faction_kind: 2, team_id: 2, attackable: true, alive: true,
                    }));
                    saw_enemy = true;
                }
                raw if raw == Fixed64::from_i32(3).raw() => {
                    assert_eq!(combat, Some(DisclosedCombatRelation {
                        faction_kind: 4, team_id: 1, attackable: false, alive: false,
                    }));
                    saw_dead_ally = true;
                }
                other => panic!("unexpected disclosed x {other}"),
            }
        }
        assert!(saw_unfactioned && saw_enemy && saw_dead_ally);
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
        .map(|state| {
            #[cfg(test)]
            {
                state.witness_worker_count = rayon::current_num_threads();
                state.witness_thread = Some(std::thread::current().id());
            }
            (state.team, state.resolve(view, transition_delay_ticks))
        })
        .collect();
    results.sort_by_key(|(team, _)| *team);
    results
}

#[derive(Clone, Debug, Default)]
pub struct TeamVisibilityRuntime {
    /// Authority-only immutable view, reused by presentation sampling after
    /// the barrier. Never serialized as client metadata.
    pub latest_read_view: Option<Arc<WaveBReadView>>,
    pub fog_geometry: Option<crate::runtime::fog_grid::FogGridGeometry>,
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
    let gold = world.read_storage::<crate::runtime::Gold>();
    let item_effects = world.read_storage::<crate::runtime::ItemEffects>();
    let match_state = world.try_fetch::<crate::runtime::MobaMatch>();
    let moba = match_state.is_some();
    let paused = world.try_fetch::<crate::runtime::GamePause>().is_some_and(|pause| pause.is_paused);
    let unit_attack_states = match_state.as_ref().map(|state| state.disclosed_unit_attack_states(paused)).unwrap_or_default();
    let attack_ranges=if moba {crate::runtime::attack_visual_state::resolved_attack_ranges(world)}else{BTreeMap::new()};
    let structures = match_state.as_ref().map(|state| state.disclosed_structure_states()).unwrap_or_default();
    let channels = match_state.as_ref().map(|state| state.disclosed_channel_states()).unwrap_or_default();
    let buffs = world.try_fetch::<crate::runtime::BuffStore>();
    let towers = world.read_storage::<crate::runtime::Tower>();
    let script_tags = world.read_storage::<crate::runtime::ScriptUnitTag>();
    let placed_wards = world.read_storage::<crate::runtime::PlacedWard>();
    let ability_projectiles = world.try_fetch::<crate::runtime::native::ability_projectile::AbilityProjectiles>();

    let mut committed_entities: Vec<_> = (&entities, &positions)
        .join()
        .map(|(entity, position)| {
            let canonical_id = ((entity.gen().id() as u32 as u64) << 32) | u64::from(entity.id());
            // Ability projectiles carry their launch team; flight state stays private.
            let flight = ability_projectiles.as_ref().and_then(|rows| rows.0.get(&entity));
            let team = flight.map(|state| state.team).unwrap_or_else(|| factions
                .get(entity)
                .map(|faction| faction.team_id.max(0) as u32)
                .unwrap_or(0));
            let scope = scopes.get(entity).copied().unwrap_or(ReplicationScope {
                kind: ReplicationScopeKind::Vision,
                owner_team: Some(team),
            });
            let render_state = DemoRenderState {
                x_raw: position.0.x.raw(),
                y_raw: position.0.y.raw(),
                team_id: team,
                kind: if flight.is_some() {
                    crate::runtime::native::ability_projectile::RENDER_KIND
                } else if heroes.get(entity).is_some() {
                    1
                } else if units.get(entity).is_some_and(|unit|matches!(unit.unit_type,UnitType::Summon)) {
                    4
                } else if units.get(entity).is_some() {
                    if factions.get(entity).is_some_and(|f| f.faction_id == FactionType::HostileNeutral) { 3 } else { 2 }
                } else {
                    0
                },
                owner_player_id: owners.get(entity).map_or(0, |owner| owner.player_id),
            };
            let mut disclosed = vec![(
                DEMO_RENDER_COMPONENT_SCHEMA_ID,
                encode_demo_render_state(render_state),
            )];
            if let Some(state)=structures.get(&canonical_id) {
                disclosed.push((DISCLOSED_STRUCTURE_COMPONENT_SCHEMA_ID,state.to_vec()));
            }
            if let Some(state) = flight {
                use crate::runtime::native::ability_projectile as projectile;
                disclosed.push((projectile::DISCLOSED_SCHEMA_ID,
                    projectile::encode_disclosed_flight(projectile::disclosed_state(state))));
                if let Some(bytes) = encode_disclosed_presentation_identity(&state.spec.ability) {
                    disclosed.push((DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID, bytes));
                }
            }
            if let Some(faction) = factions.get(entity) {
                if let Some(faction_kind) = faction_kind_discriminant(&faction.faction_id) {
                    let alive = properties.get(entity).is_some_and(|property| property.hp > Fixed64::ZERO);
                    let attackable = match match_state.as_ref() {
                        Some(state) => alive && state.current_match_target_attackable(entity),
                        None => disclosed_attackable(
                            alive,
                            structures.get(&canonical_id).map(|state| state[1] == 1),
                            false,
                        ),
                    };
                    let relation = DisclosedCombatRelation {
                        faction_kind,
                        team_id: faction.team_id,
                        attackable,
                        alive,
                    };
                    if let Some(bytes) = encode_disclosed_combat_relation(relation) {
                        disclosed.push((DISCLOSED_COMBAT_RELATION_COMPONENT_SCHEMA_ID, bytes));
                    }
                }
            }
            if let Some(ward) = placed_wards.get(entity) {
                let generation = entity.gen().id() as u32;
                let policy = match_state.as_ref().and_then(|state| state.ward_policies.get(&canonical_id));
                let (direct, automatic) = if let Some(policy) = policy {
                    (
                        crate::runtime::ward_transport::direct_attack_token(policy.direct_attack),
                        crate::runtime::ward_transport::automatic_acquisition_token(policy.automatic_acquisition),
                    )
                } else if let Some(row) = omoba_template_ids::TRINKET_CATALOG.iter().find(|row| row.catalog_id == ward.catalog_id) {
                    (
                        crate::runtime::ward_transport::direct_attack_token(row.direct_attack),
                        crate::runtime::ward_transport::automatic_acquisition_token(row.automatic_acquisition),
                    )
                } else {
                    (0, 0)
                };
                if let Some(bytes) = crate::runtime::ward_transport::encode_disclosed_ward_body(
                    ward.catalog_id, generation, direct, automatic,
                ) {
                    disclosed.push((DISCLOSED_WARD_BODY_COMPONENT_SCHEMA_ID, bytes));
                }
            }
            if let Some(property) = properties.get(entity) {
                disclosed.push((
                    DISCLOSED_PROPERTY_COMPONENT_SCHEMA_ID,
                    encode_disclosed_property(property),
                ));
                if moba {
                    if let Some(buffs) = &buffs {
                        let bonus = crate::runtime::ability_runtime::unit_stats::damage_taken_bonus(buffs, entity);
                        disclosed.push((DISCLOSED_INCOMING_DAMAGE_COMPONENT_SCHEMA_ID,
                            bonus.raw().to_be_bytes().to_vec()));
                    }
                }
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
                        disclosed.push(($schema, canonical_disclosed_json(value)));
                    }
                };
            }
            disclose_json!(heroes, DISCLOSED_HERO_COMPONENT_SCHEMA_ID);
            disclose_json!(attacks, DISCLOSED_ATTACK_COMPONENT_SCHEMA_ID);
            if moba {if let Some(range)=attack_ranges.get(&canonical_id){disclosed.push((crate::runtime::attack_visual_state::RANGE_SCHEMA_ID,crate::runtime::attack_visual_state::encode_range(*range).expect("bounded resolved range")));}}
            disclose_json!(facings, DISCLOSED_FACING_COMPONENT_SCHEMA_ID);
            disclose_json!(turn_speeds, DISCLOSED_TURN_SPEED_COMPONENT_SCHEMA_ID);
            disclose_json!(
                collision_radii,
                DISCLOSED_COLLISION_RADIUS_COMPONENT_SCHEMA_ID
            );
            disclose_json!(inventories, DISCLOSED_INVENTORY_COMPONENT_SCHEMA_ID);
            if moba && heroes.get(entity).is_some() {
                disclose_json!(gold, DISCLOSED_GOLD_COMPONENT_SCHEMA_ID);
                disclose_json!(item_effects, DISCLOSED_ITEM_EFFECTS_COMPONENT_SCHEMA_ID);
            }
            disclose_json!(script_tags, DISCLOSED_SCRIPT_UNIT_TAG_COMPONENT_SCHEMA_ID);
            // Unit identity is already public for this disclosed entity. Do not
            // synthesize ScriptUnitTag: that component also enables callbacks.
            let presentation_id = script_tags.get(entity).map(|tag| tag.unit_id.as_str())
                .filter(|id| !id.is_empty())
                .or_else(|| units.get(entity).map(|unit| unit.id.as_str()));
            if let Some(bytes) = presentation_id.and_then(encode_disclosed_presentation_identity) {
                disclosed.push((DISCLOSED_PRESENTATION_IDENTITY_SCHEMA_ID, bytes));
            }
            if moba && (heroes.get(entity).is_some() || attacks.get(entity).is_some() || unit_attack_states.contains_key(&crate::runtime::canonical_entity_id(entity))) {
                let state = unit_attack_states.get(&crate::runtime::canonical_entity_id(entity)).copied()
                    .unwrap_or_else(|| attacks.get(entity).map(crate::runtime::attack_visual_state::AttackVisualState::from_attack).unwrap_or_default());
                disclosed.push((crate::runtime::attack_visual_state::SCHEMA_ID, state.encode().expect("bounded resolved attack timing")));
                let channel = channels.get(&canonical_id).copied().unwrap_or_default();
                disclosed.push((crate::runtime::channel_visual_state::SCHEMA_ID, channel.encode()));
            }
            if moba && heroes.get(entity).is_some() && owners.get(entity).is_some() && team != 0 {
                let state=buffs.as_ref().map(|store|crate::runtime::buff_visual_state::BuffVisualSnapshot::from_store(store,entity))
                    .unwrap_or(crate::runtime::buff_visual_state::BuffVisualSnapshot {buffs:crate::runtime::buff_visual_state::BuffVisualState(vec![]),channel_totals:vec![]});
                disclosed.push((crate::runtime::buff_visual_state::SCHEMA_ID,state.encode().expect("bounded generated buff IDs")));
            }
            // Ground zones, tethers and the illusion flag (runtime_fact_state); the team projector
            // re-sends them per team and never forwards this component to a replica.
            if moba {
                if let Some(state)=buffs.as_ref().and_then(|store|crate::runtime::runtime_fact_state::RuntimeFactState::from_store(store,entity)) {
                    disclosed.push((crate::runtime::runtime_fact_state::SCHEMA_ID,state.encode().expect("bounded runtime facts")));
                }
            }
            if let Some(tower) = towers.get(entity) {
                let mut safe = tower.clone();
                safe.nearby_creeps.clear();
                safe.block_creeps.clear();
                disclosed.push((
                    DISCLOSED_TOWER_COMPONENT_SCHEMA_ID,
                    canonical_disclosed_json(&safe),
                ));
            }
            let baseline = encode_disclosed_baseline(&disclosed);
            let mut entity_overrides:Vec<_>=overrides.get(entity).map(|rule|CommittedVisibilityOverride {
                team:rule.team,kind:rule.kind,priority:rule.priority,stable_rule_id:rule.stable_rule_id,
            }).into_iter().collect();
            if properties.get(entity).is_some_and(|p|p.hp>Fixed64::ZERO) {
                if let Some(state)=match_state.as_ref() {
                    entity_overrides.extend(state.brush_reveal_teams(entity).map(|team|CommittedVisibilityOverride {
                        team:Some(team),kind:VisibilityOverrideKind::RevealBrush,priority:0,stable_rule_id:canonical_id,
                    }));
                }
            }
            CommittedEntityView {
                canonical_id,
                team,
                position: position.0,
                scope: scope.kind,
                owner_team: scope.owner_team,
                stealth_level: stealth
                    .get(entity)
                    .map(|value| value.stealth_level)
                    .unwrap_or(0)
                    .max(buffs.as_ref().map_or(0, |buffs| buffs.stealth_level(entity))),
                overrides: entity_overrides,
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
    // G-4 reveal zones: team sight; true sight detects every stealth level,
    // a plain zone (原作 AddFOWViewer) detects none.
    if let Some(zones) = world.try_fetch::<crate::runtime::native::reveal_zone::RevealZones>() {
        vision_sources.extend(zones.sources().map(|(canonical_id, zone)| CommittedVisionSource {
            canonical_id, team: zone.team, position: zone.center, radius: zone.radius, detection_level: zone.detection_level,
        }));
    }
    vision_sources.sort_by_key(|source| (source.team, source.canonical_id));
    // E187 §5.5: the indexed form, built once per distinct occluder set.
    let vision_occluders = world.read_resource::<VisionOccluderSet>().committed();
    WaveBReadView {
        tick,
        entities: committed_entities.into(),
        vision_sources: vision_sources.into(),
        vision_occluders,
        brush_regions: world.try_fetch::<crate::runtime::brush_vision::BrushVisionRegions>()
            .map_or_else(Default::default, |regions| (*regions).clone()),
    }
}

/// Called only after Wave A outcome/fact reduction and `World::maintain`.
pub fn run_committed_visibility_wave_b(world: &mut World, tick: u64, delay: u64) {
    let view = build_wave_b_read_view(world, tick);
    let mut grace: BTreeMap<u32, BTreeSet<u64>> = BTreeMap::new();
    if let Some(projection) = world.try_fetch::<crate::runtime::TeamProjectionRuntime>() {
        for input in &projection.pending_accepted_inputs {
            if let Some(target) = input.target_canonical_id { grace.entry(input.team_id).or_default().insert(target); }
        }
    }
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
    for (team, targets) in grace {
        if let Some(state) = runtime.teams.get_mut(&team) { state.input_grace = targets; }
    }
    let mut states: Vec<_> = std::mem::take(&mut runtime.teams).into_values().collect();
    let results = run_team_wave_b_parallel(&view, &mut states, delay);
    runtime.teams = states
        .into_iter()
        .map(|state| (state.team, state))
        .collect();
    runtime.last_transitions = results.into_iter().collect();
    runtime.latest_read_view = Some(Arc::new(view));
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
