//! scheduled_effect：產生的技能 Rust 的 `delay`／`repeat`／`periodic` 帶值排程（v3 `delay`
//! 捕捉單位與點、`repeat` 帶第幾次）。delayed_effect 只帶一個點、同一招同一步只能有一筆；這裡每筆
//! 有自己的 instance 與最多 [`MAX_ARGS`] 個捕捉值。
//!
//! A script schedules one by sending the owner an ordinary buff
//! `add_stat_buff(owner, spec.buff_id(), spec.lifetime(after), spec.payload())`. The row lives in the
//! owner's `BuffStore` (replicated and hashed like every marker; the DLL keeps nothing). Its
//! lifetime is the delay plus [`GRACE`]: the script dispatch fires every row whose remaining time
//! has reached `GRACE` ([`due`]), removes it and calls `AbilityScript::on_scheduled` once, in
//! (owner, buff id) order. So it never depends on when `buff_tick` expires rows. Like a delayed
//! effect the hook may only apply effects (no charge, cast or spawn); the owner's death clears its
//! buffs and with them every pending row; `remove_buff(owner, id)` cancels one.
use omb_script_abi::types::{EntityHandle, ScheduleArg, Scheduled};
use omoba_sim::{Fixed64, Vec2};
use serde_json::{json, Value};
use specs::Entity;

use crate::runtime::BuffStore;

pub const KEY: &str = "__scheduled_effect_v1";
/// Buff id prefix: `scheduled:<ability>:<instance>:<step>`.
pub const PREFIX: &str = "scheduled:";
/// Kept past the delay so the dispatch, not natural expiry, resolves the row.
pub const GRACE: Fixed64 = Fixed64::from_raw(1024);
pub const MAX_DELAY: i32 = 120;
pub const MAX_ARGS: usize = 8;
/// Pending rows one owner may hold.
pub const MAX_PER_OWNER: usize = 128;

pub const ARG_NUM: u8 = 0;
pub const ARG_UNIT: u8 = 1;
pub const ARG_POINT: u8 = 2;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Spec {
    pub ability: String,
    pub level: u8,
    pub step: u16,
    /// Distinguishes concurrent schedules of one owner, ability and step.
    pub instance: u32,
    pub args: Vec<ScheduleArg>,
}

fn arg_json(a: &ScheduleArg) -> Option<Value> {
    Some(match a.kind {
        ARG_NUM => json!({ "n": a.num.raw() }),
        ARG_UNIT => json!({ "u": [a.unit.id, a.unit.gen] }),
        ARG_POINT => json!({ "p": [a.point.x.raw(), a.point.y.raw()] }),
        _ => return None,
    })
}

fn arg_of(v: &Value) -> Option<ScheduleArg> {
    let o = v.as_object()?;
    if o.len() != 1 { return None; }
    let blank = ScheduleArg { kind: ARG_NUM, num: Fixed64::ZERO, unit: EntityHandle::INVALID, point: Vec2::ZERO };
    if let Some(n) = o.get("n") { return Some(ScheduleArg { num: Fixed64::from_raw(n.as_i64()?), ..blank }); }
    let pair = |k: &str| -> Option<(i64, i64)> {
        let a = o.get(k)?.as_array()?;
        (a.len() == 2).then_some(())?;
        Some((a[0].as_i64()?, a[1].as_i64()?))
    };
    if let Some((id, gen)) = pair("u") {
        return Some(ScheduleArg { kind: ARG_UNIT, unit: EntityHandle { id: u32::try_from(id).ok()?, gen: u32::try_from(gen).ok()? }, ..blank });
    }
    let (x, y) = pair("p")?;
    let max = omoba_sim::terrain::MAX_GEOMETRY_RAW as u64;
    (x.unsigned_abs() <= max && y.unsigned_abs() <= max).then_some(())?;
    Some(ScheduleArg { kind: ARG_POINT, point: Vec2::new(Fixed64::from_raw(x), Fixed64::from_raw(y)), ..blank })
}

