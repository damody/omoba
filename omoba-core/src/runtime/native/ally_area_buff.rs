//! Closed instant stat buff on the caster's allies in an area (G-4
//! `ally_area_buff_query`). A script cannot list allies, so it sends this
//! packet to its own caster: `{KEY: {source, center, radius, buff, seconds,
//! stats}}`. Serial settlement adds that buff (stat payload only) to every
//! live same-team non-building unit within `radius` of `center`, the caster
//! included. Refreshing the same buff id refreshes, it never stacks, so an
//! aura is one packet every short interval with a slightly longer duration.
//! Targets are a pure function of committed positions (as ally_area_restore),
//! so every world that settles the packet buffs the same units.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{Map, Value, json};
use specs::{Entity, World, WorldExt};
use super::comp::{CProperty, IsBuilding, Pos};
use omb_script_abi::stat_keys::StatKey;

pub const KEY: &str = "__ally_area_buff_v1";
pub const BUFF: &str = "__ally_area_buff";
const MAX_STATS: usize = 8;
/// Upper bound of the radius: covers the whole map for team-wide auras (E163, 日本第一武士).
pub const MAX_RADIUS: i32 = 100_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub center: Vec2,
    pub radius: Fixed64,
    /// Buff id stored on each ally; the same id refreshes.
    pub buff: String,
    pub seconds: Fixed64,
    /// StatKey name -> raw Fixed64 value, canonical (sorted) order.
    pub stats: Vec<(String, i64)>,
}

fn valid_buff_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 96 && !id.starts_with("__")
        && id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b':' || c == b'-')
}

impl Spec {
    pub fn checked(mut self) -> Option<Self> {
        let bounded = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        self.stats.sort();
        let stats_ok = !self.stats.is_empty() && self.stats.len() <= MAX_STATS
            && self.stats.windows(2).all(|w| w[0].0 != w[1].0)
            && self.stats.iter().all(|(key, raw)| StatKey::from_str_key(key).is_some() && raw.unsigned_abs() <= 1_000_000 * 1024);
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && self.center.x.raw().abs() <= bounded && self.center.y.raw().abs() <= bounded
            && self.radius > Fixed64::ZERO && self.radius <= Fixed64::from_i32(MAX_RADIUS)
            && valid_buff_id(&self.buff)
            && self.seconds > Fixed64::ZERO && self.seconds <= Fixed64::from_i32(60) && stats_ok).then_some(self)
    }
    pub fn payload(&self) -> Value {
        let stats: Map<String, Value> = self.stats.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "center": [self.center.x.raw(), self.center.y.raw()], "radius_raw": self.radius.raw(),
            "buff": self.buff, "seconds_raw": self.seconds.raw(), "stats": stats } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 7 { return None; }
        let center = row.get("center")?.as_array()?;
        if center.len() != 2 { return None; }
        let mut stats = Vec::new();
        for (key, value) in row.get("stats")?.as_object()? { stats.push((key.clone(), value.as_i64()?)); }
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            center: Vec2::new(Fixed64::from_raw(center[0].as_i64()?), Fixed64::from_raw(center[1].as_i64()?)),
            radius: Fixed64::from_raw(row.get("radius_raw")?.as_i64()?), buff: row.get("buff")?.as_str()?.to_owned(),
            seconds: Fixed64::from_raw(row.get("seconds_raw")?.as_i64()?), stats }.checked()
    }
    fn restore_view(&self) -> super::ally_area_restore::Spec {
        super::ally_area_restore::Spec::new(self.source_id, self.source_generation, self.center,
            self.radius, Fixed64::ZERO, Fixed64::ZERO, Fixed64::ONE)
    }
}

/// The packet is valid only on its own live caster (the aura owner).
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value, expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    if source != target || source.gen().id() as u32 != spec.source_generation || !entities.is_alive(source)
        || expected_source.is_some_and(|expected| expected != source) { return None; }
    if world.read_storage::<CProperty>().get(source).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    world.read_storage::<Pos>().get(source)?;
    Some(spec)
}

/// Live same-team non-building units in the area, canonical order.
pub fn targets(world: &World, spec: &Spec) -> Vec<Entity> {
    let buildings = world.read_storage::<IsBuilding>();
    super::ally_area_restore::targets(world, &spec.restore_view()).into_iter().filter(|e| buildings.get(*e).is_none()).collect()
}

/// (buff id, seconds, stat payload) applied to each target.
pub fn buff_payload(spec: &Spec) -> Value {
    Value::Object(spec.stats.iter().map(|(k, v)| (k.clone(), json!(v))).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec { source_id: 1, source_generation: 1, center: Vec2::new(Fixed64::from_i32(5), Fixed64::ZERO), radius: Fixed64::from_i32(900),
            buff: "b29_morale:1:1".into(), seconds: Fixed64::from_raw(1024 + 512),
            stats: vec![("totaldamageoutgoing_percentage".into(), 72)] }
    }
    #[test]
    fn packet_is_closed_canonical_and_bounded() {
        let s = spec().checked().unwrap();
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &s.payload()), Some(s.clone()));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(2), &s.payload()), None, "instant only");
        for bad in [
            Spec { buff: "__ally_area_buff".into(), ..spec() }, Spec { buff: "has space".into(), ..spec() },
            Spec { stats: vec![], ..spec() }, Spec { stats: vec![("not_a_stat".into(), 1)], ..spec() },
            Spec { seconds: Fixed64::ZERO, ..spec() }, Spec { radius: Fixed64::from_i32(MAX_RADIUS + 1), ..spec() },
            Spec { stats: vec![("health_bonus".into(), 1), ("health_bonus".into(), 2)], ..spec() },
        ] { let shown = format!("{bad:?}"); assert!(bad.checked().is_none(), "{shown}"); }
        let mut extra = s.payload(); extra[KEY]["mana"] = json!(1);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &extra), None);
        assert_eq!(buff_payload(&s), json!({"totaldamageoutgoing_percentage": 72}));
    }
}
