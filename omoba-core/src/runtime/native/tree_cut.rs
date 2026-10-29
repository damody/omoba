//! Destructible map trees (E187 §5.5, E242). A script cuts the trees around a point by
//! sending its own caster the request envelope `add_stat_buff(caster, BUFF, _, {KEY: ..})`
//! (v3 `destroy_trees`, 原作 `GridNav:DestroyTreesAroundPoint(point, radius, false)`).
//!
//! Flow:
//! 1. Request: settlement turns the envelope into a pending cut on the authority
//!    ([`PendingTreeCuts`], installed only on a map with trees). Replicas have no queue
//!    and drop it, so a request never touches replica state.
//! 2. Apply: the next active match tick, before any movement (pre-step), every standing
//!    tree whose point lies within the radius falls; trees whose regrow time came stand
//!    again unless a unit body overlaps them. Both rebuild the collision regions and the
//!    vision trees from the compiled map minus the fallen set, in authored order.
//! 3. Disclose: each change is one public pre-step HUD fact ([`CUT_METRIC_ID`] /
//!    [`REGROW_METRIC_ID`], value = tree index) to every team. Replicas apply them at the
//!    start of the same step ([`apply_disclosed`]), so authority and replicas walk the
//!    same terrain from the same tick. A mid-match bootstrap carries the current regions,
//!    from which the fallen set is recovered ([`cut_set_of`]).
//!
//! Regrowth: 300 s after the cut (原作 libraries/buildinghelper.lua:129 TreeRegrowTime,
//! never changed by the addon). Dota pushes units out of a regrown tree; here the tree
//! waits while a unit body stands on its square (inferred, keeps every unit free).
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Join, World, WorldExt};
use std::collections::{BTreeMap, BTreeSet};

use super::comp::{BlockedRegions, Pos, VisionOccluder, VisionOccluderSet};

pub const KEY: &str = "__tree_cut_v1";
/// Request envelope id; never stored as a buff.
pub const BUFF: &str = "__destroy_trees";
pub const MAX_RADIUS: i32 = 2_000;
/// 原作 BuildingHelper.TreeRegrowTime (libraries/buildinghelper.lua:129).
pub const REGROW_SECONDS: i32 = 300;
/// Public pre-step HUD metrics (team 0, value = tree index).
pub const CUT_METRIC_ID: u64 = 0x4d54524545435554;
pub const REGROW_METRIC_ID: u64 = 0x4d54524545524757;
/// Local ordinals of the tree facts: base + 2 × index (+1 for a regrowth).
const ORDINAL_BASE: u32 = 0x4000_0000;
/// Clearance around a tree square a unit body keeps a regrowth waiting (inferred).
const BODY_CLEARANCE: i32 = 32;
/// Pending requests kept per tick; more in one tick are dropped (none in the source).
const MAX_PENDING: usize = 64;

/// The request payload of `destroy_trees(at, radius)`.
pub fn payload(at: Vec2, radius: Fixed64) -> Value {
    json!({KEY: {"x": at.x.raw(), "y": at.y.raw(), "radius": radius.raw()}})
}

fn decode(payload: &Value) -> Option<(Vec2, Fixed64)> {
    let row = payload.get(KEY)?;
    let raw = |key: &str| row.get(key).and_then(Value::as_i64);
    let max = omoba_sim::terrain::MAX_GEOMETRY_RAW;
    let (x, y, radius) = (raw("x")?, raw("y")?, raw("radius")?);
    (x.abs() <= max && y.abs() <= max && radius > 0 && radius <= i64::from(MAX_RADIUS) * omoba_sim::fixed::SCALE)
        .then(|| (Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)), Fixed64::from_raw(radius)))
}

/// Authority queue of requests settled since the last applied tick.
#[derive(Clone, Debug, Default)]
pub struct PendingTreeCuts(pub Vec<(Vec2, Fixed64)>);

