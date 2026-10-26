//! True-sight area (G-4 detection). A script reveals an area by sending its own
//! caster a launch envelope `add_stat_buff(caster, BUFF, duration, {KEY: ..})`.
//! The host turns it into a timed vision source of the caster's team at the
//! point: normal sight within `radius` plus detection of every stealth level,
//! so stealthed enemies inside are disclosed while it lasts. Terrain and trees
//! still block its line of sight, and it never reveals through fog elsewhere.
//!
//! The zone is authority state (`RevealZones`), never a unit: nothing can target
//! or see it. It belongs to the team, not the caster's life, so it lasts its full
//! duration even if the caster dies. Visibility is decided only on the authority;
//! team replicas receive the disclosed units like any other vision.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::{Entity, World, WorldExt};
use std::collections::BTreeMap;

use super::comp::{DeltaTime, Faction};

pub const KEY: &str = "__reveal_zone_v1";
/// Launch envelope id; never stored as a buff.
pub const BUFF: &str = "__reveal_zone";
pub const MAX_RADIUS: i32 = 2_000;
pub const MAX_DURATION: i32 = 30;
/// Live zones per team; a new one past the cap replaces the oldest.
pub const MAX_PER_TEAM: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub center: Vec2,
    pub radius: Fixed64,
    pub cast_range: Fixed64,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let max = omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && self.center.x.raw().unsigned_abs() <= max && self.center.y.raw().unsigned_abs() <= max
            && self.radius > Fixed64::ZERO && self.radius <= Fixed64::from_i32(MAX_RADIUS)
            && self.cast_range > Fixed64::ZERO && self.cast_range <= Fixed64::from_i32(10_000)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "x": self.center.x.raw(), "y": self.center.y.raw(), "radius_raw": self.radius.raw(), "range_raw": self.cast_range.raw() } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || !valid_duration(duration) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 6 { return None; }
        let raw = |name: &str| row.get(name)?.as_i64().map(Fixed64::from_raw);
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            center: Vec2::new(raw("x")?, raw("y")?), radius: raw("radius_raw")?, cast_range: raw("range_raw")? }.checked()
    }
    fn source(&self) -> Entity { Entity::new(self.source_id, specs::world::Generation::new(self.source_generation as i32)) }
}

pub fn valid_duration(duration: Fixed64) -> bool {
    duration >= Fixed64::from_raw(1) && duration <= Fixed64::from_i32(MAX_DURATION)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub team: u32,
    pub center: Vec2,
    pub radius: Fixed64,
    pub remaining: Fixed64,
}

/// Live zones by admission sequence (canonical order).
#[derive(Clone, Debug, Default)]
pub struct RevealZones { pub next: u64, pub zones: BTreeMap<u64, Zone> }

impl RevealZones {
    /// Canonical authority state for the lockstep desync hash.
    pub fn hash_value(&self) -> Value {
        Value::Array(self.zones.iter().map(|(sequence, zone)| json!({
            "sequence": sequence, "team": zone.team, "center": [zone.center.x.raw(), zone.center.y.raw()],
            "radius": zone.radius.raw(), "remaining": zone.remaining.raw(),
        })).collect())
    }
    /// Vision sources `(stable id, zone)`; ids never collide with entity ids.
    pub fn sources(&self) -> impl Iterator<Item = (u64, &Zone)> {
        self.zones.iter().map(|(sequence, zone)| ((1 << 63) | sequence, zone))
    }
}

/// The exact live caster sent it, the point is within cast range.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value,
    expected_source: Option<Entity>) -> Option<(Spec, u32)> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    let source = spec.source();
    if target != source || expected_source.is_some_and(|expected| expected != source) || !world.entities().is_alive(source)
        || world.read_storage::<super::comp::CProperty>().get(source).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    let team = u32::try_from(world.read_storage::<Faction>().get(source)?.team_id).ok().filter(|team| *team > 0)?;
    let at = world.read_storage::<super::comp::Pos>().get(source)?.0;
    ((spec.center - at).length_squared() <= spec.cast_range * spec.cast_range).then_some((spec, team))
}

/// Serial settlement of the envelope: opens the zone. False drops it.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let Some((spec, team)) = admitted(world, target, buff_id, duration, payload, None) else { return false; };
    if world.try_fetch::<RevealZones>().is_none() { world.insert(RevealZones::default()); }
    let mut rows = world.write_resource::<RevealZones>();
    let own: Vec<u64> = rows.zones.iter().filter(|(_, zone)| zone.team == team).map(|(sequence, _)| *sequence).collect();
    if own.len() >= MAX_PER_TEAM { rows.zones.remove(&own[0]); }
    let sequence = rows.next;
    rows.next += 1;
    rows.zones.insert(sequence, Zone { team, center: spec.center, radius: spec.radius, remaining: duration });
    true
}

/// Counts every zone down by this tick's `DeltaTime` (authority pre-step).
pub fn step(world: &mut World) {
    let dt = world.read_resource::<DeltaTime>().0;
    if dt <= Fixed64::ZERO { return; }
    let Some(mut rows) = world.try_fetch_mut::<RevealZones>() else { return; };
    rows.zones.retain(|_, zone| { zone.remaining = zone.remaining - dt; zone.remaining > Fixed64::ZERO });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec { source_id: 3, source_generation: 1, center: Vec2::new(Fixed64::from_i32(400), Fixed64::from_i32(-20)),
            radius: Fixed64::from_i32(450), cast_range: Fixed64::from_i32(1000) }
    }
    #[test]
    fn payload_round_trips_strictly() {
        let s = spec();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(4), &s.payload()), Some(s));
        assert_eq!(Spec::from_payload("reveal", Fixed64::from_i32(4), &s.payload()), None);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ZERO, &s.payload()), None);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(MAX_DURATION + 1), &s.payload()), None);
        let mut extra = s.payload(); extra[KEY]["detection"] = 1.into();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &extra), None);
        for (field, value) in [("radius_raw", 0), ("radius_raw", Fixed64::from_i32(MAX_RADIUS + 1).raw()), ("range_raw", 0), ("source_generation", 0)] {
            let mut bad = s.payload(); bad[KEY][field] = value.into();
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None, "{field}={value}");
        }
    }
    #[test]
    fn source_ids_are_disjoint_from_entities_and_canonical() {
        let mut rows = RevealZones::default();
        for (sequence, team) in [(5, 2), (2, 1)] {
            rows.zones.insert(sequence, Zone { team, center: Vec2::ZERO, radius: Fixed64::ONE, remaining: Fixed64::ONE });
        }
        let ids: Vec<u64> = rows.sources().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![(1 << 63) | 2, (1 << 63) | 5]);
    }
}