impl Spec {
    pub fn checked(self) -> Option<Self> {
        (!self.ability.is_empty() && self.ability.len() <= 64 && !self.ability.contains(':') && self.level > 0
            && self.args.len() <= MAX_ARGS && self.args.iter().all(|a| arg_json(a).is_some())).then_some(self)
    }
    pub fn buff_id(&self) -> String { format!("{PREFIX}{}:{}:{}", self.ability, self.instance, self.step) }
    /// The row's lifetime for a delay of `after`.
    pub fn lifetime(after: Fixed64) -> Fixed64 { after + GRACE }
    pub fn payload(&self) -> Value {
        let args: Vec<Value> = self.args.iter().filter_map(arg_json).collect();
        json!({ (KEY): { "ability": self.ability, "level": self.level, "step": self.step, "instance": self.instance, "args": args } })
    }
    /// Strict: only the exact id and payload this module writes.
    pub fn from_buff(buff_id: &str, payload: &Value) -> Option<Self> {
        if !buff_id.starts_with(PREFIX) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        if row.len() != 5 { return None; }
        let args = row.get("args")?.as_array()?;
        if args.len() > MAX_ARGS { return None; }
        let spec = Self {
            ability: row.get("ability")?.as_str()?.to_owned(),
            level: u8::try_from(row.get("level")?.as_u64()?).ok()?,
            step: u16::try_from(row.get("step")?.as_u64()?).ok()?,
            instance: u32::try_from(row.get("instance")?.as_u64()?).ok()?,
            args: args.iter().map(arg_of).collect::<Option<Vec<_>>>()?,
        }.checked()?;
        (spec.buff_id() == buff_id).then_some(spec)
    }
    pub fn scheduled(&self) -> Scheduled { Scheduled { level: self.level, step: self.step, instance: self.instance } }
}

pub fn valid_delay(after: Fixed64) -> bool { after >= Fixed64::ZERO && after <= Fixed64::from_i32(MAX_DELAY) }

/// Admission of a scheduling row: a lifetime of a valid delay plus the grace, within the bound of
/// the unit that holds it (rescheduling the same row always fits). The holder may be another unit
/// than the scheduling caster: a status' periodic trigger runs on the status' holder.
pub fn admitted(buffs: &BuffStore, target: Entity, buff_id: &str, duration: Fixed64, payload: &Value) -> bool {
    duration >= GRACE && valid_delay(duration - GRACE) && Spec::from_buff(buff_id, payload).is_some()
        && (buffs.has(target, buff_id)
            || buffs.iter_for(target).filter(|(id, _)| id.starts_with(PREFIX)).count() < MAX_PER_OWNER)
}