/// Settlement of the request envelope: queued on the authority, dropped elsewhere.
pub fn request(world: &World, payload: &Value) {
    let Some(cut) = decode(payload) else { return };
    if let Some(mut pending) = world.try_fetch_mut::<PendingTreeCuts>() {
        if pending.0.len() < MAX_PENDING { pending.0.push(cut); }
    }
}

/// The fallen trees of one match, with the time each stands again.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TreeCuts {
    pub regrow_at: BTreeMap<u32, Fixed64>,
}

impl TreeCuts {
    pub fn is_empty(&self) -> bool { self.regrow_at.is_empty() }
    pub fn fallen(&self) -> BTreeSet<u32> { self.regrow_at.keys().copied().collect() }
}

/// Collision regions of `map` without the fallen trees (authored order kept).
pub fn regions(map: &omoba_template_ids::MobaMapConst, fallen: &BTreeSet<u32>) -> BlockedRegions {
    let mut regions = crate::runtime::moba_map_layout::compiled_blocked_regions(map);
    if !fallen.is_empty() {
        let names: BTreeSet<String> = fallen.iter().map(|index| crate::runtime::moba_map_layout::tree_region_name(*index as usize)).collect();
        regions.0.retain(|region| !names.contains(&region.name));
    }
    regions
}

/// Vision occluders of `map` without the fallen trees (a tree's stable id is its index + 1).
pub fn occluders(map: &omoba_template_ids::MobaMapConst, fallen: &BTreeSet<u32>) -> VisionOccluderSet {
    let mut set = crate::runtime::moba_map_layout::compiled_vision_occluders(map);
    if !fallen.is_empty() {
        set.0.retain(|occluder| !matches!(occluder, VisionOccluder::Tree(tree) if fallen.contains(&((tree.stable_id - 1) as u32))));
    }
    set
}

/// The fallen set `regions` stands for: the compiled regions of `map` minus some of its
/// trees, in authored order. `None` for anything else.
pub fn cut_set_of(map: &omoba_template_ids::MobaMapConst, regions: &BlockedRegions) -> Option<BTreeSet<u32>> {
    let compiled = crate::runtime::moba_map_layout::compiled_blocked_regions(map);
    let trees = map.trees().trees.len();
    let mut fallen = BTreeSet::new();
    let mut given = regions.0.iter().peekable();
    for (index, region) in compiled.0.iter().enumerate() {
        let same = given.peek().is_some_and(|next| next.name == region.name && next.points == region.points);
        if same { given.next(); continue; }
        // Only trees can be missing; they follow the terrain in authored order.
        let tree = index.checked_sub(map.terrain.len()).filter(|tree| *tree < trees)?;
        fallen.insert(tree as u32);
    }
    given.next().is_none().then_some(fallen)
}

fn tree_point(map: &omoba_template_ids::MobaMapConst, index: u32) -> Vec2 {
    let (x, y) = map.trees().trees[index as usize].position;
    Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y))
}

