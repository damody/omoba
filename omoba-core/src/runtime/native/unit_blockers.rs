//! Summons that block pathing (E426; v3 unit_template `blocks_pathing`, 原作 B22 屏障岩石 and B24T
//! 禁錮岩石: buildings whose hull is a GridNav obstruction). Hero, creep and summon navigation
//! collides with public static geometry only (hero_move_tick::path_hits_regions), so a blocker is
//! public geometry too: one square `blocker_<x>_<y>_<r>` of half side `r` around its point in
//! [`BlockedRegions`], after the map's terrain and standing trees, in canonical order.
//!
//! Flow (the tree pattern of tree_cut, E242):
//! 1. Authority pre-step ([`tick`]): the live immobile summons whose template blocks pathing are
//!    the wanted set; where it differs from the blocker squares the regions hold, the regions are
//!    recomposed and each change is one public pre-step HUD fact ([`ADD_METRIC_ID`] /
//!    [`REMOVE_METRIC_ID`], value = the packed [`Blocker`]) to every team.
//! 2. Replicas apply them at the start of the same step ([`apply_disclosed`]), so every peer walks
//!    the same terrain from the same tick. A mid-match start carries the current regions.
//! 3. Rebuilding the trees keeps the blocker squares ([`keep`]).
//!
//! The current set is always read back from the regions themselves ([`current`]); nothing else
//! is stored. A unit standing where a blocker appears is not pushed out (原作 FindClearSpace): it
//! walks again once the blocker is gone.
use std::collections::BTreeMap;

use omoba_sim::Fixed64;
use specs::{Join, World, WorldExt};

use super::comp::{BlockedRegion, BlockedRegions, CProperty, Pos, SummonedUnit};
use crate::runtime::scripting::ScriptUnitTag;

pub const ADD_METRIC_ID: u64 = 0x4d42_4c4f_434b_4144;
pub const REMOVE_METRIC_ID: u64 = 0x4d42_4c4f_434b_5245;
const PREFIX: &str = "blocker_";
/// Local ordinals of the blocker facts: base + sequence within the tick.
const ORDINAL_BASE: u32 = 0x5000_0000;
/// Map coordinates a blocker point may take (24-bit packed).
const MAX_COORD: i32 = (1 << 23) - 1;
pub const MAX_RADIUS: i32 = 512;

/// A blocker square: its point (whole map units) and half side.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blocker { pub x: i32, pub y: i32, pub r: i32 }

impl Blocker {
    pub fn valid(self) -> bool {
        self.x.abs() <= MAX_COORD && self.y.abs() <= MAX_COORD && (1..=MAX_RADIUS).contains(&self.r)
    }
    /// x (24 bits), y (24 bits), r (16 bits).
    pub fn pack(self) -> i64 {
        (((self.x as i64) & 0xff_ffff) << 40) | (((self.y as i64) & 0xff_ffff) << 16) | (self.r as i64 & 0xffff)
    }
    pub fn unpack(value: i64) -> Option<Self> {
        let signed = |raw: i64| ((raw << 40) >> 40) as i32;
        let blocker = Blocker { x: signed((value >> 40) & 0xff_ffff), y: signed((value >> 16) & 0xff_ffff), r: (value & 0xffff) as i32 };
        (blocker.valid() && blocker.pack() == value).then_some(blocker)
    }
    fn name(self) -> String { format!("{PREFIX}{}_{}_{}", self.x, self.y, self.r) }
    fn parse(name: &str) -> Option<Self> {
        let mut parts = name.strip_prefix(PREFIX)?.split('_');
        let blocker = Blocker { x: parts.next()?.parse().ok()?, y: parts.next()?.parse().ok()?, r: parts.next()?.parse().ok()? };
        (parts.next().is_none() && blocker.valid()).then_some(blocker)
    }
    fn region(self) -> BlockedRegion {
        let (x0, y0, x1, y1) = ((self.x - self.r) as f32, (self.y - self.r) as f32, (self.x + self.r) as f32, (self.y + self.r) as f32);
        BlockedRegion { name: self.name(), points: vec![vek::Vec2::new(x0, y0), vek::Vec2::new(x1, y0), vek::Vec2::new(x1, y1), vek::Vec2::new(x0, y1)] }
    }
}

/// Whether a region is a blocker square.
pub fn is_blocker_region(region: &BlockedRegion) -> bool { region.name.starts_with(PREFIX) }

/// The regions without their blocker squares.
pub fn without(regions: &BlockedRegions) -> BlockedRegions {
    BlockedRegions(regions.0.iter().filter(|region| !is_blocker_region(region)).cloned().collect())
}

/// The blocker squares the regions hold, with their counts. `None` when one is malformed.
pub fn current(regions: &BlockedRegions) -> Option<BTreeMap<Blocker, u32>> {
    let mut set = BTreeMap::new();
    for region in regions.0.iter().filter(|region| is_blocker_region(region)) {
        let blocker = Blocker::parse(&region.name)?;
        if region.points != blocker.region().points { return None; }
        *set.entry(blocker).or_insert(0) += 1;
    }
    Some(set)
}

