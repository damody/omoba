//! Distance-settled tether (G-4 `tether_distance_scaled`). A script links its
//! caster to a unit by a timed buff on the target whose payload is `{KEY: ..}`.
//! When that buff runs out (not when it is removed early: the target's death
//! ends it without effect) the host measures the caster–target distance `d`,
//! clamps it to `range` and settles, with `f = d / range`:
//! - damage `amount × f` of `kind` (farther hurts more), and
//! - root for `root_far + (root − root_far) × (1 − f)` seconds (closer holds
//!   longer; `root_far` is the minimum root at or beyond `range`, default 0),
//!   plus `root_damage` of `kind` when that root lands (default 0).
//!
//! Optional rules (canonical: present in the payload only when set):
//! - `edge`: `d` is measured edge to edge, the centre distance minus both
//!   collision radii (not below 0), approximating an OBB distance.
//! - `root_after_source_death`: a dead (or gone) caster skips the distance
//!   damage but the root (and its `root_damage`) still lands. Without it a dead
//!   caster ends the tether without effect.
//!
//! The tether buff carries no control or slow key, so control cleansing never
//! removes it (`BuffStore::cleanse_controls`); only the root it settles into is
//! cleansable. Settlement runs in the buff tick of every world that holds the
//! buff, so a team replica that ran the cast recomputes it from the same
//! positions.
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
    /// Root at distance 0.
    pub root: Fixed64,
    pub range: Fixed64,
    /// Root at or beyond `range`; never above `root`.
    pub root_far: Fixed64,
    /// Damage of `kind` dealt when the root lands.
    pub root_damage: Fixed64,
    pub edge: bool,
    pub root_after_source_death: bool,
}

fn kind_name(kind: DamageKind) -> &'static str {
    match kind { DamageKind::Physical => "physical", DamageKind::Magical => "magical", DamageKind::Pure => "pure" }
}

