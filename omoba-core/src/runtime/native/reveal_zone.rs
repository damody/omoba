//! Plain-sight variant (`Spec::detect = false`, payload `"detect": false`): the
//! same team sight circle without detection, like 原作 `AddFOWViewer`
//! (C15E 每段 200／12 秒、B29T 施法點視野); stealthed units stay hidden.
//!
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
//!
//! Optional envelope rows (E185; absent = the caster's team at a fixed point), added by
//! helpers so the closed `Spec` literal every script builds stays unchanged:
//! - `teams: "all"` ([`with_all_teams`]): one zone per team (原作 AddFOWViewer for both
//!   teams, e.g. B08D, B29T at the caster, C17W around the tether).
//! - `follow` ([`with_follow`]): the zone follows that unit (or projectile entity); its
//!   centre is the unit's position every step and it closes when the unit is gone.
//! Ability projectiles open a following plain zone for their own flight (原作
//! ProvidesVision, `ability_projectile::with_vision`) through [`open_following`].
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
/// (C15E alone opens 25 plain sight circles for 12 s.)
pub const MAX_PER_TEAM: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub center: Vec2,
    pub radius: Fixed64,
    pub cast_range: Fixed64,
    /// True sight (detects every stealth level). `false` is a plain sight
    /// circle like 原作 `AddFOWViewer`, which never discloses stealth.
    pub detect: bool,
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
        let mut payload = json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "x": self.center.x.raw(), "y": self.center.y.raw(), "radius_raw": self.radius.raw(), "range_raw": self.cast_range.raw() } });
        // A true-sight zone keeps the original six-field envelope.
        if !self.detect { payload[KEY]["detect"] = false.into(); }
        payload
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || !valid_duration(duration) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let detect = match row.get("detect") { None => true, Some(Value::Bool(false)) => false, Some(_) => return None };
        let teams = match row.get("teams") { None => 0, Some(value) if value == "all" => 1, Some(_) => return None };
        let follow = match row.get("follow") { None => 0, Some(_) => { follow_of(payload)?; 1 } };
        if row.len() != if detect { 6 } else { 7 } + teams + follow { return None; }
        let raw = |name: &str| row.get(name)?.as_i64().map(Fixed64::from_raw);
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            center: Vec2::new(raw("x")?, raw("y")?), radius: raw("radius_raw")?, cast_range: raw("range_raw")?, detect }.checked()
    }
    fn source(&self) -> Entity { Entity::new(self.source_id, specs::world::Generation::new(self.source_generation as i32)) }
}

/// Every team gets the zone (原作 AddFOWViewer for each team), not only the caster's.
pub fn with_all_teams(mut payload: Value) -> Value {
    if let Some(row) = payload.get_mut(KEY).and_then(Value::as_object_mut) { row.insert("teams".into(), json!("all")); }
    payload
}

/// The zone follows this unit's position and closes when it is gone.
pub fn with_follow(mut payload: Value, id: u32, generation: u32) -> Value {
    if let Some(row) = payload.get_mut(KEY).and_then(Value::as_object_mut) {
        row.insert("follow".into(), json!({"id": id, "generation": generation}));
    }
    payload
}

/// The envelope of a plain sight circle (原作 AddFOWViewer) a hero script opens at `center`
/// (any distance from the caster): `all_teams` gives it to every team, `follow` tracks a unit.
pub fn sight_payload(source_id: u32, source_generation: u32, center: Vec2, radius: Fixed64, all_teams: bool,
    follow: Option<(u32, u32)>) -> Option<Value> {
    let spec = Spec { source_id, source_generation, center, radius, cast_range: Fixed64::from_i32(10_000), detect: false }.checked()?;
    let mut payload = spec.payload();
    if all_teams { payload = with_all_teams(payload); }
    if let Some((id, generation)) = follow { payload = with_follow(payload, id, generation); }
    Some(payload)
}

fn all_teams(payload: &Value) -> bool {
    payload.get(KEY).and_then(|row| row.get("teams")).is_some_and(|teams| teams == "all")
}