/// `base` (terrain and trees, no blockers) followed by the blocker squares in canonical order.
pub fn compose(mut base: BlockedRegions, blockers: &BTreeMap<Blocker, u32>) -> BlockedRegions {
    base.0.retain(|region| !is_blocker_region(region));
    for (blocker, count) in blockers {
        for _ in 0..*count { base.0.push(blocker.region()); }
    }
    base
}

/// Rebuilt terrain and trees `base` with the blocker squares `world` holds now.
pub fn keep(world: &World, base: BlockedRegions) -> BlockedRegions {
    let blockers = current(&world.read_resource::<BlockedRegions>()).unwrap_or_default();
    compose(base, &blockers)
}

/// The blocker a live summon stands for: an immobile template that blocks pathing.
fn wanted(world: &World) -> BTreeMap<Blocker, u32> {
    let (entities, summons, tags, positions, properties) = (world.entities(), world.read_storage::<SummonedUnit>(),
        world.read_storage::<ScriptUnitTag>(), world.read_storage::<Pos>(), world.read_storage::<CProperty>());
    let mut set = BTreeMap::new();
    for (entity, summon, tag, pos) in (&entities, &summons, &tags, &positions).join() {
        if summon.is_expired() || !summon.immobile || properties.get(entity).is_none_or(|p| p.hp <= Fixed64::ZERO) { continue; }
        let Some(stats) = omoba_template_ids::summon_by_name(&tag.unit_id).and_then(omoba_template_ids::active_summon_stats) else { continue };
        if !stats.blocks_pathing { continue; }
        let round = |v: Fixed64| ((v.raw() + 512).div_euclid(1024)) as i32;
        let blocker = Blocker { x: round(pos.0.x), y: round(pos.0.y), r: round(stats.collision_radius) };
        if blocker.valid() { *set.entry(blocker).or_insert(0) += 1; }
    }
    set
}

/// Authority pre-step: recompose the regions when the wanted blockers changed, and disclose each
/// change to every team.
pub(crate) fn tick(world: &World) {
    let wanted = wanted(world);
    let had = current(&world.read_resource::<BlockedRegions>()).unwrap_or_default();
    if wanted == had { return; }
    let mut changes = Vec::new();
    for blocker in had.keys().chain(wanted.keys()).copied().collect::<std::collections::BTreeSet<_>>() {
        let (before, after) = (had.get(&blocker).copied().unwrap_or(0), wanted.get(&blocker).copied().unwrap_or(0));
        for _ in after..before { changes.push((REMOVE_METRIC_ID, blocker)); }
        for _ in before..after { changes.push((ADD_METRIC_ID, blocker)); }
    }
    let regions = compose((*world.read_resource::<BlockedRegions>()).clone(), &wanted);
    *world.write_resource::<BlockedRegions>() = regions;
    let tick = world.read_resource::<crate::runtime::Tick>().0;
    let Some(facts) = world.try_fetch::<crate::runtime::ObservableFactBuffer>() else { return };
    for (sequence, (metric_id, blocker)) in changes.into_iter().enumerate() {
        facts.emit(crate::runtime::OrderedFact {
            key: crate::runtime::FactOrderingKey { tick, phase: crate::runtime::FactPhase::PreStep, canonical_source_order: 0,
                local_ordinal: ORDINAL_BASE + sequence as u32, fact_kind: crate::runtime::FactKind::Hud },
            audience: crate::runtime::FactAudience::AllPlayers,
            fact: crate::runtime::ObservableFact::Hud { team: 0, metric_id, value: blocker.pack() },
        }).expect("valid public blocker change");
    }
}

/// Replica step start: apply this step's disclosed blocker changes (HUD payload: team u32, metric
/// u64, value i64, little endian), in order.
pub fn apply_disclosed(world: &World, payloads: &[&[u8]]) -> Result<(), &'static str> {
    let changes: Vec<(bool, i64)> = payloads.iter().filter(|bytes| bytes.len() == 20 && bytes[0..4] == 0_u32.to_le_bytes())
        .filter_map(|bytes| {
            let metric = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
            (metric == ADD_METRIC_ID || metric == REMOVE_METRIC_ID)
                .then(|| (metric == ADD_METRIC_ID, i64::from_le_bytes(bytes[12..20].try_into().unwrap())))
        }).collect();
    if changes.is_empty() { return Ok(()); }
    let mut set = current(&world.read_resource::<BlockedRegions>()).ok_or("malformed blocker region")?;
    for (add, value) in changes {
        let blocker = Blocker::unpack(value).ok_or("invalid blocker")?;
        if add { *set.entry(blocker).or_insert(0) += 1; continue; }
        match set.get_mut(&blocker) {
            Some(count) if *count > 1 => *count -= 1,
            Some(_) => { set.remove(&blocker); }
            None => return Err("inconsistent blocker change"),
        }
    }
    let regions = compose((*world.read_resource::<BlockedRegions>()).clone(), &set);
    *world.write_resource::<BlockedRegions>() = regions;
    Ok(())
}

#[cfg(test)]
#[path = "unit_blockers_tests.rs"]
mod tests;