/// Authority pre-step: apply the queued cuts and the due regrowths, rebuild the terrain
/// and the vision trees when anything changed and disclose each change to every team.
pub(crate) fn tick(world: &World, cuts: &mut TreeCuts, map: &omoba_template_ids::MobaMapConst, elapsed: Fixed64) {
    let requests = world.try_fetch_mut::<PendingTreeCuts>().map(|mut pending| std::mem::take(&mut pending.0)).unwrap_or_default();
    if requests.is_empty() && !cuts.regrow_at.values().any(|at| elapsed >= *at) { return; }
    let trees = map.trees();
    let mut fallen = Vec::new();
    for (at, radius) in requests {
        let r2 = radius * radius;
        for index in 0..trees.trees.len() as u32 {
            if cuts.regrow_at.contains_key(&index) || (tree_point(map, index) - at).length_squared() > r2 { continue; }
            cuts.regrow_at.insert(index, elapsed + Fixed64::from_i32(REGROW_SECONDS));
            fallen.push(index);
        }
    }
    let due: Vec<u32> = cuts.regrow_at.iter().filter(|(_, at)| elapsed >= **at).map(|(index, _)| *index).collect();
    let mut regrown = Vec::new();
    if !due.is_empty() {
        let bodies: Vec<Vec2> = (&world.read_storage::<Pos>(), &world.read_storage::<super::comp::CProperty>()).join()
            .filter(|(_, property)| property.hp > Fixed64::ZERO).map(|(pos, _)| pos.0).collect();
        let reach = Fixed64::from_i32(trees.radius + BODY_CLEARANCE);
        for index in due {
            let point = tree_point(map, index);
            if bodies.iter().any(|body| (body.x - point.x).raw().abs() <= reach.raw() && (body.y - point.y).raw().abs() <= reach.raw()) { continue; }
            cuts.regrow_at.remove(&index);
            regrown.push(index);
        }
    }
    if fallen.is_empty() && regrown.is_empty() { return; }
    let set = cuts.fallen();
    *world.write_resource::<BlockedRegions>() = regions(map, &set);
    *world.write_resource::<VisionOccluderSet>() = occluders(map, &set);
    let tick = world.read_resource::<crate::runtime::Tick>().0;
    let facts = world.read_resource::<crate::runtime::ObservableFactBuffer>();
    for (index, metric_id, odd) in fallen.into_iter().map(|index| (index, CUT_METRIC_ID, 0))
        .chain(regrown.into_iter().map(|index| (index, REGROW_METRIC_ID, 1))) {
        facts.emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey { tick, phase: crate::runtime::FactPhase::PreStep, canonical_source_order: 0,
                local_ordinal: ORDINAL_BASE + index * 2 + odd, fact_kind: crate::runtime::FactKind::Hud },
            audience: crate::runtime::FactAudience::AllPlayers,
            fact: crate::runtime::ObservableFact::Hud { team: 0, metric_id, value: i64::from(index) },
        }).expect("valid public tree change");
    }
}

/// Replica: the map and the fallen set it walks with.
#[derive(Clone, Debug)]
pub struct ReplicaTrees {
    pub map: &'static omoba_template_ids::MobaMapConst,
    pub fallen: BTreeSet<u32>,
}

/// Replica bootstrap: a compiled map with trees, from the regions the start carried.
pub fn replica_trees(map: Option<&'static omoba_template_ids::MobaMapConst>, regions: &BlockedRegions) -> Option<ReplicaTrees> {
    let map = map.filter(|map| !map.trees().trees.is_empty())?;
    Some(ReplicaTrees { map, fallen: cut_set_of(map, regions)? })
}

/// Replica step start: apply the disclosed tree changes of this step (HUD payload: team u32,
/// metric u64, value i64, little endian). A replica started without the map's terrain (no
/// [`ReplicaTrees`]) walks without trees and ignores them; one with it errors on a change it
/// cannot hold.
pub fn apply_disclosed(world: &World, payloads: &[&[u8]]) -> Result<(), &'static str> {
    let changes: Vec<(bool, u32)> = payloads.iter().filter(|bytes| bytes.len() == 20 && bytes[0..4] == 0_u32.to_le_bytes())
        .filter_map(|bytes| {
            let metric = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
            let value = i64::from_le_bytes(bytes[12..20].try_into().unwrap());
            (metric == CUT_METRIC_ID || metric == REGROW_METRIC_ID).then_some((metric == CUT_METRIC_ID, value))
        })
        .map(|(cut, value)| u32::try_from(value).map(|index| (cut, index)).map_err(|_| "invalid tree index"))
        .collect::<Result<_, _>>()?;
    if changes.is_empty() { return Ok(()); }
    let Some(mut trees) = world.try_fetch_mut::<ReplicaTrees>() else { return Ok(()) };
    let count = trees.map.trees().trees.len() as u32;
    for (cut, index) in changes {
        if index >= count || (if cut { !trees.fallen.insert(index) } else { !trees.fallen.remove(&index) }) {
            return Err("inconsistent tree change");
        }
    }
    *world.write_resource::<BlockedRegions>() = regions(trees.map, &trees.fallen);
    Ok(())
}

#[cfg(test)]
#[path = "tree_cut_tests.rs"]
mod tests;