/// The followed unit `(id, generation)`; None when absent or malformed.
pub fn follow_of(payload: &Value) -> Option<(u32, u32)> {
    let row = payload.get(KEY)?.get("follow")?.as_object()?;
    if row.len() != 2 { return None; }
    let id = u32::try_from(row.get("id")?.as_u64()?).ok().filter(|id| *id != u32::MAX)?;
    let generation = u32::try_from(row.get("generation")?.as_u64()?).ok().filter(|g| *g > 0 && *g <= i32::MAX as u32)?;
    Some((id, generation))
}

fn live_position(world: &World, entity: Entity) -> Option<Vec2> {
    if !world.entities().is_alive(entity) { return None; }
    world.read_storage::<super::comp::Pos>().get(entity).map(|p| p.0)
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
    /// `u16::MAX` for true sight, 0 for a plain sight circle.
    pub detection_level: u16,
    /// The unit the centre follows (E185); the zone closes when it is gone.
    pub follow: Option<Entity>,
}

/// Live zones by admission sequence (canonical order).
#[derive(Clone, Debug, Default)]
pub struct RevealZones { pub next: u64, pub zones: BTreeMap<u64, Zone> }

impl RevealZones {
    /// Canonical authority state for the lockstep desync hash.
    pub fn hash_value(&self) -> Value {
        Value::Array(self.zones.iter().map(|(sequence, zone)| {
            let mut row = json!({
            "sequence": sequence, "team": zone.team, "center": [zone.center.x.raw(), zone.center.y.raw()],
            "radius": zone.radius.raw(), "remaining": zone.remaining.raw(), "detection": zone.detection_level,
            });
            // Present only for a following zone, so fixed zones hash as before.
            if let Some(unit) = zone.follow { row["follow"] = json!([unit.id(), unit.gen().id() as u32]); }
            row
        }).collect())
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
    // A following zone starts where the followed unit is now.
    let center = match follow_of(payload) {
        Some((id, generation)) => {
            let unit = world.entities().entity(id);
            if unit.gen().id() as u32 != generation { return None; }
            live_position(world, unit)?
        }
        None => spec.center,
    };
    ((center - at).length_squared() <= spec.cast_range * spec.cast_range).then_some((Spec { center, ..spec }, team))
}

/// The teams a zone opens for: the caster's, or (`teams: "all"`) every team that keeps a
/// vision view, plus every hero team, in ascending order.
fn audience(world: &World, caster_team: u32, all: bool) -> Vec<u32> {
    let mut teams = std::collections::BTreeSet::from([caster_team]);
    if all {
        if let Some(runtime) = world.try_fetch::<crate::runtime::TeamVisibilityRuntime>() { teams.extend(runtime.teams.keys().copied()); }
        let heroes = world.read_storage::<super::comp::Hero>();
        let factions = world.read_storage::<Faction>();
        use specs::Join;
        teams.extend((&heroes, &factions).join().filter_map(|(_, f)| u32::try_from(f.team_id).ok().filter(|t| *t > 0)));
    }
    teams.into_iter().collect()
}

fn insert(world: &mut World, team: u32, zone: Zone) {
    if world.try_fetch::<RevealZones>().is_none() { world.insert(RevealZones::default()); }
    let mut rows = world.write_resource::<RevealZones>();
    let own: Vec<u64> = rows.zones.iter().filter(|(_, zone)| zone.team == team).map(|(sequence, _)| *sequence).collect();
    if own.len() >= MAX_PER_TEAM { rows.zones.remove(&own[0]); }
    let sequence = rows.next;
    rows.next += 1;
    rows.zones.insert(sequence, Zone { team, ..zone });
}

/// A plain sight circle of `team` that follows `unit` for `duration` (ability projectile
/// vision). False when the values are out of range or the unit has no position.
pub fn open_following(world: &mut World, team: u32, unit: Entity, radius: Fixed64, duration: Fixed64) -> bool {
    if team == 0 || radius <= Fixed64::ZERO || radius > Fixed64::from_i32(MAX_RADIUS) || !valid_duration(duration) { return false; }
    let Some(center) = live_position(world, unit) else { return false; };
    insert(world, team, Zone { team, center, radius, remaining: duration, detection_level: 0, follow: Some(unit) });
    true
}

/// Serial settlement of the envelope: opens the zone. False drops it.
pub fn apply(world: &mut World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    let Some((spec, team)) = admitted(world, target, buff_id, duration, payload, None) else { return false; };
    let follow = follow_of(payload).map(|(id, _)| world.entities().entity(id));
    let zone = Zone { team, center: spec.center, radius: spec.radius, remaining: duration,
        detection_level: if spec.detect { u16::MAX } else { 0 }, follow };
    for team in audience(world, team, all_teams(payload)) { insert(world, team, zone); }
    true
}

/// Counts every zone down by this tick's `DeltaTime` (authority pre-step).
pub fn step(world: &mut World) {
    let dt = world.read_resource::<DeltaTime>().0;
    if dt <= Fixed64::ZERO { return; }
    let Some(followed) = world.try_fetch::<RevealZones>().map(|rows| rows.zones.iter()
        .filter_map(|(sequence, zone)| zone.follow.map(|unit| (*sequence, unit))).collect::<Vec<_>>()) else { return; };
    let moved: Vec<(u64, Option<Vec2>)> = followed.into_iter().map(|(sequence, unit)| (sequence, live_position(world, unit))).collect();
    let mut rows = world.write_resource::<RevealZones>();
    for (sequence, at) in moved {
        match at {
            Some(at) => if let Some(zone) = rows.zones.get_mut(&sequence) { zone.center = at; },
            None => { rows.zones.remove(&sequence); }
        }
    }
    rows.zones.retain(|_, zone| { zone.remaining = zone.remaining - dt; zone.remaining > Fixed64::ZERO });
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec { source_id: 3, source_generation: 1, center: Vec2::new(Fixed64::from_i32(400), Fixed64::from_i32(-20)),
            radius: Fixed64::from_i32(450), cast_range: Fixed64::from_i32(1000), detect: true }
    }
    #[test]
    fn plain_sight_zones_round_trip_and_never_detect() {
        let plain = Spec { detect: false, ..spec() };
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(12), &plain.payload()), Some(plain));
        assert_eq!(spec().payload()[KEY].as_object().unwrap().len(), 6, "true sight keeps the original envelope");
        for value in [serde_json::json!(true), serde_json::json!(0)] {
            let mut bad = plain.payload(); bad[KEY]["detect"] = value;
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None);
        }
        let mut world = World::new();
        world.register::<Faction>(); world.register::<super::super::comp::CProperty>(); world.register::<super::super::comp::Pos>();
        use specs::Builder;
        let caster = world.create_entity().with(Faction { faction_id: super::super::comp::FactionType::Player, team_id: 1 })
            .with(super::super::comp::CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO,
                def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(super::super::comp::Pos(Vec2::ZERO)).build();
        let at = |x: i32| Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32,
            center: Vec2::new(Fixed64::from_i32(x), Fixed64::ZERO), radius: Fixed64::from_i32(200),
            cast_range: Fixed64::from_i32(3000), detect: x != 0 };
        assert!(apply(&mut world, caster, BUFF, Fixed64::from_i32(12), &at(0).payload()));
        assert!(apply(&mut world, caster, BUFF, Fixed64::from_i32(12), &at(100).payload()));
        let rows = world.read_resource::<RevealZones>();
        let levels: Vec<_> = rows.zones.values().map(|zone| zone.detection_level).collect();
        assert_eq!(levels, vec![0, u16::MAX]);
        assert_eq!(rows.hash_value()[0]["detection"], 0);
        drop(rows);
        // 25 segments of one C15E fit under the per-team cap.
        for x in 0..30 { assert!(apply(&mut world, caster, BUFF, Fixed64::from_i32(12), &at(200 + x).payload())); }
        assert_eq!(world.read_resource::<RevealZones>().zones.len(), 32);
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
    fn all_team_and_following_zones_open_per_team_track_the_unit_and_close_with_it() {
        use specs::Builder;
        use super::super::comp::{CProperty, FactionType, Hero, Pos};
        let mut world = World::new();
        world.register::<Faction>(); world.register::<CProperty>(); world.register::<Pos>(); world.register::<Hero>();
        world.insert(DeltaTime(Fixed64::from_raw(17)));
        let alive = CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO };
        let caster = world.create_entity().with(Faction { faction_id: FactionType::Player, team_id: 1 }).with(alive.clone())
            .with(Pos(Vec2::ZERO)).with(Hero::default()).build();
        let enemy = world.create_entity().with(Faction { faction_id: FactionType::Enemy, team_id: 2 }).with(alive)
            .with(Pos(Vec2::new(Fixed64::from_i32(300), Fixed64::ZERO))).with(Hero::default()).build();
        let spec = Spec { source_id: caster.id(), source_generation: caster.gen().id() as u32, center: Vec2::ZERO,
            radius: Fixed64::from_i32(100), cast_range: Fixed64::from_i32(1000), detect: false };
        let both = with_all_teams(spec.payload());
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(6), &both), Some(spec));
        assert!(apply(&mut world, caster, BUFF, Fixed64::from_i32(6), &both));
        let teams: Vec<u32> = world.read_resource::<RevealZones>().zones.values().map(|zone| zone.team).collect();
        assert_eq!(teams, vec![1, 2], "one plain zone per team");
        let mut bad = both.clone(); bad[KEY]["teams"] = json!("enemy");
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None);
        // Following the enemy: starts at it, moves with it, closes when it is gone.
        world.write_resource::<RevealZones>().zones.clear();
        let follow = with_follow(with_all_teams(spec.payload()), enemy.id(), enemy.gen().id() as u32);
        assert!(apply(&mut world, caster, BUFF, Fixed64::from_i32(3), &follow));
        assert!(world.read_resource::<RevealZones>().zones.values().all(|zone| zone.center.x == Fixed64::from_i32(300)));
        world.write_storage::<Pos>().get_mut(enemy).unwrap().0 = Vec2::new(Fixed64::from_i32(500), Fixed64::ZERO);
        step(&mut world);
        assert!(world.read_resource::<RevealZones>().zones.values().all(|zone| zone.center.x == Fixed64::from_i32(500)));
        assert!(world.read_resource::<RevealZones>().hash_value()[0]["follow"].is_array());
        world.delete_entity(enemy).unwrap(); world.maintain();
        step(&mut world);
        assert!(world.read_resource::<RevealZones>().zones.is_empty(), "the zone closes with the followed unit");
        let stale = with_follow(spec.payload(), enemy.id(), enemy.gen().id() as u32);
        assert!(!apply(&mut world, caster, BUFF, Fixed64::ONE, &stale), "a gone unit cannot be followed");
        // Projectile vision: the caster team only.
        assert!(open_following(&mut world, 1, caster, Fixed64::from_i32(175), Fixed64::ONE));
        assert!(!open_following(&mut world, 1, caster, Fixed64::ZERO, Fixed64::ONE));
        assert_eq!(world.read_resource::<RevealZones>().zones.len(), 1);
    }

    #[test]
    fn source_ids_are_disjoint_from_entities_and_canonical() {
        let mut rows = RevealZones::default();
        for (sequence, team) in [(5, 2), (2, 1)] {
            rows.zones.insert(sequence, Zone { team, center: Vec2::ZERO, radius: Fixed64::ONE, remaining: Fixed64::ONE, detection_level: u16::MAX, follow: None });
        }
        let ids: Vec<u64> = rows.sources().map(|(id, _)| id).collect();
        assert_eq!(ids, vec![(1 << 63) | 2, (1 << 63) | 5]);
    }
}