impl Spec {
    /// The plain tether: no minimum root, no root damage, centre distance,
    /// nothing after the caster's death.
    pub fn new(source_id: u32, source_generation: u32, amount: Fixed64, kind: DamageKind, root: Fixed64, range: Fixed64) -> Self {
        Self { source_id, source_generation, amount, kind, root, range, root_far: Fixed64::ZERO, root_damage: Fixed64::ZERO,
            edge: false, root_after_source_death: false }
    }
    pub fn checked(self) -> Option<Self> {
        let bounded = |v: Fixed64, max: i32| v >= Fixed64::ZERO && v <= Fixed64::from_i32(max);
        (self.source_id != u32::MAX && self.source_generation > 0 && self.source_generation <= i32::MAX as u32
            && bounded(self.amount, 1_000_000) && bounded(self.root, 60) && bounded(self.root_far, 60) && self.root_far <= self.root
            && bounded(self.root_damage, 1_000_000)
            && self.range > Fixed64::ZERO && self.range <= Fixed64::from_i32(10_000)
            && (self.amount > Fixed64::ZERO || self.root > Fixed64::ZERO)
            && (self.root_damage == Fixed64::ZERO || self.root > Fixed64::ZERO)).then_some(self)
    }
    pub fn payload(&self) -> Value {
        let mut row = json!({ "source_id": self.source_id, "source_generation": self.source_generation,
            "amount_raw": self.amount.raw(), "kind": kind_name(self.kind), "root_raw": self.root.raw(), "range_raw": self.range.raw() });
        if self.root_far > Fixed64::ZERO { row["root_far_raw"] = json!(self.root_far.raw()); }
        if self.root_damage > Fixed64::ZERO { row["root_damage_raw"] = json!(self.root_damage.raw()); }
        if self.edge { row["edge"] = json!(true); }
        if self.root_after_source_death { row["root_after_source_death"] = json!(true); }
        json!({ (KEY): row })
    }
    pub fn from_payload(payload: &Value) -> Option<Self> {
        if payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let optional = ["root_far_raw", "root_damage_raw", "edge", "root_after_source_death"].iter().filter(|k| row.contains_key(**k)).count();
        if row.len() != 6 + optional { return None; }
        let kind = match row.get("kind")?.as_str()? {
            "physical" => DamageKind::Physical, "magical" => DamageKind::Magical, "pure" => DamageKind::Pure, _ => return None };
        // Canonical optional rows: a positive value or `true`, never a default.
        let positive = |name: &str| -> Option<Fixed64> { match row.get(name) {
            None => Some(Fixed64::ZERO),
            Some(v) => Some(Fixed64::from_raw(v.as_i64()?)).filter(|v| *v > Fixed64::ZERO) } };
        let flag = |name: &str| -> Option<bool> { match row.get(name) { None => Some(false), Some(Value::Bool(true)) => Some(true), _ => None } };
        Self { source_id: u32::try_from(row.get("source_id")?.as_u64()?).ok()?,
            source_generation: u32::try_from(row.get("source_generation")?.as_u64()?).ok()?,
            amount: Fixed64::from_raw(row.get("amount_raw")?.as_i64()?), kind,
            root: Fixed64::from_raw(row.get("root_raw")?.as_i64()?), range: Fixed64::from_raw(row.get("range_raw")?.as_i64()?),
            root_far: positive("root_far_raw")?, root_damage: positive("root_damage_raw")?,
            edge: flag("edge")?, root_after_source_death: flag("root_after_source_death")? }.checked()
    }
    pub fn source(&self) -> Entity { Entity::new(self.source_id, Generation::new(self.source_generation as i32)) }
    pub fn profile(&self) -> u32 {
        match self.kind { DamageKind::Physical => DamageProfile::NORMAL, DamageKind::Magical => DamageProfile::ENERGY,
            DamageKind::Pure => DamageProfile::TRUE }.bits()
    }
    /// The settled distance: centre distance, or edge to edge (both collision
    /// radii removed, not below 0) when `edge` is set.
    pub fn measured(&self, centre: Fixed64, source_radius: Fixed64, target_radius: Fixed64) -> Fixed64 {
        if !self.edge { return centre; }
        (centre - source_radius.max(Fixed64::ZERO) - target_radius.max(Fixed64::ZERO)).max(Fixed64::ZERO)
    }
    /// (damage, root seconds) for a caster–target distance.
    pub fn settle(&self, distance: Fixed64) -> (Fixed64, Fixed64) {
        let d = distance.max(Fixed64::ZERO).min(self.range);
        let scale = |v: Fixed64, part: Fixed64| Fixed64::from_raw((i128::from(v.raw()) * i128::from(part.raw()) / i128::from(self.range.raw())) as i64);
        (scale(self.amount, d), self.root_far + scale(self.root - self.root_far, self.range - d))
    }
    /// What settles: `None` when nothing lands (dead caster without
    /// `root_after_source_death`), else (damage, root seconds, root damage).
    /// `distance` is `None` when the dead caster has no position (then the
    /// far end applies).
    pub fn settlement(&self, source_alive: bool, distance: Option<Fixed64>) -> Option<(Fixed64, Fixed64, Fixed64)> {
        if !source_alive && !self.root_after_source_death { return None; }
        let (damage, root) = self.settle(distance.unwrap_or(self.range));
        let damage = if source_alive { damage } else { Fixed64::ZERO };
        let root_damage = if root > Fixed64::ZERO { self.root_damage } else { Fixed64::ZERO };
        Some((damage, root, root_damage))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec::new(3, 1, Fixed64::from_i32(860), DamageKind::Magical, Fixed64::from_i32(4), Fixed64::from_i32(750))
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

    /// C17W：定身 ((1000 − r)/200 + 1) × k 在 r∈[0,1000] 是線性：r=0 為 6k、r=1000 為 k。
    #[test]
    fn far_root_floor_matches_the_source_affine_formula() {
        for k in [307i64, 512, 717, 922] { // 0.3/0.5/0.7/0.9 s (Q10)
            let s = Spec { root: Fixed64::from_raw(6 * k), root_far: Fixed64::from_raw(k), range: Fixed64::from_i32(1000),
                ..Spec::new(3, 1, Fixed64::from_i32(350), DamageKind::Magical, Fixed64::ONE, Fixed64::ONE) }.checked().unwrap();
            assert_eq!(Spec::from_payload(&s.payload()), Some(s));
            for r in [0i32, 200, 500, 800, 1000, 1500] {
                let rc = r.min(1000);
                let source = Fixed64::from_raw(((i64::from(1000 - rc) * 1024 / 200 + 1024) * k) / 1024);
                let (_, root) = s.settle(Fixed64::from_i32(r));
                assert!((root.raw() - source.raw()).abs() <= 1, "k {k} r {r}: {} vs {}", root.raw(), source.raw());
            }
        }
        let bad = Spec { root_far: Fixed64::from_i32(5), ..spec() };
        assert_eq!(bad.checked(), None, "the far root never exceeds the near root");
    }

    #[test]
    fn optional_rules_are_canonical_and_settle_after_source_death() {
        let s = Spec { root_far: Fixed64::from_raw(307), root_damage: Fixed64::from_i32(5), edge: true, root_after_source_death: true,
            ..spec() }.checked().unwrap();
        assert_eq!(Spec::from_payload(&s.payload()), Some(s));
        for (field, value) in [("edge", json!(false)), ("root_after_source_death", json!(1)), ("root_far_raw", json!(0)), ("root_damage_raw", json!(-1))] {
            let mut bad = s.payload(); bad[KEY][field] = value;
            assert_eq!(Spec::from_payload(&bad), None, "{field}");
        }
        assert_eq!(s.measured(Fixed64::from_i32(100), Fixed64::from_i32(30), Fixed64::from_i32(30)), Fixed64::from_i32(40));
        assert_eq!(s.measured(Fixed64::from_i32(50), Fixed64::from_i32(30), Fixed64::from_i32(30)), Fixed64::ZERO);
        assert_eq!(spec().measured(Fixed64::from_i32(100), Fixed64::from_i32(30), Fixed64::from_i32(30)), Fixed64::from_i32(100));
        // Live caster: distance damage, root and its flat damage.
        assert_eq!(s.settlement(true, Some(Fixed64::from_i32(750))), Some((Fixed64::from_i32(860), Fixed64::from_raw(307), Fixed64::from_i32(5))));
        // Dead caster: no distance damage, the root and its flat damage still land.
        assert_eq!(s.settlement(false, Some(Fixed64::ZERO)), Some((Fixed64::ZERO, Fixed64::from_i32(4), Fixed64::from_i32(5))));
        assert_eq!(s.settlement(false, None), Some((Fixed64::ZERO, Fixed64::from_raw(307), Fixed64::from_i32(5))), "no position: far end");
        assert_eq!(spec().settlement(false, Some(Fixed64::ZERO)), None, "plain tether ends with its caster");
        assert_eq!(spec().settlement(true, Some(Fixed64::from_i32(750))), Some((Fixed64::from_i32(860), Fixed64::ZERO, Fixed64::ZERO)));
    }

    #[test]
    fn control_cleansing_never_removes_a_tether() {
        use crate::runtime::BuffStore;
        use specs::WorldExt;
        let world = specs::World::new();
        let target = world.entities().create();
        let mut buffs = BuffStore::default();
        buffs.add(target, "c17_light_chain:3:1", Fixed64::from_i32(3), spec().payload());
        buffs.add(target, "root", Fixed64::from_i32(2), json!({}));
        assert!(buffs.has_cleanseable_controls(target));
        buffs.cleanse_controls(target);
        assert!(!buffs.has(target, "root"), "the settled root is cleansable");
        assert!(buffs.has(target, "c17_light_chain:3:1"), "the tether itself is not");
        assert_eq!(Spec::from_payload(&buffs.get(target, "c17_light_chain:3:1").unwrap().payload), Some(spec()));
    }
}
