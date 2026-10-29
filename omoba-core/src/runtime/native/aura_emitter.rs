//! Aura emitters (E173): one aura mechanism for hero skills and items. A hero script cannot
//! list its allies, so it marks its own caster with an emitter buff for the aura's window:
//! `{KEY: {name, radius_raw, stats}}` under the buff id `buff_id(name)`. Every settle
//! (item_modifiers::settle_item_auras, authority and replicas alike) pulses each live emitter
//! through the closed area settlement of ally_area_buff: living same-team non-building units
//! within `radius` of the emitter, the emitter included, carry buff `name` with `stats` for
//! AURA_LINGER. The receiver buff id is the aura name, so one aura never stacks with itself;
//! a script that wants per-caster stacking puts the caster in the name. Item auras use the same
//! pulse with their catalog spec, so a hero aura and an item aura behave identically.
use omoba_sim::Fixed64;
use serde_json::{json, Map, Value};
use specs::{Entity, World, WorldExt};
use super::comp::{CProperty, Pos};
use omb_script_abi::stat_keys::StatKey;

pub const KEY: &str = "__aura_emitter_v1";
pub const PREFIX: &str = "__aura_emitter:";
const MAX_STATS: usize = 8;
/// The longest window one emitter buff may declare (a script refreshes it for longer auras).
pub const MAX_WINDOW: i32 = 600;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Buff id each receiver carries.
    pub name: String,
    pub radius: Fixed64,
    /// StatKey name -> raw Fixed64, canonical (sorted) order.
    pub stats: Vec<(String, i64)>,
}

pub fn buff_id(name: &str) -> String { format!("{PREFIX}{name}") }

impl Spec {
    pub fn checked(mut self) -> Option<Self> {
        self.stats.sort();
        let name_ok = !self.name.is_empty() && self.name.len() <= 96 && !self.name.starts_with("__")
            && self.name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b':' || c == b'-');
        let stats_ok = !self.stats.is_empty() && self.stats.len() <= MAX_STATS
            && self.stats.windows(2).all(|w| w[0].0 != w[1].0)
            && self.stats.iter().all(|(key, raw)| StatKey::from_str_key(key).is_some() && raw.unsigned_abs() <= 1_000_000 * 1024);
        (name_ok && stats_ok && self.radius > Fixed64::ZERO
            && self.radius <= Fixed64::from_i32(super::ally_area_buff::MAX_RADIUS)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        let stats: Map<String, Value> = self.stats.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        json!({ (KEY): { "name": self.name, "radius_raw": self.radius.raw(), "stats": stats } })
    }
    pub fn from_payload(payload: &Value) -> Option<Self> {
        if payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 3 { return None; }
        let mut stats = Vec::new();
        for (key, value) in row.get("stats")?.as_object()? { stats.push((key.clone(), value.as_i64()?)); }
        Self { name: row.get("name")?.as_str()?.to_owned(), radius: Fixed64::from_raw(row.get("radius_raw")?.as_i64()?), stats }.checked()
    }
}

/// An emitter buff is stored only on a live unit, under its own aura's id, for a bounded window.
pub fn admitted(world: &World, target: Entity, buff_id_: &str, duration: Fixed64, payload: &Value, expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(payload)?;
    if buff_id_ != buff_id(&spec.name) || duration <= Fixed64::ZERO || duration > Fixed64::from_i32(MAX_WINDOW)
        || expected_source.is_some_and(|source| source != target) || !world.entities().is_alive(target)
        || world.read_storage::<CProperty>().get(target).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    Some(spec)
}

/// Every live emitter's aura pulse this settle: (emitter, ally_area_buff spec), canonical order.
pub fn pulses(world: &World, linger: Fixed64) -> Vec<super::ally_area_buff::Spec> {
    let Some(buffs) = world.try_fetch::<crate::runtime::BuffStore>() else { return Vec::new() };
    let mut emitters: Vec<Entity> = buffs.entities_with_key(KEY).collect();
    emitters.sort_by_key(|e| (e.id(), e.gen().id()));
    emitters.dedup();
    let (props, positions) = (world.read_storage::<CProperty>(), world.read_storage::<Pos>());
    let mut out = Vec::new();
    for emitter in emitters {
        if !world.entities().is_alive(emitter) || props.get(emitter).is_none_or(|p| p.hp <= Fixed64::ZERO) { continue; }
        let Some(center) = positions.get(emitter).map(|p| p.0) else { continue };
        let mut specs: Vec<Spec> = buffs.iter_for(emitter)
            .filter(|(id, entry)| id.starts_with(PREFIX) && entry.remaining > Fixed64::ZERO)
            .filter_map(|(_, entry)| Spec::from_payload(&entry.payload)).collect();
        specs.sort_by(|a, b| a.name.cmp(&b.name));
        for spec in specs {
            if let Some(pulse) = (super::ally_area_buff::Spec { source_id: emitter.id(), source_generation: emitter.gen().id() as u32,
                center, radius: spec.radius, buff: spec.name, seconds: linger, stats: spec.stats }).checked() { out.push(pulse); }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emitter_packet_is_closed_and_bounded() {
        let spec = Spec { name: "b06_haste:1:1".into(), radius: Fixed64::from_i32(100_000),
            stats: vec![("movespeed_bonus_percentage".into(), 82)] }.checked().unwrap();
        assert_eq!(Spec::from_payload(&spec.payload()), Some(spec.clone()));
        for bad in [Spec { name: "__x".into(), ..spec.clone() }, Spec { stats: vec![], ..spec.clone() },
            Spec { stats: vec![("nope".into(), 1)], ..spec.clone() }, Spec { radius: Fixed64::ZERO, ..spec.clone() }] {
            assert!(bad.clone().checked().is_none(), "{bad:?}");
        }
        let mut extra = spec.payload(); extra[KEY]["seconds"] = json!(1);
        assert_eq!(Spec::from_payload(&extra), None);
        assert_eq!(buff_id(&spec.name), "__aura_emitter:b06_haste:1:1");
    }
}