/// The rows whose delay has passed after this tick's `dt`, in canonical (owner, buff id) order.
/// The caller removes them and resolves each once.
pub fn due(buffs: &BuffStore, dt: Fixed64) -> Vec<(Entity, String, Spec)> {
    let mut owners: Vec<Entity> = buffs.entities_with_key(KEY).collect();
    owners.sort_by_key(|e| (e.id(), e.gen().id()));
    owners.dedup();
    let mut out = Vec::new();
    for owner in owners {
        let mut rows: Vec<(String, Spec)> = buffs.iter_for(owner)
            .filter(|(_, b)| b.remaining - dt <= GRACE)
            .filter_map(|(id, b)| Spec::from_buff(id, &b.payload).map(|s| (id.to_owned(), s)))
            .collect();
        rows.sort_by(|a, b| a.0.cmp(&b.0));
        out.extend(rows.into_iter().map(|(id, s)| (owner, id, s)));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use specs::{Builder, World, WorldExt};

    fn unit(id: u32) -> ScheduleArg {
        ScheduleArg { kind: ARG_UNIT, num: Fixed64::ZERO, unit: EntityHandle { id, gen: 1 }, point: Vec2::ZERO }
    }
    fn num(raw: i64) -> ScheduleArg {
        ScheduleArg { kind: ARG_NUM, num: Fixed64::from_raw(raw), unit: EntityHandle::INVALID, point: Vec2::ZERO }
    }
    fn point(x: i32, y: i32) -> ScheduleArg {
        ScheduleArg { kind: ARG_POINT, num: Fixed64::ZERO, unit: EntityHandle::INVALID, point: Vec2::new(Fixed64::from_i32(x), Fixed64::from_i32(y)) }
    }
    fn spec(instance: u32, step: u16) -> Spec {
        Spec { ability: "c08_devour".into(), level: 3, step, instance, args: vec![unit(9), num(2048), point(300, -40)] }
    }

    #[test]
    fn payload_round_trips_strictly() {
        let s = spec(77, 2);
        assert_eq!(s.buff_id(), "scheduled:c08_devour:77:2");
        assert_eq!(Spec::from_buff(&s.buff_id(), &s.payload()), Some(s.clone()));
        assert_eq!(Spec::from_buff("scheduled:c08_devour:77:3", &s.payload()), None, "id must match the payload");
        let mut extra = s.payload(); extra[KEY]["x"] = 1.into();
        assert_eq!(Spec::from_buff(&s.buff_id(), &extra), None);
        let mut bad = s.payload(); bad[KEY]["args"][0] = json!({ "q": 1 });
        assert_eq!(Spec::from_buff(&s.buff_id(), &bad), None, "unknown argument kind");
        let too_many = Spec { args: vec![num(1); MAX_ARGS + 1], ..spec(1, 0) };
        assert!(too_many.checked().is_none());
    }

    #[test]
    fn admission_is_bounded_and_needs_the_grace() {
        let mut world = World::new();
        let (owner, other) = (world.create_entity().build(), world.create_entity().build());
        let buffs = BuffStore::default();
        let s = spec(1, 0);
        let life = Spec::lifetime(Fixed64::from_i32(3));
        assert!(admitted(&buffs, owner, &s.buff_id(), life, &s.payload()));
        assert!(admitted(&buffs, owner, &s.buff_id(), GRACE, &s.payload()), "a zero delay fires on the next dispatch");
        assert!(admitted(&buffs, other, &s.buff_id(), life, &s.payload()), "a status holder may hold its periodic schedule");
        assert!(!admitted(&buffs, owner, &s.buff_id(), Fixed64::from_raw(10), &s.payload()), "shorter than the grace");
        assert!(!admitted(&buffs, owner, &s.buff_id(), Spec::lifetime(Fixed64::from_i32(MAX_DELAY + 1)), &s.payload()));
        let mut full = BuffStore::default();
        for i in 0..MAX_PER_OWNER as u32 { let r = spec(i, 0); full.add(other, &r.buff_id(), life, r.payload()); }
        assert!(!admitted(&full, other, &spec(9999, 0).buff_id(), life, &spec(9999, 0).payload()), "bounded per holder");
    }

    /// Each row resolves exactly once, when its delay has passed, in canonical order, and the
    /// row's own lifetime never runs out first.
    #[test]
    fn rows_come_due_once_in_canonical_order() {
        let mut world = World::new();
        let (a, b) = (world.create_entity().build(), world.create_entity().build());
        let mut buffs = BuffStore::new();
        let half = Fixed64::from_raw(461);
        for (owner, inst) in [(b, 2u32), (a, 9), (a, 3)] {
            buffs.add(owner, &spec(inst, 0).buff_id(), Spec::lifetime(half), spec(inst, 0).payload());
        }
        buffs.add(a, &spec(4, 1).buff_id(), Spec::lifetime(Fixed64::from_i32(2)), spec(4, 1).payload());
        let tick = Fixed64::from_raw(17);
        let mut ticks = 0;
        let fired = loop {
            let due = due(&buffs, tick);
            assert!(buffs.tick(tick).is_empty(), "the grace keeps every row until the dispatch takes it");
            ticks += 1;
            if !due.is_empty() { break due; }
        };
        assert_eq!(ticks, 28, "fires on the tick whose end reaches the delay (0.45 s at 60 Hz)");
        let order: Vec<_> = fired.iter().map(|(e, _, s)| (*e, s.instance)).collect();
        assert_eq!(order, vec![(a, 3), (a, 9), (b, 2)]);
        for (owner, id, _) in &fired { buffs.remove(*owner, id); }
        assert!(due(&buffs, tick).is_empty(), "never twice");
        assert!(buffs.has(a, &spec(4, 1).buff_id()), "the later row waits");
    }
}

#[cfg(test)]
#[path = "script_hooks_tests.rs"]
mod script_hooks_tests;
