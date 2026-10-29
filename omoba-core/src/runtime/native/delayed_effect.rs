//! delayed_effect：技能「施放或產生後 N 秒才結算」的階段（原作 `Timers:CreateTimer`
//! 延遲結算，例：B29E 猛擊 0.2 秒後依施法者當下位置、B29T 每擊產生後 0.45 秒落下）。
//!
//! A script schedules one by sending its own caster an ordinary buff
//! `add_stat_buff(caster, spec.buff_id(), delay, spec.payload())`. The row lives
//! in the caster's `BuffStore`, so the owning team's replica that predicts the
//! cast holds the same row, like any other caster marker. When the row expires
//! naturally, `buff_tick` queues it in [`DueDelayedEffects`] and the script
//! dispatch of that tick calls `AbilityScript::on_delayed_effect` once, after
//! the channel pulses, in (caster, buff id) order. The hook may only apply
//! effects (the channel-pulse rule): it cannot charge, cast or spawn.
//!
//! Lifetime: the row belongs to the caster's life. Death clears the caster's
//! buffs, so a pending effect of a dead caster never resolves; a channel ending
//! does not touch it (B29T 引導結束後已排程的打擊仍落下).
use omb_script_abi::types::ChannelPulse;
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::Entity;

pub const KEY: &str = "__delayed_effect_v1";
/// Buff id prefix: `delayed_effect:<ability>:<index>`.
pub const PREFIX: &str = "delayed_effect:";
pub const MAX_DELAY: i32 = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    /// Ability id whose `on_delayed_effect` resolves it.
    pub ability: String,
    pub level: u8,
    /// Distinguishes concurrent effects of one caster and ability.
    pub index: u16,
    /// The point decided when it was scheduled (the hook may ignore it).
    pub point: Vec2,
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        let max = omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
        (!self.ability.is_empty() && self.ability.len() <= 64 && !self.ability.contains(':') && self.level > 0
            && self.point.x.raw().unsigned_abs() <= max && self.point.y.raw().unsigned_abs() <= max).then_some(self)
    }
    pub fn buff_id(&self) -> String { format!("{PREFIX}{}:{}", self.ability, self.index) }
    pub fn payload(&self) -> Value {
        json!({ (KEY): { "ability": self.ability, "level": self.level, "index": self.index,
            "x": self.point.x.raw(), "y": self.point.y.raw() } })
    }
    /// Strict: only the exact id and payload this module writes.
    pub fn from_buff(buff_id: &str, payload: &Value) -> Option<Self> {
        if !buff_id.starts_with(PREFIX) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 5 { return None; }
        let raw = |name: &str| row.get(name)?.as_i64().map(Fixed64::from_raw);
        let spec = Self { ability: row.get("ability")?.as_str()?.to_owned(),
            level: u8::try_from(row.get("level")?.as_u64()?).ok()?,
            index: u16::try_from(row.get("index")?.as_u64()?).ok()?,
            point: Vec2::new(raw("x")?, raw("y")?) }.checked()?;
        (spec.buff_id() == buff_id).then_some(spec)
    }
    /// The hook argument: `level`, `index` and `anchor` (= `point`); `total` and
    /// `elapsed` are zero for a delayed effect.
    pub fn pulse(&self) -> ChannelPulse {
        ChannelPulse { level: self.level, index: self.index, total: 0, elapsed: Fixed64::ZERO, anchor: self.point }
    }
}

pub fn valid_delay(delay: Fixed64) -> bool {
    delay >= Fixed64::from_raw(1) && delay <= Fixed64::from_i32(MAX_DELAY)
}

/// Admission of a scheduling buff: only on the scheduling caster itself.
pub fn admitted(target: Entity, buff_id: &str, duration: Fixed64, payload: &Value, caster: Entity) -> bool {
    target == caster && valid_delay(duration) && Spec::from_buff(buff_id, payload).is_some()
}

/// Effects that came due this tick, filled by `buff_tick` and drained by the
/// script dispatch.
#[derive(Default, Debug)]
pub struct DueDelayedEffects(pub Vec<(Entity, Spec)>);

