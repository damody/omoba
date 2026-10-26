//! Distance-settled tether (G-4 `tether_distance_scaled`). A script links its
//! caster to a unit by a timed buff on the target whose payload is `{KEY: ..}`.
//! When that buff runs out (not when it is removed early: death, cleanse or a
//! dead caster end the tether without effect) the host measures the
//! caster–target distance `d`, clamps it to `range` and settles, with
//! `f = d / range`:
//! - damage `amount × f` of `kind` (farther hurts more), and
//! - root for `root × (1 − f)` seconds (closer holds longer).
//! Settlement runs in the buff tick of every world that holds the buff, so a
//! team replica that ran the cast recomputes it from the same positions.
use omoba_sim::Fixed64;
use omb_script_abi::types::{DamageKind, DamageProfile};
use serde_json::{json, Value};
use specs::{Entity, world::Generation};

pub const KEY: &str = "__tether_v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spec {
    pub source_id: u32,
    pub source_generation: u32,
    pub amount: Fixed64,
    pub kind: DamageKind,
    pub root: Fixed64,
    pub range: Fixed64,
}

fn kind_name(kind: DamageKind) -> &'static str {
    match kind { DamageKind::Physical => "physical", DamageKind::Magical => "magical", DamageKind::Pure => "pure" }
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let bounded = |v: Fixed64, max: i32| v >= Fixed64::ZERO && v <= Fixed64::from_i32(max);
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && bounded(self.amount, 1_000_000) && bounded(self.root, 60)
            && self.range > Fixed64::ZERO && self.range <= Fixed64::from_i32(10_000)
            && (self.amount > Fixed64::ZERO || self.root > Fixed64::ZERO)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "source_id": self.source_id, "source_generation": self.source_generation,
            "amount_raw": self.amount.raw(), "kind": kind_name(self.kind), "root_raw": self.root.raw(), "range_raw": self.range.raw() } })
    }
    pub fn from_payload(payload: &Value) -> Option<Self> {
        if payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 6 { return None; }
        let kind = match row.get("kind")?.as_str()? {
            "physical" => DamageKind::Physical, "magical" => DamageKind::Magical, "pure" => DamageKind::Pure, _ => return None };
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            amount: Fixed64::from_raw(row.get("amount_raw")?.as_i64()?), kind,
            root: Fixed64::from_raw(row.get("root_raw")?.as_i64()?), range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?) }.checked()
    }
    pub fn source(&self) -> Entity { Entity::new(self.source_id, Generation::new(self.source_generation as i32)) }
    pub fn profile(&self) -> u32 {
        match self.kind { DamageKind::Physical => DamageProfile::NORMAL, DamageKind::Magical => DamageProfile::ENERGY,
            DamageKind::Pure => DamageProfile::TRUE }.bits()
    }
    /// (damage, root seconds) for a caster–target distance.
    pub fn settle(&self, distance: Fixed64) -> (Fixed64, Fixed64) {
        let d = distance.max(Fixed64::ZERO).min(self.range);
        let scale = |v: Fixed64, part: Fixed64| Fixed64::from_raw((i128::from(v.raw()) * i128::from(part.raw()) / i128::from(self.range.raw())) as i64);
        (scale(self.amount, d), scale(self.root, self.range - d))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec { source_id: 3, source_generation: 1, amount: Fixed64::from_i32(860), kind: DamageKind::Magical,
            root: Fixed64::from_i32(4), range: Fixed64::from_i32(750) }
    }
    #[test]
    fn payload_round_trips_strictly_and_settles_by_distance() {
        let s = spec();
        assert_eq!(Spec::from_payload(&s.payload()), Some(s));
        let mut extra = s.payload(); extra[KEY]["bonus"] = 1.into();
        assert_eq!(Spec::from_payload(&extra), None);
        let mut bad = s.payload(); bad[KEY]["range_raw"] = 0.into();
        assert_eq!(Spec::from_payload(&bad), None);
        assert_eq!(s.settle(Fixed64::ZERO), (Fixed64::ZERO, Fixed64::from_i32(4)));
        assert_eq!(s.settle(Fixed64::from_i32(750)), (Fixed64::from_i32(860), Fixed64::ZERO));
        assert_eq!(s.settle(Fixed64::from_i32(5000)), (Fixed64::from_i32(860), Fixed64::ZERO), "clamped");
        let (damage, root) = s.settle(Fixed64::from_i32(375));
        assert_eq!((damage, root), (Fixed64::from_i32(430), Fixed64::from_i32(2)));
    }
}
