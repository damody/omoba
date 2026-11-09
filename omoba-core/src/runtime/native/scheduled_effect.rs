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
//!
//! A row scheduled by an item's cast (`delay`/`repeat` inside an active item's `on_cast`) keeps
//! that item's inventory slot ([`Spec::item_slot`]): its `on_scheduled` runs as that item
//! instance again (ext EVENT_ITEM_SLOT), so the instance's `scope = "item_instance"` state
//! (item_grant::instance_var_key), `remove_item` of itself and the rows it schedules in turn
//! stay with the item, as in the cast.
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
    /// The inventory slot of the item whose cast scheduled it (the scheduling unit's own item).
    pub item_slot: Option<u8>,
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
        let mut row = json!({ "ability": self.ability, "level": self.level, "step": self.step, "instance": self.instance, "args": args });
        if let Some(slot) = self.item_slot { row["item_slot"] = json!(slot); }
        json!({ (KEY): row })
    }
    /// Strict: only the exact id and payload this module writes.
    pub fn from_buff(buff_id: &str, payload: &Value) -> Option<Self> {
        if !buff_id.starts_with(PREFIX) || payload.as_object()?.len() != 1 { return None; }
        let row = payload.get(KEY)?.as_object()?;
        let item_slot = match row.get("item_slot") { Some(slot) => Some(u8::try_from(slot.as_u64()?).ok()?), None => None };
        if row.len() != 5 + usize::from(item_slot.is_some()) { return None; }
        let args = row.get("args")?.as_array()?;
        if args.len() > MAX_ARGS { return None; }
        let spec = Self {
            ability: row.get("ability")?.as_str()?.to_owned(),
            level: u8::try_from(row.get("level")?.as_u64()?).ok()?,
            step: u16::try_from(row.get("step")?.as_u64()?).ok()?,
            instance: u32::try_from(row.get("instance")?.as_u64()?).ok()?,
            args: args.iter().map(arg_of).collect::<Option<Vec<_>>>()?,
            item_slot,
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

/// The rows whose delay has passed, to the nearest tick (less than half of this tick's `dt` left),
/// in canonical (owner, buff id) order. The script dispatch runs after this tick's `buff_tick`, so
/// `remaining` already counts this tick: a row added in tick N with a delay of k ticks comes due in
/// tick N + k (a 0.25 s period is 15 ticks at 60 Hz, also when `on_scheduled` reschedules itself).
/// A delay between ticks rounds to the nearest: 原作's `duration 1` with `interval 0.99` pulses
/// once, in the tick before the 1 s status expires (59 at 60 Hz), not in the tick it expires.
/// The caller removes them and resolves each once.
pub fn due(buffs: &BuffStore, dt: Fixed64) -> Vec<(Entity, String, Spec)> {

    let mut owners: Vec<Entity> = buffs.entities_with_key(KEY).collect();
    owners.sort_by_key(|e| (e.id(), e.gen().id()));
    owners.dedup();
    let mut out = Vec::new();
    for owner in owners {
        let mut rows: Vec<(String, Spec)> = buffs.iter_for(owner)
            .filter(|(_, b)| b.remaining <= GRACE || (b.remaining - GRACE) + (b.remaining - GRACE) < dt)
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
        Spec { ability: "c08_devour".into(), level: 3, step, instance, args: vec![unit(9), num(2048), point(300, -40)], item_slot: None }
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
        let item = Spec { item_slot: Some(4), ..spec(5, 1) };
        assert_eq!(Spec::from_buff(&item.buff_id(), &item.payload()), Some(item.clone()), "an item cast's row keeps its slot");
        let mut slot = item.payload(); slot[KEY]["item_slot"] = json!(300);
        assert_eq!(Spec::from_buff(&item.buff_id(), &slot), None, "a slot is a byte");
        let mut slot = item.payload(); slot[KEY]["item_slot"] = json!("4");
        assert_eq!(Spec::from_buff(&item.buff_id(), &slot), None);
    }

    /// An item cast's row runs `on_scheduled` as that item instance (ext EVENT_ITEM_SLOT), and a
    /// row it schedules from there with the slot it read keeps it; a row of no item has none.
    #[test]
    fn an_item_casts_rows_run_as_that_item_instance() {
        use crate::runtime::native::comp::{CProperty, DeltaTime, Faction, Pos};
        use abi_stable::std_types::{RResult, RStr, RString, RVec, ROk};
        use omb_script_abi::ext::{op, ExtValue};
        use omb_script_abi::types::Target;
        use omb_script_abi::world::GameWorldDyn;
        use std::sync::{Arc, Mutex};
        #[derive(Clone)]
        struct Item { seen: Arc<Mutex<Vec<(u16, Option<i64>)>>> }
        impl omb_script_abi::ability::AbilityScript for Item {
            fn ability_id(&self) -> RStr<'_> { RStr::from_str("test_item_repeat") }
            fn execute(&self, _: EntityHandle, _: Target, _: u8, _: RStr<'_>, _: &mut GameWorldDyn<'_>) -> RResult<(), RString> { ROk(()) }
            fn on_scheduled(&self, owner: EntityHandle, at: Scheduled, _: RVec<ScheduleArg>, world: &mut GameWorldDyn<'_>) {
                let slot = match world.ext(op::EVENT_ITEM_SLOT, abi_stable::std_types::RSlice::from_slice(&[])).into_result() {
                    Ok(ExtValue::Num(n)) => Some(n.raw() >> 10), _ => None,
                };
                self.seen.lock().unwrap().push((at.step, slot));
                if at.step == 1 && at.instance == 1 {
                    // the next repeat, as ability_kit `schedule` writes it inside an item cast
                    let next = Spec { ability: "test_item_repeat".into(), level: 1, step: 2, instance: 1, args: Vec::new(),
                        item_slot: slot.and_then(|slot| u8::try_from(slot).ok()) };
                    world.add_stat_buff(owner, RStr::from_str(&next.buff_id()), Spec::lifetime(Fixed64::ZERO), RStr::from_str(&next.payload().to_string()));
                }
            }
        }
        let mut world = crate::runtime::StateInitializer::setup_campaign_ecs_world(&crate::runtime::StateInitializer::create_thread_pool());
        let dt = Fixed64::from_raw(17);
        world.insert(DeltaTime(dt));
        let owner = world.create_entity().with(Pos(Vec2::ZERO))
            .with(CProperty { hp: Fixed64::ONE, mhp: Fixed64::ONE, msd: Fixed64::ZERO, def_physic: Fixed64::ZERO, def_magic: Fixed64::ZERO })
            .with(Faction::new(crate::runtime::comp::FactionType::Player, 1)).build();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let mut registry = crate::runtime::ScriptRegistry::new();
        let level = serde_json::json!({"cooldown": 1, "mana_cost": 0, "cast_time": 0, "range": 300});
        registry.insert_ability_for_test(serde_json::from_value(serde_json::json!({ "id": "test_item_repeat", "name": "test_item_repeat",
            "ability_type": "active", "target_type": "none", "max_level": 1, "levels": {"1": level} })).unwrap(),
            omb_script_abi::ability::AbilityScript_TO::from_value(Item { seen: seen.clone() }, abi_stable::sabi_trait::prelude::TD_Opaque));
        for (instance, item_slot) in [(1, Some(3)), (2, None)] {
            let row = Spec { ability: "test_item_repeat".into(), level: 1, step: 1, instance, args: Vec::new(), item_slot };
            world.write_resource::<BuffStore>().add(owner, &row.buff_id(), Spec::lifetime(Fixed64::ZERO), row.payload());
        }
        crate::runtime::run_script_dispatch(&mut world, &registry, 1, dt);
        // what the dispatch scheduled settles as a game tick would
        let added: Vec<_> = std::mem::take(&mut *world.write_resource::<Vec<crate::comp::Outcome>>()).into_iter()
            .filter_map(|outcome| match outcome {
                crate::comp::Outcome::AddBuff { target, buff_id, duration, payload, .. } if buff_id.starts_with(PREFIX) => Some((target, buff_id, duration, payload)),
                _ => None,
            }).collect();
        assert_eq!(added.len(), 1, "the item row scheduled its next repeat");
        for (target, id, duration, payload) in added { world.write_resource::<BuffStore>().add(target, &id, duration, payload); }
        crate::runtime::run_script_dispatch(&mut world, &registry, 1, dt);
        assert_eq!(*seen.lock().unwrap(), vec![(1, Some(3)), (1, None), (2, Some(3))]);
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
        // One tick: buff_tick, then the script dispatch.
        let fired = loop {
            assert!(buffs.tick(tick).is_empty(), "the grace keeps every row until the dispatch takes it");
            let due = due(&buffs, tick);
            ticks += 1;
            if !due.is_empty() { break due; }
        };
        assert_eq!(ticks, 27, "fires on the tick nearest the delay (0.45 s at 60 Hz: 27.1 ticks)");
        let order: Vec<_> = fired.iter().map(|(e, _, s)| (*e, s.instance)).collect();
        assert_eq!(order, vec![(a, 3), (a, 9), (b, 2)]);
        for (owner, id, _) in &fired { buffs.remove(*owner, id); }
        assert!(due(&buffs, tick).is_empty(), "never twice");
        assert!(buffs.has(a, &spec(4, 1).buff_id()), "the later row waits");
    }

    /// A periodic step that reschedules itself from `on_scheduled` (its AddBuff settles after this
    /// tick's buff_tick) keeps the authored period: 0.25 s is every 15 ticks at 60 Hz, every 30 at 120 Hz.
    #[test]
    fn a_row_rescheduled_in_the_dispatch_keeps_its_period() {
        for (hz, period) in [(60, 15), (120, 30)] {
            let mut world = World::new();
            let owner = world.create_entity().build();
            let mut buffs = BuffStore::new();
            let interval = Fixed64::from_raw(256);
            let s = spec(1, 3);
            // Scheduled by a cast in tick 0, after that tick's buff_tick.
            buffs.add(owner, &s.buff_id(), Spec::lifetime(interval), s.payload());
            let mut fired = Vec::new();
            for tick in 1..=4 * period {
                let dt = Fixed64::from_raw((tick * 1024 / hz - (tick - 1) * 1024 / hz) as i64);
                buffs.tick(dt);
                for (holder, id, _) in due(&buffs, dt) {
                    buffs.remove(holder, &id);
                    fired.push(tick);
                    buffs.add(owner, &s.buff_id(), Spec::lifetime(interval), s.payload());
                }
            }
            assert_eq!(fired, (1..=4).map(|n| n * period).collect::<Vec<_>>(), "{hz} Hz");
        }
    }

    /// 原作 `duration 1, interval 0.99`: the pulse rounds to the nearest tick, the one before the
    /// status's last (E385).
    #[test]
    fn a_delay_between_ticks_comes_due_in_the_nearest_tick() {
        for (hz, expected) in [(60, 59), (30, 30), (120, 119)] {
            let mut world = World::new();
            let owner = world.create_entity().build();
            let mut buffs = BuffStore::new();
            let s = spec(1, 4);
            buffs.add(owner, &s.buff_id(), Spec::lifetime(Fixed64::from_raw(1014)), s.payload());
            let mut fired = None;
            for tick in 1..=2 * hz {
                let dt = Fixed64::from_raw((tick * 1024 / hz - (tick - 1) * 1024 / hz) as i64);
                buffs.tick(dt);
                if fired.is_none() && !due(&buffs, dt).is_empty() { fired = Some(tick); }
            }
            assert_eq!(fired, Some(expected), "0.99 s at {hz} Hz");
        }
    }
}

#[cfg(test)]
#[path = "script_hooks_tests.rs"]
mod script_hooks_tests;