/// The due effects among the rows `BuffStore::tick` expired (already in
/// canonical (entity, id) order).
pub fn collect(expired: &[(Entity, String, Value)]) -> Vec<(Entity, Spec)> {
    expired.iter().filter_map(|(entity, id, payload)| Spec::from_buff(id, payload).map(|spec| (*entity, spec))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::BuffStore;
    use specs::{Builder, World, WorldExt};

    fn spec(index: u16) -> Spec {
        Spec { ability: "b29_earth_tremor".into(), level: 2, index, point: Vec2::new(Fixed64::from_i32(300), Fixed64::from_i32(-40)) }
    }

    #[test]
    fn payload_round_trips_strictly() {
        let s = spec(7);
        assert_eq!(s.buff_id(), "delayed_effect:b29_earth_tremor:7");
        assert_eq!(Spec::from_buff(&s.buff_id(), &s.payload()), Some(s.clone()));
        assert_eq!(Spec::from_buff("delayed_effect:b29_earth_tremor:8", &s.payload()), None, "id must match the payload");
        assert_eq!(Spec::from_buff("other", &s.payload()), None);
        let mut extra = s.payload(); extra[KEY]["extra"] = 1.into();
        assert_eq!(Spec::from_buff(&s.buff_id(), &extra), None);
        for (field, value) in [("level", json!(0)), ("level", json!(300)), ("index", json!(70000)), ("ability", json!("a:b")), ("x", json!(0.5))] {
            let mut bad = s.payload(); bad[KEY][field] = value.clone();
            assert_eq!(Spec::from_buff(&s.buff_id(), &bad), None, "{field}={value}");
        }
        assert!(valid_delay(Fixed64::from_raw(1)) && valid_delay(Fixed64::from_i32(MAX_DELAY)));
        assert!(!valid_delay(Fixed64::ZERO) && !valid_delay(Fixed64::from_i32(MAX_DELAY) + Fixed64::from_raw(1)));
        let pulse = s.pulse();
        assert_eq!((pulse.level, pulse.index, pulse.total, pulse.anchor), (2, 7, 0, s.point));
    }

    #[test]
    fn admission_is_caster_only_and_bounded() {
        let mut world = World::new();
        let (caster, other) = (world.create_entity().build(), world.create_entity().build());
        let s = spec(1);
        assert!(admitted(caster, &s.buff_id(), Fixed64::from_raw(461), &s.payload(), caster));
        assert!(!admitted(other, &s.buff_id(), Fixed64::from_raw(461), &s.payload(), caster));
        assert!(!admitted(caster, &s.buff_id(), Fixed64::ZERO, &s.payload(), caster));
        assert!(!admitted(caster, &s.buff_id(), Fixed64::from_i32(11), &s.payload(), caster));
    }

    /// Resolves exactly once, at natural expiry, in canonical order; removal
    /// (caster death clears buffs) drops it; a payload-less row is no effect.
    #[test]
    fn expiry_queues_each_effect_once_in_canonical_order() {
        let mut world = World::new();
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        let half = Fixed64::from_raw(461); // 0.45 s
        for (entity, index) in [(b, 2u16), (a, 9), (a, 3)] {
            buffs.add(entity, &spec(index).buff_id(), half, spec(index).payload());
        }
        buffs.add(a, "delayed_effect:b29_earth_tremor:4", half, json!({}));
        buffs.add(b, &spec(5).buff_id(), half, spec(5).payload());
        buffs.remove(b, &spec(5).buff_id());
        let tick = Fixed64::from_raw(17);
        let mut due = Vec::new();
        let mut ticks = 0;
        while due.is_empty() { due = collect(&buffs.tick(tick)); ticks += 1; }
        assert_eq!(ticks, 28, "0.45 s at 60 Hz resolves on the first tick at or past the delay");
        let order: Vec<_> = due.iter().map(|(e, s)| (*e, s.index)).collect();
        assert_eq!(order, vec![(a, 3), (a, 9), (b, 2)]);
        assert!(collect(&buffs.tick(tick)).is_empty(), "never twice");
    }
}
