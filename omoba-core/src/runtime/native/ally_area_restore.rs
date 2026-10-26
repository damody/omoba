//! Closed instant area heal of the caster's allies (G-4 `ally_area_query`).
//! A script cannot list allies, so it sends this packet to its own caster:
//! `{KEY: {source, center, radius, max-hp fraction, cast range}}`. Serial
//! settlement heals every live same-team unit whose position lies within
//! `radius` of `center` (caster included) by that fraction of its maximum HP,
//! through the ordinary received-healing policy. Never stored as a buff.
//! Targets are a pure function of committed positions, so every world that
//! settles the packet heals the same units; opponents get committed vitals.
use omoba_sim::{Fixed64, Vec2};
use serde_json::{Value, json};
use specs::{Entity, Join, World, WorldExt};
use super::comp::{CProperty, Faction, Pos};

pub const KEY: &str = "__ally_area_restore_v1";
pub const BUFF: &str = "__ally_area_restore";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub center: Vec2,
    pub radius: Fixed64,
    /// Fractions of each target's maximum HP and mana, each in [0, 1], not both zero.
    pub hp_fraction: Fixed64,
    pub mana_fraction: Fixed64,
    pub cast_range: Fixed64,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let bounded = omoba_sim::terrain::MAX_GEOMETRY_RAW;
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && self.center.x.raw().abs() <= bounded && self.center.y.raw().abs() <= bounded
            && self.radius > Fixed64::ZERO && self.radius <= Fixed64::from_i32(5_000)
            && self.hp_fraction >= Fixed64::ZERO && self.hp_fraction <= Fixed64::ONE
            && self.mana_fraction >= Fixed64::ZERO && self.mana_fraction <= Fixed64::ONE
            && (self.hp_fraction > Fixed64::ZERO || self.mana_fraction > Fixed64::ZERO)
            && self.cast_range > Fixed64::ZERO && self.cast_range <= Fixed64::from_i32(10_000)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "center": [self.center.x.raw(), self.center.y.raw()], "radius_raw": self.radius.raw(),
            "hp_fraction_raw": self.hp_fraction.raw(), "mana_fraction_raw": self.mana_fraction.raw(),
            "range_raw": self.cast_range.raw() } })
    }
    pub fn from_payload(buff_id: &str, duration: Fixed64, payload: &Value) -> Option<Self> {
        if buff_id != BUFF || duration != Fixed64::ONE || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 7 { return None; }
        let center = row.get("center")?.as_array()?;
        if center.len() != 2 { return None; }
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            center: Vec2::new(Fixed64::from_raw(center[0].as_i64()?), Fixed64::from_raw(center[1].as_i64()?)),
            radius: Fixed64::from_raw(row.get("radius_raw")?.as_i64()?),
            hp_fraction: Fixed64::from_raw(row.get("hp_fraction_raw")?.as_i64()?),
            mana_fraction: Fixed64::from_raw(row.get("mana_fraction_raw")?.as_i64()?),
            cast_range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?) }.checked()
    }
}

/// The packet is valid only on its own live caster, with the centre in range.
pub fn admitted(world: &World, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value, expected_source: Option<Entity>) -> Option<Spec> {
    let spec = Spec::from_payload(buff_id, duration, payload)?;
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    if source != target || source.gen().id() as u32 != spec.source_generation || !entities.is_alive(source)
        || expected_source.is_some_and(|expected| expected != source) { return None; }
    if world.read_storage::<CProperty>().get(source).is_none_or(|p| p.hp <= Fixed64::ZERO) { return None; }
    let origin = world.read_storage::<Pos>().get(source)?.0;
    ((spec.center - origin).length_squared() <= spec.cast_range * spec.cast_range).then_some(spec)
}

/// Live same-team units inside the area, canonical entity order.
pub fn targets(world: &World, spec: &Spec) -> Vec<Entity> {
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    let factions = world.read_storage::<Faction>();
    let Some(team) = factions.get(source).map(|f| f.team_id) else { return Vec::new(); };
    let (positions, props) = (world.read_storage::<Pos>(), world.read_storage::<CProperty>());
    let mut rows: Vec<_> = (&entities, &factions, &positions, &props).join()
        .filter(|(_, faction, pos, prop)| faction.team_id == team && prop.hp > Fixed64::ZERO
            && (pos.0 - spec.center).length_squared() <= spec.radius * spec.radius)
        .map(|(entity, ..)| entity).collect();
    rows.sort_by_key(|entity| (entity.id(), entity.gen().id()));
    rows
}

/// Mana restored to each target hero with a pool: fraction of its maximum.
pub fn restore_mana(world: &mut World, spec: &Spec) {
    if spec.mana_fraction <= Fixed64::ZERO { return; }
    let targets = targets(world, spec);
    let mut heroes = world.write_storage::<super::comp::Hero>();
    for entity in targets {
        if let Some(pool) = heroes.get_mut(entity).and_then(|hero| hero.mana_pool.as_mut()) {
            let amount = Fixed64::from_raw((i128::from(pool.maximum().raw()) * i128::from(spec.mana_fraction.raw()) / 1024) as i64);
            let _ = pool.restore(amount);
        }
    }
}

/// (target, heal amount) rows in canonical entity order.
pub fn heals(world: &World, spec: &Spec) -> Vec<(Entity, Fixed64)> {
    let entities = world.entities();
    let source = entities.entity(spec.source_id);
    let factions = world.read_storage::<Faction>();
    let Some(team) = factions.get(source).map(|f| f.team_id) else { return Vec::new(); };
    let (positions, props) = (world.read_storage::<Pos>(), world.read_storage::<CProperty>());
    let mut rows: Vec<_> = (&entities, &factions, &positions, &props).join()
        .filter(|(_, faction, pos, prop)| faction.team_id == team && prop.hp > Fixed64::ZERO
            && (pos.0 - spec.center).length_squared() <= spec.radius * spec.radius)
        .map(|(entity, _, _, prop)| (entity, Fixed64::from_raw((i128::from(prop.mhp.raw()) * i128::from(spec.hp_fraction.raw()) / 1024) as i64)))
        .filter(|(_, amount)| *amount > Fixed64::ZERO)
        .collect();
    rows.sort_by_key(|(entity, _)| (entity.id(), entity.gen().id()));
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_is_closed_and_bounded() {
        let spec = Spec { source_id: 1, source_generation: 1, center: Vec2::new(Fixed64::from_i32(10), Fixed64::ZERO),
            radius: Fixed64::from_i32(800), hp_fraction: Fixed64::from_raw(410), mana_fraction: Fixed64::ZERO, cast_range: Fixed64::from_i32(1050) };
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &spec.payload()), Some(spec));
        assert_eq!(Spec::from_payload(BUFF, Fixed64::from_i32(2), &spec.payload()), None, "instant only");
        assert_eq!(Spec::from_payload("heal", Fixed64::ONE, &spec.payload()), None);
        for (field, value) in [("hp_fraction_raw", json!(-1)), ("hp_fraction_raw", json!(1025)), ("mana_fraction_raw", json!(1025)), ("radius_raw", json!(-1)), ("range_raw", json!(0))] {
            let mut bad = spec.payload(); bad[KEY][field] = value;
            assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &bad), None, "{field}");
        }
        let mut extra = spec.payload(); extra[KEY]["mana"] = json!(1);
        assert_eq!(Spec::from_payload(BUFF, Fixed64::ONE, &extra), None);
    }
}
